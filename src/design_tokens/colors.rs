//! Color tokens for the Kimi Code Desktop design system.
//!
//! Dark mode only. All colors are defined as `&'static str` hex / rgba
//! literals so they can be used directly in inline `class:` attributes or
//! CSS custom properties. Values are kept in sync with `DESIGN_SYSTEM.md`
//! and `assets/css/01-tokens.css`.

pub struct Colors;

#[allow(dead_code)]
impl Colors {
    // Kimi Brand
    pub const KIMI_BLUE: &str = "#6EA1FF";
    pub const KIMI_BLUE_HOVER: &str = "#9BBDFF";
    pub const KIMI_BLUE_MUTED: &str = "rgba(110, 161, 255, 0.16)";

    // Backgrounds (dark theme only — Ethereal Glass)
    pub const BG_DEEPEST: &str = "#050505";
    pub const BG_DARK: &str = "#0A0A0C";
    pub const BG_SURFACE: &str = "#101014";
    pub const BG_ELEVATED: &str = "#18181D";
    pub const BG_HOVER: &str = "#18181D";
    pub const BG_ACTIVE: &str = "#202026";
    pub const BG_CODE: &str = "#060608";
    pub const BG_OVERLAY: &str = "rgba(3, 3, 4, 0.72)";

    // Borders
    pub const BORDER_SUBTLE: &str = "rgba(255, 255, 255, 0.045)";
    pub const BORDER_DEFAULT: &str = "rgba(255, 255, 255, 0.085)";
    pub const BORDER_STRONG: &str = "rgba(255, 255, 255, 0.145)";
    pub const BORDER_ACTIVE: &str = "#6EA1FF";
    pub const BORDER_HIGHLIGHT: &str = "rgba(255, 255, 255, 0.22)";

    // Text
    pub const TEXT_PRIMARY: &str = "#F7F8FB";
    pub const TEXT_SECONDARY: &str = "#C5C9D3";
    pub const TEXT_TERTIARY: &str = "#8A8F9C";
    pub const TEXT_DISABLED: &str = "#5C6270";

    // Semantic
    pub const SUCCESS: &str = "#34D399";
    pub const SUCCESS_MUTED: &str = "rgba(52, 211, 153, 0.12)";
    pub const WARNING: &str = "#FBBF24";
    pub const WARNING_MUTED: &str = "rgba(251, 191, 36, 0.12)";
    pub const ERROR: &str = "#F87171";
    pub const ERROR_MUTED: &str = "rgba(248, 113, 113, 0.12)";
    pub const INFO: &str = "#6EA1FF";
    pub const INFO_MUTED: &str = "rgba(110, 161, 255, 0.16)";

    // Ethereal accent orbs
    pub const GLOW_PURPLE: &str = "rgba(139, 92, 246, 0.34)";
    pub const GLOW_EMERALD: &str = "rgba(16, 185, 129, 0.22)";
    pub const GLOW_AMBER: &str = "rgba(245, 158, 11, 0.24)";

    // Scrollbar
    pub const SCROLLBAR_THUMB: &str = "rgba(255, 255, 255, 0.12)";
    pub const SCROLLBAR_THUMB_HOVER: &str = "rgba(255, 255, 255, 0.2)";

    // Legacy aliases for gradual migration
    pub const ACCENT: &str = Self::KIMI_BLUE;
    pub const ACCENT_HOVER: &str = Self::KIMI_BLUE_HOVER;
    pub const ACCENT_DIM: &str = Self::KIMI_BLUE_MUTED;
}
