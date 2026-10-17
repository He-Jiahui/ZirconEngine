use super::*;

#[path = "../../../../../test_support/frame_smoke.rs"]
mod frame_smoke;

#[test]
fn decals_feature_registers_projector_component_and_pass() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert!(!report.manifest.enabled_by_default);
    assert_eq!(
        report.extensions.components()[0].type_id,
        DECAL_PROJECTOR_COMPONENT_TYPE
    );
    assert_eq!(
        report.extensions.render_features()[0].stage_passes[0].pass_name,
        "decal-projector-composite"
    );
}

#[test]
fn decals_feature_executes_in_nonempty_wgpu_product_frame() {
    let (frame, stats) = frame_smoke::render_nonempty_product_frame(
        [render_feature_descriptor()],
        [render_pass_executor_registration()],
    );

    assert!(
        !frame.rgba.is_empty(),
        "the WGPU product capture should contain pixels"
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|id| id == EXECUTOR_ID),
        "decal projector executor should run in the captured product frame: {:?}",
        stats.last_graph_executed_executor_ids
    );
}
