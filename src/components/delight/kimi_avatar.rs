//! KimiAvatar — compact visual agent-state indicator.

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AgentState {
    Idle,
    Listening,
    Thinking,
    Working,
    Complete,
    Error,
    Approval,
}

pub fn derive_agent_state(
    has_error: bool,
    has_permission: bool,
    running: bool,
    has_active_tool: bool,
    has_draft: bool,
    completed_recently: bool,
) -> AgentState {
    if has_error {
        AgentState::Error
    } else if has_permission {
        AgentState::Approval
    } else if running && has_active_tool {
        AgentState::Working
    } else if running {
        AgentState::Thinking
    } else if has_draft {
        AgentState::Listening
    } else if completed_recently {
        AgentState::Complete
    } else {
        AgentState::Idle
    }
}

#[component]
pub fn KimiAvatar(
    #[props(default = AgentState::Idle)] state: AgentState,
    #[props(default = 34)] size: u32,
) -> Element {
    let class = state.class_name();
    let dot_size = (size / 4).max(6);
    let glyph_size = (size as f32 * 0.56) as u32;

    rsx! {
        div {
            class: "kimi-avatar state-{class}",
            style: "width: {size}px; height: {size}px;",
            span {
                class: "kimi-avatar-glyph",
                style: "font-size: {glyph_size}px;",
                "K"
            }
            span {
                class: "kimi-avatar-dot",
                style: "width: {dot_size}px; height: {dot_size}px;",
            }
            if state == AgentState::Thinking {
                span { class: "kimi-avatar-orbit one" }
                span { class: "kimi-avatar-orbit two" }
                span { class: "kimi-avatar-orbit three" }
            }
            if state == AgentState::Working {
                span { class: "kimi-avatar-ring" }
            }
            if state == AgentState::Complete {
                span { class: "kimi-avatar-check", "✓" }
            }
        }
    }
}

impl AgentState {
    pub fn class_name(self) -> &'static str {
        match self {
            AgentState::Idle => "idle",
            AgentState::Listening => "listening",
            AgentState::Thinking => "thinking",
            AgentState::Working => "working",
            AgentState::Complete => "complete",
            AgentState::Error => "error",
            AgentState::Approval => "approval",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AgentState::Idle => "Ready",
            AgentState::Listening => "Listening",
            AgentState::Thinking => "Thinking",
            AgentState::Working => "Working",
            AgentState::Complete => "Complete",
            AgentState::Error => "Needs attention",
            AgentState::Approval => "Approval needed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_priority_order_for_agent_presence() {
        assert_eq!(
            derive_agent_state(true, true, true, true, true, true),
            AgentState::Error
        );
        assert_eq!(
            derive_agent_state(false, true, true, true, true, true),
            AgentState::Approval
        );
        assert_eq!(
            derive_agent_state(false, false, true, true, false, false),
            AgentState::Working
        );
        assert_eq!(
            derive_agent_state(false, false, true, false, false, false),
            AgentState::Thinking
        );
        assert_eq!(
            derive_agent_state(false, false, false, false, true, false),
            AgentState::Listening
        );
        assert_eq!(
            derive_agent_state(false, false, false, false, false, true),
            AgentState::Complete
        );
        assert_eq!(
            derive_agent_state(false, false, false, false, false, false),
            AgentState::Idle
        );
    }
}
