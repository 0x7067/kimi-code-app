# Kimi Code Desktop — Design System Implementation Document

**Version:** 1.0  
**Date:** 2026-06-11  
**Stack:** Tauri (Rust backend) + Dioxus (Rust frontend)  
**Framework:** Dioxus 0.7+ with Tailwind CSS  
**Priority:** P0 — Must be implemented before any feature work

---

## 1. Purpose

This document specifies the complete design system for the Kimi Code Desktop application. It must be implemented **first** — all subsequent feature implementations depend on these tokens, components, and patterns.

The design system is derived from:
- **Kimi AI Brand Guidelines** (https://moonshotai.github.io/Branding-Guide/)
- **Kimi web application** visual language (dark mode, blue accent)
- **Codex UI patterns** (layout, interaction, component structure) — layout only, colors/branding from Kimi

---

## 2. Technology Stack

| Layer | Technology | Crate/Tool |
|-------|-----------|------------|
| UI Framework | Dioxus | `dioxus = "0.7"` |
| Desktop Shell | Tauri | `tauri = "2.0+"` |
| Styling | Tailwind CSS | `tailwindcss` via CDN or build |
| Icons | Lucide | `dioxus-lucide` or inline SVG |
| Fonts | System + JetBrains Mono | Google Fonts or bundled |
| State | Dioxus Signals | Built-in |

**Dioxus-specific notes:**
- All components use RSX syntax (Rust JSX equivalent)
- State via `use_signal`, `use_memo`, `use_context` — no external state library
- Styling via Tailwind utility classes or inline `class:` attributes
- Icons as Lucide SVG components or `img` with SVG source

---

## 3. Design Tokens

### 3.1 Color Tokens

Define as a Rust module with CSS custom properties injected at app root.

```rust
// src/design_tokens/colors.rs
pub struct Colors;

impl Colors {
    // Kimi Brand
    pub const KIMI_BLUE: &str = "#1E90FF";
    pub const KIMI_BLUE_HOVER: &str = "#4AA8FF";
    pub const KIMI_BLUE_MUTED: &str = "rgba(30, 144, 255, 0.2)"; // #1E90FF33

    // Backgrounds (dark theme only)
    pub const BG_DEEPEST: &str = "#0A0A0A";
    pub const BG_DARK: &str = "#141414";
    pub const BG_SURFACE: &str = "#1E1E1E";
    pub const BG_HOVER: &str = "#262626";
    pub const BG_CODE: &str = "#0F0F0F";

    // Borders
    pub const BORDER_SUBTLE: &str = "#2E2E2E";
    pub const BORDER_ACTIVE: &str = "#1E90FF";

    // Text
    pub const TEXT_PRIMARY: &str = "#F5F5F5";
    pub const TEXT_SECONDARY: &str = "#A3A3A3";
    pub const TEXT_TERTIARY: &str = "#737373";
    pub const TEXT_DISABLED: &str = "#525252";

    // Semantic
    pub const SUCCESS: &str = "#22C55E";
    pub const WARNING: &str = "#EAB308";
    pub const ERROR: &str = "#EF4444";
    pub const INFO: &str = "#1E90FF";
}
```

**CSS Custom Properties (injected in `index.html` or root component):**

```css
:root {
  --kimi-blue: #1E90FF;
  --kimi-blue-hover: #4AA8FF;
  --kimi-blue-muted: rgba(30, 144, 255, 0.2);

  --bg-deepest: #0A0A0A;
  --bg-dark: #141414;
  --bg-surface: #1E1E1E;
  --bg-hover: #262626;
  --bg-code: #0F0F0F;

  --border-subtle: #2E2E2E;
  --border-active: #1E90FF;

  --text-primary: #F5F5F5;
  --text-secondary: #A3A3A3;
  --text-tertiary: #737373;
  --text-disabled: #525252;

  --success: #22C55E;
  --warning: #EAB308;
  --error: #EF4444;
  --info: #1E90FF;
}
```

**Usage in Dioxus RSX:**
```rust
rsx! {
    div {
        class: "bg-[#141414] text-[#F5F5F5] border border-[#2E2E2E] rounded-xl",
        // or via Tailwind config with custom colors
    }
}
```

---

### 3.2 Typography Tokens

```rust
// src/design_tokens/typography.rs
pub struct Typography;

impl Typography {
    // Font families
    pub const FONT_UI: &str = "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif";
    pub const FONT_MONO: &str = "'JetBrains Mono', 'Fira Code', 'SF Mono', Menlo, Consolas, monospace";

    // Type scale
    pub const DISPLAY_SIZE: &str = "24px";
    pub const H1_SIZE: &str = "20px";
    pub const H2_SIZE: &str = "16px";
    pub const H3_SIZE: &str = "14px";
    pub const BODY_SIZE: &str = "14px";
    pub const SMALL_SIZE: &str = "13px";
    pub const CAPTION_SIZE: &str = "12px";
    pub const CODE_SIZE: &str = "13px";

    // Weights
    pub const WEIGHT_NORMAL: &str = "400";
    pub const WEIGHT_MEDIUM: &str = "500";
    pub const WEIGHT_SEMIBOLD: &str = "600";
    pub const WEIGHT_BOLD: &str = "700";

    // Line heights
    pub const LH_TIGHT: &str = "1.2";
    pub const LH_SNUG: &str = "1.3";
    pub const LH_NORMAL: &str = "1.4";
    pub const LH_RELAXED: &str = "1.5";
    pub const LH_CODE: &str = "1.6";

    // Letter spacing
    pub const LS_TIGHT: &str = "-0.02em";
    pub const LS_SNUG: &str = "-0.01em";
    pub const LS_NORMAL: &str = "0";
    pub const LS_WIDE: &str = "0.01em";
}
```

**Tailwind Config Extension:**

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        'kimi-blue': '#1E90FF',
        'kimi-blue-hover': '#4AA8FF',
        'bg-deepest': '#0A0A0A',
        'bg-dark': '#141414',
        'bg-surface': '#1E1E1E',
        'bg-hover': '#262626',
        'bg-code': '#0F0F0F',
        'border-subtle': '#2E2E2E',
        'text-primary': '#F5F5F5',
        'text-secondary': '#A3A3A3',
        'text-tertiary': '#737373',
        'text-disabled': '#525252',
      },
      fontFamily: {
        'ui': ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
        'mono': ['JetBrains Mono', 'Fira Code', 'SF Mono', 'Menlo', 'Consolas', 'monospace'],
      },
      fontSize: {
        'display': ['24px', { lineHeight: '1.2', letterSpacing: '-0.02em' }],
        'h1': ['20px', { lineHeight: '1.3', letterSpacing: '-0.01em' }],
        'h2': ['16px', { lineHeight: '1.4' }],
        'h3': ['14px', { lineHeight: '1.4' }],
        'body': ['14px', { lineHeight: '1.5' }],
        'small': ['13px', { lineHeight: '1.5' }],
        'caption': ['12px', { lineHeight: '1.4', letterSpacing: '0.01em' }],
        'code': ['13px', { lineHeight: '1.6' }],
      },
      spacing: {
        '1': '4px',
        '2': '8px',
        '3': '12px',
        '4': '16px',
        '5': '20px',
        '6': '24px',
        '8': '32px',
        '10': '40px',
      },
      borderRadius: {
        'card': '12px',
        'button': '8px',
        'input': '12px',
        'tooltip': '8px',
        'modal': '16px',
      },
      boxShadow: {
        'card': '0 0 0 1px #2E2E2E, 0 4px 12px rgba(0,0,0,0.2)',
        'card-active': '0 0 0 1px #1E90FF, 0 4px 12px rgba(30,144,255,0.1)',
        'dropdown': '0 8px 24px rgba(0,0,0,0.4)',
        'modal-backdrop': 'rgba(10,10,10,0.8)',
      },
    },
  },
}
```

---

### 3.3 Spacing Tokens

```rust
// src/design_tokens/spacing.rs
pub struct Spacing;

impl Spacing {
    pub const UNIT: u32 = 4; // base unit in pixels

    pub const S1: u32 = 4;   // 1 * UNIT
    pub const S2: u32 = 8;   // 2 * UNIT
    pub const S3: u32 = 12;  // 3 * UNIT
    pub const S4: u32 = 16;  // 4 * UNIT
    pub const S5: u32 = 20;  // 5 * UNIT
    pub const S6: u32 = 24;  // 6 * UNIT
    pub const S8: u32 = 32;  // 8 * UNIT
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
```

---

### 3.4 Animation Tokens

```rust
// src/design_tokens/animation.rs
pub struct Animation;

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
```

**CSS Animation Definitions:**

```css
@keyframes pulse-dot {
  0%, 100% { transform: scale(1); opacity: 1; }
  50% { transform: scale(1.2); opacity: 0.8; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes slide-in-top {
  from { transform: translateY(-8px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes scale-in {
  from { transform: scale(0.95); opacity: 0; }
  to { transform: scale(1); opacity: 1; }
}

.kimi-pulse {
  animation: pulse-dot 2s ease-in-out infinite;
}

.kimi-spinner {
  animation: spin 1s linear infinite;
}

.kimi-slide-in {
  animation: slide-in-top 200ms cubic-bezier(0.4, 0, 0.2, 1);
}

.kimi-fade-in {
  animation: fade-in 150ms ease-out;
}

.kimi-scale-in {
  animation: scale-in 150ms cubic-bezier(0.4, 0, 0.2, 1);
}
```

---

### 3.5 Shadow & Elevation Tokens

```rust
// src/design_tokens/elevation.rs
pub struct Elevation;

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
}
```

---

## 4. Base Components

All components are Dioxus components in Rust. Implement these in order — each depends on previous.

### 4.1 KimiIcon (Logo Component)

**Purpose:** Render the Kimi K icon with blue dot in various sizes and variants.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiIconProps {
    #[props(default = 32)]
    pub size: u32,
    #[props(default = "rounded")]
    pub variant: String, // "rounded", "round", "right-angle", "k-only"
    #[props(default = false)]
    pub animate_dot: bool,
}
```

**Variants:**
- `rounded`: Rounded square background, K + blue dot (app icon)
- `round`: Circular background, K + blue dot (alternative app icon)
- `right-angle`: Sharp corners, K + blue dot (minimal)
- `k-only`: Just the K letterform, no dot (favicon, tray)

**Implementation:**
```rust
// src/components/base/kimi_icon.rs
use dioxus::prelude::*;

#[component]
pub fn KimiIcon(props: KimiIconProps) -> Element {
    let size = props.size;
    let dot_size = size / 4;
    let dot_offset = size / 8;

    rsx! {
        svg {
            width: "{size}px",
            height: "{size}px",
            view_box: "0 0 {size} {size}",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",

            // Background shape (if not k-only)
            if props.variant != "k-only" {
                rect {
                    x: "0",
                    y: "0",
                    width: "{size}",
                    height: "{size}",
                    rx: if props.variant == "round" { "{size/2}" } else if props.variant == "rounded" { "{size/4}" } else { "0" },
                    fill: "#141414",
                }
            }

            // K letterform
            text {
                x: "{size/2}",
                y: "{size/2 + size/8}",
                "text-anchor": "middle",
                "dominant-baseline": "middle",
                fill: "#F5F5F5",
                "font-family": "-apple-system, BlinkMacSystemFont, sans-serif",
                "font-weight": "700",
                "font-size": "{size * 0.6}px",
                "K"
            }

            // Blue dot
            circle {
                cx: "{size - dot_offset - dot_size/2}",
                cy: "{dot_offset + dot_size/2}",
                r: "{dot_size/2}",
                fill: "#1E90FF",
                class: if props.animate_dot { "kimi-pulse" } else { "" },
            }
        }
    }
}
```

**Usage:**
```rust
KimiIcon { size: 32, variant: "rounded", animate_dot: true }
KimiIcon { size: 16, variant: "k-only" }
```

---

### 4.2 KimiButton

**Purpose:** Primary, secondary, and ghost buttons with Kimi branding.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiButtonProps {
    pub children: Element,
    #[props(default = "primary")]
    pub variant: String, // "primary", "secondary", "ghost", "danger"
    #[props(default = "standard")]
    pub size: String, // "standard", "compact", "icon"
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub loading: bool,
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(default = false)]
    pub full_width: bool,
}
```

**Styles:**

| Variant | Background | Text | Border | Hover | Active |
|---------|-----------|------|--------|-------|--------|
| primary | `#1E90FF` | white | none | `#4AA8FF` | scale(0.98) |
| secondary | `#262626` | `#F5F5F5` | none | `#333333` | scale(0.98) |
| ghost | transparent | `#A3A3A3` | none | `#F5F5F5` | — |
| danger | `#EF4444` | white | none | `#F87171` | scale(0.98) |

| Size | Height | Padding | Font Size |
|------|--------|---------|-----------|
| standard | 32px | 0 12px | 14px |
| compact | 28px | 0 8px | 13px |
| icon | 32px | 0 | — |

**Implementation:**
```rust
// src/components/base/kimi_button.rs
use dioxus::prelude::*;

#[component]
pub fn KimiButton(props: KimiButtonProps) -> Element {
    let base_classes = "inline-flex items-center justify-center font-medium rounded-lg transition-all duration-150 ease-out focus:outline-none focus:ring-2 focus:ring-[#1E90FF] focus:ring-offset-2 focus:ring-offset-[#141414]";

    let variant_classes = match props.variant.as_str() {
        "primary" => "bg-[#1E90FF] text-white hover:bg-[#4AA8FF] active:scale-[0.98]",
        "secondary" => "bg-[#262626] text-[#F5F5F5] hover:bg-[#333333] active:scale-[0.98]",
        "ghost" => "bg-transparent text-[#A3A3A3] hover:text-[#F5F5F5]",
        "danger" => "bg-[#EF4444] text-white hover:bg-[#F87171] active:scale-[0.98]",
        _ => "bg-[#1E90FF] text-white hover:bg-[#4AA8FF]",
    };

    let size_classes = match props.size.as_str() {
        "standard" => "h-8 px-3 text-sm",
        "compact" => "h-7 px-2 text-xs",
        "icon" => "h-8 w-8 p-0",
        _ => "h-8 px-3 text-sm",
    };

    let disabled_classes = if props.disabled || props.loading {
        "opacity-50 cursor-not-allowed"
    } else {
        "cursor-pointer"
    };

    let width_class = if props.full_width { "w-full" } else { "" };

    rsx! {
        button {
            class: "{base_classes} {variant_classes} {size_classes} {disabled_classes} {width_class}",
            disabled: props.disabled || props.loading,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    if !props.disabled && !props.loading {
                        handler.call(e);
                    }
                }
            },
            if props.loading {
                svg {
                    class: "animate-spin -ml-1 mr-2 h-4 w-4 text-current",
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    circle {
                        class: "opacity-25",
                        cx: "12",
                        cy: "12",
                        r: "10",
                        stroke: "currentColor",
                        "stroke-width": "4",
                    }
                    path {
                        class: "opacity-75",
                        fill: "currentColor",
                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                    }
                }
            }
            {props.children}
        }
    }
}
```

---

### 4.3 KimiInput

**Purpose:** Text input with Kimi styling, focus ring, and optional icons.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiInputProps {
    #[props(default = "")]
    pub value: String,
    #[props(default = "")]
    pub placeholder: String,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub error: bool,
    #[props(default = false)]
    pub multiline: bool,
    pub onchange: Option<EventHandler<String>>,
    pub onsubmit: Option<EventHandler<()>>,
    pub leading_icon: Option<Element>,
    pub trailing_icon: Option<Element>,
}
```

**Styles:**
- Background: `#1E1E1E`
- Border: 1px solid `#2E2E2E`
- Border radius: 12px
- Padding: 10px 12px (single line), 12px (multiline)
- Font: 14px, `#F5F5F5`
- Placeholder: `#737373`
- Focus: border `#1E90FF`, box-shadow `0 0 0 2px rgba(30,144,255,0.3)`
- Error: border `#EF4444`, box-shadow `0 0 0 2px rgba(239,68,68,0.3)`
- Disabled: opacity 0.5, cursor not-allowed

**Implementation:**
```rust
// src/components/base/kimi_input.rs
use dioxus::prelude::*;

#[component]
pub fn KimiInput(props: KimiInputProps) -> Element {
    let mut value = use_signal(|| props.value.clone());

    let base_classes = "w-full bg-[#1E1E1E] border rounded-xl text-[#F5F5F5] placeholder-[#737373] transition-all duration-150 ease-out focus:outline-none";

    let state_classes = if props.error {
        "border-[#EF4444] focus:border-[#EF4444] focus:shadow-[0_0_0_2px_rgba(239,68,68,0.3)]"
    } else if props.disabled {
        "border-[#2E2E2E] opacity-50 cursor-not-allowed"
    } else {
        "border-[#2E2E2E] focus:border-[#1E90FF] focus:shadow-[0_0_0_2px_rgba(30,144,255,0.3)]"
    };

    let size_classes = if props.multiline {
        "py-3 px-3 min-h-[80px] resize-y"
    } else {
        "h-11 py-2 px-3"
    };

    rsx! {
        div {
            class: "relative flex items-center",
            if let Some(icon) = &props.leading_icon {
                div {
                    class: "absolute left-3 text-[#737373]",
                    {icon}
                }
            }
            if props.multiline {
                textarea {
                    class: "{base_classes} {state_classes} {size_classes}",
                    value: "{value()}",
                    placeholder: "{props.placeholder}",
                    disabled: props.disabled,
                    oninput: move |e| {
                        let new_value = e.value();
                        value.set(new_value.clone());
                        if let Some(handler) = &props.onchange {
                            handler.call(new_value);
                        }
                    },
                }
            } else {
                input {
                    class: "{base_classes} {state_classes} {size_classes}",
                    r#type: "text",
                    value: "{value()}",
                    placeholder: "{props.placeholder}",
                    disabled: props.disabled,
                    oninput: move |e| {
                        let new_value = e.value();
                        value.set(new_value.clone());
                        if let Some(handler) = &props.onchange {
                            handler.call(new_value);
                        }
                    },
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            if let Some(handler) = &props.onsubmit {
                                handler.call(());
                            }
                        }
                    },
                }
            }
            if let Some(icon) = &props.trailing_icon {
                div {
                    class: "absolute right-3 text-[#737373]",
                    {icon}
                }
            }
        }
    }
}
```

---

### 4.4 KimiCard

**Purpose:** Elevated surface container for content sections.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiCardProps {
    pub children: Element,
    #[props(default = false)]
    pub hoverable: bool,
    #[props(default = false)]
    pub active: bool,
    #[props(default = false)]
    pub padding: bool, // default true
    #[props(default = "medium")]
    pub radius: String, // "small" (8px), "medium" (12px), "large" (16px)
}
```

**Styles:**
- Background: `#1E1E1E`
- Border: 1px solid `#2E2E2E`
- Border radius: 12px (medium)
- Shadow: `0 0 0 1px #2E2E2E, 0 4px 12px rgba(0,0,0,0.2)`
- Hover (if hoverable): border `#3E3E3E`, shadow `0 8px 24px rgba(0,0,0,0.3)`
- Active (if active): border `#1E90FF`, shadow `0 0 0 1px #1E90FF, 0 4px 12px rgba(30,144,255,0.1)`
- Padding: 16px (if padding true)

---

### 4.5 KimiToggle

**Purpose:** iOS-style toggle switch.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiToggleProps {
    #[props(default = false)]
    pub checked: bool,
    pub onchange: EventHandler<bool>,
    #[props(default = false)]
    pub disabled: bool,
}
```

**Styles:**
- Track: 32px wide, 16px tall, 8px radius
- Off: background `#262626`
- On: background `#1E90FF`
- Thumb: 14px circle, white, 1px offset
- Transition: 150ms ease-out

---

### 4.6 KimiBadge

**Purpose:** Status and label badges.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiBadgeProps {
    pub children: Element,
    #[props(default = "default")]
    pub variant: String, // "default", "blue", "green", "yellow", "red", "gray"
    #[props(default = "small")]
    pub size: String, // "small", "medium"
}
```

**Styles:**

| Variant | Background | Text | Border |
|---------|-----------|------|--------|
| default | `#262626` | `#F5F5F5` | none |
| blue | `rgba(30,144,255,0.2)` | `#1E90FF` | none |
| green | `rgba(34,197,94,0.2)` | `#22C55E` | none |
| yellow | `rgba(234,179,8,0.2)` | `#EAB308` | none |
| red | `rgba(239,68,68,0.2)` | `#EF4444` | none |
| gray | `#1E1E1E` | `#737373` | `#2E2E2E` |

| Size | Padding | Font Size | Radius |
|------|---------|-----------|--------|
| small | 2px 8px | 12px | 4px |
| medium | 4px 12px | 13px | 6px |

---

### 4.7 KimiTooltip

**Purpose:** Hover tooltip for icon buttons and truncated text.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiTooltipProps {
    pub children: Element, // trigger element
    pub content: String,
    #[props(default = "top")]
    pub position: String, // "top", "bottom", "left", "right"
}
```

**Styles:**
- Background: `#262626`
- Text: `#F5F5F5`, 12px
- Padding: 4px 8px
- Radius: 8px
- Shadow: `0 4px 12px rgba(0,0,0,0.3)`
- Arrow: 4px, `#262626`

---

### 4.8 KimiDropdown

**Purpose:** Dropdown menu for actions, filters, and selections.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiDropdownProps {
    pub trigger: Element,
    pub children: Element, // menu items
    #[props(default = false)]
    pub open: bool,
    pub onclose: EventHandler<()>,
}
```

**Styles:**
- Container: `#1E1E1E`, 12px radius, border `#2E2E2E`
- Shadow: `0 8px 24px rgba(0,0,0,0.4)`
- Padding: 8px 0
- Item height: 36px
- Item padding: 0 12px
- Item hover: `#262626`
- Item active: `#333333` with left border 2px `#1E90FF`
- Divider: 1px `#2E2E2E`, margin 8px 0

---

### 4.9 KimiToast

**Purpose:** Notification toast for errors, success, and info.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiToastProps {
    pub message: String,
    #[props(default = "info")]
    pub variant: String, // "info", "success", "warning", "error"
    #[props(default = 5000)]
    pub duration: u64, // ms, 0 = persistent
    pub onclose: EventHandler<()>,
}
```

**Styles:**
- Background: `#1E1E1E`
- Border left: 3px semantic color
- Text: `#F5F5F5`, 14px
- Radius: 12px
- Padding: 12px 16px
- Shadow: `0 4px 12px rgba(0,0,0,0.3)`
- Animation: slide in from top 300ms, auto-dismiss fade out 200ms

---

### 4.10 KimiScrollbar

**Purpose:** Custom scrollbar for all scrollable areas.

**Styles (CSS):**
```css
/* Global scrollbar styles */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #333333;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #4A4A4A;
}

/* Firefox */
* {
  scrollbar-width: thin;
  scrollbar-color: #333333 transparent;
}
```

---

### 4.11 KimiLoading

**Purpose:** Loading indicators.

**Variants:**
- **Spinner:** 16px, 2px stroke, `#1E90FF`, rotating
- **Dots:** 3 dots, 8px each, `#1E90FF`, staggered pulse animation
- **Skeleton:** Shimmer effect on placeholder blocks, `#262626` to `#333333` gradient

---

### 4.12 KimiEmptyState

**Purpose:** Empty state for lists, search results, and error states.

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct KimiEmptyStateProps {
    pub icon: Element,
    pub title: String,
    #[props(default = "")]
    pub description: String,
    pub action: Option<Element>,
}
```

**Styles:**
- Centered layout, flex column, gap 16px
- Icon: 48px, `#1E90FF`
- Title: 20px, `#F5F5F5`, weight 600
- Description: 14px, `#A3A3A3`
- Action: KimiButton primary or secondary

---

## 5. Layout Components

### 5.1 AppShell

**Purpose:** Root layout with sidebar, main content area, and optional right panel.

**Structure:**
```
┌─────────────────────────────────────────────────────────┐
│  Sidebar  │  Main Content Area        │  Right Panel  │
│  240px    │  (flexible)               │  280px (opt)  │
│           │                           │               │
│  - Nav    │  - Chat thread            │  - Environment│
│  - Projects│ - Input                  │  - Changes    │
│  - Chats  │  - Preview (optional)     │  - Commit     │
│           │                           │               │
└─────────────────────────────────────────────────────────┘
```

**Props:**
```rust
#[derive(Props, PartialEq, Clone)]
pub struct AppShellProps {
    pub sidebar: Element,
    pub children: Element,
    pub right_panel: Option<Element>,
    #[props(default = false)]
    pub right_panel_visible: bool,
}
```

**Breakpoints:**
- `< 1280px`: Sidebar collapses to 64px icons-only
- `< 1440px`: Right panel hides, toggle button appears in header
- `< 1024px`: Settings switches to single-pane with top tabs

---

### 5.2 Sidebar

**Purpose:** Navigation sidebar with projects, chats, and settings.

**Sections:**
1. **Logo/Brand:** KimiIcon (32px) + "Kimi Code" wordmark (optional, collapses)
2. **Navigation:** New chat, Search, Plugins, Automations
3. **Projects:** Expandable folders with task items and age badges
4. **Chats:** Recent chat list
5. **Footer:** Settings button, user profile

**Styles:**
- Background: `#141414`
- Width: 240px (64px collapsed)
- Border right: 1px `#2E2E2E`
- Padding: 12px 8px
- Nav item height: 36px
- Nav item padding: 0 8px
- Nav item radius: 8px
- Nav item hover: `#262626`
- Nav item active: `#262626` with left border 2px `#1E90FF`
- Project folder: icon + truncated name + age badge (12px, `#737373`)
- Age badge: 12px, `#737373`, right-aligned
- "Show more": 12px, `#737373`, hover `#A3A3A3`

---

### 5.3 ChatThread

**Purpose:** Scrollable message list with virtualized rendering.

**Features:**
- Virtualized scrolling (only render visible messages)
- Auto-scroll to bottom on new messages
- Scroll-to-bottom button when scrolled up
- Date separators between days

**Styles:**
- Background: `#141414`
- Padding: 16px
- Max-width: 720px centered
- Message gap: 24px
- Message padding: 0 (no bubble chrome)

---

### 5.4 ChatInput

**Purpose:** Primary input area with toolbar, context selectors, and action buttons.

**Structure:**
```
┌─────────────────────────────────────────┐
│  [Input field with placeholder]         │
│                                         │
├─────────────────────────────────────────┤
│  [+]  [Approve for me ▼]          [⚙] [🎤] [↑] │
├─────────────────────────────────────────┤
│  [📁 project ▼]  [💻 Work locally ▼]  [🔀 main ▼] │
└─────────────────────────────────────────┘
```

**Styles:**
- Container: `#1E1E1E`, 12px radius, border 1px `#2E2E2E`
- Focus: border `#1E90FF`, shadow `0 0 0 2px rgba(30,144,255,0.3)`
- Input field: transparent bg, 14px, `#F5F5F5`, placeholder `#737373`
- Toolbar: flex row, gap 8px, padding 8px 12px
- Context selectors: inline, below toolbar, 13px, `#A3A3A3`, icon + text + chevron
- Send button: 32px circle, `#1E90FF`, white arrow icon
- Model selector: 13px, `#A3A3A3`, right-aligned

---

### 5.5 StatusBar

**Purpose:** Bottom status bar with connection, model, and context info.

**Styles:**
- Background: `#0A0A0A`
- Height: 28px
- Padding: 0 12px
- Text: 12px, `#737373`
- Connection indicator: Kimi Blue dot (8px) when connected, `#737373` when disconnected
- Context usage: progress bar 4px height, `#262626` bg, `#1E90FF` fill

---

## 6. Icon System

### 6.1 Lucide Icon Mapping

All icons from the Lucide set. Use `dioxus-lucide` crate or inline SVG.

```rust
// src/icons/lucide.rs
// Re-export or wrap Lucide icons with Kimi color defaults

use dioxus::prelude::*;

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

// ... similar for all icons
```

### 6.2 Icon Color Rules

| State | Color | Usage |
|-------|-------|-------|
| Default | `#A3A3A3` | Inactive icons, navigation |
| Hover | `#F5F5F5` | Hovered icons |
| Active/Selected | `#1E90FF` | Active nav item, selected state |
| Disabled | `#525252` | Disabled icons |
| Success | `#22C55E` | Success status |
| Warning | `#EAB308` | Warning status |
| Error | `#EF4444` | Error status |

### 6.3 Icon Size Rules

| Context | Size | Stroke Width |
|---------|------|-------------|
| Navigation | 20px | 1.5px |
| Inline/Buttons | 16px | 1.5px |
| Small/Compact | 12px | 1.5px |
| Feature/Empty | 24px | 1.5px |
| Large/Decorative | 48px | 2px |

---

## 7. Animation Patterns

### 7.1 Component Animations

| Interaction | Animation | Duration | Easing |
|-------------|-----------|----------|--------|
| Button hover | Background color change | 150ms | ease-out |
| Button press | scale(0.98) | 100ms | ease-out |
| Card hover | Shadow elevation + border | 200ms | ease-out |
| Input focus | Border + glow | 150ms | ease-out |
| Toggle switch | Thumb slide | 150ms | ease-out |
| Dropdown open | scale(0.95→1) + fade | 150ms | cubic-bezier(0.4,0,0.2,1) |
| Dropdown close | fade + scale(1→0.95) | 100ms | ease-in |
| Modal open | fade + slide up 8px | 200ms | cubic-bezier(0.4,0,0.2,1) |
| Modal close | fade + slide down 8px | 150ms | ease-in |
| Toast in | slide from top + fade | 300ms | ease-out |
| Toast out | fade | 200ms | ease-in |
| Sidebar collapse | width 240→64 | 200ms | ease-out |
| Panel slide | translateX | 200ms | ease-out |
| Skeleton shimmer | gradient slide | 1.5s | linear infinite |
| Loading spinner | rotate | 1s | linear infinite |
| Loading dots | scale pulse | 1.4s | ease-in-out infinite |
| Streaming cursor | blink | 1s | step-end infinite |

### 7.2 Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

---

## 8. Responsive Breakpoints

| Name | Width | Behavior |
|------|-------|----------|
| `mobile` | < 768px | Full-screen modals, stacked layout, hidden sidebar |
| `tablet` | 768–1024px | Collapsed sidebar, single-pane settings |
| `desktop` | 1024–1280px | Full sidebar, hidden right panel |
| `wide` | 1280–1440px | Full sidebar, optional right panel |
| `ultrawide` | > 1440px | Full sidebar + right panel |

**Dioxus/Tailwind responsive prefixes:**
- `sm:` — 640px+
- `md:` — 768px+
- `lg:` — 1024px+
- `xl:` — 1280px+
- `2xl:` — 1536px+

---

## 9. File Structure

```
src/
├── design_tokens/
│   ├── mod.rs
│   ├── colors.rs
│   ├── typography.rs
│   ├── spacing.rs
│   ├── animation.rs
│   └── elevation.rs
├── components/
│   ├── base/
│   │   ├── mod.rs
│   │   ├── kimi_icon.rs
│   │   ├── kimi_button.rs
│   │   ├── kimi_input.rs
│   │   ├── kimi_card.rs
│   │   ├── kimi_toggle.rs
│   │   ├── kimi_badge.rs
│   │   ├── kimi_tooltip.rs
│   │   ├── kimi_dropdown.rs
│   │   ├── kimi_toast.rs
│   │   ├── kimi_loading.rs
│   │   └── kimi_empty_state.rs
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── app_shell.rs
│   │   ├── sidebar.rs
│   │   ├── chat_thread.rs
│   │   ├── chat_input.rs
│   │   ├── status_bar.rs
│   │   └── right_panel.rs
│   └── icons/
│       ├── mod.rs
│       └── lucide.rs
├── styles/
│   ├── global.css
│   ├── animations.css
│   └── scrollbar.css
└── main.rs
```

---

## 10. Implementation Order

Implement in this exact order — each step depends on previous:

| Step | Component(s) | Estimated Time | Why First |
|------|-------------|----------------|-----------|
| 1 | Design tokens (colors, typography, spacing, animation, elevation) | 2 hours | All other components depend on these |
| 2 | Global CSS (scrollbar, animations, reduced motion) | 1 hour | Foundation for all UI |
| 3 | KimiIcon | 1 hour | Brand identity, used everywhere |
| 4 | KimiButton | 1 hour | Most used interactive component |
| 5 | KimiInput | 1 hour | Core form element |
| 6 | KimiCard | 30 min | Container for content |
| 7 | KimiToggle | 30 min | Settings component |
| 8 | KimiBadge | 30 min | Status indicators |
| 9 | KimiTooltip | 1 hour | Accessibility, used by buttons |
| 10 | KimiDropdown | 1 hour | Menus, selectors |
| 11 | KimiToast | 1 hour | Notifications |
| 12 | KimiLoading | 30 min | Loading states |
| 13 | KimiEmptyState | 30 min | Empty views |
| 14 | AppShell + Sidebar | 2 hours | Root layout |
| 15 | ChatThread | 2 hours | Main content area |
| 16 | ChatInput | 2 hours | Primary interaction |
| 17 | StatusBar | 1 hour | Bottom bar |
| 18 | RightPanel | 1 hour | Optional panel |
| 19 | Icon system (Lucide mapping) | 2 hours | All icons |
| 20 | Responsive behavior | 2 hours | All breakpoints |

**Total estimated time:** ~20 hours for complete design system

---

## 11. Acceptance Criteria

The design system is complete when:

- [ ] All color tokens render correctly in a test page (swatch grid)
- [ ] All typography sizes render with correct line heights and weights
- [ ] All spacing values are consistent (4px base unit verified)
- [ ] KimiIcon renders in all 4 variants at multiple sizes
- [ ] KimiButton renders all 4 variants + loading + disabled states
- [ ] KimiInput shows focus ring, error state, and placeholder correctly
- [ ] KimiCard shows hover, active, and default states
- [ ] KimiToggle animates smoothly between on/off
- [ ] KimiDropdown opens/closes with correct animation
- [ ] KimiToast auto-dismisses after duration
- [ ] AppShell layout works at all breakpoints (resize browser to test)
- [ ] Sidebar collapses/expands at 1280px breakpoint
- [ ] Right panel hides/shows at 1440px breakpoint
- [ ] All Lucide icons render at correct sizes with correct colors
- [ ] Custom scrollbar appears on all scrollable areas
- [ ] Reduced motion respected when system preference is set
- [ ] No Codex colors (`#4a9eff`) remain anywhere in the codebase
- [ ] All text references "Kimi" not "Codex" or "GPT"

---

## 12. Dependencies

```toml
# Cargo.toml additions
[dependencies]
dioxus = { version = "0.5", features = ["desktop"] }
dioxus-desktop = "0.5"
# dioxus-lucide = "0.1"  # or inline SVG if crate unavailable

[build-dependencies]
tailwindcss-cli = "3.4"  # or use npm tailwindcss
```

```html
<!-- index.html -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">
<script src="https://cdn.tailwindcss.com"></script>
<script>
  tailwind.config = {
    theme: {
      extend: {
        colors: {
          'kimi-blue': '#1E90FF',
          'kimi-blue-hover': '#4AA8FF',
          'bg-deepest': '#0A0A0A',
          'bg-dark': '#141414',
          'bg-surface': '#1E1E1E',
          'bg-hover': '#262626',
          'bg-code': '#0F0F0F',
          'border-subtle': '#2E2E2E',
          'text-primary': '#F5F5F5',
          'text-secondary': '#A3A3A3',
          'text-tertiary': '#737373',
          'text-disabled': '#525252',
        },
        fontFamily: {
          'ui': ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Helvetica Neue', 'Arial', 'sans-serif'],
          'mono': ['JetBrains Mono', 'Fira Code', 'SF Mono', 'Menlo', 'Consolas', 'monospace'],
        },
      },
    },
  }
</script>
```

---

*End of Design System Implementation Document*
