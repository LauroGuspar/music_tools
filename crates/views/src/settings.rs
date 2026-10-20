use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{
    Div, FontWeight, Rgba, Styled, div,
};

use downloader::{
    check_ffmpeg_available, check_ytdlp_available, find_ffmpeg_binary, find_ytdlp_binary,
};
use storage::{
    CacheStorage, DEFAULT_CONCURRENT_DOWNLOADS, DEFAULT_FUZZY_THRESHOLD,
    SETTING_CONCURRENT_DOWNLOADS, SETTING_DOWNLOAD_DIR, SETTING_FUZZY_THRESHOLD,
    SETTING_MANUAL_YOUTUBE_SELECTION, SETTING_MUSIC_DIR, SETTING_PLAYLIST_EXPORT_DIR,
    SETTING_SPOTIFY_CLIENT_ID, SETTING_THEME_MODE, SETTING_THEME_PRESET,
};
use ui::icon::Icon;
use ui::progress::{Badge, BadgeVariant};
use ui::theme::colors;
use ui::{Button, TextInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Appearance,
    Directories,
    Integrations,
    Algorithm,
}

impl SettingsTab {
    pub const ALL: [Self; 4] = [
        Self::Appearance,
        Self::Directories,
        Self::Integrations,
        Self::Algorithm,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Appearance => "Apariencia",
            Self::Directories => "Directorios",
            Self::Integrations => "Integraciones",
            Self::Algorithm => "Algoritmo",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            Self::Appearance => Icon::Palette,
            Self::Directories => Icon::Folder,
            Self::Integrations => Icon::Key,
            Self::Algorithm => Icon::Cpu,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
    System,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
            Self::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "system" => Self::System,
            _ => Self::Dark,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemePreset {
    #[default]
    DefaultDark,
    DefaultLight,
    CatppuccinMocha,
    CatppuccinLatte,
    Dracula,
    OneDark,
    Nord,
}

impl ThemePreset {
    pub const ALL: [Self; 7] = [
        Self::DefaultDark,
        Self::DefaultLight,
        Self::CatppuccinMocha,
        Self::CatppuccinLatte,
        Self::Dracula,
        Self::OneDark,
        Self::Nord,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::DefaultDark => "default-dark",
            Self::DefaultLight => "default-light",
            Self::CatppuccinMocha => "catppuccin-mocha",
            Self::CatppuccinLatte => "catppuccin-latte",
            Self::Dracula => "dracula",
            Self::OneDark => "one-dark",
            Self::Nord => "nord",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DefaultDark => "Default Dark",
            Self::DefaultLight => "Default Light",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinLatte => "Catppuccin Latte",
            Self::Dracula => "Dracula",
            Self::OneDark => "One Dark",
            Self::Nord => "Nord",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::DefaultLight | Self::CatppuccinLatte => false,
            _ => true,
        }
    }

    pub fn swatches(&self) -> [Rgba; 3] {
        match self {
            Self::DefaultDark => [
                colors::hex_rgb(0x07130d),
                colors::hex_rgb(0x79e5a7),
                colors::hex_rgb(0x102319),
            ],
            Self::DefaultLight => [
                colors::hex_rgb(0xffffff),
                colors::hex_rgb(0x059669),
                colors::hex_rgb(0xf3f4f6),
            ],
            Self::CatppuccinMocha => [
                colors::hex_rgb(0x1e1e2e),
                colors::hex_rgb(0xcba6f7),
                colors::hex_rgb(0x313244),
            ],
            Self::CatppuccinLatte => [
                colors::hex_rgb(0xeff1f5),
                colors::hex_rgb(0x8839ef),
                colors::hex_rgb(0xdce0e8),
            ],
            Self::Dracula => [
                colors::hex_rgb(0x282a36),
                colors::hex_rgb(0xff79c6),
                colors::hex_rgb(0x44475a),
            ],
            Self::OneDark => [
                colors::hex_rgb(0x282c34),
                colors::hex_rgb(0x61afef),
                colors::hex_rgb(0x353b45),
            ],
            Self::Nord => [
                colors::hex_rgb(0x2e3440),
                colors::hex_rgb(0x88c0d0),
                colors::hex_rgb(0x434c5e),
            ],
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "default-light" => Self::DefaultLight,
            "catppuccin-mocha" => Self::CatppuccinMocha,
            "catppuccin-latte" => Self::CatppuccinLatte,
            "dracula" => Self::Dracula,
            "one-dark" => Self::OneDark,
            "nord" => Self::Nord,
            _ => Self::DefaultDark,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SettingsView {
    pub active_tab: SettingsTab,
    pub theme_mode: ThemeMode,
    pub theme_preset: ThemePreset,
    pub download_dir: String,
    pub music_dir: String,
    pub playlist_export_dir: String,
    pub spotify_client_id: String,
    pub manual_youtube_selection: bool,
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
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("Music")
            })
            .to_string_lossy()
            .to_string();

        let downloads = dirs::audio_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("Music")
            })
            .join("Downloads")
            .to_string_lossy()
            .to_string();

        let export = dirs::audio_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("/tmp"))
                    .join("Music")
            })
            .join("Playlists")
            .to_string_lossy()
            .to_string();

        let mut view = Self {
            active_tab: SettingsTab::Appearance,
            theme_mode: ThemeMode::Dark,
            theme_preset: ThemePreset::DefaultDark,
            download_dir: downloads,
            music_dir: music,
            playlist_export_dir: export,
            spotify_client_id: String::new(),
            manual_youtube_selection: false,
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

    pub fn set_active_tab(&mut self, tab: SettingsTab) {
        self.active_tab = tab;
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
        let is_target_dark = match mode {
            ThemeMode::Light => false,
            _ => true,
        };
        if self.theme_preset.is_dark() != is_target_dark {
            self.theme_preset = if is_target_dark {
                ThemePreset::DefaultDark
            } else {
                ThemePreset::DefaultLight
            };
        }
        self.is_saved = false;
    }

    pub fn set_theme_preset(&mut self, preset: ThemePreset) {
        self.theme_preset = preset;
        self.is_saved = false;
    }

    pub fn set_manual_youtube_selection(&mut self, manual: bool) {
        self.manual_youtube_selection = manual;
        self.is_saved = false;
    }

    pub fn set_spotify_client_id(&mut self, client_id: impl Into<String>) {
        self.spotify_client_id = client_id.into();
        self.is_saved = false;
    }

    pub fn set_download_dir(&mut self, dir: impl Into<String>) {
        self.download_dir = dir.into();
        self.is_saved = false;
    }

    pub fn set_music_dir(&mut self, dir: impl Into<String>) {
        self.music_dir = dir.into();
        self.is_saved = false;
    }

    pub fn set_playlist_export_dir(&mut self, dir: impl Into<String>) {
        self.playlist_export_dir = dir.into();
        self.is_saved = false;
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
        storage.set_setting(SETTING_PLAYLIST_EXPORT_DIR, &self.playlist_export_dir)?;
        storage.set_setting(
            SETTING_FUZZY_THRESHOLD,
            &format!("{:.2}", self.fuzzy_threshold),
        )?;
        storage.set_setting(
            SETTING_CONCURRENT_DOWNLOADS,
            &self.concurrent_downloads.to_string(),
        )?;
        storage.set_setting(SETTING_THEME_MODE, self.theme_mode.as_str())?;
        storage.set_setting(SETTING_THEME_PRESET, self.theme_preset.id())?;
        storage.set_setting(SETTING_SPOTIFY_CLIENT_ID, &self.spotify_client_id)?;
        storage.set_setting(
            SETTING_MANUAL_YOUTUBE_SELECTION,
            if self.manual_youtube_selection {
                "true"
            } else {
                "false"
            },
        )?;

        self.is_saved = true;
        self.status_message = Some("¡Configuración guardada correctamente!".into());
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
        if let Some(exp) = storage.get_setting(SETTING_PLAYLIST_EXPORT_DIR)? {
            self.playlist_export_dir = exp;
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
        if let Some(mode_str) = storage.get_setting(SETTING_THEME_MODE)? {
            self.theme_mode = ThemeMode::from_str(&mode_str);
        }
        if let Some(preset_str) = storage.get_setting(SETTING_THEME_PRESET)? {
            self.theme_preset = ThemePreset::from_str(&preset_str);
        }
        if let Some(client_id) = storage.get_setting(SETTING_SPOTIFY_CLIENT_ID)? {
            self.spotify_client_id = client_id;
        }
        if let Some(manual_str) = storage.get_setting(SETTING_MANUAL_YOUTUBE_SELECTION)? {
            self.manual_youtube_selection = manual_str == "true";
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
                    .child(Icon::Settings.element().size_5().text_color(colors::ACCENT_EMERALD))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::TEXT_PRIMARY)
                            .child("Configuración"),
                    ),
            );

        if let Some(ref status) = self.status_message {
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

        for tab in SettingsTab::ALL {
            let is_active = self.active_tab == tab;
            let mut tab_btn = div()
                .flex()
                .items_center()
                .gap_2p5()
                .px_3()
                .py_2()
                .rounded_md()
                .text_sm()
                .cursor_pointer();

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
        let content_pane = match self.active_tab {
            SettingsTab::Appearance => self.render_appearance_tab(),
            SettingsTab::Directories => self.render_directories_tab(),
            SettingsTab::Integrations => self.render_integrations_tab(),
            SettingsTab::Algorithm => self.render_algorithm_tab(),
        };

        layout = layout.child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_y_hidden()
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
            .child(Button::secondary("settings-btn-cancel", "Cancelar"))
            .child(Button::primary("settings-btn-save", "Guardar Cambios"));

        root = root.child(footer);

        root
    }

    fn render_appearance_tab(&self) -> Div {
        let is_effective_dark = match self.theme_mode {
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
                            .child("Elige si prefieres un esquema de tema claro, oscuro o sincronizado con el sistema."),
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
                                    .selected(self.theme_mode == ThemeMode::Dark),
                            )
                            .child(
                                Button::secondary("theme-mode-light", "Claro")
                                    .icon(Icon::Sun)
                                    .selected(self.theme_mode == ThemeMode::Light),
                            )
                            .child(
                                Button::secondary("theme-mode-system", "Sistema")
                                    .icon(Icon::Monitor)
                                    .selected(self.theme_mode == ThemeMode::System),
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
                            .children(presets.into_iter().map(|preset| {
                                let is_active = self.theme_preset == preset;
                                let swatches = preset.swatches();

                                let mut card = div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_3()
                                    .w_48()
                                    .rounded_lg()
                                    .bg(colors::BG_CARD)
                                    .border_1()
                                    .cursor_pointer();

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
                                        .child(if preset.is_dark() { "Tema Oscuro" } else { "Tema Claro" }),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .pt_1()
                                        .child(div().size_4().rounded_full().bg(swatches[0]).border_1().border_color(colors::BORDER_SUBTLE))
                                        .child(div().size_4().rounded_full().bg(swatches[1]).border_1().border_color(colors::BORDER_SUBTLE))
                                        .child(div().size_4().rounded_full().bg(swatches[2]).border_1().border_color(colors::BORDER_SUBTLE)),
                                )
                            })),
                    ),
            )
    }

    fn render_directories_tab(&self) -> Div {
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
                            .child(
                                div()
                                    .flex_1()
                                    .child(
                                        TextInput::new("settings-music-dir")
                                            .value(self.music_dir.clone())
                                            .placeholder("Ruta local...")
                                            .icon(Icon::Folder)
                                            .clearable(true),
                                    ),
                            )
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
                            .child(
                                div()
                                    .flex_1()
                                    .child(
                                        TextInput::new("settings-export-dir")
                                            .value(self.playlist_export_dir.clone())
                                            .placeholder("Ruta de exportación...")
                                            .icon(Icon::Folder)
                                            .clearable(true),
                                    ),
                            )
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
                            .child(
                                div()
                                    .flex_1()
                                    .child(
                                        TextInput::new("settings-download-dir")
                                            .value(self.download_dir.clone())
                                            .placeholder("Ruta de descargas MP3...")
                                            .icon(Icon::Folder)
                                            .clearable(true),
                                    ),
                            )
                            .child(Button::secondary("settings-btn-browse-download", "Buscar")),
                    ),
            )
    }

    fn render_integrations_tab(&self) -> Div {
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
                    .child(
                        TextInput::new("settings-spotify-client-id")
                            .value(self.spotify_client_id.clone())
                            .placeholder("Usar Client ID predeterminado...")
                            .icon(Icon::Key)
                            .clearable(true),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child("Opcional. Permite conectar tu propia clave de desarrollador en la API Web de Spotify."),
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
                            .flex()
                            .items_center()
                            .gap_2p5()
                            .cursor_pointer()
                            .child(
                                div()
                                    .size_4()
                                    .rounded_sm()
                                    .border_1()
                                    .border_color(if self.manual_youtube_selection {
                                        colors::ACCENT_EMERALD
                                    } else {
                                        colors::BORDER_BASE
                                    })
                                    .bg(if self.manual_youtube_selection {
                                        colors::ACCENT_EMERALD
                                    } else {
                                        colors::BG_SURFACE
                                    })
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(self.manual_youtube_selection, |this| {
                                        this.child(Icon::Check.element().size_3().text_color(colors::TEXT_PRIMARY))
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
                            .child("Muestra un diálogo de confirmación con 5 opciones de vídeo para seleccionar la versión preferida."),
                    ),
            )
    }

    fn render_algorithm_tab(&self) -> Div {
        let ytdlp_badge = if self.ytdlp_available {
            Badge::new("Disponible").variant(BadgeVariant::Success).icon(Icon::Check)
        } else {
            Badge::new("No encontrado").variant(BadgeVariant::Danger).icon(Icon::Close)
        };

        let ffmpeg_badge = if self.ffmpeg_available {
            Badge::new("Disponible").variant(BadgeVariant::Success).icon(Icon::Check)
        } else {
            Badge::new("No encontrado").variant(BadgeVariant::Danger).icon(Icon::Close)
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
                                    .child(format!("{:.0}%", self.fuzzy_threshold * 100.0)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Button::ghost("settings-fuzzy-minus", "").icon(Icon::Minus).small())
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
                                    .child(format!("{:.2}", self.fuzzy_threshold)),
                            )
                            .child(Button::ghost("settings-fuzzy-plus", "").icon(Icon::Plus).small()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child("Controla la precisión requerida al comparar nombres de pistas y artistas locales contra playlists remotas para detectar coincidencias parciales."),
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
                                    .child(format!("{} hilos", self.concurrent_downloads)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(Button::ghost("settings-conc-minus", "").icon(Icon::Minus).small())
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
                                    .child(self.concurrent_downloads.to_string()),
                            )
                            .child(Button::ghost("settings-conc-plus", "").icon(Icon::Plus).small()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child("Número de descargas simultáneas en paralelo recomendadas (1 - 5)."),
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
                                    .small(),
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
                                            .child(self.ytdlp_path.clone().unwrap_or_else(|| "No detectado en PATH ni runtime".into())),
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
                                            .child(self.ffmpeg_path.clone().unwrap_or_else(|| "No detectado en PATH ni runtime".into())),
                                    ),
                            )
                            .child(ffmpeg_badge),
                    ),
            )
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
        assert!(!view.playlist_export_dir.is_empty());
        assert_eq!(view.theme_mode, ThemeMode::Dark);
        assert_eq!(view.theme_preset, ThemePreset::DefaultDark);
        assert!(!view.manual_youtube_selection);
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
        let temp_dir =
            std::env::temp_dir().join(format!("mt_views_settings_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("settings.sqlite");
        let storage = CacheStorage::new(&db_path);

        let mut view = SettingsView::new();
        view.download_dir = "/custom/downloads".into();
        view.music_dir = "/custom/music".into();
        view.playlist_export_dir = "/custom/playlists".into();
        view.spotify_client_id = "test_client_id_123".into();
        view.manual_youtube_selection = true;
        view.set_theme_mode(ThemeMode::Light);
        view.set_theme_preset(ThemePreset::CatppuccinLatte);
        view.set_fuzzy_threshold(0.88);
        view.set_concurrent_downloads(4);

        assert!(view.save_to_storage(&storage).is_ok());
        assert!(view.is_saved);

        let mut loaded_view = SettingsView::new();
        assert!(loaded_view.load_from_storage(&storage).is_ok());
        assert_eq!(loaded_view.download_dir, "/custom/downloads");
        assert_eq!(loaded_view.music_dir, "/custom/music");
        assert_eq!(loaded_view.playlist_export_dir, "/custom/playlists");
        assert_eq!(loaded_view.spotify_client_id, "test_client_id_123");
        assert!(loaded_view.manual_youtube_selection);
        assert_eq!(loaded_view.theme_mode, ThemeMode::Light);
        assert_eq!(loaded_view.theme_preset, ThemePreset::CatppuccinLatte);
        assert_eq!(loaded_view.fuzzy_threshold, 0.88);
        assert_eq!(loaded_view.concurrent_downloads, 4);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
