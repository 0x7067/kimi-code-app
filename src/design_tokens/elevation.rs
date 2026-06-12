//! Elevation (shadow) tokens for the Kimi Code Desktop design system.

pub struct Elevation;

#[allow(dead_code)]
impl Elevation {
    // Card shadows
    pub const CARD: &str = "0 0 0 1px #252A33, 0 10px 32px rgba(0,0,0,0.2)";
    pub const CARD_HOVER: &str = "0 0 0 1px #374050, 0 16px 44px rgba(0,0,0,0.28)";
    pub const CARD_ACTIVE: &str = "0 0 0 1px #6EA1FF, 0 12px 32px rgba(110,161,255,0.12)";

    // Dropdown/Menu shadows
    pub const DROPDOWN: &str = "0 8px 24px rgba(0,0,0,0.4)";
    pub const DROPDOWN_UP: &str = "0 -8px 24px rgba(0,0,0,0.4)";

    // Modal shadows
    pub const MODAL_BACKDROP: &str = "rgba(9,10,12,0.58)";
    pub const MODAL: &str = "0 28px 90px rgba(0,0,0,0.5)";

    // Tooltip shadows
    pub const TOOLTIP: &str = "0 4px 12px rgba(0,0,0,0.3)";

    // Input focus glow
    pub const INPUT_FOCUS: &str = "0 0 0 2px rgba(110,161,255,0.28)";

    // Toast shadow
    pub const TOAST: &str = "0 4px 12px rgba(0,0,0,0.3)";
}
