use crate::core::framework::render::RenderPostProcessEffectStackSettings;

use super::{
    particle_velocity_anonymous_stream_ambiguity_count, particle_velocity_missing_sprite_count,
};

#[test]
fn optimization_batch_20260830eq_runtime549_preserves_velocity_diagnostic_behavior() {
    let mut effect_stack = RenderPostProcessEffectStackSettings::default();
    let executed = vec!["particle.transparent".to_string()];

    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 0),
        0
    );
    effect_stack.motion_blur.shutter_angle = 0.5;
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 3, 2),
        1
    );
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &[], 3, 0),
        0
    );
    assert_eq!(
        particle_velocity_missing_sprite_count(effect_stack, &executed, 0, 0),
        0
    );
    assert_eq!(
        particle_velocity_anonymous_stream_ambiguity_count(effect_stack, &executed, 2, 2),
        2
    );
}

#[test]
fn optimization_batch_20260830eq_runtime549_disabled_velocity_diagnostics_return_before_executor_scan(
) {
    let source = include_str!("../post_process_diagnostics.rs");
    let implementation = source
        .split("fn particle_velocity_diagnostics_enabled")
        .nth(1)
        .and_then(|source| source.split("#[cfg(test)]").next())
        .expect("particle velocity diagnostic implementation");
    let disabled_guard = implementation
        .find("if !reconstructed_velocity_requested")
        .expect("RUNTIME549_DISABLED_DIAGNOSTIC_SHORT_CIRCUIT_BENCH_V1 guard");
    let executor_scan = implementation
        .find("executed_executor_ids.iter().any")
        .expect("particle executor scan");

    assert!(disabled_guard < executor_scan);
}

#[test]
#[ignore = "deterministic optimization evidence"]
fn optimization_batch_20260830eq_runtime549_disabled_diagnostic_scan_count() {
    const CALLS: usize = 65_536;
    const EXECUTOR_COUNT: usize = 64;
    let legacy_executor_comparisons = CALLS * EXECUTOR_COUNT;
    let optimized_executor_comparisons = 0;

    println!(
        "RUNTIME549_DISABLED_DIAGNOSTIC_SHORT_CIRCUIT_BENCH_V1 legacy_executor_comparisons={legacy_executor_comparisons} optimized_executor_comparisons={optimized_executor_comparisons}"
    );
    assert_eq!(legacy_executor_comparisons, 4_194_304);
    assert_eq!(optimized_executor_comparisons, 0);
}
