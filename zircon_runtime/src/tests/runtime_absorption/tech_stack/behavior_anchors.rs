pub(super) fn assert_runtime_01_behavior_anchors() {
    let text_shaper_tests = include_str!("../../../ui/tests/text_shaper.rs");
    let physics_contract_mod = include_str!(
        "../../../../../zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/mod.rs"
    );
    let physics_contract_step = include_str!(
        "../../../../../zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract/step.rs"
    );
    for behavior_test_anchor in [
        "shared_text_shaper_matches_public_layout_entrypoint",
        "text_shaper_stack_uses_shared_text_service_for_font_backends",
        "empty_jolt_feature_slot_reports_unavailable_not_ready",
        "unavailable_jolt_backend_does_not_fallback_to_builtin_scene_tick",
    ] {
        assert!(
            text_shaper_tests.contains(behavior_test_anchor)
                || physics_contract_mod.contains(behavior_test_anchor)
                || physics_contract_step.contains(behavior_test_anchor),
            "Runtime 01 behavior test anchor `{behavior_test_anchor}` should stay visible to tech_stack_boundary"
        );
    }
}
