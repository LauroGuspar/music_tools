use gpui::{Pixels, Rgba, px};

pub const fn hex_rgb(hex: u32) -> Rgba {
    let bytes = hex.to_be_bytes();
    Rgba {
        r: (bytes[1] as f32) / 255.0,
        g: (bytes[2] as f32) / 255.0,
        b: (bytes[3] as f32) / 255.0,
        a: 1.0,
    }
}

pub const fn hex_rgba(hex: u32) -> Rgba {
    let bytes = hex.to_be_bytes();
    Rgba {
        r: (bytes[0] as f32) / 255.0,
        g: (bytes[1] as f32) / 255.0,
        b: (bytes[2] as f32) / 255.0,
        a: (bytes[3] as f32) / 255.0,
    }
}

pub mod colors {
    use super::*;
    pub use super::{hex_rgb, hex_rgba};

    // Accent greens (Spotify and Emerald)
    pub const SPOTIFY_GREEN: Rgba = hex_rgb(0x1DB954);
    pub const ACCENT_EMERALD: Rgba = hex_rgb(0x10B981);
    pub const ACCENT_EMERALD_HOVER: Rgba = hex_rgb(0x34D399);
    pub const ACCENT_EMERALD_DARK: Rgba = hex_rgb(0x059669);

    // Dark backgrounds and surfaces
    pub const BG_BASE: Rgba = hex_rgb(0x121212);
    pub const BG_SURFACE: Rgba = hex_rgb(0x181818);
    pub const BG_CARD: Rgba = hex_rgb(0x242424);
    pub const BG_SURFACE_HOVER: Rgba = hex_rgb(0x242424);
    pub const BG_SURFACE_ACTIVE: Rgba = hex_rgb(0x2E2E2E);

    // Borders
    pub const BORDER_BASE: Rgba = hex_rgb(0x2E2E2E);
    pub const BORDER_SUBTLE: Rgba = hex_rgb(0x282828);
    pub const BORDER_ACCENT: Rgba = hex_rgb(0x10B981);

    // Typography
    pub const TEXT_PRIMARY: Rgba = hex_rgb(0xFFFFFF);
    pub const TEXT_SECONDARY: Rgba = hex_rgb(0xB3B3B3);
    pub const TEXT_MUTED: Rgba = hex_rgb(0x727272);

    // Status colors
    pub const STATUS_SUCCESS: Rgba = hex_rgb(0x10B981);
    pub const STATUS_WARNING: Rgba = hex_rgb(0xF59E0B);
    pub const STATUS_ERROR: Rgba = hex_rgb(0xEF4444);
    pub const STATUS_DANGER: Rgba = hex_rgb(0xEF4444);
    pub const STATUS_INFO: Rgba = hex_rgb(0x3B82F6);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub radius_xs: Pixels,
    pub radius_sm: Pixels,
    pub radius_md: Pixels,
    pub radius_lg: Pixels,
    pub radius_xl: Pixels,
    pub radius_full: Pixels,
    pub row_height: Pixels,
    pub header_height: Pixels,
    pub pad_xs: Pixels,
    pub pad_sm: Pixels,
    pub pad_md: Pixels,
    pub pad_lg: Pixels,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            radius_xs: px(2.),
            radius_sm: px(4.),
            radius_md: px(6.),
            radius_lg: px(8.),
            radius_xl: px(12.),
            radius_full: px(9999.),
            row_height: px(36.),
            header_height: px(40.),
            pad_xs: px(4.),
            pad_sm: px(8.),
            pad_md: px(12.),
            pad_lg: px(16.),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub bg_base: Rgba,
    pub bg_surface: Rgba,
    pub bg_card: Rgba,
    pub bg_surface_hover: Rgba,
    pub bg_surface_active: Rgba,
    pub border: Rgba,
    pub border_subtle: Rgba,
    pub border_accent: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
    pub accent_dark: Rgba,
    pub accent_spotify: Rgba,
    pub danger: Rgba,
    pub warning: Rgba,
    pub info: Rgba,
    pub metrics: Metrics,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    pub const fn dark() -> Self {
        Self {
            bg_base: colors::BG_BASE,
            bg_surface: colors::BG_SURFACE,
            bg_card: colors::BG_CARD,
            bg_surface_hover: colors::BG_SURFACE_HOVER,
            bg_surface_active: colors::BG_SURFACE_ACTIVE,
            border: colors::BORDER_BASE,
            border_subtle: colors::BORDER_SUBTLE,
            border_accent: colors::BORDER_ACCENT,
            text_primary: colors::TEXT_PRIMARY,
            text_secondary: colors::TEXT_SECONDARY,
            text_muted: colors::TEXT_MUTED,
            accent: colors::ACCENT_EMERALD,
            accent_hover: colors::ACCENT_EMERALD_HOVER,
            accent_dark: colors::ACCENT_EMERALD_DARK,
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: colors::STATUS_DANGER,
            warning: colors::STATUS_WARNING,
            info: colors::STATUS_INFO,
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn light() -> Self {
        Self {
            bg_base: hex_rgb(0xF8FAFC),
            bg_surface: hex_rgb(0xFFFFFF),
            bg_card: hex_rgb(0xF1F5F9),
            bg_surface_hover: hex_rgb(0xE2E8F0),
            bg_surface_active: hex_rgb(0xCBD5E1),
            border: hex_rgb(0xE2E8F0),
            border_subtle: hex_rgb(0xEEF2F6),
            border_accent: colors::ACCENT_EMERALD,
            text_primary: hex_rgb(0x0F172A),
            text_secondary: hex_rgb(0x475569),
            text_muted: hex_rgb(0x94A3B8),
            accent: hex_rgb(0x059669),
            accent_hover: hex_rgb(0x10B981),
            accent_dark: hex_rgb(0x047857),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: colors::STATUS_DANGER,
            warning: colors::STATUS_WARNING,
            info: colors::STATUS_INFO,
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn catppuccin_mocha() -> Self {
        Self {
            bg_base: hex_rgb(0x1E1E2E),
            bg_surface: hex_rgb(0x181825),
            bg_card: hex_rgb(0x313244),
            bg_surface_hover: hex_rgb(0x45475A),
            bg_surface_active: hex_rgb(0x585B70),
            border: hex_rgb(0x45475A),
            border_subtle: hex_rgb(0x313244),
            border_accent: hex_rgb(0xCBA6F7),
            text_primary: hex_rgb(0xCDD6F4),
            text_secondary: hex_rgb(0xA6ADC8),
            text_muted: hex_rgb(0x7F849C),
            accent: hex_rgb(0xCBA6F7),
            accent_hover: hex_rgb(0xDDB6F2),
            accent_dark: hex_rgb(0xBA94E8),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: hex_rgb(0xF38BA8),
            warning: hex_rgb(0xF9E2AF),
            info: hex_rgb(0x89B4FA),
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn catppuccin_latte() -> Self {
        Self {
            bg_base: hex_rgb(0xEFF1F5),
            bg_surface: hex_rgb(0xE6E9EF),
            bg_card: hex_rgb(0xDCE0E8),
            bg_surface_hover: hex_rgb(0xCCD0DA),
            bg_surface_active: hex_rgb(0xBCC0CC),
            border: hex_rgb(0xCCD0DA),
            border_subtle: hex_rgb(0xDCE0E8),
            border_accent: hex_rgb(0x8839EF),
            text_primary: hex_rgb(0x4C4F69),
            text_secondary: hex_rgb(0x5C5F77),
            text_muted: hex_rgb(0x8C8FA1),
            accent: hex_rgb(0x8839EF),
            accent_hover: hex_rgb(0x9A55F5),
            accent_dark: hex_rgb(0x7220E0),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: hex_rgb(0xD20F39),
            warning: hex_rgb(0xDF8E1D),
            info: hex_rgb(0x1E66F5),
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn dracula() -> Self {
        Self {
            bg_base: hex_rgb(0x282A36),
            bg_surface: hex_rgb(0x21222C),
            bg_card: hex_rgb(0x44475A),
            bg_surface_hover: hex_rgb(0x52566D),
            bg_surface_active: hex_rgb(0x6272A4),
            border: hex_rgb(0x6272A4),
            border_subtle: hex_rgb(0x44475A),
            border_accent: hex_rgb(0xFF79C6),
            text_primary: hex_rgb(0xF8F8F2),
            text_secondary: hex_rgb(0xBFBFBF),
            text_muted: hex_rgb(0x6272A4),
            accent: hex_rgb(0xFF79C6),
            accent_hover: hex_rgb(0xFF92D0),
            accent_dark: hex_rgb(0xE05BA7),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: hex_rgb(0xFF5555),
            warning: hex_rgb(0xFFB86C),
            info: hex_rgb(0x8BE9FD),
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn one_dark() -> Self {
        Self {
            bg_base: hex_rgb(0x282C34),
            bg_surface: hex_rgb(0x21252B),
            bg_card: hex_rgb(0x353B45),
            bg_surface_hover: hex_rgb(0x3E4451),
            bg_surface_active: hex_rgb(0x4B5263),
            border: hex_rgb(0x4B5263),
            border_subtle: hex_rgb(0x353B45),
            border_accent: hex_rgb(0x61AFEF),
            text_primary: hex_rgb(0xABB2BF),
            text_secondary: hex_rgb(0x828997),
            text_muted: hex_rgb(0x5C6370),
            accent: hex_rgb(0x61AFEF),
            accent_hover: hex_rgb(0x74BCF7),
            accent_dark: hex_rgb(0x4E9BD8),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: hex_rgb(0xE06C75),
            warning: hex_rgb(0xE5C07B),
            info: hex_rgb(0x61AFEF),
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn nord() -> Self {
        Self {
            bg_base: hex_rgb(0x2E3440),
            bg_surface: hex_rgb(0x242933),
            bg_card: hex_rgb(0x434C5E),
            bg_surface_hover: hex_rgb(0x4C566A),
            bg_surface_active: hex_rgb(0x5E6A82),
            border: hex_rgb(0x4C566A),
            border_subtle: hex_rgb(0x3B4252),
            border_accent: hex_rgb(0x88C0D0),
            text_primary: hex_rgb(0xECEFF4),
            text_secondary: hex_rgb(0xD8DEE9),
            text_muted: hex_rgb(0x4C566A),
            accent: hex_rgb(0x88C0D0),
            accent_hover: hex_rgb(0x9DD1DE),
            accent_dark: hex_rgb(0x72ABC0),
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: hex_rgb(0xBF616A),
            warning: hex_rgb(0xEBCB8B),
            info: hex_rgb(0x81A1C1),
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub const fn oled_black() -> Self {
        Self {
            bg_base: hex_rgb(0x000000),
            bg_surface: hex_rgb(0x0A0A0A),
            bg_card: hex_rgb(0x141414),
            bg_surface_hover: hex_rgb(0x1F1F1F),
            bg_surface_active: hex_rgb(0x2A2A2A),
            border: hex_rgb(0x222222),
            border_subtle: hex_rgb(0x181818),
            border_accent: colors::ACCENT_EMERALD,
            text_primary: hex_rgb(0xFFFFFF),
            text_secondary: hex_rgb(0xA0A0A0),
            text_muted: hex_rgb(0x606060),
            accent: colors::ACCENT_EMERALD,
            accent_hover: colors::ACCENT_EMERALD_HOVER,
            accent_dark: colors::ACCENT_EMERALD_DARK,
            accent_spotify: colors::SPOTIFY_GREEN,
            danger: colors::STATUS_DANGER,
            warning: colors::STATUS_WARNING,
            info: colors::STATUS_INFO,
            metrics: Metrics {
                radius_xs: px(2.),
                radius_sm: px(4.),
                radius_md: px(6.),
                radius_lg: px(8.),
                radius_xl: px(12.),
                radius_full: px(9999.),
                row_height: px(36.),
                header_height: px(40.),
                pad_xs: px(4.),
                pad_sm: px(8.),
                pad_md: px(12.),
                pad_lg: px(16.),
            },
        }
    }

    pub fn for_preset(preset_id: &str) -> Self {
        match preset_id {
            "default-light" | "light" => Self::light(),
            "catppuccin-mocha" => Self::catppuccin_mocha(),
            "catppuccin-latte" => Self::catppuccin_latte(),
            "dracula" => Self::dracula(),
            "one-dark" => Self::one_dark(),
            "nord" => Self::nord(),
            "oled-black" => Self::oled_black(),
            _ => Self::dark(),
        }
    }
}

/// Calculate relative luminance of an sRGB color (formula from WCAG 2.1)
pub fn relative_luminance(c: Rgba) -> f32 {
    let adjust = |val: f32| -> f32 {
        if val <= 0.03928 {
            val / 12.92
        } else {
            ((val + 0.055) / 1.055).powf(2.4)
        }
    };

    let r = adjust(c.r);
    let g = adjust(c.g);
    let b = adjust(c.b);

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Calculate contrast ratio between two colors (formula from WCAG 2.1: (L1 + 0.05) / (L2 + 0.05))
pub fn contrast_ratio(c1: Rgba, c2: Rgba) -> f32 {
    let l1 = relative_luminance(c1);
    let l2 = relative_luminance(c2);

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let theme = Theme::dark();
        assert_eq!(theme.bg_base, colors::BG_BASE);
        assert_eq!(theme.bg_surface, colors::BG_SURFACE);
        assert_eq!(theme.bg_card, colors::BG_CARD);
        assert_eq!(theme.border, colors::BORDER_BASE);
        assert_eq!(theme.accent, colors::ACCENT_EMERALD);
        assert_eq!(theme.accent_spotify, colors::SPOTIFY_GREEN);
        assert_eq!(theme.danger, colors::STATUS_DANGER);
        assert_eq!(theme.warning, colors::STATUS_WARNING);
        assert_eq!(theme.text_primary, colors::TEXT_PRIMARY);
        assert_eq!(theme.text_secondary, colors::TEXT_SECONDARY);
        assert_eq!(theme.text_muted, colors::TEXT_MUTED);
    }

    #[test]
    fn test_hex_rgb_calculation() {
        let white = hex_rgb(0xFFFFFF);
        assert!((white.r - 1.0).abs() < 1e-4);
        assert!((white.g - 1.0).abs() < 1e-4);
        assert!((white.b - 1.0).abs() < 1e-4);
        assert!((white.a - 1.0).abs() < 1e-4);

        let black = hex_rgb(0x000000);
        assert!((black.r - 0.0).abs() < 1e-4);
        assert!((black.g - 0.0).abs() < 1e-4);
        assert!((black.b - 0.0).abs() < 1e-4);
        assert!((black.a - 1.0).abs() < 1e-4);

        let emerald = hex_rgb(0x10B981);
        assert!((emerald.r - (0x10 as f32 / 255.0)).abs() < 1e-4);
        assert!((emerald.g - (0xB9 as f32 / 255.0)).abs() < 1e-4);
        assert!((emerald.b - (0x81 as f32 / 255.0)).abs() < 1e-4);
    }

    #[test]
    fn test_theme_contrast_and_accessibility() {
        let theme = Theme::dark();

        // White text on dark base should have very high contrast (> 12:1)
        let text_on_bg = contrast_ratio(theme.text_primary, theme.bg_base);
        assert!(text_on_bg > 12.0, "Text on background contrast is {text_on_bg}");

        // Text secondary on dark base should meet WCAG AA (> 4.5:1)
        let secondary_on_bg = contrast_ratio(theme.text_secondary, theme.bg_base);
        assert!(secondary_on_bg > 4.5, "Secondary text on bg contrast is {secondary_on_bg}");

        // Accent on dark base should be clearly legible
        let accent_on_bg = contrast_ratio(theme.accent, theme.bg_base);
        assert!(accent_on_bg > 4.0, "Accent on bg contrast is {accent_on_bg}");
    }

    #[test]
    fn test_metrics_radii_and_spacings() {
        let metrics = Metrics::default();
        assert_eq!(metrics.radius_xs, px(2.));
        assert_eq!(metrics.radius_sm, px(4.));
        assert_eq!(metrics.radius_md, px(6.));
        assert_eq!(metrics.radius_lg, px(8.));
        assert_eq!(metrics.radius_xl, px(12.));
        assert_eq!(metrics.row_height, px(36.));
        assert_eq!(metrics.header_height, px(40.));
    }
}
