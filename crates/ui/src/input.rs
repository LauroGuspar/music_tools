use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    App, ClickEvent, CursorStyle, Div, ElementId, FocusHandle, InteractiveElement,
    IntoElement, RenderOnce, SharedString, Stateful, StatefulInteractiveElement,
    StyleRefinement, Styled, Window, div,
};

use crate::button::Button;
use crate::theme::{Theme, colors};

type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type ClearHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type SubmitHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct TextInput {
    base: Stateful<Div>,
    value: SharedString,
    placeholder: SharedString,
    icon: Option<SharedString>,
    clearable: bool,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
    on_clear: Option<ClearHandler>,
    on_submit: Option<SubmitHandler>,
}

impl TextInput {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            base: div().id(id),
            value: SharedString::default(),
            placeholder: SharedString::default(),
            icon: None,
            clearable: false,
            disabled: false,
            focus_handle: None,
            on_change: None,
            on_clear: None,
            on_submit: None,
        }
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus_handle = Some(handle.clone());
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_clear = Some(Rc::new(handler));
        self
    }

    pub fn on_submit(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    // Getters for inspection and testing
    pub fn get_value(&self) -> &SharedString {
        &self.value
    }

    pub fn get_placeholder(&self) -> &SharedString {
        &self.placeholder
    }

    pub fn is_clearable(&self) -> bool {
        self.clearable
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

impl Styled for TextInput {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for TextInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for TextInput {}

impl RenderOnce for TextInput {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = Theme::dark();
        let disabled = self.disabled;
        let is_empty = self.value.is_empty();

        let mut container = self.base
            .flex()
            .items_center()
            .gap_2()
            .min_w_0()
            .h(theme.metrics.row_height)
            .px_3()
            .rounded(theme.metrics.radius_md)
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_BASE)
            .cursor(CursorStyle::IBeam)
            .when(!disabled, |this| {
                this.hover(|s| s.border_color(colors::TEXT_MUTED))
            });

        if let Some(focus_handle) = &self.focus_handle {
            container = container.track_focus(focus_handle);
        }

        if disabled {
            container = container.opacity(0.5);
        }

        // Leading icon (e.g. search symbol "🔍" or folder "📁")
        if let Some(icon) = self.icon {
            container = container.child(
                div()
                    .flex_none()
                    .text_sm()
                    .text_color(colors::TEXT_MUTED)
                    .child(icon),
            );
        }

        // Content / Placeholder text area
        let text_element = if is_empty {
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(colors::TEXT_MUTED)
                .truncate()
                .child(self.placeholder.clone())
        } else {
            div()
                .flex_1()
                .min_w_0()
                .text_sm()
                .text_color(colors::TEXT_PRIMARY)
                .truncate()
                .child(self.value.clone())
        };

        container = container.child(text_element);

        // Clear button when clearable and has text
        if self.clearable && !is_empty && !disabled {
            let clear_btn = Button::ghost("input-clear-btn", "✕")
                .small()
                .on_click(move |ev, window, cx| {
                    if let Some(ref on_clear) = self.on_clear {
                        on_clear(ev, window, cx);
                    }
                });
            container = container.child(clear_btn);
        }

        container
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_creation_and_properties() {
        let input = TextInput::new("search-input")
            .placeholder("Buscar canciones o artistas...")
            .value("Bohemian Rhapsody")
            .icon("🔍")
            .clearable(true);

        assert_eq!(input.get_value().as_ref(), "Bohemian Rhapsody");
        assert_eq!(
            input.get_placeholder().as_ref(),
            "Buscar canciones o artistas..."
        );
        assert!(input.is_clearable());
        assert!(!input.is_disabled());

        let disabled_input = TextInput::new("url-input")
            .placeholder("URL de Spotify...")
            .disabled(true);
        assert!(disabled_input.is_disabled());
    }
}
