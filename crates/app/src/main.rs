use gpui::{
    prelude::*, App, Bounds, KeyBinding, TitlebarOptions, WindowBounds, WindowOptions, px, size,
};
use gpui_platform::application;
use ui::{
    Backspace, BackspaceToStart, BackspaceWord, Copy, Cut, Delete, DeleteToEnd, DeleteWord,
    End, Home, INPUT_CONTEXT, Left, Paste, Right, SelectAll, SelectEnd, SelectHome, SelectLeft,
    SelectRight, SelectWordLeft, SelectWordRight, Space, WordLeft, WordRight,
};
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

        // Text input keybindings (Input context)
        let input_ctx = Some(INPUT_CONTEXT);
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, input_ctx),
            KeyBinding::new("ctrl-backspace", BackspaceWord, input_ctx),
            KeyBinding::new("delete", Delete, input_ctx),
            KeyBinding::new("ctrl-delete", DeleteWord, input_ctx),
            KeyBinding::new("ctrl-u", BackspaceToStart, input_ctx),
            KeyBinding::new("ctrl-k", DeleteToEnd, input_ctx),
            KeyBinding::new("left", Left, input_ctx),
            KeyBinding::new("right", Right, input_ctx),
            KeyBinding::new("ctrl-left", WordLeft, input_ctx),
            KeyBinding::new("ctrl-right", WordRight, input_ctx),
            KeyBinding::new("shift-left", SelectLeft, input_ctx),
            KeyBinding::new("shift-right", SelectRight, input_ctx),
            KeyBinding::new("shift-ctrl-left", SelectWordLeft, input_ctx),
            KeyBinding::new("shift-ctrl-right", SelectWordRight, input_ctx),
            KeyBinding::new("home", Home, input_ctx),
            KeyBinding::new("end", End, input_ctx),
            KeyBinding::new("shift-home", SelectHome, input_ctx),
            KeyBinding::new("shift-end", SelectEnd, input_ctx),
            KeyBinding::new("ctrl-a", SelectAll, input_ctx),
            KeyBinding::new("ctrl-v", Paste, input_ctx),
            KeyBinding::new("ctrl-c", Copy, input_ctx),
            KeyBinding::new("ctrl-x", Cut, input_ctx),
            KeyBinding::new("space", Space, input_ctx),
            // macOS friendly bindings for input
            KeyBinding::new("alt-left", WordLeft, input_ctx),
            KeyBinding::new("alt-right", WordRight, input_ctx),
            KeyBinding::new("shift-alt-left", SelectWordLeft, input_ctx),
            KeyBinding::new("shift-alt-right", SelectWordRight, input_ctx),
            KeyBinding::new("alt-backspace", BackspaceWord, input_ctx),
            KeyBinding::new("alt-delete", DeleteWord, input_ctx),
            KeyBinding::new("cmd-backspace", BackspaceToStart, input_ctx),
            KeyBinding::new("cmd-delete", DeleteToEnd, input_ctx),
            KeyBinding::new("cmd-left", Home, input_ctx),
            KeyBinding::new("cmd-right", End, input_ctx),
            KeyBinding::new("shift-cmd-left", SelectHome, input_ctx),
            KeyBinding::new("shift-cmd-right", SelectEnd, input_ctx),
            KeyBinding::new("cmd-a", SelectAll, input_ctx),
            KeyBinding::new("cmd-v", Paste, input_ctx),
            KeyBinding::new("cmd-c", Copy, input_ctx),
            KeyBinding::new("cmd-x", Cut, input_ctx),
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
            cx.new(|cx| WorkspaceView::new(cx).with_focus_handle(focus_handle))
        })
        .expect("failed to open window");

        cx.activate(true);
    });

    Ok(())
}
