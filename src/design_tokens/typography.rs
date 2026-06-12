//! Typography tokens for the Kimi Code Desktop design system.

pub struct Typography;

#[allow(dead_code)]
impl Typography {
    // Font families
    pub const FONT_UI: &str = "'Geist', 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'SF Pro Text', ui-sans-serif, system-ui, sans-serif";
    pub const FONT_MONO: &str = "'JetBrains Mono', 'Fira Code', 'SF Mono', Menlo, Consolas, monospace";

    // Type scale
    pub const DISPLAY_SIZE: &str = "24px";
    pub const H1_SIZE: &str = "20px";
    pub const H2_SIZE: &str = "16px";
    pub const H3_SIZE: &str = "14px";
    pub const BODY_SIZE: &str = "14px";
    pub const SMALL_SIZE: &str = "13px";
    pub const CAPTION_SIZE: &str = "12px";
    pub const CODE_SIZE: &str = "13px";

    // Weights
    pub const WEIGHT_NORMAL: &str = "400";
    pub const WEIGHT_MEDIUM: &str = "500";
    pub const WEIGHT_SEMIBOLD: &str = "600";
    pub const WEIGHT_BOLD: &str = "700";

    // Line heights
    pub const LH_TIGHT: &str = "1.2";
    pub const LH_SNUG: &str = "1.3";
    pub const LH_NORMAL: &str = "1.4";
    pub const LH_RELAXED: &str = "1.5";
    pub const LH_CODE: &str = "1.6";

    // Letter spacing
    pub const LS_TIGHT: &str = "-0.02em";
    pub const LS_SNUG: &str = "-0.01em";
    pub const LS_NORMAL: &str = "0";
    pub const LS_WIDE: &str = "0.01em";
}
