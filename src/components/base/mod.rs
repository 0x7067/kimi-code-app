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

#[allow(unused_imports)] // public design-system surface; adopted incrementally
pub use kimi_badge::KimiBadge;
#[allow(unused_imports)]
pub use kimi_button::KimiButton;
#[allow(unused_imports)]
pub use kimi_card::KimiCard;
#[allow(unused_imports)]
pub use kimi_dropdown::{KimiDropdown, KimiDropdownDivider, KimiDropdownItem};
#[allow(unused_imports)]
pub use kimi_empty_state::KimiEmptyState;
#[allow(unused_imports)]
pub use kimi_icon::KimiIcon;
#[allow(unused_imports)]
pub use kimi_input::KimiInput;
#[allow(unused_imports)]
pub use kimi_loading::{KimiLoading, LoadingVariant};
#[allow(unused_imports)]
pub use kimi_toast::KimiToast;
#[allow(unused_imports)]
pub use kimi_toggle::KimiToggle;
#[allow(unused_imports)]
pub use kimi_tooltip::KimiTooltip;

