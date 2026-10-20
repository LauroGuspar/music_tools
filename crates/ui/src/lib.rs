use gpui::Rgba;

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

    // Acentos verdes (Spotify y Esmeralda)
    pub const SPOTIFY_GREEN: Rgba = hex_rgb(0x1DB954);
    pub const ACCENT_EMERALD: Rgba = hex_rgb(0x10B981);
    pub const ACCENT_EMERALD_HOVER: Rgba = hex_rgb(0x34D399);
    pub const ACCENT_EMERALD_DARK: Rgba = hex_rgb(0x059669);

    // Fondo oscuro y superficies
    pub const BG_BASE: Rgba = hex_rgb(0x121212);
    pub const BG_SURFACE: Rgba = hex_rgb(0x181818);
    pub const BG_SURFACE_HOVER: Rgba = hex_rgb(0x242424);
    pub const BG_SURFACE_ACTIVE: Rgba = hex_rgb(0x2E2E2E);

    // Bordes
    pub const BORDER_SUBTLE: Rgba = hex_rgb(0x282828);
    pub const BORDER_ACCENT: Rgba = hex_rgb(0x10B981);

    // Tipografía
    pub const TEXT_PRIMARY: Rgba = hex_rgb(0xFFFFFF);
    pub const TEXT_SECONDARY: Rgba = hex_rgb(0xB3B3B3);
    pub const TEXT_MUTED: Rgba = hex_rgb(0x727272);

    // Estados
    pub const STATUS_SUCCESS: Rgba = hex_rgb(0x10B981);
    pub const STATUS_WARNING: Rgba = hex_rgb(0xF59E0B);
    pub const STATUS_ERROR: Rgba = hex_rgb(0xEF4444);
    pub const STATUS_INFO: Rgba = hex_rgb(0x3B82F6);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub bg_base: Rgba,
    pub bg_surface: Rgba,
    pub bg_surface_hover: Rgba,
    pub border: Rgba,
    pub text_primary: Rgba,
    pub text_secondary: Rgba,
    pub text_muted: Rgba,
    pub accent: Rgba,
    pub accent_hover: Rgba,
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
            bg_surface_hover: colors::BG_SURFACE_HOVER,
            border: colors::BORDER_SUBTLE,
            text_primary: colors::TEXT_PRIMARY,
            text_secondary: colors::TEXT_SECONDARY,
            text_muted: colors::TEXT_MUTED,
            accent: colors::ACCENT_EMERALD,
            accent_hover: colors::ACCENT_EMERALD_HOVER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_colors() {
        let theme = Theme::dark();
        assert_eq!(theme.bg_base, colors::BG_BASE);
        assert_eq!(theme.accent, colors::ACCENT_EMERALD);
        assert_eq!(theme.text_primary, colors::TEXT_PRIMARY);
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
    }
}
