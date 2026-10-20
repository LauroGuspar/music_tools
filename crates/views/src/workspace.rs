use gpui::prelude::*;
use gpui::{
    Context, FocusHandle, FontWeight, IntoElement, Render, SharedString,
    Window, div,
};

use storage::CacheStorage;
use ui::tabs::{AppTab, TabItem, Tabs};
use ui::theme::colors;

use crate::actions::{
    EscapeAction, SelectCompare, SelectDownloads, SelectSettings, SelectStaging, SelectUtilities,
};
use crate::compare::CompareView;
use crate::downloads::DownloadsView;
use crate::settings::SettingsView;
use crate::staging::StagingView;
use crate::utilities::UtilitiesView;

pub struct WorkspaceView {
    pub active_tab: AppTab,
    pub staging: StagingView,
    pub compare: CompareView,
    pub downloads: DownloadsView,
    pub utilities: UtilitiesView,
    pub settings: SettingsView,
    pub storage: Option<CacheStorage>,
    pub focus_handle: Option<FocusHandle>,
}

impl Default for WorkspaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceView {
    pub fn new() -> Self {
        let storage = CacheStorage::standard().ok();
        let mut settings = SettingsView::new();

        if let Some(ref st) = storage {
            let _ = settings.load_from_storage(st);
        }

        let mut downloads = DownloadsView::new();
        downloads.set_download_dir(&settings.download_dir);

        Self {
            active_tab: AppTab::Staging,
            staging: StagingView::new(),
            compare: CompareView::new(),
            downloads,
            utilities: UtilitiesView::new(),
            settings,
            storage,
            focus_handle: None,
        }
    }

    pub fn with_focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn switch_tab(&mut self, tab: AppTab) {
        self.active_tab = tab;
    }

    pub fn switch_tab_by_id(&mut self, id: &str) {
        for tab in AppTab::ALL {
            if tab.id() == id {
                self.switch_tab(tab);
                break;
            }
        }
    }

    pub fn select_staging(&mut self) {
        self.switch_tab(AppTab::Staging);
    }

    pub fn select_compare(&mut self) {
        self.switch_tab(AppTab::Compare);
    }

    pub fn select_downloads(&mut self) {
        self.switch_tab(AppTab::Downloads);
    }

    pub fn select_utilities(&mut self) {
        self.switch_tab(AppTab::Utilities);
    }

    pub fn select_settings(&mut self) {
        self.switch_tab(AppTab::Settings);
    }

    /// Handles Escape key: closes modals first, or cancels/clears active table selection
    pub fn handle_escape(&mut self) -> bool {
        // 1. Close any modal open in utilities (e.g. pending deletion confirmation)
        if self.utilities.pending_delete_path.is_some() {
            self.utilities.cancel_delete();
            return true;
        }

        // 2. Clear selections in active view
        match self.active_tab {
            AppTab::Staging if !self.staging.selection.is_empty() => {
                self.staging.selection.clear();
                return true;
            }
            AppTab::Downloads if !self.downloads.selection.is_empty() => {
                self.downloads.selection.clear();
                return true;
            }
            _ => {}
        }

        // 3. Clear any remaining selections in inactive views
        if !self.staging.selection.is_empty() {
            self.staging.selection.clear();
            return true;
        }
        if !self.downloads.selection.is_empty() {
            self.downloads.selection.clear();
            return true;
        }

        false
    }

    // Action listener handlers
    fn on_select_staging(&mut self, _: &SelectStaging, _: &mut Window, cx: &mut Context<Self>) {
        self.select_staging();
        cx.notify();
    }

    fn on_select_compare(&mut self, _: &SelectCompare, _: &mut Window, cx: &mut Context<Self>) {
        self.select_compare();
        cx.notify();
    }

    fn on_select_downloads(&mut self, _: &SelectDownloads, _: &mut Window, cx: &mut Context<Self>) {
        self.select_downloads();
        cx.notify();
    }

    fn on_select_utilities(&mut self, _: &SelectUtilities, _: &mut Window, cx: &mut Context<Self>) {
        self.select_utilities();
        cx.notify();
    }

    fn on_select_settings(&mut self, _: &SelectSettings, _: &mut Window, cx: &mut Context<Self>) {
        self.select_settings();
        cx.notify();
    }

    fn on_escape(&mut self, _: &EscapeAction, _: &mut Window, cx: &mut Context<Self>) {
        if self.handle_escape() {
            cx.notify();
        }
    }

    /// Transfer tracks from staging to download queue
    pub fn send_staging_to_downloads(&mut self) -> usize {
        let items = self.staging.to_download_items();
        let count = items.len();
        self.downloads.add_items(items);
        count
    }

    /// Transfer missing tracks from compare results to download queue
    pub fn send_compare_missing_to_downloads(&mut self) -> usize {
        let items = self.compare.export_missing_to_downloads();
        let count = items.len();
        self.downloads.add_items(items);
        count
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.active_tab;
        let active_downloads_count = self.downloads.queued_count() + self.downloads.downloading_count();
        let corrupt_count = self.utilities.corrupt_count();

        let focus_handle = self
            .focus_handle
            .get_or_insert_with(|| cx.focus_handle())
            .clone();

        // 1. Build Navigation Tabs with dynamic badges
        let items: Vec<TabItem> = AppTab::ALL
            .iter()
            .copied()
            .map(|tab| {
                let mut it = TabItem::from_app_tab(tab);
                match tab {
                    AppTab::Downloads => {
                        if active_downloads_count > 0 {
                            it = it.badge(active_downloads_count);
                        }
                    }
                    AppTab::Utilities => {
                        if corrupt_count > 0 {
                            it = it.badge(corrupt_count);
                        }
                    }
                    _ => {}
                }
                it
            })
            .collect();

        let tabs = Tabs::new("workspace-tabs")
            .items(items)
            .active(active_tab.id())
            .on_select(cx.listener(|this, selected_id: &SharedString, _, _| {
                this.switch_tab_by_id(selected_id.as_ref());
            }));

        // 2. Active Tab View Content
        let view_content = match active_tab {
            AppTab::Staging => self.staging.render_view(),
            AppTab::Compare => self.compare.render_view(),
            AppTab::Downloads => self.downloads.render_view(),
            AppTab::Utilities => self.utilities.render_view(),
            AppTab::Settings => self.settings.render_view(),
        };

        // 3. Assemble Full Workspace Shell with Global Key Actions
        div()
            .id("workspace-shell")
            .key_context("workspace")
            .track_focus(&focus_handle)
            .on_action(cx.listener(Self::on_select_staging))
            .on_action(cx.listener(Self::on_select_compare))
            .on_action(cx.listener(Self::on_select_downloads))
            .on_action(cx.listener(Self::on_select_utilities))
            .on_action(cx.listener(Self::on_select_settings))
            .on_action(cx.listener(Self::on_escape))
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .text_color(colors::TEXT_PRIMARY)
            // Top App Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_6()
                    .py_2()
                    .bg(colors::BG_SURFACE)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    // Left Branding
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
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Music Tools v2.0"),
                            ),
                    )
                    // Center Navigation Tabs
                    .child(tabs)
                    // Right Platform Pill
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
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
                    ),
            )
            // Main Central Viewport
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(view_content),
            )
            // Bottom Status / Footer Bar
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .size_2()
                                    .rounded_full()
                                    .bg(colors::ACCENT_EMERALD),
                            )
                            .child(format!("Vista Activa: {}", active_tab.label())),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .child(format!("Pistas en Mesa: {}", self.staging.track_count()))
                            .child(format!("Descargas: {}", self.downloads.total_count()))
                            .child("GPUI Native Engine"),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::Track;

    #[test]
    fn test_workspace_view_initial_state() {
        let ws = WorkspaceView::new();
        assert_eq!(ws.active_tab, AppTab::Staging);
        assert_eq!(ws.staging.track_count(), 0);
        assert_eq!(ws.downloads.total_count(), 0);
    }

    #[test]
    fn test_workspace_switch_tabs() {
        let mut ws = WorkspaceView::new();
        ws.switch_tab(AppTab::Downloads);
        assert_eq!(ws.active_tab, AppTab::Downloads);

        ws.switch_tab_by_id("utilities");
        assert_eq!(ws.active_tab, AppTab::Utilities);

        ws.switch_tab_by_id("settings");
        assert_eq!(ws.active_tab, AppTab::Settings);
    }

    #[test]
    fn test_workspace_staging_to_downloads_integration() {
        let mut ws = WorkspaceView::new();
        ws.staging.tracks.push(Track::new("Track 1", vec!["Artist 1".into()], 200_000));
        ws.staging.tracks.push(Track::new("Track 2", vec!["Artist 2".into()], 180_000));

        let sent = ws.send_staging_to_downloads();
        assert_eq!(sent, 2);
        assert_eq!(ws.downloads.total_count(), 2);
        assert_eq!(ws.downloads.items[0].title, "Track 1");
    }

    #[test]
    fn test_workspace_compare_to_downloads_integration() {
        let mut ws = WorkspaceView::new();
        let source = vec![Track::new("Missing Song", vec!["Artist".into()], 210_000)];
        let target = vec![];
        ws.compare.source_tracks = source;
        ws.compare.target_tracks = target;
        ws.compare.compare();

        let sent = ws.send_compare_missing_to_downloads();
        assert_eq!(sent, 1);
        assert_eq!(ws.downloads.total_count(), 1);
        assert_eq!(ws.downloads.items[0].title, "Missing Song");
    }

    #[test]
    fn test_workspace_select_shortcuts_methods() {
        let mut ws = WorkspaceView::new();

        ws.select_compare();
        assert_eq!(ws.active_tab, AppTab::Compare);

        ws.select_downloads();
        assert_eq!(ws.active_tab, AppTab::Downloads);

        ws.select_utilities();
        assert_eq!(ws.active_tab, AppTab::Utilities);

        ws.select_settings();
        assert_eq!(ws.active_tab, AppTab::Settings);

        ws.select_staging();
        assert_eq!(ws.active_tab, AppTab::Staging);
    }

    #[test]
    fn test_workspace_escape_cancels_modal() {
        let mut ws = WorkspaceView::new();
        ws.utilities.prompt_delete("/tmp/test_song.mp3");
        assert!(ws.utilities.pending_delete_path.is_some());

        let handled = ws.handle_escape();
        assert!(handled);
        assert!(ws.utilities.pending_delete_path.is_none());
    }

    #[test]
    fn test_workspace_escape_clears_selection() {
        let mut ws = WorkspaceView::new();
        ws.staging.tracks.push(Track::new("Song 1", vec!["Artist".into()], 180_000));
        ws.staging.selection.select(0);
        assert_eq!(ws.staging.selection.count(), 1);

        let handled = ws.handle_escape();
        assert!(handled);
        assert!(ws.staging.selection.is_empty());

        // Escape when idle should return false
        assert!(!ws.handle_escape());
    }

    #[test]
    fn test_workspace_keybindings_parsing() {
        use gpui::KeyBinding;

        let b1 = KeyBinding::new("ctrl-1", SelectStaging, None);
        let b2 = KeyBinding::new("ctrl-2", SelectCompare, None);
        let b3 = KeyBinding::new("ctrl-3", SelectDownloads, None);
        let b4 = KeyBinding::new("ctrl-4", SelectUtilities, None);
        let b5 = KeyBinding::new("ctrl-5", SelectSettings, None);
        let b_comma = KeyBinding::new("ctrl-,", SelectSettings, None);
        let b_esc = KeyBinding::new("escape", EscapeAction, None);

        // Verify all keybindings were constructed successfully without panic
        assert_eq!(b1.action().name(), "music_tools::SelectStaging");
        assert_eq!(b2.action().name(), "music_tools::SelectCompare");
        assert_eq!(b3.action().name(), "music_tools::SelectDownloads");
        assert_eq!(b4.action().name(), "music_tools::SelectUtilities");
        assert_eq!(b5.action().name(), "music_tools::SelectSettings");
        assert_eq!(b_comma.action().name(), "music_tools::SelectSettings");
        assert_eq!(b_esc.action().name(), "music_tools::EscapeAction");
    }
}
