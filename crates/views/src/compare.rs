use gpui::prelude::*;
use gpui::{
    Div, FontWeight, IntoElement, Pixels, Styled, div, px,
};

use core::{DownloadItem, Track};
use matcher::{compare_tracks, ComparisonResult, DEFAULT_THRESHOLD};
use ui::icon::Icon;
use ui::progress::{Badge, BadgeVariant};
use ui::table::{ColumnSpec, VirtualTable};
use ui::theme::colors;
use ui::{Button, TextInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompareSubTab {
    #[default]
    Exact,
    Probable,
    Missing,
    Extra,
}

impl CompareSubTab {
    pub const ALL: [Self; 4] = [
        Self::Exact,
        Self::Probable,
        Self::Missing,
        Self::Extra,
    ];

    pub fn id(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Probable => "probable",
            Self::Missing => "missing",
            Self::Extra => "extra",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Exact => "Exactas",
            Self::Probable => "Probables",
            Self::Missing => "Faltantes",
            Self::Extra => "Extras",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompareView {
    pub source_input: String,
    pub target_input: String,
    pub source_tracks: Vec<Track>,
    pub target_tracks: Vec<Track>,
    pub fuzzy_threshold: f64,
    pub comparison_result: Option<ComparisonResult>,
    pub active_subtab: CompareSubTab,
    pub is_comparing: bool,
    pub status_message: Option<String>,
    pub error_message: Option<String>,
    pub scroll_top: Pixels,
    pub viewport_height: Pixels,
}

impl Default for CompareView {
    fn default() -> Self {
        Self::new()
    }
}

impl CompareView {
    pub fn new() -> Self {
        Self {
            source_input: String::new(),
            target_input: String::new(),
            source_tracks: Vec::new(),
            target_tracks: Vec::new(),
            fuzzy_threshold: DEFAULT_THRESHOLD,
            comparison_result: None,
            active_subtab: CompareSubTab::Exact,
            is_comparing: false,
            status_message: None,
            error_message: None,
            scroll_top: Pixels::ZERO,
            viewport_height: px(500.),
        }
    }

    pub fn with_tracks(source: Vec<Track>, target: Vec<Track>) -> Self {
        let mut view = Self::new();
        view.source_tracks = source;
        view.target_tracks = target;
        view
    }

    pub fn set_fuzzy_threshold(&mut self, threshold: f64) {
        self.fuzzy_threshold = threshold.clamp(0.50, 1.00);
    }

    pub fn set_active_subtab(&mut self, subtab: CompareSubTab) {
        self.active_subtab = subtab;
        self.scroll_top = Pixels::ZERO;
    }

    pub fn compare(&mut self) {
        if self.source_tracks.is_empty() && self.target_tracks.is_empty() {
            self.error_message = Some("Cargue pistas en Fuente o Destino para comparar".into());
            return;
        }

        self.is_comparing = true;
        self.error_message = None;

        let result = compare_tracks(&self.source_tracks, &self.target_tracks, self.fuzzy_threshold);
        self.status_message = Some(format!(
            "Comparación finalizada: {} exactas, {} probables, {} faltantes, {} extras",
            result.exact_matches.len(),
            result.probable_matches.len(),
            result.missing.len(),
            result.extra.len(),
        ));
        self.comparison_result = Some(result);
        self.is_comparing = false;
    }

    pub fn exact_count(&self) -> usize {
        self.comparison_result.as_ref().map(|r| r.exact_matches.len()).unwrap_or(0)
    }

    pub fn probable_count(&self) -> usize {
        self.comparison_result.as_ref().map(|r| r.probable_matches.len()).unwrap_or(0)
    }

    pub fn missing_count(&self) -> usize {
        self.comparison_result.as_ref().map(|r| r.missing.len()).unwrap_or(0)
    }

    pub fn extra_count(&self) -> usize {
        self.comparison_result.as_ref().map(|r| r.extra.len()).unwrap_or(0)
    }

    pub fn missing_tracks(&self) -> &[Track] {
        self.comparison_result
            .as_ref()
            .map(|r| r.missing.as_slice())
            .unwrap_or(&[])
    }

    pub fn export_missing_to_downloads(&self) -> Vec<DownloadItem> {
        self.missing_tracks()
            .iter()
            .enumerate()
            .map(|(idx, track)| {
                let id = track
                    .spotify_id
                    .clone()
                    .unwrap_or_else(|| format!("cmp-miss-{}", idx + 1));
                DownloadItem::new(id, track.title.clone(), track.artist_display())
            })
            .collect()
    }

    pub fn render_view(&self) -> Div {
        let mut root = div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors::BG_BASE)
            .p_4()
            .gap_3();

        // 1. Selector Bar: Source vs Destination + Fuzzy Threshold + Compare Button
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
                    // Source input
                    .child(
                        div()
                            .flex_1()
                            .child(
                                TextInput::new("cmp-source-input")
                                    .value(self.source_input.clone())
                                    .placeholder("Fuente: Carpeta local o URL de Spotify (Colección deseada)...")
                                    .icon(Icon::Folder)
                                    .clearable(true),
                            ),
                    )
                    .child(
                        Icon::Compare.element().size_4().text_color(colors::TEXT_MUTED),
                    )
                    // Destination input
                    .child(
                        div()
                            .flex_1()
                            .child(
                                TextInput::new("cmp-target-input")
                                    .value(self.target_input.clone())
                                    .placeholder("Destino: Carpeta local o Biblioteca actual...")
                                    .icon(Icon::FolderOpen)
                                    .clearable(true),
                            ),
                    ),
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
                                    .child(format!("{:.0}% ({:.2})", self.fuzzy_threshold * 100.0, self.fuzzy_threshold)),
                            )
                            .child(
                                Button::ghost("btn-threshold-minus", "")
                                    .icon(Icon::Minus)
                                    .small(),
                            )
                            .child(
                                Button::ghost("btn-threshold-plus", "")
                                    .icon(Icon::Plus)
                                    .small(),
                            ),
                    )
                    // Action Buttons
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::primary("cmp-btn-run", "Comparar")
                                    .icon(Icon::Compare)
                                    .disabled(self.is_comparing),
                            )
                            .child(
                                Button::secondary("cmp-btn-export-missing", "Enviar Faltantes a la Cola")
                                    .icon(Icon::Download)
                                    .disabled(self.missing_count() == 0),
                            ),
                    ),
            );

        root = root.child(controls_box);

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

        // 2. Result Sub-Tabs
        let exact_count = self.exact_count();
        let probable_count = self.probable_count();
        let missing_count = self.missing_count();
        let extra_count = self.extra_count();

        let mut tabs_row = div()
            .flex()
            .items_center()
            .gap_2()
            .border_b_1()
            .border_color(colors::BORDER_SUBTLE)
            .pb_2();

        for tab in CompareSubTab::ALL {
            let is_active = self.active_subtab == tab;
            let (label, count, badge_variant) = match tab {
                CompareSubTab::Exact => ("Exactas", exact_count, BadgeVariant::Success),
                CompareSubTab::Probable => ("Probables", probable_count, BadgeVariant::Warning),
                CompareSubTab::Missing => ("Faltantes", missing_count, BadgeVariant::Danger),
                CompareSubTab::Extra => ("Extras", extra_count, BadgeVariant::Info),
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
                .child(div().text_sm().font_weight(FontWeight::MEDIUM).child(label))
                .child(Badge::new(count.to_string()).variant(badge_variant));

            tabs_row = tabs_row.child(tab_btn);
        }

        root = root.child(tabs_row);

        // 3. Tab Content: Table of Results or Empty State
        if self.comparison_result.is_none() {
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
                .child(Icon::Compare.element().size_8().text_color(colors::ACCENT_EMERALD))
                .child(
                    div()
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT_PRIMARY)
                        .child("Sin comparación activa"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(colors::TEXT_SECONDARY)
                        .child("Presiona 'Comparar' para cotejar la colección de origen contra el destino."),
                );
            root = root.child(empty);
        } else if let Some(res) = &self.comparison_result {
            let content = match self.active_subtab {
                CompareSubTab::Exact => render_exact_matches_table(&res.exact_matches, "exact"),
                CompareSubTab::Probable => render_probable_matches_table(&res.probable_matches, "probable"),
                CompareSubTab::Missing => render_tracks_table(&res.missing, "missing"),
                CompareSubTab::Extra => render_tracks_table(&res.extra, "extra"),
            };
            root = root.child(content);
        }

        root
    }
}

fn render_exact_matches_table(matches: &[(Track, Track)], id_prefix: &str) -> Div {
    if matches.is_empty() {
        return div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .text_sm()
            .text_color(colors::TEXT_MUTED)
            .child("No hay coincidencias exactas.");
    }

    let cols = vec![
        ColumnSpec::new("index".to_string(), "#")
            .fixed(px(50.))
            .sortable(false),
        ColumnSpec::new("source".to_string(), "Pista Fuente")
            .fill(2.5),
        ColumnSpec::new("target".to_string(), "Coincidencia Destino")
            .fill(2.5),
        ColumnSpec::new("similarity".to_string(), "Similitud")
            .fixed(px(90.)),
        ColumnSpec::new("type".to_string(), "Tipo")
            .fixed(px(100.)),
    ];

    let table = VirtualTable::new(format!("{}-table", id_prefix))
        .columns(cols)
        .rows(matches.to_vec())
        .row_height(px(40.))
        .cell_builder(|idx, (source, matched): &(Track, Track), col_idx, _col, _width| {
            match col_idx {
                0 => div()
                    .text_xs()
                    .text_color(colors::TEXT_MUTED)
                    .child(format!("{}", idx + 1))
                    .into_any_element(),
                1 => div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(source.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(source.artist_display()),
                    )
                    .into_any_element(),
                2 => div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(matched.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(matched.artist_display()),
                    )
                    .into_any_element(),
                3 => div()
                    .text_xs()
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors::ACCENT_EMERALD)
                    .child("100%")
                    .into_any_element(),
                4 => Badge::exact().into_any_element(),
                _ => div().into_any_element(),
            }
        });

    div().flex_1().child(table)
}

fn render_probable_matches_table(matches: &[(Track, Track, f64)], id_prefix: &str) -> Div {
    if matches.is_empty() {
        return div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .text_sm()
            .text_color(colors::TEXT_MUTED)
            .child("No hay coincidencias probables.");
    }

    let cols = vec![
        ColumnSpec::new("index".to_string(), "#")
            .fixed(px(50.))
            .sortable(false),
        ColumnSpec::new("source".to_string(), "Pista Fuente")
            .fill(2.5),
        ColumnSpec::new("target".to_string(), "Coincidencia Destino")
            .fill(2.5),
        ColumnSpec::new("similarity".to_string(), "Similitud")
            .fixed(px(90.)),
        ColumnSpec::new("type".to_string(), "Tipo")
            .fixed(px(100.)),
    ];

    let table = VirtualTable::new(format!("{}-table", id_prefix))
        .columns(cols)
        .rows(matches.to_vec())
        .row_height(px(40.))
        .cell_builder(|idx, (source, matched, score): &(Track, Track, f64), col_idx, _col, _width| {
            match col_idx {
                0 => div()
                    .text_xs()
                    .text_color(colors::TEXT_MUTED)
                    .child(format!("{}", idx + 1))
                    .into_any_element(),
                1 => div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(source.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(source.artist_display()),
                    )
                    .into_any_element(),
                2 => div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT_PRIMARY)
                            .child(matched.title.clone()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors::TEXT_SECONDARY)
                            .child(matched.artist_display()),
                    )
                    .into_any_element(),
                3 => {
                    let score_pct = format!("{:.0}%", score * 100.0);
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::STATUS_WARNING)
                        .child(score_pct)
                        .into_any_element()
                }
                4 => Badge::probable().into_any_element(),
                _ => div().into_any_element(),
            }
        });

    div().flex_1().child(table)
}

fn render_tracks_table(tracks: &[Track], id_prefix: &str) -> Div {
    if tracks.is_empty() {
        return div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .p_8()
            .text_sm()
            .text_color(colors::TEXT_MUTED)
            .child("Lista vacía en esta categoría.");
    }

    let cols = vec![
        ColumnSpec::new("index".to_string(), "#")
            .fixed(px(50.))
            .sortable(false),
        ColumnSpec::new("title".to_string(), "Título")
            .fill(2.5),
        ColumnSpec::new("artists".to_string(), "Artistas")
            .fill(2.0),
        ColumnSpec::new("album".to_string(), "Álbum")
            .fill(1.5),
        ColumnSpec::new("duration".to_string(), "Duración")
            .fixed(px(85.)),
    ];

    let table = VirtualTable::new(format!("{}-table", id_prefix))
        .columns(cols)
        .rows(tracks.to_vec())
        .row_height(px(38.))
        .cell_builder(|idx, track: &Track, col_idx, _col, _width| {
            match col_idx {
                0 => div()
                    .text_xs()
                    .text_color(colors::TEXT_MUTED)
                    .child(format!("{}", idx + 1))
                    .into_any_element(),
                1 => div()
                    .text_sm()
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
                    .text_color(colors::TEXT_SECONDARY)
                    .child(crate::staging::format_duration_ms(track.duration_ms))
                    .into_any_element(),
                _ => div().into_any_element(),
            }
        });

    div().flex_1().child(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_view_initial_state() {
        let view = CompareView::new();
        assert_eq!(view.fuzzy_threshold, DEFAULT_THRESHOLD);
        assert_eq!(view.active_subtab, CompareSubTab::Exact);
        assert!(view.comparison_result.is_none());
        assert_eq!(view.exact_count(), 0);
        assert_eq!(view.missing_count(), 0);
    }

    #[test]
    fn test_compare_tracks_execution() {
        let source = vec![
            Track::new("Bohemian Rhapsody", vec!["Queen".into()], 354_000),
            Track::new("Hoteel California", vec!["Eagles".into()], 390_000), // probable typo
            Track::new("Missing Track", vec!["Unknown".into()], 120_000),
        ];

        let target = vec![
            Track::new("Bohemian Rhapsody", vec!["Queen".into()], 354_000), // exact
            Track::new("Hotel California", vec!["Eagles".into()], 390_000), // matched
            Track::new("Extra Track", vec!["Bonus".into()], 200_000), // extra
        ];

        let mut view = CompareView::with_tracks(source, target);
        view.compare();

        assert!(view.comparison_result.is_some());
        assert_eq!(view.exact_count(), 1);
        assert_eq!(view.probable_count(), 1);
        assert_eq!(view.missing_count(), 1);
        assert_eq!(view.extra_count(), 1);

        let missing = view.export_missing_to_downloads();
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].title, "Missing Track");
    }

    #[test]
    fn test_fuzzy_threshold_clamping() {
        let mut view = CompareView::new();
        view.set_fuzzy_threshold(0.2);
        assert_eq!(view.fuzzy_threshold, 0.50);

        view.set_fuzzy_threshold(1.5);
        assert_eq!(view.fuzzy_threshold, 1.00);

        view.set_fuzzy_threshold(0.85);
        assert_eq!(view.fuzzy_threshold, 0.85);
    }
}
