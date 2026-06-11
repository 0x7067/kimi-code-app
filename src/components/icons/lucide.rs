//! Inline SVG icons modeled after the Lucide set.
//!
//! Usage:
//! ```rust
//! use crate::components::icons::IconSearch;
//! rsx! { IconSearch { size: 20, color: "#A3A3A3" } }
//! ```

use crate::design_tokens::Colors;
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Props)]
pub struct IconProps {
    #[props(default = 16)]
    pub size: u32,
    #[props(default = Colors::TEXT_SECONDARY.to_string())]
    pub color: String,
    #[props(default = 1.5)]
    pub stroke_width: f32,
}

// ------------------------------------------------------------------
// Navigation & Actions
// ------------------------------------------------------------------

#[component]
pub fn IconSearch(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "11", cy: "11", r: "8" }
            path { d: "m21 21-4.3-4.3" }
        }
    }
}

#[component]
pub fn IconHome(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9z" }
            polyline { points: "9 22 9 12 15 12 15 22" }
        }
    }
}

#[component]
pub fn IconSettings(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "3" }
            path { d: "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z" }
        }
    }
}

#[component]
pub fn IconPlus(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M12 5v14M5 12h14" }
        }
    }
}

#[component]
pub fn IconSend(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m22 2-7 20-4-9-9-4 20-7z" }
            path { d: "M22 2 11 13" }
        }
    }
}

#[component]
pub fn IconMoreHorizontal(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "1" }
            circle { cx: "19", cy: "12", r: "1" }
            circle { cx: "5", cy: "12", r: "1" }
        }
    }
}

#[component]
pub fn IconChevronDown(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m6 9 6 6 6-6" }
        }
    }
}

#[component]
pub fn IconChevronRight(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

#[component]
pub fn IconChevronLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m15 18-6-6 6-6" }
        }
    }
}

#[component]
pub fn IconX(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

// ------------------------------------------------------------------
// File & Code
// ------------------------------------------------------------------

#[component]
pub fn IconFileCode(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" }
            polyline { points: "14 2 14 8 20 8" }
            path { d: "m10 13-2 2 2 2" }
            path { d: "m14 17 2-2-2-2" }
        }
    }
}

#[component]
pub fn IconFolder(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2z" }
        }
    }
}

#[component]
pub fn IconFolderOpen(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2" }
        }
    }
}

#[component]
pub fn IconGitBranch(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            line { x1: "6", y1: "3", x2: "6", y2: "15" }
            circle { cx: "18", cy: "6", r: "3" }
            circle { cx: "6", cy: "18", r: "3" }
            path { d: "M18 9a9 9 0 0 1-9 9" }
        }
    }
}

#[component]
pub fn IconGitCommit(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "3" }
            line { x1: "3", y1: "12", x2: "9", y2: "12" }
            line { x1: "15", y1: "12", x2: "21", y2: "12" }
        }
    }
}

// ------------------------------------------------------------------
// Status & Feedback
// ------------------------------------------------------------------

#[component]
pub fn IconCheck(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M20 6 9 17l-5-5" }
        }
    }
}

#[component]
pub fn IconCheckCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
            polyline { points: "22 4 12 14.01 9 11.01" }
        }
    }
}

#[component]
pub fn IconAlertCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "8", x2: "12", y2: "12" }
            line { x1: "12", y1: "16", x2: "12.01", y2: "16" }
        }
    }
}

#[component]
pub fn IconAlertTriangle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3z" }
            line { x1: "12", y1: "9", x2: "12", y2: "13" }
            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
        }
    }
}

#[component]
pub fn IconInfo(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", y1: "16", x2: "12", y2: "12" }
            line { x1: "12", y1: "8", x2: "12.01", y2: "8" }
        }
    }
}

#[component]
pub fn IconLoader(props: IconProps) -> Element {
    rsx! {
        svg {
            class: "animate-spin",
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M21 12a9 9 0 1 1-6.219-8.56" }
        }
    }
}

// ------------------------------------------------------------------
// Chat & Thread
// ------------------------------------------------------------------

#[component]
pub fn IconMessageSquare(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" }
        }
    }
}

#[component]
pub fn IconMessageCircle(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M7.9 20A9 9 0 1 0 4 16.1L2 22z" }
        }
    }
}

#[component]
pub fn IconBot(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M12 8V4H8" }
            rect { x: "4", y: "8", width: "16", height: "12", rx: "2" }
            path { d: "M2 14h2" }
            path { d: "M20 14h2" }
            path { d: "M15 13v2" }
            path { d: "M9 13v2" }
        }
    }
}

#[component]
pub fn IconUser(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2" }
            circle { cx: "12", cy: "7", r: "4" }
        }
    }
}

#[component]
pub fn IconPaperclip(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48" }
        }
    }
}

#[component]
pub fn IconMic(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M12 19v3" }
            path { d: "M19 10v2a7 7 0 0 1-14 0v-2" }
            line { x1: "12", y1: "19", x2: "12", y2: "22" }
            rect { x: "9", y: "2", width: "6", height: "11", rx: "3" }
        }
    }
}

// ------------------------------------------------------------------
// Layout & Misc
// ------------------------------------------------------------------

#[component]
pub fn IconPanelRight(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
            path { d: "M15 3v18" }
        }
    }
}

#[component]
pub fn IconPanelLeft(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            rect { x: "3", y: "3", width: "18", height: "18", rx: "2" }
            path { d: "M9 3v18" }
        }
    }
}

#[component]
pub fn IconCpu(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            rect { x: "4", y: "4", width: "16", height: "16", rx: "2" }
            rect { x: "9", y: "9", width: "6", height: "6" }
            path { d: "M15 2v2" }
            path { d: "M15 20v2" }
            path { d: "M2 15h2" }
            path { d: "M2 9h2" }
            path { d: "M20 15h2" }
            path { d: "M20 9h2" }
            path { d: "M9 2v2" }
            path { d: "M9 20v2" }
        }
    }
}

#[component]
pub fn IconZap(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            polygon { points: "13 2 3 14 12 14 11 22 21 10 12 10 13 2" }
        }
    }
}

#[component]
pub fn IconTrash(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M3 6h18" }
            path { d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }
            path { d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }
            line { x1: "10", y1: "11", x2: "10", y2: "17" }
            line { x1: "14", y1: "11", x2: "14", y2: "17" }
        }
    }
}

#[component]
pub fn IconEdit(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" }
            path { d: "M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" }
        }
    }
}

#[component]
pub fn IconCopy(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2", ry: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

#[component]
pub fn IconRefreshCw(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
            path { d: "M21 3v5h-5" }
            path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
            path { d: "M8 16H3v5" }
        }
    }
}

#[component]
pub fn IconSparkles(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3z" }
            path { d: "M5 3v4" }
            path { d: "M19 17v4" }
            path { d: "M3 5h4" }
            path { d: "M17 19h4" }
        }
    }
}

#[component]
pub fn IconTerminal(props: IconProps) -> Element {
    rsx! {
        svg {
            width: "{props.size}",
            height: "{props.size}",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "{props.color}",
            "stroke-width": "{props.stroke_width}",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            polyline { points: "4 17 10 11 4 5" }
            line { x1: "12", y1: "19", x2: "20", y2: "19" }
        }
    }
}
