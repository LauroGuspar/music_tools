use gpui::{Pixels, SharedString, TextAlign, px};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColumnWidth {
    Fixed(Pixels),
    Fill(f32),
}

impl Default for ColumnWidth {
    fn default() -> Self {
        Self::Fill(1.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

impl SortDirection {
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableSort<F> {
    pub column: F,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnSpec<F = String> {
    pub id: F,
    pub label: SharedString,
    pub width: ColumnWidth,
    pub min_width: Pixels,
    pub align: TextAlign,
    pub sortable: bool,
}

impl<F: Clone> ColumnSpec<F> {
    pub fn new(id: F, label: impl Into<SharedString>) -> Self {
        Self {
            id,
            label: label.into(),
            width: ColumnWidth::Fill(1.0),
            min_width: px(40.0),
            align: TextAlign::Left,
            sortable: true,
        }
    }

    pub fn fixed(mut self, width: Pixels) -> Self {
        self.width = ColumnWidth::Fixed(width);
        self.min_width = width;
        self
    }

    pub fn fill(mut self, weight: f32) -> Self {
        self.width = ColumnWidth::Fill(weight.max(0.1));
        self
    }

    pub fn min_width(mut self, min: Pixels) -> Self {
        self.min_width = min;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}

/// Resolves column widths given the available width and column specifications.
/// Fixed columns take their specified width (or at least `min_width`).
/// Remaining space is distributed among Fill columns proportionally to their weight.
/// Every column is clamped to at least its `min_width`.
pub fn resolve_widths<F: Clone>(columns: &[ColumnSpec<F>], available_width: Pixels) -> Vec<Pixels> {
    if columns.is_empty() {
        return Vec::new();
    }

    let mut resolved: Vec<Pixels> = Vec::with_capacity(columns.len());
    let mut total_fixed = Pixels::ZERO;
    let mut total_fill_weight = 0.0f32;

    for col in columns {
        match col.width {
            ColumnWidth::Fixed(w) => {
                let actual = w.max(col.min_width);
                total_fixed += actual;
                resolved.push(actual);
            }
            ColumnWidth::Fill(weight) => {
                total_fill_weight += weight;
                resolved.push(col.min_width); // temporary placeholder
            }
        }
    }

    let available_for_fill = (available_width - total_fixed).max(Pixels::ZERO);

    if total_fill_weight > 0.0 {
        for (i, col) in columns.iter().enumerate() {
            if let ColumnWidth::Fill(weight) = col.width {
                let share = (f32::from(available_for_fill) * (weight / total_fill_weight)).floor();
                let col_width = px(share).max(col.min_width);
                resolved[i] = col_width;
            }
        }
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_spec_builder() {
        let col = ColumnSpec::new("title", "Título")
            .fill(2.0)
            .min_width(px(120.))
            .align(TextAlign::Left)
            .sortable(true);

        assert_eq!(col.id, "title");
        assert_eq!(col.label.as_ref(), "Título");
        assert_eq!(col.width, ColumnWidth::Fill(2.0));
        assert_eq!(col.min_width, px(120.));
        assert_eq!(col.align, TextAlign::Left);
        assert!(col.sortable);
    }

    #[test]
    fn test_resolve_widths_fixed_and_fill() {
        let cols = vec![
            ColumnSpec::new("index", "#").fixed(px(50.)),
            ColumnSpec::new("title", "Título").fill(1.0).min_width(px(100.)),
            ColumnSpec::new("artist", "Artista").fill(1.0).min_width(px(100.)),
            ColumnSpec::new("duration", "Duración").fixed(px(80.)),
        ];

        // Total available width = 1000px
        // Fixed = 50 + 80 = 130px
        // Fill remaining = 1000 - 130 = 870px
        // Title (weight 1.0) = 435px, Artist (weight 1.0) = 435px
        let resolved = resolve_widths(&cols, px(1000.));
        assert_eq!(resolved.len(), 4);
        assert_eq!(resolved[0], px(50.));
        assert_eq!(resolved[1], px(435.));
        assert_eq!(resolved[2], px(435.));
        assert_eq!(resolved[3], px(80.));
    }

    #[test]
    fn test_resolve_widths_respects_min_width() {
        let cols = vec![
            ColumnSpec::new("c1", "C1").fill(1.0).min_width(px(150.)),
            ColumnSpec::new("c2", "C2").fill(1.0).min_width(px(150.)),
        ];

        // Only 200px available, but each requires min 150px
        let resolved = resolve_widths(&cols, px(200.));
        assert_eq!(resolved[0], px(150.));
        assert_eq!(resolved[1], px(150.));
    }

    #[test]
    fn test_sort_direction_toggle() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }
}
