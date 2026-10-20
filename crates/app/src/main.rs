use gpui::{
    App, Bounds, Context, FontWeight, Render, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, size,
};
use gpui_platform::application;
use ui::colors;

struct RootView {
    title: SharedString,
    subtitle: SharedString,
}

impl RootView {
    pub fn new(title: impl Into<SharedString>, subtitle: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .text_color(colors::TEXT_PRIMARY)
            // Titlebar / Header bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_3()
                    .bg(colors::BG_SURFACE)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .size_3()
                                    .rounded_full()
                                    .bg(colors::ACCENT_EMERALD),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Music Tools v2.0"),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(colors::BG_BASE)
                            .border_1()
                            .border_color(colors::BORDER_SUBTLE)
                            .text_xs()
                            .text_color(colors::ACCENT_EMERALD)
                            .child("Linux (Wayland / X11)"),
                    ),
            )
            // Main content area
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .p_8()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_4()
                            .p_8()
                            .rounded_xl()
                            .bg(colors::BG_SURFACE)
                            .border_1()
                            .border_color(colors::BORDER_SUBTLE)
                            .shadow_lg()
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_full()
                                    .bg(colors::BG_BASE)
                                    .border_1()
                                    .border_color(colors::ACCENT_EMERALD)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child("v2.0 NATIVE ARCHITECTURE"),
                            )
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child(self.title.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child(self.subtitle.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .gap_3()
                                    .mt_4()
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(colors::BG_BASE)
                                            .border_1()
                                            .border_color(colors::BORDER_SUBTLE)
                                            .text_xs()
                                            .child("⚡ GPUI GPU-Rendered"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(colors::BG_BASE)
                                            .border_1()
                                            .border_color(colors::BORDER_SUBTLE)
                                            .text_xs()
                                            .child("🗄️ SQLite Fast Cache"),
                                    )
                                    .child(
                                        div()
                                            .px_3()
                                            .py_2()
                                            .rounded_md()
                                            .bg(colors::BG_BASE)
                                            .border_1()
                                            .border_color(colors::BORDER_SUBTLE)
                                            .text_xs()
                                            .child("🎵 High-Speed Sync"),
                                    ),
                            ),
                    ),
            )
            // Footer
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_2()
                    .bg(colors::BG_SURFACE)
                    .border_t_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .text_xs()
                    .text_color(colors::TEXT_MUTED)
                    .child("Ready")
                    .child("Workspace: crates/app, crates/core, crates/storage, crates/ui"),
            )
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("Starting Music Tools v2.0 (GPUI + Rust)...");

    let app = application();
    app.run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(580.0)), cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Music Tools v2.0 - Rust & GPUI".into()),
                ..Default::default()
            }),
            app_id: Some("music-tools".into()),
            ..Default::default()
        };

        cx.open_window(window_options, |_, cx| {
            cx.new(|_| {
                RootView::new(
                    "Music Tools v2.0 - Rust & GPUI",
                    "Native High Performance Desktop Suite for Spotify & Audio Management",
                )
            })
        })
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}
