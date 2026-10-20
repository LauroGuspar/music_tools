use std::rc::Rc;

use gpui::prelude::*;
use gpui::{
    AnyElement, App, ClickEvent, Div, ElementId, FontWeight, InteractiveElement, IntoElement,
    MouseButton, Pixels, RenderOnce, SharedString, StyleRefinement, Styled, Window, div, px,
};

use crate::button::Button;
use crate::icon::Icon;
use crate::theme::{Theme, colors};

type DismissHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Modal {
    base: Div,
    id: ElementId,
    title: SharedString,
    description: Option<SharedString>,
    body: Vec<AnyElement>,
    actions: Vec<AnyElement>,
    close_button: bool,
    width: Pixels,
    on_dismiss: Option<DismissHandler>,
}

impl Modal {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            id: id.into(),
            title: title.into(),
            description: None,
            body: Vec::new(),
            actions: Vec::new(),
            close_button: true,
            width: px(520.),
            on_dismiss: None,
        }
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.body.push(child.into_any_element());
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    pub fn actions(mut self, actions: impl IntoIterator<Item = AnyElement>) -> Self {
        self.actions.extend(actions);
        self
    }

    pub fn close_button(mut self, show: bool) -> Self {
        self.close_button = show;
        self
    }

    pub fn on_dismiss(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(handler));
        self
    }

    // Getters for tests
    pub fn get_title(&self) -> &SharedString {
        &self.title
    }

    pub fn get_description(&self) -> Option<&SharedString> {
        self.description.as_ref()
    }

    pub fn has_close_button(&self) -> bool {
        self.close_button
    }

    pub fn get_body_count(&self) -> usize {
        self.body.len()
    }

    pub fn get_actions_count(&self) -> usize {
        self.actions.len()
    }
}

impl Styled for Modal {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Modal {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let theme = Theme::dark();
        let on_dismiss_backdrop = self.on_dismiss.clone();
        let on_dismiss_close = self.on_dismiss.clone();

        // Modal dialog panel
        let mut panel = div()
            .id(self.id)
            .w(self.width)
            .max_w_full()
            .flex()
            .flex_col()
            .rounded(theme.metrics.radius_xl)
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_BASE)
            .shadow_xl()
            .overflow_hidden()
            // Stop propagation on panel clicks so clicking inside dialog does not dismiss it
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());

        // Header
        let mut header = div()
            .flex()
            .items_center()
            .justify_between()
            .px_6()
            .py_4()
            .border_b_1()
            .border_color(colors::BORDER_BASE);

        let mut title_column = div().flex().flex_col().gap_0p5();
        title_column = title_column.child(
            div()
                .text_base()
                .font_weight(FontWeight::BOLD)
                .text_color(colors::TEXT_PRIMARY)
                .child(self.title),
        );

        if let Some(desc) = self.description {
            title_column = title_column.child(
                div()
                    .text_xs()
                    .text_color(colors::TEXT_SECONDARY)
                    .child(desc),
            );
        }

        header = header.child(title_column);

        if self.close_button {
            let close_btn = Button::ghost("modal-close-btn", "")
                .icon(Icon::Close)
                .small()
                .on_click(move |_ev: &ClickEvent, window, cx| {
                    if let Some(ref handler) = on_dismiss_close {
                        handler(window, cx);
                    }
                });
            header = header.child(close_btn);
        }

        panel = panel.child(header);

        // Body
        let body_content = div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .children(self.body);

        panel = panel.child(body_content);

        // Actions footer
        if !self.actions.is_empty() {
            let footer = div()
                .flex()
                .items_center()
                .justify_end()
                .gap_3()
                .px_6()
                .py_3()
                .bg(colors::BG_BASE)
                .border_t_1()
                .border_color(colors::BORDER_BASE)
                .children(self.actions);

            panel = panel.child(footer);
        }

        // Fullscreen backdrop overlay
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors::hex_rgba(0x000000B3)) // 70% black overlay
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                if let Some(ref handler) = on_dismiss_backdrop {
                    handler(window, cx);
                }
            })
            .child(panel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_builder_and_properties() {
        let modal = Modal::new("delete-dialog", "¿Eliminar archivo?")
            .description("Esta acción no se puede deshacer.")
            .child(div().child("El archivo song.mp3 será eliminado permanentemente."))
            .action(Button::secondary("cancel-btn", "Cancelar"))
            .action(Button::danger("confirm-btn", "Eliminar"));

        assert_eq!(modal.get_title().as_ref(), "¿Eliminar archivo?");
        assert_eq!(
            modal.get_description().map(|s| s.as_ref()),
            Some("Esta acción no se puede deshacer.")
        );
        assert!(modal.has_close_button());
        assert_eq!(modal.get_body_count(), 1);
        assert_eq!(modal.get_actions_count(), 2);
    }
}
