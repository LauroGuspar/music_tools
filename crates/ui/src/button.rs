use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    RenderOnce, SharedString, Stateful, StatefulInteractiveElement, StyleRefinement,
    Styled, Window, div,
};

use crate::icon::Icon;
use crate::theme::{Theme, colors};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Button {
    base: Stateful<Div>,
    label: Option<SharedString>,
    icon: Option<Icon>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    selected: bool,
    on_click: Option<ClickHandler>,
}

impl Button {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            label: None,
            icon: None,
            variant: ButtonVariant::Primary,
            size: ButtonSize::Medium,
            disabled: false,
            selected: false,
            on_click: None,
        }
    }

    pub fn primary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id).variant(ButtonVariant::Primary).label(label)
    }

    pub fn secondary(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id).variant(ButtonVariant::Secondary).label(label)
    }

    pub fn ghost(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id).variant(ButtonVariant::Ghost).label(label)
    }

    pub fn danger(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self::new(id).variant(ButtonVariant::Danger).label(label)
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn small(mut self) -> Self {
        self.size = ButtonSize::Small;
        self
    }

    pub fn large(mut self) -> Self {
        self.size = ButtonSize::Large;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    // Inspection methods for tests
    pub fn get_variant(&self) -> ButtonVariant {
        self.variant
    }

    pub fn get_size(&self) -> ButtonSize {
        self.size
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn get_label(&self) -> Option<&SharedString> {
        self.label.as_ref()
    }
}

impl Styled for Button {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Button {}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = Theme::dark();
        let disabled = self.disabled;

        let mut el = self.base
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer();

        // Size styling
        el = match self.size {
            ButtonSize::Small => el
                .px_2()
                .py_1()
                .text_xs()
                .gap_1()
                .rounded(theme.metrics.radius_sm),
            ButtonSize::Medium => el
                .px_3()
                .py_1p5()
                .text_sm()
                .gap_2()
                .rounded(theme.metrics.radius_md),
            ButtonSize::Large => el
                .px_4()
                .py_2()
                .text_base()
                .gap_2()
                .rounded(theme.metrics.radius_lg),
        };

        // Variant styling
        el = match self.variant {
            ButtonVariant::Primary => {
                let bg = if disabled {
                    colors::hex_rgb(0x1a4d38)
                } else {
                    colors::ACCENT_EMERALD
                };
                el.bg(bg)
                    .text_color(colors::TEXT_PRIMARY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(!disabled, |this| {
                        this.hover(|s| s.bg(colors::ACCENT_EMERALD_HOVER))
                            .active(|s| s.bg(colors::ACCENT_EMERALD_DARK))
                    })
            }
            ButtonVariant::Secondary => {
                let bg = if disabled {
                    colors::BG_SURFACE
                } else {
                    colors::BG_CARD
                };
                el.bg(bg)
                    .border_1()
                    .border_color(colors::BORDER_BASE)
                    .text_color(colors::TEXT_PRIMARY)
                    .when(!disabled, |this| {
                        this.hover(|s| s.bg(colors::BG_SURFACE_ACTIVE))
                            .active(|s| s.bg(colors::hex_rgb(0x383838)))
                    })
            }
            ButtonVariant::Ghost => {
                el.bg(colors::hex_rgba(0x00000000))
                    .text_color(if self.selected {
                        colors::TEXT_PRIMARY
                    } else {
                        colors::TEXT_SECONDARY
                    })
                    .when(!disabled, |this| {
                        this.hover(|s| s.bg(colors::BG_CARD).text_color(colors::TEXT_PRIMARY))
                            .active(|s| s.bg(colors::BG_SURFACE_ACTIVE))
                    })
            }
            ButtonVariant::Danger => {
                let bg = if disabled {
                    colors::hex_rgb(0x4a1d1d)
                } else {
                    colors::STATUS_DANGER
                };
                el.bg(bg)
                    .text_color(colors::TEXT_PRIMARY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(!disabled, |this| {
                        this.hover(|s| s.bg(colors::hex_rgb(0xDC2626)))
                            .active(|s| s.bg(colors::hex_rgb(0xB91C1C)))
                    })
            }
        };

        if self.selected {
            el = el.border_1().border_color(colors::ACCENT_EMERALD);
        }

        if disabled {
            el = el.opacity(0.5);
        } else if let Some(on_click) = self.on_click {
            el = el
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .on_click(move |ev, window, cx| {
                    on_click(ev, window, cx);
                });
        }

        if let Some(icon) = self.icon {
            el = el.child(icon.element().size_4());
        }

        if let Some(label) = self.label {
            if !label.is_empty() {
                el = el.child(div().child(label));
            }
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_variants_and_sizes() {
        let btn_primary = Button::primary("btn-1", "Guardar");
        assert_eq!(btn_primary.get_variant(), ButtonVariant::Primary);
        assert_eq!(btn_primary.get_size(), ButtonSize::Medium);
        assert_eq!(btn_primary.get_label().map(|s| s.as_ref()), Some("Guardar"));
        assert!(!btn_primary.is_disabled());

        let btn_sec = Button::secondary("btn-2", "Cancelar").small().disabled(true);
        assert_eq!(btn_sec.get_variant(), ButtonVariant::Secondary);
        assert_eq!(btn_sec.get_size(), ButtonSize::Small);
        assert!(btn_sec.is_disabled());

        let btn_ghost = Button::ghost("btn-3", "Ver más").large().selected(true);
        assert_eq!(btn_ghost.get_variant(), ButtonVariant::Ghost);
        assert_eq!(btn_ghost.get_size(), ButtonSize::Large);
        assert!(btn_ghost.is_selected());

        let btn_danger = Button::danger("btn-4", "Eliminar");
        assert_eq!(btn_danger.get_variant(), ButtonVariant::Danger);
    }
}
