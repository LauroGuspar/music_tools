use std::path::Path;

use gpui::prelude::*;
use gpui::{
    div, px, Context, Entity, FocusHandle, FontWeight, IntoElement, Render, SharedString,
    Window,
};

use storage::CacheStorage;
use ui::icon::Icon;
use ui::input::Input;
use ui::modal::Modal;
use ui::progress::{Badge, BadgeVariant, ProgressBar};
use ui::tabs::{AppTab, TabItem, Tabs};
use ui::theme::colors;
use ui::Button;

use crate::actions::{
    EscapeAction, SelectCompare, SelectDownloads, SelectSettings, SelectStaging, SelectUtilities,
};
use crate::compare::{CompareSubTab, CompareView};
use crate::downloads::DownloadsView;
use crate::settings::{SettingsTab, SettingsView, ThemeMode, ThemePreset};
use crate::staging::{format_duration_ms, StagingView};
use crate::utilities::{format_file_size, UtilitiesView, UtilitySubTab};

pub struct WorkspaceView {
    pub active_tab: AppTab,
    pub staging: StagingView,
    pub compare: CompareView,
    pub downloads: DownloadsView,
    pub utilities: UtilitiesView,
    pub settings: SettingsView,
    pub storage: Option<CacheStorage>,
    pub focus_handle: Option<FocusHandle>,
    pub omnibar_input: Option<Entity<Input>>,
    pub settings_music_input: Option<Entity<Input>>,
    pub settings_download_input: Option<Entity<Input>>,
    pub settings_export_input: Option<Entity<Input>>,
    pub settings_spotify_id_input: Option<Entity<Input>>,
    pub compare_source_input: Option<Entity<Input>>,
    pub compare_target_input: Option<Entity<Input>>,
    pub utilities_folder_input: Option<Entity<Input>>,
}

impl Default for WorkspaceView {
    fn default() -> Self {
        Self::new_headless()
    }
}

impl WorkspaceView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let storage = CacheStorage::standard().ok();
        let mut settings = SettingsView::new();

        if let Some(ref st) = storage {
            let _ = settings.load_from_storage(st);
        }

        let mut downloads = DownloadsView::new();
        downloads.set_download_dir(&settings.download_dir);

        let utilities = UtilitiesView::new();

        let omnibar_input = cx.new(|cx| {
            Input::new(
                "Introduce URL de Spotify (playlist/álbum), ruta local de música o búsqueda...",
                cx,
            )
            .icon(Icon::Search)
            .clearable()
        });

        let settings_music_input = cx.new(|cx| {
            Input::new("Ruta local...", cx)
                .icon(Icon::Folder)
                .clearable()
                .with_value(settings.music_dir.clone())
        });

        let settings_download_input = cx.new(|cx| {
            Input::new("Ruta de descargas MP3...", cx)
                .icon(Icon::Folder)
                .clearable()
                .with_value(settings.download_dir.clone())
        });

        let settings_export_input = cx.new(|cx| {
            Input::new("Ruta de exportación...", cx)
                .icon(Icon::Folder)
                .clearable()
                .with_value(settings.playlist_export_dir.clone())
        });

        let settings_spotify_id_input = cx.new(|cx| {
            Input::new("Usar Client ID predeterminado...", cx)
                .icon(Icon::Key)
                .clearable()
                .with_value(settings.spotify_client_id.clone())
        });

        let compare_source_input = cx.new(|cx| {
            Input::new(
                "Fuente: Carpeta local o URL de Spotify (Colección deseada)...",
                cx,
            )
            .icon(Icon::Folder)
            .clearable()
        });

        let compare_target_input = cx.new(|cx| {
            Input::new("Destino: Carpeta local o Biblioteca actual...", cx)
                .icon(Icon::FolderOpen)
                .clearable()
        });

        let utilities_folder_input = cx.new(|cx| {
            Input::new("Carpeta de música a inspeccionar...", cx)
                .icon(Icon::Folder)
                .clearable()
                .with_value(utilities.library_path.clone())
        });

        Self {
            active_tab: AppTab::Staging,
            staging: StagingView::new(),
            compare: CompareView::new(),
            downloads,
            utilities,
            settings,
            storage,
            focus_handle: None,
            omnibar_input: Some(omnibar_input),
            settings_music_input: Some(settings_music_input),
            settings_download_input: Some(settings_download_input),
            settings_export_input: Some(settings_export_input),
            settings_spotify_id_input: Some(settings_spotify_id_input),
            compare_source_input: Some(compare_source_input),
            compare_target_input: Some(compare_target_input),
            utilities_folder_input: Some(utilities_folder_input),
        }
    }

    pub fn new_headless() -> Self {
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
            omnibar_input: None,
            settings_music_input: None,
            settings_download_input: None,
            settings_export_input: None,
            settings_spotify_id_input: None,
            compare_source_input: None,
            compare_target_input: None,
            utilities_folder_input: None,
        }
    }

    pub fn with_focus_handle(mut self, focus_handle: FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle);
        self
    }

    pub fn omnibar_input(&self) -> &Entity<Input> {
        self.omnibar_input
            .as_ref()
            .expect("omnibar_input is available when WorkspaceView is initialized with Context")
    }

    pub fn settings_music_input(&self) -> &Entity<Input> {
        self.settings_music_input
            .as_ref()
            .expect("settings_music_input is available when WorkspaceView is initialized with Context")
    }

    pub fn settings_download_input(&self) -> &Entity<Input> {
        self.settings_download_input
            .as_ref()
            .expect("settings_download_input is available when WorkspaceView is initialized with Context")
    }

    pub fn settings_export_input(&self) -> &Entity<Input> {
        self.settings_export_input
            .as_ref()
            .expect("settings_export_input is available when WorkspaceView is initialized with Context")
    }

    pub fn settings_spotify_id_input(&self) -> &Entity<Input> {
        self.settings_spotify_id_input
            .as_ref()
            .expect("settings_spotify_id_input is available when WorkspaceView is initialized with Context")
    }

    pub fn compare_source_input(&self) -> &Entity<Input> {
        self.compare_source_input
            .as_ref()
            .expect("compare_source_input is available when WorkspaceView is initialized with Context")
    }

    pub fn compare_target_input(&self) -> &Entity<Input> {
        self.compare_target_input
            .as_ref()
            .expect("compare_target_input is available when WorkspaceView is initialized with Context")
    }

    pub fn utilities_folder_input(&self) -> &Entity<Input> {
        self.utilities_folder_input
            .as_ref()
            .expect("utilities_folder_input is available when WorkspaceView is initialized with Context")
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

    // -------------------------------------------------------------------------
    // View Rendering Methods
    // -------------------------------------------------------------------------

    pub fn render_staging(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        // 1. Omnibar Header
        let mut omnibar_row = div()
            .flex()
            .items_center()
            .gap_3()
            .w_full()
            .p_2()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE);

        if let Some(ref input) = self.omnibar_input {
            omnibar_row = omnibar_row.child(div().flex_1().child(input.clone()));
        } else {
            omnibar_row = omnibar_row.child(
                div().flex_1().child(
                    ui::TextInput::new("staging-omnibar-input")
                        .value(self.staging.omnibar_input.clone())
                        .placeholder(
                            "Introduce URL de Spotify (playlist/álbum), ruta local de música o búsqueda...",
                        )
                        .icon(Icon::Search)
                        .clearable(true),
                ),
            );
        }

        let load_btn = Button::primary("staging-btn-load", "Cargar")
            .icon(Icon::Staging)
            .on_click(cx.listener(|this, _, _, cx| {
                let text = if let Some(ref input) = this.omnibar_input {
                    input.read(cx).text().to_string()
                } else {
                    this.staging.omnibar_input.clone()
                };
                let _ = this.staging.load_input(&text);
                cx.notify();
            }));

        let add_dl_btn = Button::secondary("staging-btn-add-downloads", "Añadir a Descargas")
            .icon(Icon::Download)
            .disabled(self.staging.tracks.is_empty())
            .on_click(cx.listener(|this, _, _, cx| {
                this.send_staging_to_downloads();
                cx.notify();
            }));

        let clear_btn = Button::ghost("staging-btn-clear", "Limpiar")
            .icon(Icon::Close)
            .disabled(self.staging.tracks.is_empty() && self.staging.omnibar_input.is_empty())
            .on_click(cx.listener(|this, _, _, cx| {
                if let Some(ref input) = this.omnibar_input {
                    input.update(cx, |input, cx| {
                        input.clear_text(cx);
                    });
                }
                this.staging.clear_tracks();
                cx.notify();
            }));

        omnibar_row = omnibar_row.child(load_btn).child(add_dl_btn).child(clear_btn);
        root = root.child(omnibar_row);

        // 2. Status / Error Message if any
        if let Some(err) = &self.staging.error_message {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .bg(colors::hex_rgb(0x3B1818))
                    .border_1()
                    .border_color(colors::STATUS_DANGER)
                    .text_sm()
                    .text_color(colors::hex_rgb(0xFCA5A5))
                    .child(
                        Icon::Warning
                            .element()
                            .size_4()
                            .text_color(colors::hex_rgb(0xFCA5A5)),
                    )
                    .child(err.clone()),
            );
        } else if let Some(status) = &self.staging.status_message {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_4()
                    .py_1()
                    .rounded_md()
                    .bg(colors::BG_SURFACE)
                    .border_1()
                    .border_color(colors::ACCENT_EMERALD)
                    .text_xs()
                    .text_color(colors::ACCENT_EMERALD)
                    .child(
                        Icon::Check
                            .element()
                            .size_4()
                            .text_color(colors::ACCENT_EMERALD),
                    )
                    .child(status.clone()),
            );
        }

        // 3. Table or Empty State
        if self.staging.tracks.is_empty() {
            root = root.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_3()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .bg(colors::BG_SURFACE)
                    .p_8()
                    .child(
                        div()
                            .size_12()
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded_full()
                            .bg(colors::BG_CARD)
                            .child(
                                Icon::Staging
                                    .element()
                                    .size_6()
                                    .text_color(colors::ACCENT_EMERALD),
                            ),
                    )
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Mesa de Trabajo Vacía"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(
                                "Introduce una URL de Spotify o una ruta de carpeta local en el Omnibar superior para inspeccionar pistas.",
                            ),
                    ),
            );
        } else {
            let mut table_container = div()
                .id("staging-table-container")
                .flex_1()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .rounded_lg()
                .border_1()
                .border_color(colors::BORDER_SUBTLE)
                .bg(colors::BG_SURFACE);

            table_container = table_container.child(
                div()
                    .flex()
                    .items_center()
                    .h(px(36.))
                    .px_4()
                    .bg(colors::BG_CARD)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_SECONDARY)
                    .child(div().w(px(40.)).child("#"))
                    .child(div().flex_grow(2.0).child("Título"))
                    .child(div().flex_1().child("Artistas"))
                    .child(div().flex_1().child("Álbum"))
                    .child(div().w(px(80.)).child("Duración"))
                    .child(div().w(px(60.)).child("Acción")),
            );

            for (idx, track) in self.staging.tracks.iter().enumerate() {
                let is_selected = self.staging.selection.is_selected(idx);
                let row_bg = if is_selected {
                    colors::BG_SURFACE_ACTIVE
                } else if idx % 2 == 1 {
                    colors::BG_BASE
                } else {
                    colors::BG_SURFACE
                };

                let row = div()
                    .id(gpui::ElementId::named_usize("staging-row", idx))
                    .flex()
                    .items_center()
                    .h(px(38.))
                    .px_4()
                    .bg(row_bg)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .text_xs()
                    .cursor_pointer()
                    .hover(|s| s.bg(colors::BG_CARD))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.staging.selection.toggle(idx);
                        cx.notify();
                    }))
                    .child(
                        div()
                            .w(px(40.))
                            .text_color(colors::TEXT_MUTED)
                            .child(format!("{}", idx + 1)),
                    )
                    .child(
                        div()
                            .flex_grow(2.0)
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(track.title.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(track.artist_display()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(colors::TEXT_MUTED)
                            .child(track.album.clone().unwrap_or_else(|| "-".into())),
                    )
                    .child(
                        div()
                            .w(px(80.))
                            .text_color(colors::TEXT_SECONDARY)
                            .child(format_duration_ms(track.duration_ms)),
                    )
                    .child(div().w(px(60.)).child(
                        Button::ghost(format!("del-{}", idx), "")
                            .icon(Icon::Close)
                            .small()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.staging.remove_track(idx);
                                cx.notify();
                            })),
                    ));

                table_container = table_container.child(row);
            }

            root = root.child(table_container);
        }

        // 4. Summary Bar
        let track_count = self.staging.tracks.len();
        let total_duration = self.staging.formatted_total_duration();
        let selected_count = self.staging.selection.count();
        let summary_text = if track_count > 0 {
            let mut s = format!("{} pistas • {}", track_count, total_duration);
            if selected_count > 0 {
                s.push_str(&format!(" ({} seleccionadas)", selected_count));
            }
            if let Some(title) = &self.staging.playlist_title {
                s.push_str(&format!(" — {}", title));
            }
            s
        } else {
            "0 pistas cargadas".to_string()
        };

        let summary_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .text_xs()
            .text_color(colors::TEXT_SECONDARY)
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
                    .child(summary_text),
            );

        root = root.child(summary_bar);

        root
    }

    pub fn render_settings(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_6()
            .gap_4();

        // 1. Header with title and Status Notice
        let mut header_row = div()
            .flex()
            .items_center()
            .justify_between()
            .border_b_1()
            .border_color(colors::BORDER_SUBTLE)
            .pb_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Icon::Settings
                            .element()
                            .size_5()
                            .text_color(colors::ACCENT_EMERALD),
                    )
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Configuración"),
                    ),
            );

        if let Some(ref status) = self.settings.status_message {
            header_row = header_row.child(
                div()
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .bg(colors::hex_rgba(0x10B98126))
                    .border_1()
                    .border_color(colors::STATUS_SUCCESS)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(colors::ACCENT_EMERALD)
                    .child(status.clone()),
            );
        }

        root = root.child(header_row);

        // 2. Main layout: Navigation Sidebar + Active Tab Content
        let mut layout = div().flex().flex_1().gap_6().min_h_0();

        // Sidebar Navigation
        let mut sidebar = div()
            .flex()
            .flex_col()
            .gap_1()
            .w_48()
            .flex_none();

        for (tab_idx, tab) in SettingsTab::ALL.iter().copied().enumerate() {
            let is_active = self.settings.active_tab == tab;
            let mut tab_btn = div()
                .id(gpui::ElementId::named_usize("settings-tab", tab_idx))
                .flex()
                .items_center()
                .gap_2p5()
                .px_3()
                .py_2()
                .rounded_md()
                .text_sm()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.settings.active_tab = tab;
                    cx.notify();
                }));

            if is_active {
                tab_btn = tab_btn
                    .bg(colors::BG_SURFACE_ACTIVE)
                    .text_color(colors::TEXT_PRIMARY)
                    .font_weight(FontWeight::SEMIBOLD)
                    .border_l_2()
                    .border_color(colors::ACCENT_EMERALD);
            } else {
                tab_btn = tab_btn
                    .text_color(colors::TEXT_SECONDARY)
                    .hover(|s| s.bg(colors::BG_SURFACE).text_color(colors::TEXT_PRIMARY));
            }

            tab_btn = tab_btn
                .child(tab.icon().element().size_4())
                .child(tab.label());

            sidebar = sidebar.child(tab_btn);
        }

        layout = layout.child(sidebar);

        // Tab Content Pane
        let content_pane = match self.settings.active_tab {
            SettingsTab::Appearance => self.render_settings_appearance(cx).into_any_element(),
            SettingsTab::Directories => self.render_settings_directories(cx).into_any_element(),
            SettingsTab::Integrations => self.render_settings_integrations(cx).into_any_element(),
            SettingsTab::Algorithm => self.render_settings_algorithm(cx).into_any_element(),
        };

        layout = layout.child(
            div()
                .id("settings-content-pane")
                .flex_1()
                .min_w_0()
                .overflow_y_scroll()
                .child(content_pane),
        );

        root = root.child(layout);

        // 3. Dialog Footer
        let footer = div()
            .flex()
            .items_center()
            .justify_end()
            .gap_3()
            .pt_3()
            .border_t_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                Button::secondary("settings-btn-cancel", "Cancelar").on_click(cx.listener(
                    |this, _, _, cx| {
                        if let Some(ref storage) = this.storage {
                            let _ = this.settings.load_from_storage(storage);
                        }
                        if let Some(ref input) = this.settings_music_input {
                            input.update(cx, |input, cx| {
                                input.set_text(&this.settings.music_dir, cx)
                            });
                        }
                        if let Some(ref input) = this.settings_export_input {
                            input.update(cx, |input, cx| {
                                input.set_text(&this.settings.playlist_export_dir, cx)
                            });
                        }
                        if let Some(ref input) = this.settings_download_input {
                            input.update(cx, |input, cx| {
                                input.set_text(&this.settings.download_dir, cx)
                            });
                        }
                        if let Some(ref input) = this.settings_spotify_id_input {
                            input.update(cx, |input, cx| {
                                input.set_text(&this.settings.spotify_client_id, cx)
                            });
                        }
                        this.settings.status_message = None;
                        cx.notify();
                    },
                )),
            )
            .child(
                Button::primary("settings-btn-save", "Guardar Cambios").on_click(cx.listener(
                    |this, _, _, cx| {
                        if let Some(ref input) = this.settings_music_input {
                            this.settings.music_dir = input.read(cx).text().to_string();
                        }
                        if let Some(ref input) = this.settings_export_input {
                            this.settings.playlist_export_dir = input.read(cx).text().to_string();
                        }
                        if let Some(ref input) = this.settings_download_input {
                            this.settings.download_dir = input.read(cx).text().to_string();
                            this.downloads
                                .set_download_dir(&this.settings.download_dir);
                        }
                        if let Some(ref input) = this.settings_spotify_id_input {
                            this.settings.spotify_client_id = input.read(cx).text().to_string();
                        }
                        if let Some(ref storage) = this.storage {
                            let _ = this.settings.save_to_storage(storage);
                        } else {
                            this.settings.is_saved = true;
                            this.settings.status_message =
                                Some("¡Configuración guardada correctamente!".into());
                        }
                        cx.notify();
                    },
                )),
            );

        root = root.child(footer);

        root
    }

    fn render_settings_appearance(&self, cx: &mut Context<Self>) -> gpui::Div {
        let is_effective_dark = match self.settings.theme_mode {
            ThemeMode::Light => false,
            _ => true,
        };

        let presets: Vec<ThemePreset> = ThemePreset::ALL
            .iter()
            .copied()
            .filter(|p| p.is_dark() == is_effective_dark)
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_6()
            // Section 1: Color Scheme Mode
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Modo de Esquema de Color"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(
                                "Elige si prefieres un esquema de tema claro, oscuro o sincronizado con el sistema.",
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .pt_1()
                            .child(
                                Button::secondary("theme-mode-dark", "Oscuro")
                                    .icon(Icon::Moon)
                                    .selected(self.settings.theme_mode == ThemeMode::Dark)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_theme_mode(ThemeMode::Dark);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::secondary("theme-mode-light", "Claro")
                                    .icon(Icon::Sun)
                                    .selected(self.settings.theme_mode == ThemeMode::Light)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_theme_mode(ThemeMode::Light);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::secondary("theme-mode-system", "Sistema")
                                    .icon(Icon::Monitor)
                                    .selected(self.settings.theme_mode == ThemeMode::System)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_theme_mode(ThemeMode::System);
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Section 2: Available Theme Presets
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(format!(
                                "Temas {} Disponibles ({})",
                                if is_effective_dark { "Oscuros" } else { "Claros" },
                                presets.len()
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(format!(
                                "Selecciona un estilo visual {}. Las muestras reflejan el fondo, el color de acento y las tarjetas.",
                                if is_effective_dark { "oscuro" } else { "claro" }
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_3()
                            .children(presets.into_iter().enumerate().map(|(p_idx, preset)| {
                                let is_active = self.settings.theme_preset == preset;
                                let swatches = preset.swatches();

                                let mut card = div()
                                    .id(gpui::ElementId::named_usize("theme-preset", p_idx))
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_3()
                                    .w_48()
                                    .rounded_lg()
                                    .bg(colors::BG_CARD)
                                    .border_1()
                                    .cursor_pointer()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.settings.set_theme_preset(preset);
                                        cx.notify();
                                    }));

                                if is_active {
                                    card = card
                                        .border_color(colors::ACCENT_EMERALD)
                                        .bg(colors::BG_SURFACE_ACTIVE);
                                } else {
                                    card = card
                                        .border_color(colors::BORDER_BASE)
                                        .hover(|s| s.border_color(colors::TEXT_MUTED));
                                }

                                card.child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::TEXT_PRIMARY)
                                        .child(preset.name()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(colors::TEXT_MUTED)
                                        .child(if preset.is_dark() {
                                            "Tema Oscuro"
                                        } else {
                                            "Tema Claro"
                                        }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .pt_1()
                                        .child(
                                            div()
                                                .size_4()
                                                .rounded_full()
                                                .bg(swatches[0])
                                                .border_1()
                                                .border_color(colors::BORDER_SUBTLE),
                                        )
                                        .child(
                                            div()
                                                .size_4()
                                                .rounded_full()
                                                .bg(swatches[1])
                                                .border_1()
                                                .border_color(colors::BORDER_SUBTLE),
                                        )
                                        .child(
                                            div()
                                                .size_4()
                                                .rounded_full()
                                                .bg(swatches[2])
                                                .border_1()
                                                .border_color(colors::BORDER_SUBTLE),
                                        ),
                                )
                            })),
                    ),
            )
    }

    fn render_settings_directories(&self, _cx: &mut Context<Self>) -> gpui::Div {
        let music_input_el = if let Some(ref input) = self.settings_music_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("settings-music-dir")
                    .value(self.settings.music_dir.clone())
                    .placeholder("Ruta local...")
                    .icon(Icon::Folder)
                    .clearable(true),
            )
        };

        let export_input_el = if let Some(ref input) = self.settings_export_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("settings-export-dir")
                    .value(self.settings.playlist_export_dir.clone())
                    .placeholder("Ruta de exportación...")
                    .icon(Icon::Folder)
                    .clearable(true),
            )
        };

        let download_input_el = if let Some(ref input) = self.settings_download_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("settings-download-dir")
                    .value(self.settings.download_dir.clone())
                    .placeholder("Ruta de descargas MP3...")
                    .icon(Icon::Folder)
                    .clearable(true),
            )
        };

        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::ACCENT_EMERALD)
                    .child("RUTAS DE ARCHIVOS"),
            )
            // Field 1: Local Music Folder
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Carpeta de música local"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(music_input_el)
                            .child(Button::secondary("settings-btn-browse-music", "Buscar")),
                    ),
            )
            // Field 2: Playlist Export Folder
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Carpeta para exportar playlists (JSON / TXT)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(export_input_el)
                            .child(Button::secondary("settings-btn-browse-export", "Buscar")),
                    ),
            )
            // Field 3: Song Download Folder
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Carpeta para descargas de audio (.mp3)"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(download_input_el)
                            .child(Button::secondary(
                                "settings-btn-browse-download",
                                "Buscar",
                            )),
                    ),
            )
    }

    fn render_settings_integrations(&self, cx: &mut Context<Self>) -> gpui::Div {
        let spotify_id_el = if let Some(ref input) = self.settings_spotify_id_input {
            div().child(input.clone())
        } else {
            div().child(
                ui::TextInput::new("settings-spotify-client-id")
                    .value(self.settings.spotify_client_id.clone())
                    .placeholder("Usar Client ID predeterminado...")
                    .icon(Icon::Key)
                    .clearable(true),
            )
        };

        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::ACCENT_EMERALD)
                    .child("SPOTIFY & MOTOR DE BÚSQUEDA"),
            )
            // Spotify Client ID
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Spotify Client ID (Personalizado)"),
                    )
                    .child(spotify_id_el)
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Opcional. Permite conectar tu propia clave de desarrollador en la API Web de Spotify.",
                            ),
                    ),
            )
            // Manual YouTube Selection Checkbox
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1p5()
                    .pt_2()
                    .child(
                        div()
                            .id("settings-toggle-manual-yt")
                            .flex()
                            .items_center()
                            .gap_2p5()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.settings.manual_youtube_selection =
                                    !this.settings.manual_youtube_selection;
                                this.settings.is_saved = false;
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .size_4()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(if self.settings.manual_youtube_selection {
                                        colors::ACCENT_EMERALD
                                    } else {
                                        colors::BORDER_BASE
                                    })
                                    .bg(if self.settings.manual_youtube_selection {
                                        colors::ACCENT_EMERALD
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(self.settings.manual_youtube_selection, |this| {
                                        this.child(
                                            Icon::Check
                                                .element()
                                                .size_3()
                                                .text_color(colors::TEXT_PRIMARY),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Elegir vídeo manualmente en YouTube antes de descargar"),
                            ),
                    )
                    .child(
                        div()
                            .pl_6()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Muestra un diálogo de confirmación con 5 opciones de vídeo para seleccionar la versión preferida.",
                            ),
                    ),
            )
    }

    fn render_settings_algorithm(&self, cx: &mut Context<Self>) -> gpui::Div {
        let ytdlp_badge = if self.settings.ytdlp_available {
            Badge::new("Disponible")
                .variant(BadgeVariant::Success)
                .icon(Icon::Check)
        } else {
            Badge::new("No encontrado")
                .variant(BadgeVariant::Danger)
                .icon(Icon::Close)
        };

        let ffmpeg_badge = if self.settings.ffmpeg_available {
            Badge::new("Disponible")
                .variant(BadgeVariant::Success)
                .icon(Icon::Check)
        } else {
            Badge::new("No encontrado")
                .variant(BadgeVariant::Danger)
                .icon(Icon::Close)
        };

        div()
            .flex()
            .flex_col()
            .gap_6()
            // Fuzzy match threshold
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Tolerancia de Coincidencia (Fuzzy Match Threshold)"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child(format!("{:.0}%", self.settings.fuzzy_threshold * 100.0)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::ghost("settings-fuzzy-minus", "")
                                    .icon(Icon::Minus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_fuzzy_threshold(
                                            this.settings.fuzzy_threshold - 0.05,
                                        );
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors::BG_CARD)
                                    .border_1()
                                    .border_color(colors::BORDER_BASE)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child(format!("{:.2}", self.settings.fuzzy_threshold)),
                            )
                            .child(
                                Button::ghost("settings-fuzzy-plus", "")
                                    .icon(Icon::Plus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_fuzzy_threshold(
                                            this.settings.fuzzy_threshold + 0.05,
                                        );
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Controla la precisión requerida al comparar nombres de pistas y artistas locales contra playlists remotas para detectar coincidencias parciales.",
                            ),
                    ),
            )
            // Concurrent downloads
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Descargas Simultáneas"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child(format!("{} hilos", self.settings.concurrent_downloads)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                Button::ghost("settings-conc-minus", "")
                                    .icon(Icon::Minus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_concurrent_downloads(
                                            this.settings.concurrent_downloads.saturating_sub(1),
                                        );
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors::BG_CARD)
                                    .border_1()
                                    .border_color(colors::BORDER_BASE)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child(self.settings.concurrent_downloads.to_string()),
                            )
                            .child(
                                Button::ghost("settings-conc-plus", "")
                                    .icon(Icon::Plus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.set_concurrent_downloads(
                                            this.settings.concurrent_downloads + 1,
                                        );
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Número de descargas simultáneas en paralelo recomendadas (1 - 5).",
                            ),
                    ),
            )
            // Tool Diagnostics
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .pt_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Diagnóstico de Dependencias Externas"),
                            )
                            .child(
                                Button::secondary("settings-btn-refresh-diag", "Actualizar")
                                    .icon(Icon::Refresh)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.settings.refresh_diagnostics();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3()
                            .rounded_lg()
                            .bg(colors::BG_CARD)
                            .border_1()
                            .border_color(colors::BORDER_BASE)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT_PRIMARY)
                                            .child("yt-dlp (Descargador de Audio/Vídeo)"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::TEXT_MUTED)
                                            .child(
                                                self.settings
                                                    .ytdlp_path
                                                    .clone()
                                                    .unwrap_or_else(|| {
                                                        "No detectado en PATH ni runtime".into()
                                                    }),
                                            ),
                                    ),
                            )
                            .child(ytdlp_badge),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3()
                            .rounded_lg()
                            .bg(colors::BG_CARD)
                            .border_1()
                            .border_color(colors::BORDER_BASE)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT_PRIMARY)
                                            .child("ffmpeg (Conversión & Extracción de Audio)"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors::TEXT_MUTED)
                                            .child(
                                                self.settings
                                                    .ffmpeg_path
                                                    .clone()
                                                    .unwrap_or_else(|| {
                                                        "No detectado en PATH ni runtime".into()
                                                    }),
                                            ),
                                    ),
                            )
                            .child(ffmpeg_badge),
                    ),
            )
    }

    pub fn render_compare(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        let source_el = if let Some(ref input) = self.compare_source_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("cmp-source-input")
                    .value(self.compare.source_input.clone())
                    .placeholder("Fuente: Carpeta local o URL de Spotify (Colección deseada)...")
                    .icon(Icon::Folder)
                    .clearable(true),
            )
        };

        let target_el = if let Some(ref input) = self.compare_target_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("cmp-target-input")
                    .value(self.compare.target_input.clone())
                    .placeholder("Destino: Carpeta local o Biblioteca actual...")
                    .icon(Icon::FolderOpen)
                    .clearable(true),
            )
        };

        // Controls box
        let controls_box = div()
            .flex()
            .flex_col()
            .gap_3()
            .p_3()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(source_el)
                    .child(
                        Icon::Compare
                            .element()
                            .size_4()
                            .text_color(colors::TEXT_MUTED),
                    )
                    .child(target_el),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    // Fuzzy threshold control
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child("Umbral de Similitud Difusa:"),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors::BG_BASE)
                                    .border_1()
                                    .border_color(colors::BORDER_BASE)
                                    .text_xs()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child(format!(
                                        "{:.0}% ({:.2})",
                                        self.compare.fuzzy_threshold * 100.0,
                                        self.compare.fuzzy_threshold
                                    )),
                            )
                            .child(
                                Button::ghost("btn-threshold-minus", "")
                                    .icon(Icon::Minus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.compare.set_fuzzy_threshold(
                                            this.compare.fuzzy_threshold - 0.05,
                                        );
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::ghost("btn-threshold-plus", "")
                                    .icon(Icon::Plus)
                                    .small()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.compare.set_fuzzy_threshold(
                                            this.compare.fuzzy_threshold + 0.05,
                                        );
                                        cx.notify();
                                    })),
                            ),
                    )
                    // Action Buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::primary("btn-execute-compare", "Comparar")
                                    .icon(Icon::Compare)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(ref input) = this.compare_source_input {
                                            this.compare.source_input =
                                                input.read(cx).text().to_string();
                                        }
                                        if let Some(ref input) = this.compare_target_input {
                                            this.compare.target_input =
                                                input.read(cx).text().to_string();
                                        }
                                        let src_p = Path::new(&this.compare.source_input);
                                        if src_p.is_dir() {
                                            this.compare.source_tracks =
                                                scanner::scan_directory(src_p).unwrap_or_default();
                                        }
                                        let tgt_p = Path::new(&this.compare.target_input);
                                        if tgt_p.is_dir() {
                                            this.compare.target_tracks =
                                                scanner::scan_directory(tgt_p).unwrap_or_default();
                                        }
                                        this.compare.compare();
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::secondary(
                                    "btn-export-missing",
                                    "Exportar Faltantes a Descargas",
                                )
                                .icon(Icon::Download)
                                .disabled(self.compare.missing_count() == 0)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.send_compare_missing_to_downloads();
                                    cx.notify();
                                })),
                            ),
                    ),
            );

        root = root.child(controls_box);

        // Subtabs Navigation
        let mut subtabs_row = div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(colors::BORDER_SUBTLE)
            .pb_2();

        for (tab_idx, tab) in CompareSubTab::ALL.iter().copied().enumerate() {
            let is_active = self.compare.active_subtab == tab;
            let count = match tab {
                CompareSubTab::Exact => self.compare.exact_count(),
                CompareSubTab::Probable => self.compare.probable_count(),
                CompareSubTab::Missing => self.compare.missing_count(),
                CompareSubTab::Extra => self.compare.extra_count(),
            };

            let mut tab_btn = div()
                .id(gpui::ElementId::named_usize("compare-subtab", tab_idx))
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1p5()
                .rounded_md()
                .text_xs()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.compare.active_subtab = tab;
                    cx.notify();
                }));

            if is_active {
                tab_btn = tab_btn
                    .bg(colors::BG_SURFACE_ACTIVE)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_PRIMARY)
                    .border_b_2()
                    .border_color(colors::ACCENT_EMERALD);
            } else {
                tab_btn = tab_btn
                    .text_color(colors::TEXT_SECONDARY)
                    .hover(|s| s.bg(colors::BG_SURFACE).text_color(colors::TEXT_PRIMARY));
            }

            tab_btn = tab_btn.child(tab.label()).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_full()
                    .bg(colors::BG_CARD)
                    .text_color(if is_active {
                        colors::ACCENT_EMERALD
                    } else {
                        colors::TEXT_MUTED
                    })
                    .text_xs()
                    .child(count.to_string()),
            );

            subtabs_row = subtabs_row.child(tab_btn);
        }

        root = root.child(subtabs_row);

        // Subtab Content or Empty State
        let items_count = match self.compare.active_subtab {
            CompareSubTab::Exact => self.compare.exact_count(),
            CompareSubTab::Probable => self.compare.probable_count(),
            CompareSubTab::Missing => self.compare.missing_count(),
            CompareSubTab::Extra => self.compare.extra_count(),
        };

        if items_count == 0 {
            root = root.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .bg(colors::BG_SURFACE)
                    .p_8()
                    .child(
                        Icon::Compare
                            .element()
                            .size_8()
                            .text_color(colors::TEXT_MUTED),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Sin elementos en esta categoría"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Ejecuta una comparación entre la colección deseada y la biblioteca local para ver resultados.",
                            ),
                    ),
            );
        } else {
            let mut table = div()
                .id("compare-table-container")
                .flex_1()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .rounded_lg()
                .border_1()
                .border_color(colors::BORDER_SUBTLE)
                .bg(colors::BG_SURFACE);

            table = table.child(
                div()
                    .flex()
                    .items_center()
                    .h(px(36.))
                    .px_4()
                    .bg(colors::BG_CARD)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_SECONDARY)
                    .child(div().w(px(40.)).child("#"))
                    .child(div().flex_grow(2.0).child("Pista Fuente"))
                    .child(div().flex_grow(2.0).child("Pista Destino / Coincidencia"))
                    .child(div().w(px(100.)).child("Puntuación")),
            );

            match self.compare.active_subtab {
                CompareSubTab::Exact => {
                    if let Some(ref res) = self.compare.comparison_result {
                        for (i, (source, target)) in res.exact_matches.iter().enumerate() {
                            table = table.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(36.))
                                    .px_4()
                                    .bg(if i % 2 == 1 {
                                        colors::BG_BASE
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .text_xs()
                                    .child(
                                        div()
                                            .w(px(40.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child(format!("{}", i + 1)),
                                    )
                                    .child(div().flex_grow(2.0).text_color(colors::TEXT_PRIMARY).child(
                                        format!(
                                            "{} - {}",
                                            source.artist_display(),
                                            source.title
                                        ),
                                    ))
                                    .child(
                                        div().flex_grow(2.0).text_color(colors::TEXT_SECONDARY).child(
                                            format!(
                                                "{} - {}",
                                                target.artist_display(),
                                                target.title
                                            ),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .w(px(100.))
                                            .text_color(colors::ACCENT_EMERALD)
                                            .child("100%"),
                                    ),
                            );
                        }
                    }
                }
                CompareSubTab::Probable => {
                    if let Some(ref res) = self.compare.comparison_result {
                        for (i, (source, target, score)) in res.probable_matches.iter().enumerate() {
                            table = table.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(36.))
                                    .px_4()
                                    .bg(if i % 2 == 1 {
                                        colors::BG_BASE
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .text_xs()
                                    .child(
                                        div()
                                            .w(px(40.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child(format!("{}", i + 1)),
                                    )
                                    .child(div().flex_grow(2.0).text_color(colors::TEXT_PRIMARY).child(
                                        format!(
                                            "{} - {}",
                                            source.artist_display(),
                                            source.title
                                        ),
                                    ))
                                    .child(
                                        div().flex_grow(2.0).text_color(colors::TEXT_SECONDARY).child(
                                            format!(
                                                "{} - {}",
                                                target.artist_display(),
                                                target.title
                                            ),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .w(px(100.))
                                            .text_color(colors::STATUS_WARNING)
                                            .child(format!("{:.0}%", score * 100.0)),
                                    ),
                            );
                        }
                    }
                }
                CompareSubTab::Missing => {
                    if let Some(ref res) = self.compare.comparison_result {
                        for (i, t) in res.missing.iter().enumerate() {
                            table = table.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(36.))
                                    .px_4()
                                    .bg(if i % 2 == 1 {
                                        colors::BG_BASE
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .text_xs()
                                    .child(
                                        div()
                                            .w(px(40.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child(format!("{}", i + 1)),
                                    )
                                    .child(div().flex_grow(2.0).text_color(colors::TEXT_PRIMARY).child(
                                        format!("{} - {}", t.artist_display(), t.title),
                                    ))
                                    .child(
                                        div()
                                            .flex_grow(2.0)
                                            .text_color(colors::STATUS_DANGER)
                                            .child("Faltante en biblioteca local"),
                                    )
                                    .child(
                                        div()
                                            .w(px(100.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child("-"),
                                    ),
                            );
                        }
                    }
                }
                CompareSubTab::Extra => {
                    if let Some(ref res) = self.compare.comparison_result {
                        for (i, t) in res.extra.iter().enumerate() {
                            table = table.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(36.))
                                    .px_4()
                                    .bg(if i % 2 == 1 {
                                        colors::BG_BASE
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .text_xs()
                                    .child(
                                        div()
                                            .w(px(40.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child(format!("{}", i + 1)),
                                    )
                                    .child(
                                        div()
                                            .flex_grow(2.0)
                                            .text_color(colors::TEXT_MUTED)
                                            .child("No requerida"),
                                    )
                                    .child(div().flex_grow(2.0).text_color(colors::TEXT_PRIMARY).child(
                                        format!("{} - {}", t.artist_display(), t.title),
                                    ))
                                    .child(
                                        div()
                                            .w(px(100.))
                                            .text_color(colors::TEXT_MUTED)
                                            .child("-"),
                                    ),
                            );
                        }
                    }
                }
            }

            root = root.child(table);
        }

        root
    }

    pub fn render_downloads(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        // Top Actions Bar
        let actions_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .p_3()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        Button::secondary(
                            "btn-pause-resume-all",
                            if self.downloads.is_paused {
                                "Reanudar Todo"
                            } else {
                                "Pausar Todo"
                            },
                        )
                        .icon(if self.downloads.is_paused {
                            Icon::Play
                        } else {
                            Icon::Pause
                        })
                        .on_click(cx.listener(|this, _, _, cx| {
                            if this.downloads.is_paused {
                                this.downloads.resume_all();
                            } else {
                                this.downloads.pause_all();
                            }
                            cx.notify();
                        })),
                    )
                    .child(
                        Button::secondary("btn-retry-failed", "Reintentar Fallidas")
                            .icon(Icon::Refresh)
                            .disabled(self.downloads.failed_count() == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.downloads.retry_failed();
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::ghost("btn-clear-completed", "Limpiar Completadas")
                            .icon(Icon::Close)
                            .disabled(self.downloads.completed_count() == 0)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.downloads.clear_completed();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors::TEXT_MUTED)
                    .child(format!("Carpeta: {}", self.downloads.download_dir.display())),
            );

        root = root.child(actions_bar);

        // Filter chips bar
        let filter_chips = div()
            .flex()
            .items_center()
            .gap_2()
            .child(
                Button::ghost(
                    "filter-all",
                    format!("Todas ({})", self.downloads.total_count()),
                )
                .small()
                .selected(self.downloads.status_filter.is_none())
                .on_click(cx.listener(|this, _, _, cx| {
                    this.downloads.status_filter = None;
                    cx.notify();
                })),
            )
            .child(
                Button::ghost(
                    "filter-downloading",
                    format!("Descargando ({})", self.downloads.downloading_count()),
                )
                .small()
                .selected(self.downloads.status_filter == Some(core::DownloadStatus::Downloading))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.downloads.status_filter = Some(core::DownloadStatus::Downloading);
                    cx.notify();
                })),
            )
            .child(
                Button::ghost(
                    "filter-queued",
                    format!("En Cola ({})", self.downloads.queued_count()),
                )
                .small()
                .selected(self.downloads.status_filter == Some(core::DownloadStatus::Queued))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.downloads.status_filter = Some(core::DownloadStatus::Queued);
                    cx.notify();
                })),
            )
            .child(
                Button::ghost(
                    "filter-completed",
                    format!("Completadas ({})", self.downloads.completed_count()),
                )
                .small()
                .selected(self.downloads.status_filter == Some(core::DownloadStatus::Completed))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.downloads.status_filter = Some(core::DownloadStatus::Completed);
                    cx.notify();
                })),
            )
            .child(
                Button::ghost(
                    "filter-failed",
                    format!("Fallidas ({})", self.downloads.failed_count()),
                )
                .small()
                .selected(self.downloads.status_filter == Some(core::DownloadStatus::Failed))
                .on_click(cx.listener(|this, _, _, cx| {
                    this.downloads.status_filter = Some(core::DownloadStatus::Failed);
                    cx.notify();
                })),
            );

        root = root.child(filter_chips);

        // Downloads Table or Empty State
        let filtered_items: Vec<&core::DownloadItem> = self
            .downloads
            .items
            .iter()
            .filter(|item| {
                if let Some(status) = self.downloads.status_filter {
                    item.status == status
                } else {
                    true
                }
            })
            .collect();

        if filtered_items.is_empty() {
            root = root.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .bg(colors::BG_SURFACE)
                    .p_8()
                    .child(
                        Icon::Download
                            .element()
                            .size_8()
                            .text_color(colors::TEXT_MUTED),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Cola de Descargas Vacía"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(
                                "Añade canciones desde la Mesa de Trabajo o Comparador para comenzar a descargar.",
                            ),
                    ),
            );
        } else {
            let mut table = div()
                .id("downloads-table-container")
                .flex_1()
                .flex()
                .flex_col()
                .overflow_y_scroll()
                .rounded_lg()
                .border_1()
                .border_color(colors::BORDER_SUBTLE)
                .bg(colors::BG_SURFACE);

            table = table.child(
                div()
                    .flex()
                    .items_center()
                    .h(px(36.))
                    .px_4()
                    .bg(colors::BG_CARD)
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .text_xs()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_SECONDARY)
                    .child(div().w(px(40.)).child("#"))
                    .child(div().flex_grow(2.0).child("Título / Artista"))
                    .child(div().w(px(140.)).child("Estado"))
                    .child(div().w(px(200.)).child("Progreso"))
                    .child(div().w(px(60.)).child("Acción")),
            );

            for (i, item) in filtered_items.into_iter().enumerate() {
                let item_id = item.id.clone();
                let status_badge = match item.status {
                    core::DownloadStatus::Queued => {
                        Badge::new("En Cola").variant(BadgeVariant::Neutral)
                    }
                    core::DownloadStatus::Downloading => {
                        Badge::new("Descargando").variant(BadgeVariant::Info)
                    }
                    core::DownloadStatus::Completed => Badge::new("Completada")
                        .variant(BadgeVariant::Success)
                        .icon(Icon::Check),
                    core::DownloadStatus::Failed => {
                        Badge::new("Error").variant(BadgeVariant::Danger).icon(Icon::Warning)
                    }
                    core::DownloadStatus::Cancelled => {
                        Badge::new("Cancelada").variant(BadgeVariant::Warning)
                    }
                };

                let row = div()
                    .flex()
                    .items_center()
                    .h(px(42.))
                    .px_4()
                    .bg(if i % 2 == 1 {
                        colors::BG_BASE
                    } else {
                        colors::BG_SURFACE
                    })
                    .text_xs()
                    .border_b_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .child(
                        div()
                            .w(px(40.))
                            .text_color(colors::TEXT_MUTED)
                            .child(format!("{}", i + 1)),
                    )
                    .child(
                        div()
                            .flex_grow(2.0)
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child(item.title.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child(item.artist.clone()),
                            ),
                    )
                    .child(div().w(px(140.)).child(status_badge))
                    .child(div().w(px(200.)).child(
                        ProgressBar::new(format!("dl-progress-{}", i), item.progress_percent)
                            .detail(format!("{:.0}%", item.progress_percent)),
                    ))
                    .child(div().w(px(60.)).child(
                        Button::ghost(format!("cancel-{}", i), "")
                            .icon(Icon::Close)
                            .small()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.downloads.cancel_item(&item_id);
                                cx.notify();
                            })),
                    ));

                table = table.child(row);
            }

            root = root.child(table);
        }

        root
    }

    pub fn render_utilities(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        let folder_el = if let Some(ref input) = self.utilities_folder_input {
            div().flex_1().child(input.clone())
        } else {
            div().flex_1().child(
                ui::TextInput::new("util-library-path")
                    .value(self.utilities.library_path.clone())
                    .placeholder("Carpeta de música a inspeccionar...")
                    .icon(Icon::Folder)
                    .clearable(true),
            )
        };

        // Top Folder & Scan Bar
        let scan_bar = div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(folder_el)
            .child(
                Button::primary("util-btn-scan", "Escanear Biblioteca")
                    .icon(Icon::Refresh)
                    .on_click(cx.listener(|this, _, _, cx| {
                        if let Some(ref input) = this.utilities_folder_input {
                            this.utilities.library_path = input.read(cx).text().to_string();
                        }
                        this.utilities.scan();
                        cx.notify();
                    })),
            );

        root = root.child(scan_bar);

        // Subtabs Navigation
        let mut subtabs_row = div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(colors::BORDER_SUBTLE)
            .pb_2();

        for (tab_idx, tab) in UtilitySubTab::ALL.iter().copied().enumerate() {
            let is_active = self.utilities.active_subtab == tab;
            let count = match tab {
                UtilitySubTab::ExactDuplicates => self.utilities.exact_count(),
                UtilitySubTab::SemanticDuplicates => self.utilities.semantic_count(),
                UtilitySubTab::CorruptFiles => self.utilities.corrupt_count(),
            };

            let mut tab_btn = div()
                .id(gpui::ElementId::named_usize("util-subtab", tab_idx))
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1p5()
                .rounded_md()
                .text_xs()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.utilities.active_subtab = tab;
                    cx.notify();
                }));

            if is_active {
                tab_btn = tab_btn
                    .bg(colors::BG_SURFACE_ACTIVE)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_PRIMARY)
                    .border_b_2()
                    .border_color(colors::ACCENT_EMERALD);
            } else {
                tab_btn = tab_btn
                    .text_color(colors::TEXT_SECONDARY)
                    .hover(|s| s.bg(colors::BG_SURFACE).text_color(colors::TEXT_PRIMARY));
            }

            tab_btn = tab_btn.child(tab.label()).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_full()
                    .bg(colors::BG_CARD)
                    .text_color(if is_active {
                        colors::ACCENT_EMERALD
                    } else {
                        colors::TEXT_MUTED
                    })
                    .text_xs()
                    .child(count.to_string()),
            );

            subtabs_row = subtabs_row.child(tab_btn);
        }

        root = root.child(subtabs_row);

        // Subtab Content Pane
        let content_count = match self.utilities.active_subtab {
            UtilitySubTab::ExactDuplicates => self.utilities.exact_count(),
            UtilitySubTab::SemanticDuplicates => self.utilities.semantic_count(),
            UtilitySubTab::CorruptFiles => self.utilities.corrupt_count(),
        };

        if content_count == 0 {
            root = root.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .rounded_lg()
                    .border_1()
                    .border_color(colors::BORDER_SUBTLE)
                    .bg(colors::BG_SURFACE)
                    .p_8()
                    .child(
                        Icon::Check
                            .element()
                            .size_8()
                            .text_color(colors::ACCENT_EMERALD),
                    )
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("No se encontraron problemas"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child("La biblioteca analizada está limpia en esta categoría."),
                    ),
            );
        } else {
            let mut list = div()
                .id("utilities-subtab-list")
                .flex_1()
                .flex()
                .flex_col()
                .gap_2()
                .overflow_y_scroll()
                .p_2();

            match self.utilities.active_subtab {
                UtilitySubTab::ExactDuplicates => {
                    for (g_idx, group) in self.utilities.exact_duplicates.iter().enumerate() {
                        let original = group.tracks.first();
                        let original_label = original
                            .and_then(|t| t.path.as_deref().or(Some(&t.title)))
                            .unwrap_or("Pista");
                        let original_size = original
                            .and_then(|t| t.file_size)
                            .map(format_file_size)
                            .unwrap_or_else(|| "-".into());

                        let mut group_card = div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_3()
                            .rounded_lg()
                            .bg(colors::BG_SURFACE)
                            .border_1()
                            .border_color(colors::BORDER_SUBTLE)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_xs()
                                            .text_color(colors::TEXT_PRIMARY)
                                            .child(format!(
                                                "Original: {} ({})",
                                                original_label,
                                                original_size
                                            )),
                                    )
                                    .child(Badge::new("Exacto").variant(BadgeVariant::Warning)),
                            );

                        for (t_idx, dup) in group.tracks.iter().skip(1).enumerate() {
                            let dup_path = dup.path.clone().unwrap_or_default();
                            let dup_label = dup.path.as_deref().unwrap_or(&dup.title);
                            group_card = group_card.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .pl_4()
                                    .py_1()
                                    .text_xs()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child(div().child(format!("Duplicado: {}", dup_label)))
                                    .child(
                                        Button::ghost(format!("del-dup-{}-{}", g_idx, t_idx), "Eliminar")
                                            .icon(Icon::Close)
                                            .small()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.utilities.prompt_delete(dup_path.clone());
                                                cx.notify();
                                            })),
                                    ),
                            );
                        }
                        list = list.child(group_card);
                    }
                }
                UtilitySubTab::SemanticDuplicates => {
                    for (g_idx, group) in self.utilities.semantic_duplicates.iter().enumerate() {
                        let original = group.tracks.first();
                        let original_label = original
                            .map(|t| format!("{} - {}", t.artist_display(), t.title))
                            .unwrap_or_else(|| "Pista".into());

                        let mut group_card = div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_3()
                            .rounded_lg()
                            .bg(colors::BG_SURFACE)
                            .border_1()
                            .border_color(colors::BORDER_SUBTLE)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_xs()
                                            .text_color(colors::TEXT_PRIMARY)
                                            .child(format!("Pista: {}", original_label)),
                                    )
                                    .child(Badge::new("Semántico").variant(BadgeVariant::Info)),
                            );

                        for (t_idx, dup) in group.tracks.iter().skip(1).enumerate() {
                            let dup_path = dup.path.clone().unwrap_or_default();
                            let dup_label = format!("{} - {}", dup.artist_display(), dup.title);
                            group_card = group_card.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .pl_4()
                                    .py_1()
                                    .text_xs()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child(div().child(format!("Similar: {}", dup_label)))
                                    .child(
                                        Button::ghost(format!("del-sem-{}-{}", g_idx, t_idx), "Eliminar")
                                            .icon(Icon::Close)
                                            .small()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.utilities.prompt_delete(dup_path.clone());
                                                cx.notify();
                                            })),
                                    ),
                            );
                        }
                        list = list.child(group_card);
                    }
                }
                UtilitySubTab::CorruptFiles => {
                    for corrupt in &self.utilities.corrupt_files {
                        let corrupt_path = corrupt.path.clone();
                        let item = div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_3()
                            .rounded_lg()
                            .bg(colors::BG_SURFACE)
                            .border_1()
                            .border_color(colors::STATUS_DANGER)
                            .text_xs()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT_PRIMARY)
                                            .child(corrupt.filename.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_color(colors::STATUS_DANGER)
                                            .child(corrupt.error_reason.clone()),
                                    ),
                            )
                            .child(
                                Button::ghost(format!("del-corrupt-{}", corrupt.filename), "Eliminar")
                                    .icon(Icon::Close)
                                    .small()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.utilities.prompt_delete(corrupt_path.clone());
                                        cx.notify();
                                    })),
                            );
                        list = list.child(item);
                    }
                }
            }

            root = root.child(list);
        }

        // Modal confirmation if pending delete
        if let Some(ref path) = self.utilities.pending_delete_path {
            let filename = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("archivo")
                .to_string();
            let modal = Modal::new("modal-delete-confirm", "Confirmar Eliminación")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(format!(
                            "¿Estás seguro de que deseas eliminar permanentemente el archivo '{}'?",
                            filename
                        ))
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::TEXT_MUTED)
                                .child(path.to_string_lossy().to_string()),
                        ),
                )
                .action(
                    Button::secondary("btn-cancel-del", "Cancelar").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.utilities.cancel_delete();
                            cx.notify();
                        },
                    )),
                )
                .action(
                    Button::danger("btn-confirm-del", "Eliminar Archivo").on_click(cx.listener(
                        |this, _, _, cx| {
                            let _ = this.utilities.confirm_delete();
                            cx.notify();
                        },
                    )),
                );

            root = root.child(modal);
        }

        root
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.active_tab;
        let active_downloads_count =
            self.downloads.queued_count() + self.downloads.downloading_count();
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
            .on_select(cx.listener(|this, selected_id: &SharedString, _, cx| {
                this.switch_tab_by_id(selected_id.as_ref());
                cx.notify();
            }));

        // 2. Active Tab View Content rendered dynamically with &mut self, window and cx
        let view_content = match active_tab {
            AppTab::Staging => self.render_staging(window, cx).into_any_element(),
            AppTab::Compare => self.render_compare(window, cx).into_any_element(),
            AppTab::Downloads => self.render_downloads(window, cx).into_any_element(),
            AppTab::Utilities => self.render_utilities(window, cx).into_any_element(),
            AppTab::Settings => self.render_settings(window, cx).into_any_element(),
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
        let ws = WorkspaceView::default();
        assert_eq!(ws.active_tab, AppTab::Staging);
        assert_eq!(ws.staging.track_count(), 0);
        assert_eq!(ws.downloads.total_count(), 0);
    }

    #[test]
    fn test_workspace_switch_tabs() {
        let mut ws = WorkspaceView::default();
        ws.switch_tab(AppTab::Downloads);
        assert_eq!(ws.active_tab, AppTab::Downloads);

        ws.switch_tab_by_id("utilities");
        assert_eq!(ws.active_tab, AppTab::Utilities);

        ws.switch_tab_by_id("settings");
        assert_eq!(ws.active_tab, AppTab::Settings);
    }

    #[test]
    fn test_workspace_staging_to_downloads_integration() {
        let mut ws = WorkspaceView::default();
        ws.staging
            .tracks
            .push(Track::new("Track 1", vec!["Artist 1".into()], 200_000));
        ws.staging
            .tracks
            .push(Track::new("Track 2", vec!["Artist 2".into()], 180_000));

        let sent = ws.send_staging_to_downloads();
        assert_eq!(sent, 2);
        assert_eq!(ws.downloads.total_count(), 2);
        assert_eq!(ws.downloads.items[0].title, "Track 1");
    }

    #[test]
    fn test_workspace_compare_to_downloads_integration() {
        let mut ws = WorkspaceView::default();
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
        let mut ws = WorkspaceView::default();

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
        let mut ws = WorkspaceView::default();
        ws.utilities.prompt_delete("/tmp/test_song.mp3");
        assert!(ws.utilities.pending_delete_path.is_some());

        let handled = ws.handle_escape();
        assert!(handled);
        assert!(ws.utilities.pending_delete_path.is_none());
    }

    #[test]
    fn test_workspace_escape_clears_selection() {
        let mut ws = WorkspaceView::default();
        ws.staging
            .tracks
            .push(Track::new("Song 1", vec!["Artist".into()], 180_000));
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
