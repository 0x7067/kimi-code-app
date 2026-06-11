//! Spec-conformance tests for design tokens (DESIGN_SYSTEM.md §3).

use super::*;
use spacing::Spacing;

#[test]
fn brand_and_semantic_colors_match_spec() {
    assert_eq!(Colors::KIMI_BLUE, "#1E90FF");
    assert_eq!(Colors::BG_DEEPEST, "#0A0A0A");
    assert_eq!(Colors::BG_DARK, "#141414");
    assert_eq!(Colors::BG_SURFACE, "#1E1E1E");
    assert_eq!(Colors::BG_HOVER, "#262626");
    assert_eq!(Colors::BG_CODE, "#0F0F0F");
    assert_eq!(Colors::BORDER_SUBTLE, "#2E2E2E");
    assert_eq!(Colors::BORDER_ACTIVE, "#1E90FF");
    assert_eq!(Colors::TEXT_PRIMARY, "#F5F5F5");
    assert_eq!(Colors::TEXT_SECONDARY, "#A3A3A3");
    assert_eq!(Colors::TEXT_TERTIARY, "#737373");
    assert_eq!(Colors::TEXT_DISABLED, "#525252");
    assert_eq!(Colors::SUCCESS, "#22C55E");
    assert_eq!(Colors::WARNING, "#EAB308");
    assert_eq!(Colors::ERROR, "#EF4444");
    assert_eq!(Colors::INFO, "#1E90FF");
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
