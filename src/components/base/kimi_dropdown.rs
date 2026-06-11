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
    // True while the 100ms close animation plays; the menu stays mounted
    // with the `kimi-scale-out` class until the timer unmounts it.
    let mut closing = use_signal(|| false);

    let mut close = move || {
        if closing() || !is_open() {
            return;
        }
        closing.set(true);
        let handle = gloo_timers::callback::Timeout::new(100, move || {
            is_open.set(false);
            closing.set(false);
            if let Some(h) = onclose {
                h.call(());
            }
        });
        std::mem::forget(handle);
    };

    let anim_cls = if closing() { "kimi-scale-out" } else { "kimi-fade-in kimi-scale-in" };

    rsx! {
        div {
            style: "position: relative; display: inline-block;",
            div {
                onclick: move |_| {
                    if is_open() {
                        close();
                    } else {
                        is_open.set(true);
                    }
                },
                {trigger}
            }
            if is_open() {
                div {
                    class: "{anim_cls}",
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
                        close();
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
    let left_border =
        if active { "2px solid var(--accent)" } else { "2px solid transparent" };

    rsx! {
        div {
            class: "kimi-dropdown-item",
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
