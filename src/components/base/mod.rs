//! Base (primitive) components for the Kimi Code Desktop design system.
//!
//! These are the lowest-level building blocks used by layout and feature
//! components. Every component here depends only on `design_tokens`.

mod kimi_icon;
mod kimi_button;
mod kimi_input;
mod kimi_card;
mod kimi_toggle;
mod kimi_badge;
mod kimi_tooltip;
mod kimi_dropdown;
mod kimi_toast;
mod kimi_loading;
mod kimi_empty_state;

pub use kimi_icon::KimiIcon;
pub use kimi_button::KimiButton;
pub use kimi_input::KimiInput;
pub use kimi_card::KimiCard;
pub use kimi_toggle::KimiToggle;
pub use kimi_badge::KimiBadge;
pub use kimi_tooltip::KimiTooltip;
pub use kimi_dropdown::KimiDropdown;
pub use kimi_toast::KimiToast;
pub use kimi_loading::{KimiLoading, LoadingVariant};
pub use kimi_empty_state::KimiEmptyState;
