use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    RenderOnce, SharedString, Stateful, StyleRefinement, Styled, Window, div,
};

use crate::theme::{Theme, colors};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppTab {
    Staging,
    Compare,
    Downloads,
    Utilities,
    Settings,
}

impl AppTab {
    pub const ALL: [Self; 5] = [
        Self::Staging,
        Self::Compare,
        Self::Downloads,
        Self::Utilities,
        Self::Settings,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Staging => "staging",
            Self::Compare => "compare",
            Self::Downloads => "downloads",
            Self::Utilities => "utilities",
            Self::Settings => "settings",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Staging => "Mesa de Trabajo",
            Self::Compare => "Comparador",
            Self::Downloads => "Descargas",
            Self::Utilities => "Utilidades",
            Self::Settings => "Configuración",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Staging => "⚡",
            Self::Compare => "⇄",
            Self::Downloads => "↓",
            Self::Utilities => "🛠",
            Self::Settings => "⚙",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub badge: Option<usize>,
}

impl TabItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn badge(mut self, count: usize) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn from_app_tab(tab: AppTab) -> Self {
        Self::new(tab.id(), tab.label()).icon(tab.icon())
    }
}

type TabSelectHandler = Rc<dyn Fn(&SharedString, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Tabs {
    base: Stateful<Div>,
    items: Vec<TabItem>,
    active_tab: SharedString,
    on_select: Option<TabSelectHandler>,
}

impl Tabs {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            items: Vec::new(),
            active_tab: SharedString::default(),
            on_select: None,
        }
    }

    /// Convenience builder for the standard Music Tools navigation tabs
    pub fn for_app(active: AppTab) -> Self {
        let items: Vec<TabItem> = AppTab::ALL.iter().copied().map(TabItem::from_app_tab).collect();
        Self::new("app-tabs")
            .items(items)
            .active(active.id())
    }

    pub fn item(mut self, item: TabItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn active(mut self, active_id: impl Into<SharedString>) -> Self {
        self.active_tab = active_id.into();
        self
    }

    pub fn set_badge(mut self, tab_id: &str, count: usize) -> Self {
        if let Some(item) = self.items.iter_mut().find(|it| it.id.as_ref() == tab_id) {
            item.badge = Some(count);
        }
        self
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(&SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }

    // Inspection methods for tests
    pub fn get_active_tab(&self) -> &SharedString {
        &self.active_tab
    }

    pub fn get_items(&self) -> &[TabItem] {
        &self.items
    }
}

impl Styled for Tabs {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Tabs {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = Theme::dark();
        let active_id = self.active_tab.clone();
        let on_select = self.on_select.clone();

        let tab_elements: Vec<_> = self
            .items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_active = item.id == active_id;
                let item_id = item.id.clone();
                let handler = on_select.clone();

                let mut tab_button = div()
                    .id(ElementId::named_usize("tab-item", idx))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .cursor_pointer()
                    .text_sm()
                    .rounded_t(theme.metrics.radius_md);

                if is_active {
                    tab_button = tab_button
                        .bg(colors::BG_CARD)
                        .border_b_2()
                        .border_color(colors::ACCENT_EMERALD)
                        .text_color(colors::TEXT_PRIMARY)
                        .font_weight(FontWeight::SEMIBOLD);
                } else {
                    tab_button = tab_button
                        .text_color(colors::TEXT_SECONDARY)
                        .hover(|s| {
                            s.bg(colors::BG_SURFACE_HOVER)
                                .text_color(colors::TEXT_PRIMARY)
                        });
                }

                if let Some(icon) = item.icon {
                    tab_button = tab_button.child(
                        div()
                            .text_sm()
                            .child(icon),
                    );
                }

                tab_button = tab_button.child(div().child(item.label));

                if let Some(badge) = item.badge {
                    if badge > 0 {
                        let badge_el = div()
                            .px_1p5()
                            .py_0p5()
                            .rounded_full()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .bg(if is_active {
                                colors::ACCENT_EMERALD
                            } else {
                                colors::BG_SURFACE_ACTIVE
                            })
                            .text_color(colors::TEXT_PRIMARY)
                            .child(badge.to_string());
                        tab_button = tab_button.child(badge_el);
                    }
                }

                if let Some(on_select) = handler {
                    let id_for_click = item_id.clone();
                    tab_button = tab_button.on_click(move |_: &ClickEvent, window, cx| {
                        on_select(&id_for_click, window, cx);
                    });
                }

                tab_button
            })
            .collect();

        self.base
            .flex()
            .items_center()
            .gap_1()
            .w_full()
            .bg(colors::BG_BASE)
            .border_b_1()
            .border_color(colors::BORDER_BASE)
            .px_4()
            .children(tab_elements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_tabs_enum() {
        assert_eq!(AppTab::ALL.len(), 5);
        assert_eq!(AppTab::Staging.id(), "staging");
        assert_eq!(AppTab::Compare.id(), "compare");
        assert_eq!(AppTab::Downloads.id(), "downloads");
        assert_eq!(AppTab::Utilities.id(), "utilities");
        assert_eq!(AppTab::Settings.id(), "settings");
    }

    #[test]
    fn test_tabs_builder_and_badges() {
        let tabs = Tabs::for_app(AppTab::Downloads).set_badge("downloads", 3);

        assert_eq!(tabs.get_active_tab().as_ref(), "downloads");
        assert_eq!(tabs.get_items().len(), 5);

        let dl_item = tabs
            .get_items()
            .iter()
            .find(|it| it.id.as_ref() == "downloads")
            .unwrap();
        assert_eq!(dl_item.badge, Some(3));
    }
}
