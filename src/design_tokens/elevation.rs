//! Elevation (shadow) tokens for the Kimi Code Desktop design system.

pub struct Elevation;

#[allow(dead_code)]
impl Elevation {
    // Card shadows
    pub const CARD: &str = "0 0 0 1px #2E2E2E, 0 4px 12px rgba(0,0,0,0.2)";
    pub const CARD_HOVER: &str = "0 0 0 1px #3E3E3E, 0 8px 24px rgba(0,0,0,0.3)";
    pub const CARD_ACTIVE: &str = "0 0 0 1px #1E90FF, 0 4px 12px rgba(30,144,255,0.1)";

    // Dropdown/Menu shadows
    pub const DROPDOWN: &str = "0 8px 24px rgba(0,0,0,0.4)";
    pub const DROPDOWN_UP: &str = "0 -8px 24px rgba(0,0,0,0.4)";

    // Modal shadows
    pub const MODAL_BACKDROP: &str = "rgba(10,10,10,0.8)";
    pub const MODAL: &str = "0 16px 48px rgba(0,0,0,0.5)";

    // Tooltip shadows
    pub const TOOLTIP: &str = "0 4px 12px rgba(0,0,0,0.3)";

    // Input focus glow
    pub const INPUT_FOCUS: &str = "0 0 0 2px rgba(30,144,255,0.3)";

    // Toast shadow
    pub const TOAST: &str = "0 4px 12px rgba(0,0,0,0.3)";
}
