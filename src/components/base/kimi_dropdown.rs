//! KimiDropdown — Dropdown menu for actions, filters, and selections.

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[component]
pub fn KimiDropdown(
    trigger: Element,
    children: Element,
    #[props(default = false)] open: bool,
    onclose: Option<EventHandler<()>>,
) -> Element {
    let mut is_open = use_signal(|| open);

    rsx! {
        div { class: "relative inline-block",
            div {
                onclick: move |_| is_open.set(!is_open()),
                {trigger}
            }
            if is_open() {
                div {
                    class: "kimi-fade-in kimi-scale-in",
                    style: "
                        position: absolute;
                        top: calc(100% + 6px);
                        right: 0;
                        background: {BG_SURFACE};
                        border: 1px solid {BORDER};
                        border-radius: 12px;
                        box-shadow: 0 8px 24px rgba(0,0,0,0.4);
                        padding: 8px 0;
                        min-width: 160px;
                        z-index: 40;
                        overflow: hidden;
                    ",
                    onclick: move |_e| {
                        // Close when clicking inside (typical dropdown behavior)
                        // If child items want to keep it open they can call e.stop_propagation()
                        is_open.set(false);
                        if let Some(h) = onclose { h.call(()); }
                    },
                    {children}
                }
            }
        }
    }
}

/// A single item inside a `KimiDropdown`.
#[component]
pub fn KimiDropdownItem(
    children: Element,
    #[props(default = false)] active: bool,
    onclick: Option<EventHandler<MouseEvent>>,
) -> Element {
    let bg = if active { Colors::BG_HOVER } else { "transparent" };
    let left_border = if active { "2px solid #1E90FF" } else { "2px solid transparent" };

    rsx! {
        div {
            style: "
                display: flex;
                align-items: center;
                height: 36px;
                padding: 0 12px;
                cursor: pointer;
                background: {bg};
                border-left: {left_border};
                font-size: 14px;
                color: {TEXT};
                transition: background 150ms ease-out;
                white-space: nowrap;
            ",
            class: "hover:bg-[#262626]",
            onclick: move |e| {
                e.stop_propagation();
                if let Some(h) = onclick { h.call(e); }
            },
            {children}
        }
    }
}

/// A divider inside a `KimiDropdown`.
#[component]
pub fn KimiDropdownDivider() -> Element {
    rsx! {
        div {
            style: "
                height: 1px;
                background: {BORDER};
                margin: 8px 0;
            ",
        }
    }
}

#[allow(dead_code)] // used via rsx attribute interpolation
const BG_SURFACE: &str = Colors::BG_SURFACE;
#[allow(dead_code)] // used via rsx attribute interpolation
const BORDER: &str = Colors::BORDER_SUBTLE;
#[allow(dead_code)] // used via rsx attribute interpolation
const TEXT: &str = Colors::TEXT_PRIMARY;
