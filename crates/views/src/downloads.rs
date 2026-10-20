use std::path::PathBuf;

use gpui::prelude::*;
use gpui::{
    Div, FontWeight, IntoElement, Pixels, Styled, div, px,
};

use core::{DownloadItem, DownloadStatus};
use ui::progress::{Badge, BadgeVariant, ProgressBar};
use ui::table::{ColumnSpec, TableSelection, VirtualTable};
use ui::theme::colors;
use ui::Button;

#[derive(Clone, Debug)]
pub struct DownloadsView {
    pub items: Vec<DownloadItem>,
    pub download_dir: PathBuf,
    pub is_paused: bool,
    pub status_filter: Option<DownloadStatus>,
    pub selection: TableSelection,
    pub scroll_top: Pixels,
    pub viewport_height: Pixels,
}

impl Default for DownloadsView {
    fn default() -> Self {
        Self::new()
    }
}

impl DownloadsView {
    pub fn new() -> Self {
        let default_dir = dirs::audio_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("Music"))
            .join("Downloads");

        Self {
            items: Vec::new(),
            download_dir: default_dir,
            is_paused: false,
            status_filter: None,
            selection: TableSelection::new(),
            scroll_top: Pixels::ZERO,
            viewport_height: px(500.),
        }
    }

    pub fn with_items(items: Vec<DownloadItem>) -> Self {
        let mut view = Self::new();
        view.items = items;
        view
    }

    pub fn set_download_dir(&mut self, dir: impl Into<PathBuf>) {
        self.download_dir = dir.into();
    }

    pub fn add_item(&mut self, item: DownloadItem) {
        if !self.items.iter().any(|i| i.id == item.id) {
            self.items.push(item);
        }
    }

    pub fn add_items(&mut self, items: Vec<DownloadItem>) {
        for item in items {
            self.add_item(item);
        }
    }

    pub fn pause_all(&mut self) {
        self.is_paused = true;
        for item in &mut self.items {
            if item.status == DownloadStatus::Downloading {
                item.status = DownloadStatus::Queued;
            }
        }
    }

    pub fn resume_all(&mut self) {
        self.is_paused = false;
    }

    pub fn retry_failed(&mut self) {
        for item in &mut self.items {
            if item.status == DownloadStatus::Failed || item.status == DownloadStatus::Cancelled {
                item.status = DownloadStatus::Queued;
                item.progress_percent = 0.0;
                item.speed = None;
                item.eta = None;
            }
        }
    }

    pub fn clear_completed(&mut self) {
        self.items.retain(|i| i.status != DownloadStatus::Completed);
        self.selection.clear();
    }

    pub fn cancel_item(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = DownloadStatus::Cancelled;
        }
    }

    pub fn retry_item(&mut self, id: &str) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = DownloadStatus::Queued;
            item.progress_percent = 0.0;
            item.speed = None;
            item.eta = None;
        }
    }

    pub fn remove_item(&mut self, id: &str) {
        self.items.retain(|i| i.id != id);
        self.selection.clear();
    }

    pub fn update_progress(
        &mut self,
        id: &str,
        percent: f32,
        speed: Option<String>,
        eta: Option<String>,
    ) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.progress_percent = percent.clamp(0.0, 100.0);
            item.speed = speed;
            item.eta = eta;
            if item.progress_percent >= 100.0 {
                item.status = DownloadStatus::Completed;
            } else if item.status == DownloadStatus::Queued {
                item.status = DownloadStatus::Downloading;
            }
        }
    }

    pub fn apply_progress(&mut self, progress: &downloader::DownloadProgress) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == progress.id) {
            item.status = progress.status;
            item.progress_percent = progress.percent.clamp(0.0, 100.0);
            item.speed = progress.speed.clone();
            item.eta = progress.eta.clone();
        } else {
            self.items.push(progress.to_item());
        }
    }

    pub fn set_status(&mut self, id: &str, status: DownloadStatus) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.status = status;
        }
    }

    // Statistics counts
    pub fn total_count(&self) -> usize {
        self.items.len()
    }

    pub fn queued_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == DownloadStatus::Queued).count()
    }

    pub fn downloading_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == DownloadStatus::Downloading).count()
    }

    pub fn completed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == DownloadStatus::Completed).count()
    }

    pub fn failed_count(&self) -> usize {
        self.items.iter().filter(|i| i.status == DownloadStatus::Failed).count()
    }

    pub fn render_view(&self) -> Div {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        // 1. Top Controls Bar
        let total = self.total_count();
        let queued = self.queued_count();
        let downloading = self.downloading_count();
        let completed = self.completed_count();
        let failed = self.failed_count();

        let dir_display = self.download_dir.to_string_lossy().to_string();

        let controls_bar = div()
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
                    .gap_2()
                    .child(
                        if self.is_paused {
                            Button::primary("dl-btn-resume", "Reanudar Todo")
                                .icon("▶")
                                .disabled(total == 0)
                        } else {
                            Button::secondary("dl-btn-pause", "Pausar Todo")
                                .icon("⏸")
                                .disabled(downloading == 0 && queued == 0)
                        },
                    )
                    .child(
                        Button::secondary("dl-btn-retry-failed", "Reintentar Fallidas")
                            .icon("↺")
                            .disabled(failed == 0),
                    )
                    .child(
                        Button::ghost("dl-btn-clear-completed", "Limpiar Completadas")
                            .icon("🗑")
                            .disabled(completed == 0),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_xs()
                    .text_color(colors::TEXT_SECONDARY)
                    .child(
                        div()
                            .text_color(colors::TEXT_MUTED)
                            .child("Directorio:"),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(colors::BG_BASE)
                            .border_1()
                            .border_color(colors::BORDER_BASE)
                            .text_color(colors::ACCENT_EMERALD)
                            .font_weight(FontWeight::MEDIUM)
                            .child(dir_display),
                    ),
            );

        root = root.child(controls_bar);

        // 2. Statistics Pills Bar
        let stats_bar = div()
            .flex()
            .items_center()
            .gap_3()
            .px_3()
            .py_2()
            .rounded_lg()
            .bg(colors::BG_CARD)
            .text_xs()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).text_color(colors::TEXT_PRIMARY).child(format!("{}", total)))
                    .child(div().text_color(colors::TEXT_MUTED).child("totales")),
            )
            .child(div().text_color(colors::BORDER_BASE).child("•"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).text_color(colors::TEXT_SECONDARY).child(format!("{}", queued)))
                    .child(div().text_color(colors::TEXT_MUTED).child("en cola")),
            )
            .child(div().text_color(colors::BORDER_BASE).child("•"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).text_color(colors::ACCENT_EMERALD).child(format!("{}", downloading)))
                    .child(div().text_color(colors::TEXT_MUTED).child("descargando")),
            )
            .child(div().text_color(colors::BORDER_BASE).child("•"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).text_color(colors::STATUS_SUCCESS).child(format!("{}", completed)))
                    .child(div().text_color(colors::TEXT_MUTED).child("completadas")),
            )
            .child(div().text_color(colors::BORDER_BASE).child("•"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(div().font_weight(FontWeight::SEMIBOLD).text_color(colors::STATUS_DANGER).child(format!("{}", failed)))
                    .child(div().text_color(colors::TEXT_MUTED).child("fallidas")),
            );

        root = root.child(stats_bar);

        // 3. Downloads Table or Empty State
        if self.items.is_empty() {
            let empty = div()
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
                .child(div().text_3xl().child("↓"))
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT_PRIMARY)
                        .child("Cola de Descargas Vacía"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::TEXT_SECONDARY)
                        .child("Añade canciones desde la Mesa de Trabajo o desde el Comparador para iniciar descargas."),
                );
            root = root.child(empty);
        } else {
            let cols = vec![
                ColumnSpec::new("index".to_string(), "#")
                    .fixed(px(50.))
                    .sortable(false),
                ColumnSpec::new("title".to_string(), "Título")
                    .fill(2.2),
                ColumnSpec::new("artist".to_string(), "Artista")
                    .fill(1.8),
                ColumnSpec::new("status".to_string(), "Estado")
                    .fixed(px(130.)),
                ColumnSpec::new("progress".to_string(), "Progreso")
                    .fill(2.0),
                ColumnSpec::new("actions".to_string(), "Acciones")
                    .fixed(px(110.))
                    .sortable(false),
            ];

            let table = VirtualTable::new("downloads-table")
                .columns(cols)
                .rows(self.items.clone())
                .row_height(px(44.))
                .scroll_top(self.scroll_top)
                .viewport_height(self.viewport_height)
                .cell_builder(|idx, item: &DownloadItem, col_idx, _col, _width| {
                    match col_idx {
                        0 => div()
                            .text_xs()
                            .text_color(colors::TEXT_MUTED)
                            .child(format!("{}", idx + 1))
                            .into_any_element(),
                        1 => div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(item.title.clone())
                            .into_any_element(),
                        2 => div()
                            .text_sm()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(item.artist.clone())
                            .into_any_element(),
                        3 => {
                            let badge = match item.status {
                                DownloadStatus::Queued => Badge::queued(),
                                DownloadStatus::Downloading => Badge::downloading(),
                                DownloadStatus::Completed => Badge::completed(),
                                DownloadStatus::Failed => Badge::failed(),
                                DownloadStatus::Cancelled => Badge::new("Cancelada").variant(BadgeVariant::Neutral),
                            };
                            badge.into_any_element()
                        }
                        4 => {
                            let mut detail_str = String::new();
                            if let Some(spd) = &item.speed {
                                detail_str.push_str(spd);
                            }
                            if let Some(eta) = &item.eta {
                                if !detail_str.is_empty() {
                                    detail_str.push_str(" • ETA ");
                                }
                                detail_str.push_str(eta);
                            }

                            let mut pb = ProgressBar::new(format!("pb-{}", item.id), item.progress_percent);
                            if !detail_str.is_empty() {
                                pb = pb.detail(detail_str);
                            }
                            pb.into_any_element()
                        }
                        5 => {
                            let action_btn = match item.status {
                                DownloadStatus::Downloading | DownloadStatus::Queued => {
                                    Button::ghost(format!("dl-cancel-{}", item.id), "Cancelar")
                                        .small()
                                }
                                DownloadStatus::Failed | DownloadStatus::Cancelled => {
                                    Button::secondary(format!("dl-retry-{}", item.id), "Reintentar")
                                        .small()
                                }
                                DownloadStatus::Completed => {
                                    Button::ghost(format!("dl-del-{}", item.id), "✕")
                                        .small()
                                }
                            };
                            action_btn.into_any_element()
                        }
                        _ => div().into_any_element(),
                    }
                });

            root = root.child(div().flex_1().child(table));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloads_view_initial_state() {
        let view = DownloadsView::new();
        assert_eq!(view.total_count(), 0);
        assert_eq!(view.queued_count(), 0);
        assert_eq!(view.downloading_count(), 0);
        assert_eq!(view.completed_count(), 0);
        assert_eq!(view.failed_count(), 0);
        assert!(!view.is_paused);
    }

    #[test]
    fn test_downloads_add_and_counts() {
        let mut view = DownloadsView::new();
        view.add_item(DownloadItem::new("dl-1", "Track 1", "Artist 1"));
        view.add_item(DownloadItem::new("dl-2", "Track 2", "Artist 2"));

        assert_eq!(view.total_count(), 2);
        assert_eq!(view.queued_count(), 2);

        view.update_progress("dl-1", 50.0, Some("2.5 MB/s".into()), Some("00:15".into()));
        assert_eq!(view.downloading_count(), 1);
        assert_eq!(view.items[0].speed.as_deref(), Some("2.5 MB/s"));

        view.update_progress("dl-1", 100.0, None, None);
        assert_eq!(view.completed_count(), 1);

        view.set_status("dl-2", DownloadStatus::Failed);
        assert_eq!(view.failed_count(), 1);

        view.retry_failed();
        assert_eq!(view.queued_count(), 1);
        assert_eq!(view.failed_count(), 0);

        view.clear_completed();
        assert_eq!(view.total_count(), 1);
    }

    #[test]
    fn test_downloads_pause_and_resume_all() {
        let mut view = DownloadsView::new();
        let mut item1 = DownloadItem::new("dl-1", "Track 1", "Artist 1");
        item1.status = DownloadStatus::Downloading;
        view.add_item(item1);

        view.pause_all();
        assert!(view.is_paused);
        assert_eq!(view.items[0].status, DownloadStatus::Queued);

        view.resume_all();
        assert!(!view.is_paused);
    }
}
