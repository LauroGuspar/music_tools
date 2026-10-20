use gpui::{
    prelude::*, App, Bounds, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;
use views::WorkspaceView;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("Starting Music Tools v2.0 (GPUI + Rust Native)...");

    let app = application();
    app.run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1080.0), px(720.0)), cx);

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
            cx.new(|_| WorkspaceView::new())
        })
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}
