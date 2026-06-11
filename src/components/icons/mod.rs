//! Icon system — inline SVG Lucide-style icons.
//!
//! Each icon is a Dioxus component accepting `size`, `color`, and
//! `stroke_width` props with sensible defaults.

mod lucide;

#[allow(unused_imports)] // public design-system surface; adopted incrementally
pub use lucide::*;

