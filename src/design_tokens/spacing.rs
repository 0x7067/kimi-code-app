//! Spacing tokens for the Kimi Code Desktop design system.
//!
//! Based on a 4px grid. All values are `u32` pixels.

pub struct Spacing;

#[allow(dead_code)]
impl Spacing {
    pub const UNIT: u32 = 4; // base unit in pixels

    pub const S1: u32 = 4; // 1 * UNIT
    pub const S2: u32 = 8; // 2 * UNIT
    pub const S3: u32 = 12; // 3 * UNIT
    pub const S4: u32 = 16; // 4 * UNIT
    pub const S5: u32 = 20; // 5 * UNIT
    pub const S6: u32 = 24; // 6 * UNIT
    pub const S8: u32 = 32; // 8 * UNIT
    pub const S10: u32 = 40; // 10 * UNIT
    pub const S12: u32 = 48; // 12 * UNIT
    pub const S16: u32 = 64; // 16 * UNIT

    // Layout dimensions
    pub const SIDEBAR_WIDTH: u32 = 240;
    pub const SIDEBAR_COLLAPSED: u32 = 64;
    pub const CHAT_MAX_WIDTH: u32 = 720;
    pub const RIGHT_PANEL_WIDTH: u32 = 280;
    pub const SETTINGS_MAX_WIDTH: u32 = 640;
    pub const INPUT_MIN_HEIGHT: u32 = 44;
    pub const BUTTON_STANDARD_HEIGHT: u32 = 32;
    pub const BUTTON_COMPACT_HEIGHT: u32 = 28;
    pub const ROW_HEIGHT: u32 = 44; // for lists, commands
    pub const ROW_HEIGHT_SMALL: u32 = 36;
}
