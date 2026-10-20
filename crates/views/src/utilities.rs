use std::path::{Path, PathBuf};

use gpui::prelude::*;
use gpui::{
    Div, FontWeight, Pixels, Styled, div,
};

use scanner::{
    detect_exact_duplicates, detect_semantic_duplicates, scan_corrupt_files, CorruptFile,
    DuplicateGroup, DuplicateReason,
};
use ui::modal::Modal;
use ui::progress::{Badge, BadgeVariant};
use ui::theme::colors;
use ui::{Button, TextInput};

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UtilitySubTab {
    #[default]
    ExactDuplicates,
    SemanticDuplicates,
    CorruptFiles,
}

impl UtilitySubTab {
    pub const ALL: [Self; 3] = [
        Self::ExactDuplicates,
        Self::SemanticDuplicates,
        Self::CorruptFiles,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::ExactDuplicates => "exact_dups",
            Self::SemanticDuplicates => "semantic_dups",
            Self::CorruptFiles => "corrupt_files",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::ExactDuplicates => "Duplicados Exactos",
            Self::SemanticDuplicates => "Duplicados Semánticos",
            Self::CorruptFiles => "Archivos Dañados",
        }
    }
}

#[derive(Clone, Debug)]
pub struct UtilitiesView {
    pub library_path: String,
    pub is_scanning: bool,
    pub active_subtab: UtilitySubTab,
    pub exact_duplicates: Vec<DuplicateGroup>,
    pub semantic_duplicates: Vec<DuplicateGroup>,
    pub corrupt_files: Vec<CorruptFile>,
    pub pending_delete_path: Option<PathBuf>,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub scroll_top: Pixels,
    pub viewport_height: Pixels,
}

impl Default for UtilitiesView {
    fn default() -> Self {
        Self::new()
    }
}

impl UtilitiesView {
    pub fn new() -> Self {
        let default_path = dirs::audio_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Music"))
            .to_string_lossy()
            .to_string();

        Self {
            library_path: default_path,
            is_scanning: false,
            active_subtab: UtilitySubTab::ExactDuplicates,
            exact_duplicates: Vec::new(),
            semantic_duplicates: Vec::new(),
            corrupt_files: Vec::new(),
            pending_delete_path: None,
            status_message: None,
            error_message: None,
            scroll_top: Pixels::ZERO,
            viewport_height: gpui::px(500.),
        }
    }

    pub fn set_library_path(&mut self, path: impl Into<String>) {
        self.library_path = path.into();
    }

    pub fn set_active_subtab(&mut self, tab: UtilitySubTab) {
        self.active_subtab = tab;
        self.scroll_top = Pixels::ZERO;
    }

    pub fn scan(&mut self) {
        let path = Path::new(&self.library_path);
        if !path.exists() {
            self.error_message = Some(format!("La ruta '{}' no existe", self.library_path));
            return;
        }

        self.is_scanning = true;
        self.error_message = None;

        let scanned_tracks = scanner::scan_directory(path).unwrap_or_default();
        self.exact_duplicates = detect_exact_duplicates(&scanned_tracks);
        self.semantic_duplicates = detect_semantic_duplicates(&scanned_tracks, 0.85);

        if let Ok(corrupt) = scan_corrupt_files(path) {
            self.corrupt_files = corrupt;
        }

        self.status_message = Some(format!(
            "Escaneo completado: {} pistas analizadas. {} grupos exactos, {} semánticos, {} dañados",
            scanned_tracks.len(),
            self.exact_duplicates.len(),
            self.semantic_duplicates.len(),
            self.corrupt_files.len()
        ));
        self.is_scanning = false;
    }

    pub fn prompt_delete(&mut self, path: impl Into<PathBuf>) {
        self.pending_delete_path = Some(path.into());
    }

    pub fn cancel_delete(&mut self) {
        self.pending_delete_path = None;
    }

    pub fn confirm_delete(&mut self) -> anyhow::Result<bool> {
        if let Some(target) = self.pending_delete_path.take() {
            if target.exists() {
                std::fs::remove_file(&target)?;
            }

            // Remove from duplicate lists
            for group in &mut self.exact_duplicates {
                group.tracks.retain(|t| t.path.as_deref() != target.to_str());
            }
            self.exact_duplicates.retain(|g| g.tracks.len() > 1);

            for group in &mut self.semantic_duplicates {
                group.tracks.retain(|t| t.path.as_deref() != target.to_str());
            }
            self.semantic_duplicates.retain(|g| g.tracks.len() > 1);

            // Remove from corrupt list
            self.corrupt_files.retain(|f| f.path != target);

            self.status_message = Some(format!("Archivo eliminado: {}", target.display()));
            return Ok(true);
        }
        Ok(false)
    }

    pub fn exact_count(&self) -> usize {
        self.exact_duplicates.len()
    }

    pub fn semantic_count(&self) -> usize {
        self.semantic_duplicates.len()
    }

    pub fn corrupt_count(&self) -> usize {
        self.corrupt_files.len()
    }

    pub fn render_view(&self) -> Div {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        // 1. Directory Input + Scan Button
        let controls = div()
            .flex()
            .items_center()
            .gap_3()
            .p_3()
            .rounded_lg()
            .bg(colors::BG_SURFACE)
            .border_1()
            .border_color(colors::BORDER_SUBTLE)
            .child(
                div()
                    .flex_1()
                    .child(
                        TextInput::new("util-path-input")
                            .value(self.library_path.clone())
                            .placeholder("Ruta de la carpeta de música a analizar...")
                            .icon("📁")
                            .clearable(true),
                    ),
            )
            .child(
                Button::primary("util-btn-scan", "Escanear Biblioteca")
                    .icon("🔍")
                    .disabled(self.is_scanning || self.library_path.is_empty()),
            );

        root = root.child(controls);

        // Status or Error banner
        if let Some(err) = &self.error_message {
            root = root.child(
                div()
                    .px_4()
                    .py_2()
                    .rounded_md()
                    .bg(colors::hex_rgb(0x3B1818))
                    .border_1()
                    .border_color(colors::STATUS_DANGER)
                    .text_sm()
                    .text_color(colors::hex_rgb(0xFCA5A5))
                    .child(err.clone()),
            );
        } else if let Some(status) = &self.status_message {
            root = root.child(
                div()
                    .px_4()
                    .py_1()
                    .rounded_md()
                    .bg(colors::BG_SURFACE)
                    .border_1()
                    .border_color(colors::ACCENT_EMERALD)
                    .text_xs()
                    .text_color(colors::ACCENT_EMERALD)
                    .child(status.clone()),
            );
        }

        // 2. Sub-Tabs Header
        let mut tabs_row = div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(colors::BORDER_SUBTLE)
            .pb_2();

        for tab in UtilitySubTab::ALL {
            let is_active = self.active_subtab == tab;
            let (count, badge_variant) = match tab {
                UtilitySubTab::ExactDuplicates => (self.exact_count(), BadgeVariant::Warning),
                UtilitySubTab::SemanticDuplicates => (self.semantic_count(), BadgeVariant::Info),
                UtilitySubTab::CorruptFiles => (self.corrupt_count(), BadgeVariant::Danger),
            };

            let tab_btn = div()
                .flex()
                .items_center()
                .gap_2()
                .px_3()
                .py_1()
                .rounded_md()
                .cursor_pointer()
                .when(is_active, |this| {
                    this.bg(colors::BG_SURFACE_ACTIVE)
                        .border_1()
                        .border_color(colors::ACCENT_EMERALD)
                        .text_color(colors::TEXT_PRIMARY)
                })
                .when(!is_active, |this| {
                    this.bg(colors::BG_SURFACE)
                        .text_color(colors::TEXT_SECONDARY)
                })
                .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(tab.label()))
                .child(Badge::new(count.to_string()).variant(badge_variant));

            tabs_row = tabs_row.child(tab_btn);
        }

        root = root.child(tabs_row);

        // 3. Sub-Tab Content
        let content = match self.active_subtab {
            UtilitySubTab::ExactDuplicates => self.render_duplicates_list(&self.exact_duplicates, "exact"),
            UtilitySubTab::SemanticDuplicates => self.render_duplicates_list(&self.semantic_duplicates, "semantic"),
            UtilitySubTab::CorruptFiles => self.render_corrupt_list(),
        };

        root = root.child(content);

        // 4. Modal overlay if file deletion is pending
        if let Some(target) = &self.pending_delete_path {
            let modal = Modal::new("delete-file-modal", "Confirmar Eliminación")
                .description(format!(
                    "¿Estás seguro de que deseas eliminar permanentemente este archivo del disco?\n\n{}",
                    target.display()
                ))
                .action(
                    Button::ghost("modal-cancel-btn", "Cancelar")
                )
                .action(
                    Button::danger("modal-delete-btn", "Eliminar Definitivamente")
                );

            root = root.child(modal);
        }

        root
    }

    fn render_duplicates_list(&self, groups: &[DuplicateGroup], prefix: &str) -> Div {
        if groups.is_empty() {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .p_8()
                .text_sm()
                .text_color(colors::TEXT_MUTED)
                .child(div().text_2xl().child("✓"))
                .child("No se han detectado duplicados en esta categoría.");
        }

        let mut list = div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_3();

        for (g_idx, group) in groups.iter().enumerate() {
            let group_title = match group.reason {
                DuplicateReason::ExactBytes => format!("Grupo #{} — Contenido binario idéntico (SHA-256)", g_idx + 1),
                DuplicateReason::SemanticMatch => format!("Grupo #{} — Coincidencia por Artista/Título ({:.0}%)", g_idx + 1, group.score * 100.0),
            };

            let mut group_card = div()
                .p_3()
                .rounded_lg()
                .bg(colors::BG_SURFACE)
                .border_1()
                .border_color(colors::BORDER_SUBTLE)
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
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::ACCENT_EMERALD)
                                .child(group_title),
                        )
                        .child(
                            Badge::new(format!("{} archivos", group.tracks.len()))
                                .variant(BadgeVariant::Warning),
                        ),
                );

            for (t_idx, track) in group.tracks.iter().enumerate() {
                let path_str = track.path.clone().unwrap_or_else(|| "Sin ruta".into());
                let size_str = track.file_size.map(format_file_size).unwrap_or_else(|| "-".into());

                let track_row = div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_2()
                    .rounded_md()
                    .bg(colors::BG_CARD)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT_PRIMARY)
                                    .child(format!("{} - {}", track.artist_display(), track.title)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_MUTED)
                                    .child(path_str),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors::TEXT_SECONDARY)
                                    .child(size_str),
                            )
                            .child(
                                Button::ghost(format!("cp-{}-{}-{}", prefix, g_idx, t_idx), "Copiar")
                                    .small(),
                            )
                            .child(
                                Button::danger(format!("del-{}-{}-{}", prefix, g_idx, t_idx), "Eliminar")
                                    .small(),
                            ),
                    );

                group_card = group_card.child(track_row);
            }

            list = list.child(group_card);
        }

        list
    }

    fn render_corrupt_list(&self) -> Div {
        if self.corrupt_files.is_empty() {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .p_8()
                .text_sm()
                .text_color(colors::TEXT_MUTED)
                .child(div().text_2xl().child("✓"))
                .child("No se han detectado archivos dañados en la biblioteca.");
        }

        let mut list = div()
            .flex_1()
            .flex()
            .flex_col()
            .gap_2();

        for (idx, corrupt) in self.corrupt_files.iter().enumerate() {
            let item = div()
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
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::TEXT_PRIMARY)
                                        .child(corrupt.filename.clone()),
                                )
                                .child(
                                    Badge::new(format_file_size(corrupt.size))
                                        .variant(BadgeVariant::Danger),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::hex_rgb(0xF87171))
                                .child(format!("Motivo: {}", corrupt.error_reason)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(colors::TEXT_MUTED)
                                .child(corrupt.path.to_string_lossy().to_string()),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Button::ghost(format!("corrupt-cp-{}", idx), "Copiar")
                                .small(),
                        )
                        .child(
                            Button::danger(format!("corrupt-del-{}", idx), "Eliminar")
                                .small(),
                        ),
                );

            list = list.child(item);
        }

        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utilities_view_initial_state() {
        let view = UtilitiesView::new();
        assert!(!view.library_path.is_empty());
        assert_eq!(view.active_subtab, UtilitySubTab::ExactDuplicates);
        assert_eq!(view.exact_count(), 0);
        assert_eq!(view.semantic_count(), 0);
        assert_eq!(view.corrupt_count(), 0);
        assert!(view.pending_delete_path.is_none());
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2048), "2 KB");
        assert_eq!(format_file_size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(format_file_size(2 * 1024 * 1024 * 1024), "2.00 GB");
    }

    #[test]
    fn test_utilities_delete_confirmation_flow() {
        let mut view = UtilitiesView::new();
        let fake_path = PathBuf::from("/tmp/non_existent_music_file_test.mp3");

        view.prompt_delete(fake_path.clone());
        assert_eq!(view.pending_delete_path, Some(fake_path));

        view.cancel_delete();
        assert!(view.pending_delete_path.is_none());
    }
}
