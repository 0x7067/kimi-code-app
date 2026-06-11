//! Animation tokens for the Kimi Code Desktop design system.
//!
//! Durations, easings, and ready-made Tailwind transition classes.

pub struct Animation;

#[allow(dead_code)]
impl Animation {
    // Durations
    pub const MICRO: &str = "150ms";
    pub const FAST: &str = "200ms";
    pub const NORMAL: &str = "300ms";
    pub const SLOW: &str = "500ms";

    // Easing
    pub const EASE_DEFAULT: &str = "cubic-bezier(0.4, 0, 0.2, 1)";
    pub const EASE_ENTER: &str = "cubic-bezier(0, 0, 0.2, 1)";
    pub const EASE_EXIT: &str = "cubic-bezier(0.4, 0, 1, 1)";
    pub const EASE_BOUNCE: &str = "cubic-bezier(0.34, 1.56, 0.64, 1)";

    // Transitions (Tailwind classes)
    pub const TRANSITION_MICRO: &str = "transition-all duration-150 ease-out";
    pub const TRANSITION_FAST: &str = "transition-all duration-200 ease-out";
    pub const TRANSITION_NORMAL: &str = "transition-all duration-300 ease-out";
}
