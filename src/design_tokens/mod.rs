//! Design tokens for the Kimi Code Desktop design system.
//!
//! All visual constants (colors, typography, spacing, animation, elevation)
//! live here. Components import these values rather than hard-coding
//! hex codes or pixel values.

pub mod colors;
pub mod typography;
pub mod spacing;
pub mod animation;
pub mod elevation;

#[cfg(test)]
mod tests;

pub use colors::Colors;
pub use typography::Typography;
