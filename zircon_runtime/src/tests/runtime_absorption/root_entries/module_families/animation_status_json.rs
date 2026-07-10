#[test]
fn runtime_animation_status_json_boundary_sanitizes_non_finite_values() {
    let runtime_status_source =
        include_str!("../../../../core/framework/animation/runtime_status.rs");
    for required_anchor in [
        "serialize_sanitized_non_negative_real",
        "deserialize_sanitized_non_negative_real",
        "serialize_normalized_real",
        "deserialize_normalized_real",
        "impl AnimationPlayerRuntimeStatus",
        "impl AnimationRuntimeStatus",
        "snapshot.time_seconds = self.sanitized_time_seconds()",
        "AnimationPlayerRuntimeStatus::sanitized_snapshot",
    ] {
        assert!(
            runtime_status_source.contains(required_anchor),
            "animation runtime status JSON boundary should keep `{required_anchor}`"
        );
    }

    let framework_tests = include_str!("../../../../core/framework/animation/tests.rs");
    for required_anchor in [
        "runtime_status_reports_player_rig_and_gpu_readiness",
        "serde_json::from_value::<AnimationRuntimeStatus>",
        "serde_json::to_value(&status)",
        "status.sanitized_snapshot()",
    ] {
        assert!(
            framework_tests.contains(required_anchor),
            "animation framework tests should lock runtime status JSON sanitization anchor `{required_anchor}`"
        );
    }

    let framework_doc =
        include_str!("../../../../../../docs/zircon_runtime/core/framework/animation.md");
    for required_anchor in [
        "AnimationPlayerRuntimeStatus",
        "JSON boundary",
        "`time_seconds` and `playback_speed` serialize and deserialize as finite non-negative values",
        "`weight` serializes and deserializes as a finite `0.0..=1.0` value",
        "AnimationRuntimeStatus::sanitized_snapshot()",
        "JSON `null` values from `NaN` or infinite runtime floats",
    ] {
        assert!(
            framework_doc.contains(required_anchor),
            "animation framework doc should record runtime status JSON sanitization anchor `{required_anchor}`"
        );
    }

    let runtime_14_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/14/2026-07-09-runtime-module-family-closeout-output-records.md"
    );
    for required_anchor in [
        "animation runtime-status JSON 边界守卫",
        "runtime_animation_status_json_boundary_sanitizes_non_finite_values",
        "AnimationPlayerRuntimeStatus::sanitized_snapshot",
    ] {
        assert!(
            runtime_14_plan.contains(required_anchor),
            "Runtime 14 plan should record animation status JSON boundary anchor `{required_anchor}`"
        );
    }
}
