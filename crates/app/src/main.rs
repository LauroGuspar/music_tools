use gpui::{
    prelude::*, App, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;
use views::{
    EscapeAction, SelectCompare, SelectDownloads, SelectSettings, SelectStaging, SelectUtilities,
    WorkspaceView,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    log::info!("Starting Music Tools v2.0 (GPUI + Rust Native)...");

    let app = application();
    app.run(|cx: &mut App| {
        // Global keybindings
        cx.bind_keys([
            KeyBinding::new("ctrl-1", SelectStaging, None),
            KeyBinding::new("ctrl-2", SelectCompare, None),
            KeyBinding::new("ctrl-3", SelectDownloads, None),
            KeyBinding::new("ctrl-4", SelectUtilities, None),
            KeyBinding::new("ctrl-5", SelectSettings, None),
            KeyBinding::new("ctrl-,", SelectSettings, None),
            KeyBinding::new("escape", EscapeAction, None),
            // macOS friendly bindings
            KeyBinding::new("cmd-1", SelectStaging, None),
            KeyBinding::new("cmd-2", SelectCompare, None),
            KeyBinding::new("cmd-3", SelectDownloads, None),
            KeyBinding::new("cmd-4", SelectUtilities, None),
            KeyBinding::new("cmd-5", SelectSettings, None),
            KeyBinding::new("cmd-,", SelectSettings, None),
        ]);

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

        cx.open_window(window_options, |window, cx| {
            let focus_handle = cx.focus_handle();
            window.focus(&focus_handle, cx);
            cx.new(|_| WorkspaceView::new().with_focus_handle(focus_handle))
        })
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}
