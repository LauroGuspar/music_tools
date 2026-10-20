use gpui::prelude::*;
use gpui::{
    App, Bounds, Element, ElementInputHandler, Entity, GlobalElementId,
    InspectorElementId, LayoutId, PaintQuad, Pixels, ShapedLine, Style, TextRun,
    UnderlineStyle, Window, fill, point, px, relative, size,
};

use crate::theme::colors;
use super::{CARET, CARET_LINES, Input, clamp_offset, clamp_range};

pub(crate) struct Text {
    pub(crate) input: Entity<Input>,
}

pub(crate) struct Painted {
    line: Option<ShapedLine>,
    caret: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for Text {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for Text {
    type RequestLayoutState = ();
    type PrepaintState = Painted;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let empty = input.content.is_empty();
        let style = window.text_style();

        let text = if empty {
            input.hint.clone()
        } else {
            input.content.clone()
        };

        let color = if empty {
            colors::TEXT_MUTED.into()
        } else {
            style.color
        };

        let selected = if empty {
            0..0
        } else {
            clamp_range(&text, &input.selected_range)
        };

        let cursor = if empty {
            0
        } else {
            clamp_offset(&text, input.cursor())
        };

        let marked = if empty {
            None
        } else {
            input.marked_range.as_ref().map(|range| clamp_range(&text, range))
        };

        let run = TextRun {
            len: text.len(),
            font: style.font(),
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = match marked {
            Some(marked) => [
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: text.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect(),
            None => vec![run],
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(text, font_size, &runs, None);

        let (selection, caret) = if selected.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(
                            bounds.left() + line.x_for_index(cursor),
                            bounds.top() + (bounds.size.height - font_size * CARET_LINES) / 2.,
                        ),
                        size(CARET, font_size * CARET_LINES),
                    ),
                    colors::ACCENT_EMERALD,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected.end),
                            bounds.bottom(),
                        ),
                    ),
                    colors::ACCENT_EMERALD.opacity(0.3),
                )),
                None,
            )
        };

        Painted {
            line: Some(line),
            caret,
            selection,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        painted: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        let focused = focus.is_focused(window);
        if focused {
            if let Some(selection) = painted.selection.take() {
                window.paint_quad(selection);
            }
        }

        if let Some(line) = painted.line.take() {
            line.paint(
                bounds.origin,
                window.line_height(),
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            )
            .ok();

            self.input.update(cx, |input, _| {
                input.last_layout = (!input.content.is_empty()).then_some(line);
                input.last_bounds = Some(bounds);
            });
        }

        if focused {
            if let Some(caret) = painted.caret.take() {
                window.paint_quad(caret);
            }
        }
    }
}
