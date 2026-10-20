use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{
    Div, FontWeight, Styled, div,
};

use downloader::{check_ffmpeg_available, check_ytdlp_available, find_ffmpeg_binary, find_ytdlp_binary};
use storage::{
    CacheStorage, DEFAULT_CONCURRENT_DOWNLOADS, DEFAULT_FUZZY_THRESHOLD,
    SETTING_CONCURRENT_DOWNLOADS, SETTING_DOWNLOAD_DIR, SETTING_FUZZY_THRESHOLD, SETTING_MUSIC_DIR,
};
use ui::progress::{Badge, BadgeVariant};
use ui::theme::colors;
use ui::{Button, TextInput};

#[derive(Clone, Debug)]
pub struct SettingsView {
    pub download_dir: String,
    pub music_dir: String,
    pub fuzzy_threshold: f64,
    pub concurrent_downloads: usize,
    pub ytdlp_available: bool,
    pub ytdlp_path: Option<String>,
    pub ffmpeg_available: bool,
    pub ffmpeg_path: Option<String>,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub is_saved: bool,
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsView {
    pub fn new() -> Self {
        let music = dirs::audio_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Music"))
            .to_string_lossy()
            .to_string();

        let downloads = dirs::audio_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Music"))
            .join("Downloads")
            .to_string_lossy()
            .to_string();

        let mut view = Self {
            download_dir: downloads,
            music_dir: music,
            fuzzy_threshold: DEFAULT_FUZZY_THRESHOLD,
            concurrent_downloads: DEFAULT_CONCURRENT_DOWNLOADS,
            ytdlp_available: false,
            ytdlp_path: None,
            ffmpeg_available: false,
            ffmpeg_path: None,
            status_message: None,
            error_message: None,
            is_saved: false,
        };

        view.refresh_diagnostics();
        view
    }

    pub fn refresh_diagnostics(&mut self) {
        self.ytdlp_available = check_ytdlp_available(None);
        self.ytdlp_path = find_ytdlp_binary(None).map(|p| p.to_string_lossy().to_string());
        self.ffmpeg_available = check_ffmpeg_available();
        self.ffmpeg_path = find_ffmpeg_binary().map(|p| p.to_string_lossy().to_string());
    }

    pub fn set_fuzzy_threshold(&mut self, threshold: f64) {
        self.fuzzy_threshold = threshold.clamp(0.50, 1.00);
        self.is_saved = false;
    }

    pub fn set_concurrent_downloads(&mut self, count: usize) {
        self.concurrent_downloads = count.clamp(1, 5);
        self.is_saved = false;
    }

    pub fn save_to_storage(&mut self, storage: &CacheStorage) -> anyhow::Result<()> {
        storage.set_setting(SETTING_DOWNLOAD_DIR, &self.download_dir)?;
        storage.set_setting(SETTING_MUSIC_DIR, &self.music_dir)?;
        storage.set_setting(SETTING_FUZZY_THRESHOLD, &format!("{:.2}", self.fuzzy_threshold))?;
        storage.set_setting(SETTING_CONCURRENT_DOWNLOADS, &self.concurrent_downloads.to_string())?;

        self.is_saved = true;
        self.status_message = Some("Configuración guardada exitosamente".into());
        self.error_message = None;
        Ok(())
    }

    pub fn load_from_storage(&mut self, storage: &CacheStorage) -> anyhow::Result<()> {
        if let Some(dl) = storage.get_setting(SETTING_DOWNLOAD_DIR)? {
            self.download_dir = dl;
        }
        if let Some(m) = storage.get_setting(SETTING_MUSIC_DIR)? {
            self.music_dir = m;
        }
        if let Some(thresh_str) = storage.get_setting(SETTING_FUZZY_THRESHOLD)? {
            if let Ok(thresh) = thresh_str.parse::<f64>() {
                self.fuzzy_threshold = thresh.clamp(0.50, 1.00);
            }
        }
        if let Some(conc_str) = storage.get_setting(SETTING_CONCURRENT_DOWNLOADS)? {
            if let Ok(conc) = conc_str.parse::<usize>() {
                self.concurrent_downloads = conc.clamp(1, 5);
            }
        }
        Ok(())
    }

    pub fn render_view(&self) -> Div {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_6()
            .gap_5();

        // Title header
        root = root.child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(colors::BORDER_SUBTLE)
                .pb_3()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(FontWeight::BOLD)
                                .text_color(colors::TEXT_PRIMARY)
                                .child("Configuración del Sistema"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::TEXT_SECONDARY)
                                .child("Ajustes generales, directorios, parámetros de comparación y diagnóstico de herramientas."),
                        ),
                )
                .child(
                    Button::primary("settings-btn-save", "Guardar Configuración")
                        .icon("💾"),
                ),
        );

        // Feedback Banner
        if let Some(status) = &self.status_message {
            root = root.child(
                div()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .bg(colors::BG_SURFACE)
                    .border_1()
                    .border_color(colors::ACCENT_EMERALD)
                    .text_sm()
                    .text_color(colors::ACCENT_EMERALD)
                    .child(format!("✓ {}", status)),
            );
        }

        // Section 1: Directories
        let dir_section = div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_PRIMARY)
                    .child("📁 Directorios de Biblioteca y Descargas"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Directorio de Descargas:"),
                    )
                    .child(
                        TextInput::new("settings-download-dir")
                            .value(self.download_dir.clone())
                            .placeholder("Ruta para guardar archivos descargados...")
                            .clearable(true),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_SECONDARY)
                            .child("Directorio Principal de Música:"),
                    )
                    .child(
                        TextInput::new("settings-music-dir")
                            .value(self.music_dir.clone())
                            .placeholder("Ruta raíz de tu colección de música local...")
                            .clearable(true),
                    ),
            );

        root = root.child(dir_section);

        // Section 2: Audio & Comparison Parameters
        let params_section = div()
            .flex()
            .flex_col()
            .gap_4()
            .p_4()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT_PRIMARY)
                    .child("⚡ Parámetros de Comparación y Descarga"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Umbral de Similitud Sørensen-Dice:"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_MUTED)
                                    .child("Controla el grado de tolerancia difusa ante faltas ortográficas y sufijos (0.50 a 1.00)."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::ghost("btn-thresh-dec", "-")
                                    .small(),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors::BG_BASE)
                                    .border_1()
                                    .border_color(colors::BORDER_BASE)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child(format!("{:.0}% ({:.2})", self.fuzzy_threshold * 100.0, self.fuzzy_threshold)),
                            )
                            .child(
                                Button::ghost("btn-thresh-inc", "+")
                                    .small(),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("Descargas Concurrentes Simultáneas:"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_MUTED)
                                    .child("Número máximo de procesos de descarga en paralelo (1 a 5, recomendado 3)."),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::ghost("btn-conc-dec", "-")
                                    .small(),
                            )
                            .child(
                                div()
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors::BG_BASE)
                                    .border_1()
                                    .border_color(colors::BORDER_BASE)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(colors::ACCENT_EMERALD)
                                    .child(format!("{}", self.concurrent_downloads)),
                            )
                            .child(
                                Button::ghost("btn-conc-inc", "+")
                                    .small(),
                            ),
                    ),
            );

        root = root.child(params_section);

        // Section 3: Diagnostic of system dependencies
        let ytdlp_badge = if self.ytdlp_available {
            Badge::new("Disponible").variant(BadgeVariant::Success).icon("✓")
        } else {
            Badge::new("No encontrado").variant(BadgeVariant::Danger).icon("✕")
        };

        let ffmpeg_badge = if self.ffmpeg_available {
            Badge::new("Disponible").variant(BadgeVariant::Success).icon("✓")
        } else {
            Badge::new("No encontrado").variant(BadgeVariant::Danger).icon("✕")
        };

        let diag_section = div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
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
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("🛠 Diagnóstico de Dependencias Externas"),
                    )
                    .child(
                        Button::ghost("diag-refresh-btn", "Comprobar")
                            .small()
                            .icon("↺"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .rounded_md()
                    .bg(colors::BG_CARD)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("yt-dlp (Extractor de audio)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_MUTED)
                                    .child(self.ytdlp_path.clone().unwrap_or_else(|| "Instálalo vía paquete o gestor del sistema".into())),
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
                    .rounded_md()
                    .bg(colors::BG_CARD)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child("ffmpeg (Conversor & Muxer)"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_MUTED)
                                    .child(self.ffmpeg_path.clone().unwrap_or_else(|| "Necesario para remuestreo y codificación a MP3".into())),
                            ),
                    )
                    .child(ffmpeg_badge),
            );

        root = root.child(diag_section);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_view_initial_state() {
        let view = SettingsView::new();
        assert_eq!(view.fuzzy_threshold, DEFAULT_FUZZY_THRESHOLD);
        assert_eq!(view.concurrent_downloads, DEFAULT_CONCURRENT_DOWNLOADS);
        assert!(!view.download_dir.is_empty());
        assert!(!view.music_dir.is_empty());
    }

    #[test]
    fn test_settings_clamping() {
        let mut view = SettingsView::new();

        view.set_fuzzy_threshold(0.1);
        assert_eq!(view.fuzzy_threshold, 0.50);

        view.set_fuzzy_threshold(1.2);
        assert_eq!(view.fuzzy_threshold, 1.00);

        view.set_concurrent_downloads(0);
        assert_eq!(view.concurrent_downloads, 1);

        view.set_concurrent_downloads(10);
        assert_eq!(view.concurrent_downloads, 5);
    }

    #[test]
    fn test_settings_storage_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("mt_views_settings_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("settings.sqlite");
        let storage = CacheStorage::new(&db_path);

        let mut view = SettingsView::new();
        view.download_dir = "/custom/downloads".into();
        view.music_dir = "/custom/music".into();
        view.set_fuzzy_threshold(0.88);
        view.set_concurrent_downloads(4);

        assert!(view.save_to_storage(&storage).is_ok());
        assert!(view.is_saved);

        let mut loaded_view = SettingsView::new();
        assert!(loaded_view.load_from_storage(&storage).is_ok());
        assert_eq!(loaded_view.download_dir, "/custom/downloads");
        assert_eq!(loaded_view.music_dir, "/custom/music");
        assert_eq!(loaded_view.fuzzy_threshold, 0.88);
        assert_eq!(loaded_view.concurrent_downloads, 4);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
