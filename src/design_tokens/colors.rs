//! Color tokens for the Kimi Code Desktop design system.
//!
//! Dark mode only. All colors are defined as `&'static str` hex / rgba
//! literals so they can be used directly in inline Tailwind-style
//! `class:` attributes or CSS custom properties.

pub struct Colors;

#[allow(dead_code)]
impl Colors {
    // Kimi Brand
    pub const KIMI_BLUE: &str = "#6EA1FF";
    pub const KIMI_BLUE_HOVER: &str = "#8AB4FF";
    pub const KIMI_BLUE_MUTED: &str = "rgba(110, 161, 255, 0.18)";

    // Backgrounds (dark theme only)
    pub const BG_DEEPEST: &str = "#0B0D10";
    pub const BG_DARK: &str = "#15171B";
    pub const BG_SURFACE: &str = "#1B1E24";
    pub const BG_HOVER: &str = "#232832";
    pub const BG_ELEVATED: &str = "#2B313C";
    pub const BG_CODE: &str = "#0B0D10";

    // Borders
    pub const BORDER_SUBTLE: &str = "#252A33";
    pub const BORDER_ACTIVE: &str = "#6EA1FF";
    pub const BORDER_HOVER: &str = "#374050";

    // Text
    pub const TEXT_PRIMARY: &str = "#F4F6FA";
    pub const TEXT_SECONDARY: &str = "#BAC2CE";
    pub const TEXT_TERTIARY: &str = "#87909E";
    pub const TEXT_DISABLED: &str = "#5F6876";

    // Semantic
    pub const SUCCESS: &str = "#22C55E";
    pub const SUCCESS_MUTED: &str = "rgba(34, 197, 94, 0.2)";
    pub const WARNING: &str = "#EAB308";
    pub const WARNING_MUTED: &str = "rgba(234, 179, 8, 0.2)";
    pub const ERROR: &str = "#EF4444";
    pub const ERROR_MUTED: &str = "rgba(239, 68, 68, 0.2)";
    pub const INFO: &str = "#6EA1FF";
    pub const INFO_MUTED: &str = "rgba(110, 161, 255, 0.18)";

    // Scrollbar
    pub const SCROLLBAR_THUMB: &str = "#2B313C";
    pub const SCROLLBAR_THUMB_HOVER: &str = "#424B5C";

    // Legacy aliases for gradual migration
    pub const ACCENT: &str = Self::KIMI_BLUE;
    pub const ACCENT_HOVER: &str = Self::KIMI_BLUE_HOVER;
    pub const ACCENT_DIM: &str = Self::KIMI_BLUE_MUTED;
}
