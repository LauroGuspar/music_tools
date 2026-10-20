use std::ops::Range;
use std::rc::Rc;

use gpui::actions;
use gpui::prelude::*;
use gpui::{
    App, Bounds, ClickEvent, ClipboardItem, Context, CursorStyle, Div, ElementId,
    EntityInputHandler, FocusHandle, Focusable, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point,
    Render, RenderOnce, ShapedLine, SharedString, Stateful, StatefulInteractiveElement,
    StyleRefinement, Styled, UTF16Selection, Window, div, point, px,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::button::Button;
use crate::icon::Icon;
use crate::theme::{Theme, colors};

mod element;

actions!(
    input,
    [
        Backspace,
        BackspaceWord,
        BackspaceToStart,
        Delete,
        DeleteWord,
        DeleteToEnd,
        Left,
        Right,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectWordLeft,
        SelectWordRight,
        SelectAll,
        Home,
        End,
        SelectHome,
        SelectEnd,
        Paste,
        Dismiss,
        Cut,
        Copy,
        Space,
    ]
);

pub const INPUT_CONTEXT: &str = "Input";

const CARET: Pixels = px(2.);
const CARET_LINES: f32 = 1.25;

type Motion = fn(&str, usize) -> usize;

pub(crate) fn clamp_offset(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub(crate) fn clamp_range(text: &str, range: &Range<usize>) -> Range<usize> {
    let start = clamp_offset(text, range.start);
    start..clamp_offset(text, range.end).max(start)
}

fn previous_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_offset(text, offset);
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .rev()
        .find(|&index| index < offset)
        .unwrap_or(0)
}

fn next_boundary(text: &str, offset: usize) -> usize {
    let offset = clamp_offset(text, offset);
    text.grapheme_indices(true)
        .map(|(index, _)| index)
        .find(|&index| index > offset)
        .unwrap_or(text.len())
}

fn previous_word(text: &str, offset: usize) -> usize {
    let head = text[..clamp_offset(text, offset)].trim_end();
    head.char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0)
}

fn next_word(text: &str, offset: usize) -> usize {
    let offset = clamp_offset(text, offset);
    let tail = &text[offset..];
    let skipped = tail.len() - tail.trim_start().len();
    let word = &tail[skipped..];
    let end = word
        .char_indices()
        .find(|(_, character)| character.is_whitespace())
        .map(|(index, _)| index)
        .unwrap_or(word.len());
    offset + skipped + end
}

fn word_at(text: &str, offset: usize) -> Range<usize> {
    let offset = clamp_offset(text, offset);
    let after = text[offset..].chars().next();
    let before = text[..offset].chars().next_back();
    let Some(anchor) = after
        .filter(|character| !character.is_whitespace())
        .or(before)
        .or(after)
    else {
        return offset..offset;
    };

    let space = anchor.is_whitespace();
    let start = text[..offset]
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace() != space)
        .map(|(index, character)| index + character.len_utf8())
        .unwrap_or(0);
    let end = text[offset..]
        .char_indices()
        .find(|(_, character)| character.is_whitespace() != space)
        .map(|(index, _)| offset + index)
        .unwrap_or(text.len());
    start..end
}

fn offset_from_utf16(text: &str, offset: usize) -> usize {
    let mut utf8 = 0;
    let mut utf16 = 0;
    for character in text.chars() {
        if utf16 >= offset {
            break;
        }
        utf16 += character.len_utf16();
        utf8 += character.len_utf8();
    }
    utf8
}

fn offset_to_utf16(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    let mut utf8 = 0;
    for character in text.chars() {
        if utf8 >= offset {
            break;
        }
        utf8 += character.len_utf8();
        utf16 += character.len_utf16();
    }
    utf16
}

pub struct Input {
    focus_handle: FocusHandle,
    hint: SharedString,
    icon: Option<Icon>,
    clearable: bool,
    disabled: bool,
    content: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    selecting: bool,
    selected_word: Option<Range<usize>>,
    on_change: Option<Rc<dyn Fn(&str, &mut Context<Self>) + 'static>>,
}

impl Input {
    pub fn new(hint: impl Into<SharedString>, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            hint: hint.into(),
            icon: None,
            clearable: false,
            disabled: false,
            content: SharedString::default(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            selecting: false,
            selected_word: None,
            on_change: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn clearable(mut self) -> Self {
        self.clearable = true;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(
        mut self,
        handler: impl Fn(&str, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    pub fn text(&self) -> &str {
        &self.content
    }

    pub fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        self.selected_range = self.content.len()..self.content.len();
        self.selection_reversed = false;
        self.marked_range = None;
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.content, cx);
        }
        cx.notify();
    }

    pub fn clear_text(&mut self, cx: &mut Context<Self>) {
        self.set_text("", cx);
    }

    pub fn set_hint(&mut self, hint: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.hint = hint.into();
        cx.notify();
    }

    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    pub fn clear(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.set_text("", cx);
        self.focus(window, cx);
    }

    fn cursor(&self) -> usize {
        match self.selection_reversed {
            true => self.selected_range.start,
            false => self.selected_range.end,
        }
    }

    fn step(&mut self, motion: Motion, backward: bool, cx: &mut Context<Self>) {
        let offset = match (self.selected_range.is_empty(), backward) {
            (true, _) => motion(&self.content, self.cursor()),
            (false, true) => self.selected_range.start,
            (false, false) => self.selected_range.end,
        };
        self.move_to(offset, cx);
    }

    fn extend(&mut self, motion: Motion, cx: &mut Context<Self>) {
        self.select_to(motion(&self.content, self.cursor()), cx);
    }

    fn erase(&mut self, motion: Motion, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.extend(motion, cx);
        }
        self.replace_text_in_range(None, "", window, cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = previous_boundary(&self.content, self.cursor());
            if prev == self.cursor() {
                return;
            }
            self.move_to(prev, cx);
        } else {
            self.move_to(self.selected_range.start, cx);
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = next_boundary(&self.content, self.cursor());
            if next == self.cursor() {
                return;
            }
            self.move_to(next, cx);
        } else {
            self.move_to(self.selected_range.end, cx);
        }
    }

    fn word_left(&mut self, _: &WordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.step(previous_word, true, cx);
    }

    fn word_right(&mut self, _: &WordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.step(next_word, false, cx);
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.extend(previous_boundary, cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.extend(next_boundary, cx);
    }

    fn select_word_left(&mut self, _: &SelectWordLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.extend(previous_word, cx);
    }

    fn select_word_right(&mut self, _: &SelectWordRight, _: &mut Window, cx: &mut Context<Self>) {
        self.extend(next_word, cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx);
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(0, cx);
    }

    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.content.len(), cx);
    }

    fn space(&mut self, _: &Space, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, " ", window, cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let prev = previous_boundary(&self.content, self.cursor());
            if prev == self.cursor() {
                return;
            }
        }
        self.erase(previous_boundary, window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            let next = next_boundary(&self.content, self.cursor());
            if next == self.cursor() {
                return;
            }
        }
        self.erase(next_boundary, window, cx);
    }

    fn backspace_word(&mut self, _: &BackspaceWord, window: &mut Window, cx: &mut Context<Self>) {
        self.erase(previous_word, window, cx);
    }

    fn delete_word(&mut self, _: &DeleteWord, window: &mut Window, cx: &mut Context<Self>) {
        self.erase(next_word, window, cx);
    }

    fn backspace_to_start(
        &mut self,
        _: &BackspaceToStart,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.erase(|_, _| 0, window, cx);
    }

    fn delete_to_end(&mut self, _: &DeleteToEnd, window: &mut Window, cx: &mut Context<Self>) {
        self.erase(|text, _| text.len(), window, cx);
    }

    fn write_selection(&self, cx: &mut Context<Self>) -> bool {
        let range = clamp_range(&self.content, &self.selected_range);
        if range.is_empty() {
            return false;
        }
        cx.write_to_clipboard(ClipboardItem::new_string(self.content[range].to_owned()));
        true
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        self.write_selection(cx);
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if self.write_selection(cx) {
            self.replace_text_in_range(None, "", window, cx);
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        let Some(pasted) = cx.read_from_clipboard().and_then(|item| item.text()) else {
            return;
        };
        let single_line = pasted.replace('\n', " ");
        self.replace_text_in_range(None, &single_line, window, cx);
    }

    fn select_word(&mut self, offset: usize, cx: &mut Context<Self>) {
        let word = word_at(&self.content, offset);
        self.selected_word = Some(word.clone());
        self.move_to(word.start, cx);
        self.select_to(word.end, cx);
    }

    fn select_words(&mut self, anchor: Range<usize>, offset: usize, cx: &mut Context<Self>) {
        let hovered = word_at(&self.content, offset);
        self.selected_range = hovered.start.min(anchor.start)..hovered.end.max(anchor.end);
        self.selection_reversed = offset < anchor.start;
        cx.notify();
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        self.focus(window, cx);
        self.selecting = true;
        let offset = self.offset_for(event.position);
        match (event.click_count, event.modifiers.shift) {
            (0 | 1, true) => {
                self.selected_word = None;
                self.select_to(offset, cx);
            }
            (0 | 1, false) => {
                self.selected_word = None;
                self.move_to(offset, cx);
            }
            (2, _) => self.select_word(offset, cx),
            _ => {
                self.selected_word = None;
                self.select_all(&SelectAll, window, cx);
            }
        }
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        let offset = self.offset_for(event.position);
        match self.selected_word.clone() {
            Some(anchor) => self.select_words(anchor, offset, cx),
            None => self.select_to(offset, cx),
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
        self.selected_word = None;
    }

    fn offset_for(&self, position: Point<Pixels>) -> usize {
        let Some((bounds, line)) = self.last_bounds.as_ref().zip(self.last_layout.as_ref()) else {
            return 0;
        };
        if position.x < bounds.left() {
            return 0;
        }
        let offset = line.closest_index_for_x(position.x - bounds.left());
        clamp_offset(&self.content, offset)
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_offset(&self.content, offset);
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = clamp_offset(&self.content, offset);
        match self.selection_reversed {
            true => self.selected_range.start = offset,
            false => self.selected_range.end = offset,
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        let range = clamp_range(&self.content, range);
        offset_to_utf16(&self.content, range.start)..offset_to_utf16(&self.content, range.end)
    }

    fn range_from_utf16(&self, range: &Range<usize>) -> Range<usize> {
        let start = offset_from_utf16(&self.content, range.start);
        start..offset_from_utf16(&self.content, range.end).max(start)
    }

    fn edited_range(&self, range_utf16: Option<&Range<usize>>) -> Range<usize> {
        let range = range_utf16
            .map(|range| self.range_from_utf16(range))
            .or_else(|| self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());
        clamp_range(&self.content, &range)
    }

    fn splice(&mut self, range: &Range<usize>, text: &str) {
        self.content =
            (self.content[..range.start].to_owned() + text + &self.content[range.end..]).into();
    }
}

impl Focusable for Input {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for Input {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let range = clamp_range(&self.content, &self.range_from_utf16(&range_utf16));
        actual.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.edited_range(range_utf16.as_ref());
        self.splice(&range, text);

        let caret = range.start + text.len();
        self.selected_range = caret..caret;
        self.selection_reversed = false;
        self.marked_range.take();
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.content, cx);
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = self.edited_range(range_utf16.as_ref());
        self.splice(&range, text);

        let caret = range.start + text.len();
        self.marked_range = (!text.is_empty()).then_some(range.start..caret);
        let selected = selected
            .map(|utf16| {
                range.start + offset_from_utf16(text, utf16.start)
                    ..range.start + offset_from_utf16(text, utf16.end)
            })
            .unwrap_or(caret..caret);
        self.selected_range = clamp_range(&self.content, &selected);
        self.selection_reversed = false;
        if let Some(on_change) = self.on_change.clone() {
            on_change(&self.content, cx);
        }
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let line = self.last_layout.as_ref()?;
        let range = clamp_range(&self.content, &self.range_from_utf16(&range_utf16));
        Some(Bounds::from_corners(
            point(bounds.left() + line.x_for_index(range.start), bounds.top()),
            point(
                bounds.left() + line.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_for(point))
    }
}

impl Render for Input {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let disabled = self.disabled;
        let is_empty = self.content.is_empty();

        let mut container = div()
            .flex()
            .items_center()
            .gap_2()
            .min_w_0()
            .h(px(36.))
            .px_3()
            .rounded_md()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_BASE)
            .cursor(CursorStyle::IBeam)
            .key_context(INPUT_CONTEXT)
            .track_focus(&self.focus_handle)
            .when(!disabled, |this| {
                this.hover(|s| s.border_color(colors::TEXT_MUTED))
            })
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::backspace_word))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::delete_word))
            .on_action(cx.listener(Self::backspace_to_start))
            .on_action(cx.listener(Self::delete_to_end))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::word_left))
            .on_action(cx.listener(Self::word_right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_word_left))
            .on_action(cx.listener(Self::select_word_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::space))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up));

        if disabled {
            container = container.opacity(0.5);
        }

        if let Some(icon) = self.icon {
            container = container.child(
                div()
                    .flex_none()
                    .text_color(colors::TEXT_MUTED)
                    .child(icon.element().size_4()),
            );
        }

        container = container.child(
            div()
                .flex_1()
                .min_w_0()
                .child(element::Text {
                    input: cx.entity().clone(),
                }),
        );

        if self.clearable && !is_empty && !disabled {
            let clear_btn = Button::ghost("input-clear-btn", "")
                .icon(Icon::Close)
                .small()
                .on_click(cx.listener(|this, _, window, cx| {
                    this.clear(window, cx);
                }));
            container = container.child(clear_btn);
        }

        container
    }
}

// Declarative TextInput adapter for backward compatibility with existing tests and simple views
type ChangeHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type ClearHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type SubmitHandler = Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct TextInput {
    base: Stateful<Div>,
    value: SharedString,
    placeholder: SharedString,
    icon: Option<Icon>,
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

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
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

        if let Some(icon) = self.icon {
            container = container.child(
                div()
                    .flex_none()
                    .text_color(colors::TEXT_MUTED)
                    .child(icon.element().size_4()),
            );
        }

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

        if self.clearable && !is_empty && !disabled {
            let clear_btn = Button::ghost("input-clear-btn", "")
                .icon(Icon::Close)
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
            .icon(Icon::Search)
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
