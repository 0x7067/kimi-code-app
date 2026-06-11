//! Color tokens for the Kimi Code Desktop design system.
//!
//! Dark mode only. All colors are defined as `&'static str` hex / rgba
//! literals so they can be used directly in inline Tailwind-style
//! `class:` attributes or CSS custom properties.

pub struct Colors;

#[allow(dead_code)]
impl Colors {
    // Kimi Brand
    pub const KIMI_BLUE: &str = "#1E90FF";
    pub const KIMI_BLUE_HOVER: &str = "#4AA8FF";
    pub const KIMI_BLUE_MUTED: &str = "rgba(30, 144, 255, 0.2)";

    // Backgrounds (dark theme only)
    pub const BG_DEEPEST: &str = "#0A0A0A";
    pub const BG_DARK: &str = "#141414";
    pub const BG_SURFACE: &str = "#1E1E1E";
    pub const BG_HOVER: &str = "#262626";
    pub const BG_ELEVATED: &str = "#333333";
    pub const BG_CODE: &str = "#0F0F0F";

    // Borders
    pub const BORDER_SUBTLE: &str = "#2E2E2E";
    pub const BORDER_ACTIVE: &str = "#1E90FF";
    pub const BORDER_HOVER: &str = "#3E3E3E";

    // Text
    pub const TEXT_PRIMARY: &str = "#F5F5F5";
    pub const TEXT_SECONDARY: &str = "#A3A3A3";
    pub const TEXT_TERTIARY: &str = "#737373";
    pub const TEXT_DISABLED: &str = "#525252";

    // Semantic
    pub const SUCCESS: &str = "#22C55E";
    pub const SUCCESS_MUTED: &str = "rgba(34, 197, 94, 0.2)";
    pub const WARNING: &str = "#EAB308";
    pub const WARNING_MUTED: &str = "rgba(234, 179, 8, 0.2)";
    pub const ERROR: &str = "#EF4444";
    pub const ERROR_MUTED: &str = "rgba(239, 68, 68, 0.2)";
    pub const INFO: &str = "#1E90FF";
    pub const INFO_MUTED: &str = "rgba(30, 144, 255, 0.2)";

    // Scrollbar
    pub const SCROLLBAR_THUMB: &str = "#333333";
    pub const SCROLLBAR_THUMB_HOVER: &str = "#4A4A4A";

    // Legacy aliases for gradual migration
    pub const ACCENT: &str = Self::KIMI_BLUE;
    pub const ACCENT_HOVER: &str = Self::KIMI_BLUE_HOVER;
    pub const ACCENT_DIM: &str = Self::KIMI_BLUE_MUTED;
}
