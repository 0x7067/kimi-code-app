//! Spec-conformance tests for design tokens (DESIGN_SYSTEM.md §3).

use super::*;
use spacing::Spacing;

#[test]
fn brand_and_semantic_colors_match_spec() {
    assert_eq!(Colors::KIMI_BLUE, "#6EA1FF");
    assert_eq!(Colors::KIMI_BLUE_HOVER, "#9BBDFF");
    assert_eq!(Colors::KIMI_BLUE_MUTED, "rgba(110, 161, 255, 0.16)");
    assert_eq!(Colors::BG_DEEPEST, "#050505");
    assert_eq!(Colors::BG_DARK, "#0A0A0C");
    assert_eq!(Colors::BG_SURFACE, "#101014");
    assert_eq!(Colors::BG_ELEVATED, "#18181D");
    assert_eq!(Colors::BG_HOVER, "#18181D");
    assert_eq!(Colors::BG_ACTIVE, "#202026");
    assert_eq!(Colors::BG_CODE, "#060608");
    assert_eq!(Colors::BORDER_SUBTLE, "rgba(255, 255, 255, 0.045)");
    assert_eq!(Colors::BORDER_DEFAULT, "rgba(255, 255, 255, 0.085)");
    assert_eq!(Colors::BORDER_STRONG, "rgba(255, 255, 255, 0.145)");
    assert_eq!(Colors::BORDER_ACTIVE, "#6EA1FF");
    assert_eq!(Colors::BORDER_HIGHLIGHT, "rgba(255, 255, 255, 0.22)");
    assert_eq!(Colors::TEXT_PRIMARY, "#F7F8FB");
    assert_eq!(Colors::TEXT_SECONDARY, "#C5C9D3");
    assert_eq!(Colors::TEXT_TERTIARY, "#8A8F9C");
    assert_eq!(Colors::TEXT_DISABLED, "#5C6270");
    assert_eq!(Colors::SUCCESS, "#34D399");
    assert_eq!(Colors::WARNING, "#FBBF24");
    assert_eq!(Colors::ERROR, "#F87171");
    assert_eq!(Colors::INFO, "#6EA1FF");
}

#[test]
fn no_codex_accent_color() {
    for c in [
        Colors::KIMI_BLUE,
        Colors::ACCENT,
        Colors::ACCENT_HOVER,
        Colors::BORDER_ACTIVE,
        Colors::INFO,
    ] {
        assert_ne!(c.to_lowercase(), "#4a9eff", "Codex accent color must not be used");
    }
}

#[test]
fn spacing_is_4px_grid() {
    assert_eq!(Spacing::UNIT, 4);
    for (s, n) in [
        (Spacing::S1, 1),
        (Spacing::S2, 2),
        (Spacing::S3, 3),
        (Spacing::S4, 4),
        (Spacing::S5, 5),
        (Spacing::S6, 6),
        (Spacing::S8, 8),
        (Spacing::S10, 10),
        (Spacing::S12, 12),
        (Spacing::S16, 16),
    ] {
        assert_eq!(s, n * Spacing::UNIT);
    }
}

#[test]
fn typography_scale_matches_spec() {
    assert_eq!(Typography::DISPLAY_SIZE, "24px");
    assert_eq!(Typography::H1_SIZE, "20px");
    assert_eq!(Typography::H2_SIZE, "16px");
    assert_eq!(Typography::H3_SIZE, "14px");
    assert_eq!(Typography::BODY_SIZE, "14px");
    assert_eq!(Typography::SMALL_SIZE, "13px");
    assert_eq!(Typography::CAPTION_SIZE, "12px");
    assert_eq!(Typography::CODE_SIZE, "13px");
}

#[test]
fn typography_weights_and_rhythm_match_spec() {
    assert_eq!(Typography::WEIGHT_NORMAL, "400");
    assert_eq!(Typography::WEIGHT_MEDIUM, "500");
    assert_eq!(Typography::WEIGHT_SEMIBOLD, "600");
    assert_eq!(Typography::WEIGHT_BOLD, "700");
    assert_eq!(Typography::LH_TIGHT, "1.2");
    assert_eq!(Typography::LH_SNUG, "1.3");
    assert_eq!(Typography::LH_NORMAL, "1.4");
    assert_eq!(Typography::LH_RELAXED, "1.5");
    assert_eq!(Typography::LH_CODE, "1.6");
    assert_eq!(Typography::LS_TIGHT, "-0.02em");
    assert_eq!(Typography::LS_SNUG, "-0.01em");
    assert_eq!(Typography::LS_NORMAL, "0");
    assert_eq!(Typography::LS_WIDE, "0.01em");
}

#[test]
fn animation_durations_match_spec() {
    assert_eq!(animation::Animation::MICRO, "150ms");
    assert_eq!(animation::Animation::FAST, "200ms");
    assert_eq!(animation::Animation::NORMAL, "300ms");
    assert_eq!(animation::Animation::SLOW, "500ms");
}

#[test]
fn layout_dimensions_match_spec() {
    assert_eq!(Spacing::SIDEBAR_WIDTH, 240);
    assert_eq!(Spacing::SIDEBAR_COLLAPSED, 64);
    assert_eq!(Spacing::CHAT_MAX_WIDTH, 720);
    assert_eq!(Spacing::RIGHT_PANEL_WIDTH, 280);
    assert_eq!(Spacing::SETTINGS_MAX_WIDTH, 640);
    assert_eq!(Spacing::INPUT_MIN_HEIGHT, 44);
}
