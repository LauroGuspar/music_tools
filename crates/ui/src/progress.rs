use gpui::prelude::*;
use gpui::{
    App, Div, ElementId, FontWeight, IntoElement, Pixels, RenderOnce, Rgba, SharedString,
    Stateful, StyleRefinement, Styled, Window, div, px, relative,
};

use crate::icon::Icon;
use crate::theme::colors;

#[derive(IntoElement)]
pub struct ProgressBar {
    base: Stateful<Div>,
    percentage: f32,
    height: Pixels,
    color: Option<Rgba>,
    bg_color: Option<Rgba>,
    label: Option<SharedString>,
    detail: Option<SharedString>,
    show_percentage_text: bool,
}

impl ProgressBar {
    #[track_caller]
    pub fn new(id: impl Into<ElementId>, percentage: f32) -> Self {
        Self {
            base: div().id(id),
            percentage: percentage.clamp(0.0, 100.0),
            height: px(8.),
            color: None,
            bg_color: None,
            label: None,
            detail: None,
            show_percentage_text: true,
        }
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = Some(color);
        self
    }

    pub fn bg_color(mut self, bg_color: Rgba) -> Self {
        self.bg_color = Some(bg_color);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage_text = show;
        self
    }

    // Getters for testing
    pub fn get_percentage(&self) -> f32 {
        self.percentage
    }

    pub fn get_label(&self) -> Option<&SharedString> {
        self.label.as_ref()
    }

    pub fn get_detail(&self) -> Option<&SharedString> {
        self.detail.as_ref()
    }
}

impl Styled for ProgressBar {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let fill_color = self.color.unwrap_or(colors::ACCENT_EMERALD);
        let track_color = self.bg_color.unwrap_or(colors::BG_CARD);
        let pct_ratio = (self.percentage / 100.0).clamp(0.0, 1.0);

        let has_text_header = self.label.is_some() || self.detail.is_some() || self.show_percentage_text;

        let mut root = self.base.flex().flex_col().gap_1().w_full();

        if has_text_header {
            let mut header = div().flex().items_center().justify_between().text_xs();

            if let Some(label) = self.label {
                header = header.child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::TEXT_PRIMARY)
                        .child(label),
                );
            } else {
                header = header.child(div());
            }

            let mut info_right = div().flex().items_center().gap_2().text_color(colors::TEXT_MUTED);

            if let Some(detail) = self.detail {
                info_right = info_right.child(div().child(detail));
            }

            if self.show_percentage_text {
                let pct_str = format!("{:.1}%", self.percentage);
                info_right = info_right.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT_SECONDARY)
                        .child(pct_str),
                );
            }

            header = header.child(info_right);
            root = root.child(header);
        }

        // Track and fill
        let bar_container = div()
            .relative()
            .w_full()
            .h(self.height)
            .rounded_full()
            .bg(track_color)
            .overflow_hidden()
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .bottom_0()
                    .w(relative(pct_ratio))
                    .rounded_full()
                    .bg(fill_color),
            );

        root.child(bar_container)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Success,
    Warning,
    Info,
    Danger,
    Spotify,
}

#[derive(IntoElement)]
pub struct Badge {
    base: Div,
    label: SharedString,
    variant: BadgeVariant,
    icon: Option<Icon>,
}

impl Badge {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            base: div(),
            label: label.into(),
            variant: BadgeVariant::Neutral,
            icon: None,
        }
    }

    pub fn exact() -> Self {
        Self::new("Exacto").variant(BadgeVariant::Success).icon(Icon::Check)
    }

    pub fn probable() -> Self {
        Self::new("Probable").variant(BadgeVariant::Warning).icon(Icon::Compare)
    }

    pub fn completed() -> Self {
        Self::new("Completado").variant(BadgeVariant::Success).icon(Icon::Check)
    }

    pub fn downloading() -> Self {
        Self::new("Descargando").variant(BadgeVariant::Info).icon(Icon::Download)
    }

    pub fn failed() -> Self {
        Self::new("Fallido").variant(BadgeVariant::Danger).icon(Icon::Close)
    }

    pub fn queued() -> Self {
        Self::new("En Cola").variant(BadgeVariant::Neutral).icon(Icon::Refresh)
    }

    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    // Getters for testing
    pub fn get_label(&self) -> &SharedString {
        &self.label
    }

    pub fn get_variant(&self) -> BadgeVariant {
        self.variant
    }

    pub fn get_icon(&self) -> Option<Icon> {
        self.icon
    }
}

impl Styled for Badge {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let (bg, border, text) = match self.variant {
            BadgeVariant::Success => (
                colors::hex_rgba(0x10B98126), // 15% opacity emerald
                colors::hex_rgb(0x10B981),
                colors::hex_rgb(0x34D399),
            ),
            BadgeVariant::Warning => (
                colors::hex_rgba(0xF59E0B26), // 15% opacity amber
                colors::hex_rgb(0xF59E0B),
                colors::hex_rgb(0xFBBF24),
            ),
            BadgeVariant::Info => (
                colors::hex_rgba(0x3B82F626), // 15% opacity blue
                colors::hex_rgb(0x3B82F6),
                colors::hex_rgb(0x60A5FA),
            ),
            BadgeVariant::Danger => (
                colors::hex_rgba(0xEF444426), // 15% opacity red
                colors::hex_rgb(0xEF4444),
                colors::hex_rgb(0xF87171),
            ),
            BadgeVariant::Spotify => (
                colors::hex_rgba(0x1DB95426), // 15% opacity spotify green
                colors::hex_rgb(0x1DB954),
                colors::hex_rgb(0x1ED760),
            ),
            BadgeVariant::Neutral => (
                colors::BG_CARD,
                colors::BORDER_BASE,
                colors::TEXT_SECONDARY,
            ),
        };

        let mut el = self.base
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_0p5()
            .rounded_full()
            .bg(bg)
            .border_1()
            .border_color(border)
            .text_xs()
            .font_weight(FontWeight::MEDIUM)
            .text_color(text);

        if let Some(icon) = self.icon {
            el = el.child(icon.element().size_3());
        }

        el.child(div().child(self.label))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_percentage_clamping() {
        let p1 = ProgressBar::new("pb-1", 45.5);
        assert_eq!(p1.get_percentage(), 45.5);

        let p2 = ProgressBar::new("pb-2", 150.0);
        assert_eq!(p2.get_percentage(), 100.0);

        let p3 = ProgressBar::new("pb-3", -20.0);
        assert_eq!(p3.get_percentage(), 0.0);
    }

    #[test]
    fn test_progress_bar_label_and_detail() {
        let pb = ProgressBar::new("pb-download", 72.3)
            .label("Descargando playlist")
            .detail("4.2 MB/s - 00:32");

        assert_eq!(pb.get_label().map(|s| s.as_ref()), Some("Descargando playlist"));
        assert_eq!(pb.get_detail().map(|s| s.as_ref()), Some("4.2 MB/s - 00:32"));
    }

    #[test]
    fn test_badge_variants() {
        let exact = Badge::exact();
        assert_eq!(exact.get_label().as_ref(), "Exacto");
        assert_eq!(exact.get_variant(), BadgeVariant::Success);

        let probable = Badge::probable();
        assert_eq!(probable.get_variant(), BadgeVariant::Warning);

        let dl = Badge::downloading();
        assert_eq!(dl.get_variant(), BadgeVariant::Info);

        let failed = Badge::failed();
        assert_eq!(failed.get_variant(), BadgeVariant::Danger);

        let custom = Badge::new("Personalizado").variant(BadgeVariant::Spotify);
        assert_eq!(custom.get_variant(), BadgeVariant::Spotify);
    }
}
