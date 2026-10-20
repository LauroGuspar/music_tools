pub mod layout;

use std::collections::HashSet;
use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, Pixels, RenderOnce, Stateful, StyleRefinement, Styled,
    Window, div, px,
};

pub use layout::{ColumnSpec, ColumnWidth, SortDirection, TableSort, resolve_widths};

use crate::theme::colors;

pub const OVERSCAN: usize = 2;
pub const OVERSCAN_ABOVE: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleRange {
    pub first: usize,
    pub count: usize,
    pub last: usize,
}

/// Computes the range of visible row indices in a virtualized list or table.
///
/// Uses Sonora's formula with overscan:
/// - `first = ((scroll_top - head) / row_height).floor() - OVERSCAN_ABOVE` (saturating_sub)
/// - `count = (viewport_height / row_height).ceil() + OVERSCAN + OVERSCAN_ABOVE`
/// - `last = (first + count).min(total_rows)`
pub fn calculate_visible_rows(
    scroll_top: Pixels,
    viewport_height: Pixels,
    row_height: Pixels,
    total_rows: usize,
) -> VisibleRange {
    if total_rows == 0 || row_height <= Pixels::ZERO {
        return VisibleRange {
            first: 0,
            count: 0,
            last: 0,
        };
    }

    let top = scroll_top.max(Pixels::ZERO);
    let under_edge = (top / row_height).floor().max(0.) as usize;
    let first = under_edge.saturating_sub(OVERSCAN_ABOVE);
    let count_needed = (viewport_height / row_height).ceil().max(0.) as usize + OVERSCAN + OVERSCAN_ABOVE;
    let last = (first + count_needed).min(total_rows);
    let count = last.saturating_sub(first);

    VisibleRange { first, count, last }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableSelection {
    selected: Option<usize>,
    marked: HashSet<usize>,
}

impl TableSelection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn select(&mut self, row: usize) {
        self.selected = Some(row);
        self.marked.clear();
        self.marked.insert(row);
    }

    pub fn toggle(&mut self, row: usize) {
        if self.marked.contains(&row) {
            self.marked.remove(&row);
            if self.selected == Some(row) {
                self.selected = self.marked.iter().copied().next();
            }
        } else {
            self.marked.insert(row);
            self.selected = Some(row);
        }
    }

    pub fn select_range(&mut self, from: usize, to: usize) {
        let start = from.min(to);
        let end = from.max(to);
        self.marked.clear();
        for r in start..=end {
            self.marked.insert(r);
        }
        self.selected = Some(to);
    }

    pub fn clear(&mut self) {
        self.selected = None;
        self.marked.clear();
    }

    pub fn is_selected(&self, row: usize) -> bool {
        self.marked.contains(&row)
    }

    pub fn primary_selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn count(&self) -> usize {
        self.marked.len()
    }

    pub fn is_empty(&self) -> bool {
        self.marked.is_empty()
    }

    pub fn selected_rows(&self) -> Vec<usize> {
        let mut rows: Vec<usize> = self.marked.iter().copied().collect();
        rows.sort_unstable();
        rows
    }
}

pub type RowSelectCallback = Rc<dyn Fn(usize, bool, bool, &mut Window, &mut App) + 'static>;
pub type RowDoubleClickedCallback = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type SortChangeCallback = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;
pub type RowCellBuilder<R> = Rc<dyn Fn(usize, &R, usize, &ColumnSpec<String>, Pixels) -> AnyElement + 'static>;

#[derive(IntoElement)]
pub struct VirtualTable<R: 'static> {
    base: Stateful<Div>,
    columns: Vec<ColumnSpec<String>>,
    rows: Vec<R>,
    scroll_top: Pixels,
    viewport_height: Pixels,
    row_height: Pixels,
    selection: TableSelection,
    sort: Option<TableSort<String>>,
    row_cell_builder: Option<RowCellBuilder<R>>,
    on_select_row: Option<RowSelectCallback>,
    on_double_click_row: Option<RowDoubleClickedCallback>,
    on_sort_change: Option<SortChangeCallback>,
}

impl<R: 'static> VirtualTable<R> {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            columns: Vec::new(),
            rows: Vec::new(),
            scroll_top: Pixels::ZERO,
            viewport_height: px(500.),
            row_height: px(36.),
            selection: TableSelection::new(),
            sort: None,
            row_cell_builder: None,
            on_select_row: None,
            on_double_click_row: None,
            on_sort_change: None,
        }
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = ColumnSpec<String>>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }

    pub fn rows(mut self, rows: Vec<R>) -> Self {
        self.rows = rows;
        self
    }

    pub fn scroll_top(mut self, scroll: Pixels) -> Self {
        self.scroll_top = scroll;
        self
    }

    pub fn viewport_height(mut self, height: Pixels) -> Self {
        self.viewport_height = height;
        self
    }

    pub fn row_height(mut self, height: Pixels) -> Self {
        self.row_height = height;
        self
    }

    pub fn selection(mut self, selection: TableSelection) -> Self {
        self.selection = selection;
        self
    }

    pub fn sort(mut self, sort: Option<TableSort<String>>) -> Self {
        self.sort = sort;
        self
    }

    pub fn cell_builder(
        mut self,
        builder: impl Fn(usize, &R, usize, &ColumnSpec<String>, Pixels) -> AnyElement + 'static,
    ) -> Self {
        self.row_cell_builder = Some(Rc::new(builder));
        self
    }

    pub fn on_select_row(
        mut self,
        handler: impl Fn(usize, bool, bool, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_row = Some(Rc::new(handler));
        self
    }

    pub fn on_double_click_row(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_double_click_row = Some(Rc::new(handler));
        self
    }

    pub fn on_sort_change(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort_change = Some(Rc::new(handler));
        self
    }
}

impl<R: 'static> Styled for VirtualTable<R> {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl<R: 'static> RenderOnce for VirtualTable<R> {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let total_rows = self.rows.len();
        let visible = calculate_visible_rows(
            self.scroll_top,
            self.viewport_height,
            self.row_height,
            total_rows,
        );

        let total_content_height = self.row_height * total_rows as f32;
        let widths = resolve_widths(&self.columns, px(1000.)); // default layout width

        // Header element
        let sort_change = self.on_sort_change.clone();
        let header_cells: Vec<_> = self
            .columns
            .iter()
            .enumerate()
            .map(|(col_idx, col)| {
                let width = widths.get(col_idx).copied().unwrap_or(px(100.));
                let is_sorted = self.sort.as_ref().map(|s| s.column == col.id).unwrap_or(false);
                let sort_indicator = if is_sorted {
                    match self.sort.as_ref().map(|s| s.direction) {
                        Some(SortDirection::Ascending) => " ↑",
                        Some(SortDirection::Descending) => " ↓",
                        None => "",
                    }
                } else {
                    ""
                };

                let mut header_cell = div()
                    .id(ElementId::named_usize("header-col", col_idx))
                    .w(width)
                    .px_3()
                    .py_2()
                    .flex_none()
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(if is_sorted {
                        colors::ACCENT_EMERALD
                    } else {
                        colors::TEXT_MUTED
                    })
                    .truncate()
                    .text_align(col.align)
                    .child(format!("{}{}", col.label, sort_indicator));

                if col.sortable {
                    header_cell = header_cell.cursor_pointer().hover(|s| {
                        s.text_color(colors::TEXT_PRIMARY)
                    });

                    if let Some(ref on_sort) = sort_change {
                        let on_sort = on_sort.clone();
                        header_cell = header_cell.on_click(move |_, window, cx| {
                            on_sort(col_idx, window, cx);
                        });
                    }
                }

                header_cell
            })
            .collect();

        let header = div()
            .flex()
            .items_center()
            .w_full()
            .h(px(32.))
            .bg(colors::BG_BASE)
            .border_b_1()
            .border_color(colors::BORDER_BASE)
            .children(header_cells);

        // Virtualized rows
        let select_handler = self.on_select_row.clone();
        let dbl_click_handler = self.on_double_click_row.clone();
        let cell_builder = self.row_cell_builder.clone();

        let visible_row_elements: Vec<_> = (visible.first..visible.last)
            .filter_map(|row_idx| {
                let item = self.rows.get(row_idx)?;
                let is_selected = self.selection.is_selected(row_idx);
                let top_pos = self.row_height * row_idx as f32;

                let row_cells: Vec<AnyElement> = self
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(col_idx, col)| {
                        let width = widths.get(col_idx).copied().unwrap_or(px(100.));
                        if let Some(ref builder) = cell_builder {
                            builder(row_idx, item, col_idx, col, width)
                        } else {
                            div()
                                .w(width)
                                .px_3()
                                .flex_none()
                                .truncate()
                                .text_sm()
                                .text_align(col.align)
                                .text_color(if is_selected {
                                    colors::TEXT_PRIMARY
                                } else {
                                    colors::TEXT_SECONDARY
                                })
                                .child(format!("Row {row_idx}"))
                                .into_any_element()
                        }
                    })
                    .collect();

                let mut row_el = div()
                    .id(ElementId::named_usize("table-row", row_idx))
                    .absolute()
                    .top(top_pos)
                    .left_0()
                    .right_0()
                    .h(self.row_height)
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE);

                if is_selected {
                    row_el = row_el.bg(colors::BG_CARD);
                } else {
                    row_el = row_el.hover(|s| s.bg(colors::BG_SURFACE_HOVER));
                }

                if let Some(ref on_select) = select_handler {
                    let on_select = on_select.clone();
                    row_el = row_el.on_mouse_down(
                        MouseButton::Left,
                        move |ev: &MouseDownEvent, window, cx| {
                            let shift = ev.modifiers.shift;
                            let secondary = ev.modifiers.secondary() && !ev.modifiers.shift;
                            on_select(row_idx, shift, secondary, window, cx);
                        },
                    );
                }

                if let Some(ref on_dbl_click) = dbl_click_handler {
                    let on_dbl_click = on_dbl_click.clone();
                    row_el = row_el.on_click(move |ev: &ClickEvent, window, cx| {
                        if ev.click_count() >= 2 {
                            on_dbl_click(row_idx, window, cx);
                        }
                    });
                }

                Some(row_el.children(row_cells))
            })
            .collect();

        // Scroll container content
        let scroll_content = div()
            .relative()
            .w_full()
            .h(total_content_height)
            .children(visible_row_elements);

        let viewport = div()
            .relative()
            .w_full()
            .h(self.viewport_height)
            .overflow_hidden()
            .child(scroll_content);

        self.base
            .flex()
            .flex_col()
            .w_full()
            .bg(colors::BG_SURFACE)
            .child(header)
            .child(viewport)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_visible_rows_at_top() {
        let total_rows = 1000;
        let row_height = px(40.);
        let viewport_height = px(400.); // 10 rows fit on screen
        let scroll_top = px(0.);

        let visible = calculate_visible_rows(scroll_top, viewport_height, row_height, total_rows);

        // At top: under_edge = 0. first = 0.saturating_sub(1) = 0.
        // count_needed = (400 / 40) + OVERSCAN(2) + OVERSCAN_ABOVE(1) = 10 + 3 = 13.
        // last = min(0 + 13, 1000) = 13.
        assert_eq!(visible.first, 0);
        assert_eq!(visible.count, 13);
        assert_eq!(visible.last, 13);
    }

    #[test]
    fn test_calculate_visible_rows_scrolled_middle() {
        let total_rows = 1000;
        let row_height = px(40.);
        let viewport_height = px(400.); // 10 rows
        let scroll_top = px(400.); // Scrolled past 10 rows

        let visible = calculate_visible_rows(scroll_top, viewport_height, row_height, total_rows);

        // under_edge = 400 / 40 = 10.
        // first = 10.saturating_sub(OVERSCAN_ABOVE: 1) = 9.
        // count_needed = 10 + 2 + 1 = 13.
        // last = min(9 + 13, 1000) = 22.
        assert_eq!(visible.first, 9);
        assert_eq!(visible.last, 22);
        assert_eq!(visible.count, 13);
    }

    #[test]
    fn test_calculate_visible_rows_near_end() {
        let total_rows = 50;
        let row_height = px(40.);
        let viewport_height = px(400.); // 10 rows
        let scroll_top = px(1800.); // Scrolled near end: 1800 / 40 = 45

        let visible = calculate_visible_rows(scroll_top, viewport_height, row_height, total_rows);

        // under_edge = 45.
        // first = 45 - 1 = 44.
        // last = min(44 + 13, 50) = 50.
        // count = 50 - 44 = 6.
        assert_eq!(visible.first, 44);
        assert_eq!(visible.last, 50);
        assert_eq!(visible.count, 6);
    }

    #[test]
    fn test_calculate_visible_rows_empty_table() {
        let visible = calculate_visible_rows(px(0.), px(400.), px(40.), 0);
        assert_eq!(visible.first, 0);
        assert_eq!(visible.count, 0);
        assert_eq!(visible.last, 0);
    }

    #[test]
    fn test_selection_management() {
        let mut selection = TableSelection::new();
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);

        // Single selection
        selection.select(5);
        assert!(selection.is_selected(5));
        assert!(!selection.is_selected(6));
        assert_eq!(selection.primary_selected(), Some(5));
        assert_eq!(selection.count(), 1);

        // Toggle selection (ctrl+click)
        selection.toggle(8);
        assert!(selection.is_selected(5));
        assert!(selection.is_selected(8));
        assert_eq!(selection.count(), 2);
        assert_eq!(selection.selected_rows(), vec![5, 8]);

        // Toggle off
        selection.toggle(5);
        assert!(!selection.is_selected(5));
        assert!(selection.is_selected(8));
        assert_eq!(selection.count(), 1);

        // Range selection (shift+click)
        selection.select_range(2, 5);
        assert_eq!(selection.selected_rows(), vec![2, 3, 4, 5]);
        assert_eq!(selection.count(), 4);

        // Clear
        selection.clear();
        assert!(selection.is_empty());
        assert_eq!(selection.count(), 0);
    }

    #[test]
    fn test_virtual_table_builder() {
        let cols = vec![
            ColumnSpec::new("title".to_string(), "Título"),
            ColumnSpec::new("artist".to_string(), "Artista"),
        ];
        let items = vec!["Song 1", "Song 2", "Song 3"];

        let table = VirtualTable::new("my-table")
            .columns(cols)
            .rows(items)
            .row_height(px(40.))
            .viewport_height(px(600.))
            .sort(Some(TableSort {
                column: "title".to_string(),
                direction: SortDirection::Ascending,
            }));

        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.row_height, px(40.));
        assert_eq!(table.viewport_height, px(600.));
        assert!(table.sort.is_some());
    }
}
