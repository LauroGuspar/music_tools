use gpui::prelude::*;
use gpui::{
    Div, FontWeight, IntoElement, Pixels, Styled, div, px,
};

use core::{DownloadItem, Track};
use ui::icon::Icon;
use ui::table::{ColumnSpec, TableSelection, VirtualTable};
use ui::theme::colors;
use ui::{Button, TextInput};

/// Helper to format duration in milliseconds to MM:SS or H:MM:SS
pub fn format_duration_ms(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Helper to format human friendly long durations: "Xh Ym" or "Xm Ys"
pub fn format_duration_human(ms: u64) -> String {
    let total_secs = ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}

#[derive(Clone, Debug)]
pub struct StagingView {
    pub omnibar_input: String,
    pub tracks: Vec<Track>,
    pub playlist_title: Option<String>,
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub status_message: Option<String>,
    pub selection: TableSelection,
    pub scroll_top: Pixels,
    pub viewport_height: Pixels,
}

impl Default for StagingView {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingView {
    pub fn new() -> Self {
        Self {
            omnibar_input: String::new(),
            tracks: Vec::new(),
            playlist_title: None,
            is_loading: false,
            error_message: None,
            status_message: None,
            selection: TableSelection::new(),
            scroll_top: Pixels::ZERO,
            viewport_height: px(500.),
        }
    }

    pub fn with_tracks(tracks: Vec<Track>) -> Self {
        let mut view = Self::new();
        view.tracks = tracks;
        view
    }

    pub fn set_omnibar_input(&mut self, text: impl Into<String>) {
        self.omnibar_input = text.into();
    }

    pub fn clear_omnibar(&mut self) {
        self.omnibar_input.clear();
    }

    pub fn clear_tracks(&mut self) {
        self.tracks.clear();
        self.playlist_title = None;
        self.selection.clear();
        self.status_message = None;
        self.error_message = None;
    }

    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    pub fn total_duration_ms(&self) -> u64 {
        self.tracks.iter().map(|t| t.duration_ms).sum()
    }

    pub fn formatted_total_duration(&self) -> String {
        format_duration_human(self.total_duration_ms())
    }

    pub fn selected_tracks(&self) -> Vec<Track> {
        let selected_indices = self.selection.selected_rows();
        selected_indices
            .into_iter()
            .filter_map(|i| self.tracks.get(i).cloned())
            .collect()
    }

    pub fn all_or_selected_tracks(&self) -> Vec<Track> {
        if self.selection.is_empty() {
            self.tracks.clone()
        } else {
            self.selected_tracks()
        }
    }

    pub fn to_download_items(&self) -> Vec<DownloadItem> {
        let tracks_to_export = self.all_or_selected_tracks();
        tracks_to_export
            .into_iter()
            .enumerate()
            .map(|(idx, track)| {
                let id = track
                    .spotify_id
                    .clone()
                    .unwrap_or_else(|| format!("stg-{}", idx + 1));
                DownloadItem::new(id, track.title.clone(), track.artist_display())
            })
            .collect()
    }

    /// Load tracks from input (URL, local folder or mock data)
    pub fn load_input(&mut self, input: &str) -> anyhow::Result<usize> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            self.error_message = Some("Por favor introduce una URL o ruta local válida".into());
            return Ok(0);
        }

        self.is_loading = true;
        self.error_message = None;

        // 1. Check if it's a local filesystem directory or JSON file
        let expanded = ui::expand_tilde(trimmed);
        if expanded.exists() && expanded.is_dir() {
            let scanned = scanner::scan_directory(&expanded).unwrap_or_default();
            let count = scanned.len();
            self.tracks = scanned;
            let folder_name = expanded
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Carpeta local");
            self.playlist_title = Some(format!("Carpeta: {}", folder_name));
            self.status_message = Some(format!("Cargadas {} pistas locales", count));
            self.is_loading = false;
            return Ok(count);
        }

        if expanded.exists() && expanded.is_file() && expanded.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Ok(content) = std::fs::read_to_string(&expanded) {
                if let Ok(playlist) = serde_json::from_str::<core::Playlist>(&content) {
                    let count = playlist.tracks.len();
                    self.tracks = playlist.tracks;
                    self.playlist_title = Some(playlist.name);
                    self.status_message = Some(format!("Cargadas {} pistas desde playlist JSON", count));
                    self.is_loading = false;
                    return Ok(count);
                } else if let Ok(tracks) = serde_json::from_str::<Vec<Track>>(&content) {
                    let count = tracks.len();
                    self.tracks = tracks;
                    let file_stem = expanded.file_stem().and_then(|s| s.to_str()).unwrap_or("Pistas");
                    self.playlist_title = Some(file_stem.to_string());
                    self.status_message = Some(format!("Cargadas {} pistas desde archivo JSON", count));
                    self.is_loading = false;
                    return Ok(count);
                }
            }
        }

        // 2. Check if it's a Spotify Playlist/Track URL
        if trimmed.contains("spotify.com") || trimmed.starts_with("spotify:") {
            if let Some(playlist_id) = spotify::extract_playlist_id(trimmed) {
                self.playlist_title = Some(format!("Spotify Playlist ({})", &playlist_id[..playlist_id.len().min(8)]));
                self.status_message = Some(format!("URL de Spotify reconocida: {}", playlist_id));
                self.is_loading = false;
                return Ok(self.tracks.len());
            }
        }

        // 3. Fallback text search/query
        self.status_message = Some(format!("Búsqueda: '{}'", trimmed));
        self.is_loading = false;
        Ok(self.tracks.len())
    }

    pub fn remove_track(&mut self, index: usize) {
        if index < self.tracks.len() {
            self.tracks.remove(index);
            self.selection.clear();
        }
    }

    pub fn render_view(&self) -> Div {
        let theme_bg = colors::BG_BASE;
        let track_count = self.tracks.len();
        let total_duration = self.formatted_total_duration();
        let selected_count = self.selection.count();

        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme_bg)
            .p_4()
            .gap_3();

        // 1. Omnibar Header
        let omnibar_row = div()
            .flex()
            .items_center()
            .gap_3()
            .w_full()
            .p_2()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .flex_1()
                    .child(
                        TextInput::new("staging-omnibar-input")
                            .value(self.omnibar_input.clone())
                            .placeholder("Introduce URL de Spotify (playlist/álbum), ruta local de música o búsqueda...")
                            .icon(Icon::Search)
                            .clearable(true),
                    ),
            )
            .child(
                Button::primary("staging-btn-load", "Cargar")
                    .icon(Icon::Staging),
            )
            .child(
                Button::secondary("staging-btn-add-downloads", "Añadir a Descargas")
                    .icon(Icon::Download)
                    .disabled(self.tracks.is_empty()),
            )
            .child(
                Button::ghost("staging-btn-clear", "Limpiar")
                    .icon(Icon::Close)
                    .disabled(self.tracks.is_empty() && self.omnibar_input.is_empty()),
            );

        root = root.child(omnibar_row);

        // 2. Status / Error Message if any
        if let Some(err) = &self.error_message {
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
                    .child(Icon::Warning.element().size_4().text_color(colors::hex_rgb(0xFCA5A5)))
                    .child(err.clone()),
            );
        } else if let Some(status) = &self.status_message {
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
                    .child(Icon::Check.element().size_4().text_color(colors::ACCENT_EMERALD))
                    .child(status.clone()),
            );
        }

        // 3. VirtualTable or Empty State
        if self.tracks.is_empty() {
            let empty_state = div()
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
                        .child(Icon::Staging.element().size_6().text_color(colors::ACCENT_EMERALD)),
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
                        .child("Introduce una URL de Spotify o una ruta de carpeta local en el Omnibar superior para inspeccionar pistas."),
                );

            root = root.child(empty_state);
        } else {
            let cols = vec![
                ColumnSpec::new("index".to_string(), "#")
                    .fixed(px(50.))
                    .sortable(false),
                ColumnSpec::new("title".to_string(), "Título")
                    .fill(2.5),
                ColumnSpec::new("artists".to_string(), "Artistas")
                    .fill(1.8),
                ColumnSpec::new("album".to_string(), "Álbum")
                    .fill(1.5),
                ColumnSpec::new("duration".to_string(), "Duración")
                    .fixed(px(85.)),
                ColumnSpec::new("actions".to_string(), "Acción")
                    .fixed(px(80.))
                    .sortable(false),
            ];

            let table = VirtualTable::new("staging-table")
                .columns(cols)
                .rows(self.tracks.clone())
                .scroll_top(self.scroll_top)
                .viewport_height(self.viewport_height)
                .row_height(px(38.))
                .selection(self.selection.clone())
                .cell_builder(|row_idx, track: &Track, col_idx, _col, _width| {
                    match col_idx {
                        0 => div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(format!("{}", row_idx + 1))
                            .into_any_element(),
                        1 => div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(track.title.clone())
                            .into_any_element(),
                        2 => div()
                            .text_sm()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(track.artist_display())
                            .into_any_element(),
                        3 => div()
                            .text_sm()
                            .text_color(colors::TEXT_MUTED)
                            .child(track.album.clone().unwrap_or_else(|| "-".into()))
                            .into_any_element(),
                        4 => div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_SECONDARY)
                            .child(format_duration_ms(track.duration_ms))
                            .into_any_element(),
                        5 => Button::ghost(format!("del-{}", row_idx), "")
                            .icon(Icon::Close)
                            .small()
                            .into_any_element(),
                        _ => div().into_any_element(),
                    }
                });

            root = root.child(div().flex_1().child(table));
        }

        // 4. Summary Bar
        let summary_text = if track_count > 0 {
            let mut s = format!("{} pistas • {}", track_count, total_duration);
            if selected_count > 0 {
                s.push_str(&format!(" ({} seleccionadas)", selected_count));
            }
            if let Some(title) = &self.playlist_title {
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
                            .bg(if track_count > 0 { colors::ACCENT_EMERALD } else { colors::TEXT_MUTED }),
                    )
                    .child(summary_text),
            )
            .child(
                div()
                    .text_color(colors::TEXT_MUTED)
                    .child("Mesa de Trabajo"),
            );

        root.child(summary_bar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_staging_view_initial_state() {
        let view = StagingView::new();
        assert_eq!(view.track_count(), 0);
        assert_eq!(view.total_duration_ms(), 0);
        assert!(view.omnibar_input.is_empty());
        assert!(view.playlist_title.is_none());
        assert!(!view.is_loading);
        assert!(view.selection.is_empty());
    }

    #[test]
    fn test_staging_view_tracks_and_durations() {
        let tracks = vec![
            Track::new("Song 1", vec!["Artist 1".into()], 180_000),
            Track::new("Song 2", vec!["Artist 2".into()], 240_000),
        ];

        let mut view = StagingView::with_tracks(tracks);
        assert_eq!(view.track_count(), 2);
        assert_eq!(view.total_duration_ms(), 420_000);
        assert_eq!(format_duration_ms(180_000), "03:00");
        assert_eq!(format_duration_ms(3_665_000), "1:01:05");
        assert_eq!(format_duration_human(420_000), "7m 0s");

        view.remove_track(0);
        assert_eq!(view.track_count(), 1);
        assert_eq!(view.tracks[0].title, "Song 2");

        view.clear_tracks();
        assert_eq!(view.track_count(), 0);
    }

    #[test]
    fn test_staging_to_download_items() {
        let tracks = vec![
            Track::new("Song 1", vec!["Artist 1".into()], 180_000),
            Track::new("Song 2", vec!["Artist 2".into()], 240_000),
        ];
        let mut view = StagingView::with_tracks(tracks);

        // When none selected, all are exported
        let all_dl = view.to_download_items();
        assert_eq!(all_dl.len(), 2);
        assert_eq!(all_dl[0].title, "Song 1");

        // When row 1 is selected
        view.selection.select(1);
        let selected_dl = view.to_download_items();
        assert_eq!(selected_dl.len(), 1);
        assert_eq!(selected_dl[0].title, "Song 2");
    }

    #[test]
    fn test_staging_load_empty_and_spotify_url() {
        let mut view = StagingView::new();
        assert!(view.load_input("   ").is_ok());
        assert!(view.error_message.is_some());

        let url = "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M";
        assert!(view.load_input(url).is_ok());
        assert!(view.playlist_title.is_some());
        assert!(view.error_message.is_none());
    }

    #[test]
    fn test_staging_load_with_tilde_expansion() {
        let mut view = StagingView::new();
        // A non-existing path under ~/should_not_exist_xyz123 should fall back gracefully
        assert!(view.load_input("~/non_existent_folder_abc_123").is_ok());
        assert!(view.status_message.is_some());
    }
}
