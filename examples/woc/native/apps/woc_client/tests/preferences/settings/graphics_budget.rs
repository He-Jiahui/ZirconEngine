use woc_client::{
    graphics_runtime_budget, GraphicsPreset, GraphicsRuntimeBudget, GraphicsTier,
    GRAPHICS_RUNTIME_BUDGETS,
};

#[test]
fn presets_resolve_to_the_target_runtime_tiers() {
    assert_eq!(GraphicsPreset::Low.runtime_tier(), GraphicsTier::Low);
    assert_eq!(GraphicsPreset::Medium.runtime_tier(), GraphicsTier::Medium);
    assert_eq!(GraphicsPreset::High.runtime_tier(), GraphicsTier::High);
    assert_eq!(GraphicsPreset::Ultra.runtime_tier(), GraphicsTier::Ultra);
    assert_eq!(GraphicsPreset::Advanced.runtime_tier(), GraphicsTier::High);
}

#[test]
fn all_tiers_present_at_sixty_hz_with_exact_target_budgets() {
    assert_eq!(
        GRAPHICS_RUNTIME_BUDGETS,
        [
            GraphicsRuntimeBudget {
                target_hz: 60,
                min_render_scale_desktop: 0.65,
                min_render_scale_mobile: 0.55,
                max_render_scale: 1.0,
                drop_frame_ms: 22.0,
                urgent_frame_ms: 34.0,
                recover_frame_ms: 17.5,
                drop_step: 0.08,
                urgent_drop_step: 0.12,
                recover_step: 0.06,
                recover_stable_seconds: 6.0,
                cooldown_seconds: 1.1,
            },
            GraphicsRuntimeBudget {
                target_hz: 60,
                min_render_scale_desktop: 0.72,
                min_render_scale_mobile: 0.55,
                max_render_scale: 1.0,
                drop_frame_ms: 24.0,
                urgent_frame_ms: 34.0,
                recover_frame_ms: 17.0,
                drop_step: 0.1,
                urgent_drop_step: 0.15,
                recover_step: 0.05,
                recover_stable_seconds: 7.0,
                cooldown_seconds: 1.35,
            },
            GraphicsRuntimeBudget {
                target_hz: 60,
                min_render_scale_desktop: 0.7,
                min_render_scale_mobile: 0.6,
                max_render_scale: 1.0,
                drop_frame_ms: 22.0,
                urgent_frame_ms: 32.0,
                recover_frame_ms: 15.0,
                drop_step: 0.1,
                urgent_drop_step: 0.15,
                recover_step: 0.05,
                recover_stable_seconds: 3.0,
                cooldown_seconds: 0.85,
            },
            GraphicsRuntimeBudget {
                target_hz: 60,
                min_render_scale_desktop: 0.78,
                min_render_scale_mobile: 0.68,
                max_render_scale: 1.0,
                drop_frame_ms: 24.0,
                urgent_frame_ms: 34.0,
                recover_frame_ms: 15.0,
                drop_step: 0.08,
                urgent_drop_step: 0.12,
                recover_step: 0.04,
                recover_stable_seconds: 3.0,
                cooldown_seconds: 0.85,
            },
        ]
    );
    for budget in GRAPHICS_RUNTIME_BUDGETS {
        assert_eq!(budget.target_hz, 60);
        assert_eq!(budget.presentation_interval_seconds(), 1.0 / 60.0);
    }
}

#[test]
fn mobile_changes_resolution_floor_without_changing_presentation_frequency() {
    for tier in GraphicsTier::ALL {
        let budget = graphics_runtime_budget(tier);
        assert_eq!(
            budget.min_render_scale(false),
            budget.min_render_scale_desktop
        );
        assert_eq!(
            budget.min_render_scale(true),
            budget.min_render_scale_mobile
        );
        assert_eq!(budget.target_hz, 60);
    }
}

#[test]
fn automatic_governor_is_enabled_for_every_tier_except_ultra() {
    assert!(GraphicsTier::Low.auto_governor_default());
    assert!(GraphicsTier::Medium.auto_governor_default());
    assert!(GraphicsTier::High.auto_governor_default());
    assert!(!GraphicsTier::Ultra.auto_governor_default());
}
