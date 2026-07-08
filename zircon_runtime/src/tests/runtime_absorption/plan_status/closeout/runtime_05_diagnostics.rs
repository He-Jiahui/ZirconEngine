#[test]
fn runtime_05_scene_failure_triage_records_minimum_lower_layer_diagnostics() {
    let runtime_05_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review = include_str!(
        "../../../../../../docs/engine-architecture/runtime-architecture-review-m0.md"
    );
    let convergence =
        include_str!("../../../../../../docs/engine-architecture/runtime-interface-convergence.md");

    let required_anchors = [
        "Runtime 05 scene:: lower-layer diagnostic matrix",
        "support-first-minimal-diagnostics-matrix",
        "graphics-scene-diagnostics",
        "render_product_streamer_tests",
        "render_product_material_property_tests",
        "render_pass_executor_registry",
        "deferred_lighting_shader_matches_scene_uniform_layout",
        "scene-asset-project-io-diagnostics",
        "scene_asset_toml_roundtrip_preserves_entities_and_bindings",
        "scene_assets_roundtrip_primitive_mesh_material_bindings",
        "scene_project_serialization_sources_do_not_store_editor_authoring_state",
        "ecs-scene-diagnostics",
        "ecs_query",
        "first_stage_updates_all_registered_event_channels",
        "render_extract_prepare_flushes_parent_reorder_and_active_changes",
        "world_bootstraps_with_renderable_defaults",
        "world_resolves_entity_paths_and_mutates_component_properties",
        "cargo test -p zircon_runtime --lib render_product_streamer_tests --locked",
        "cargo test -p zircon_runtime --lib scene_asset --locked",
        "cargo test -p zircon_runtime --lib ecs_query --locked",
    ];

    for (label, source) in [
        ("Runtime 05 closeout plan", runtime_05_plan),
        ("runtime index", runtime_index),
        ("M0 architecture review", review),
        ("runtime-interface convergence", convergence),
    ] {
        for required_anchor in required_anchors.iter().copied() {
            assert!(
                source.contains(required_anchor),
                "{label} should keep Runtime 05 lower-layer diagnostic matrix anchor `{required_anchor}`"
            );
        }
    }
}
