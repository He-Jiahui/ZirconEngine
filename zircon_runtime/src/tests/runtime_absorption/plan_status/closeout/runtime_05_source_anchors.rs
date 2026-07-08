#[test]
fn runtime_05_scene_failure_diagnostic_matrix_source_anchors_exist() {
    for (bucket, anchors) in [
        (
            "graphics-scene-diagnostics",
            GRAPHICS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS,
        ),
        (
            "scene-asset-project-io-diagnostics",
            SCENE_ASSET_PROJECT_IO_DIAGNOSTIC_SOURCE_ANCHORS,
        ),
        ("ecs-scene-diagnostics", ECS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS),
    ] {
        for (anchor, source) in anchors {
            assert!(
                source.contains(anchor),
                "Runtime 05 support-first bucket `{bucket}` should keep source anchor `{anchor}`"
            );
        }
    }
}

const GRAPHICS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS: &[(&str, &str)] = &[
    (
        "mod render_product_streamer_tests;",
        include_str!("../../../../graphics/scene/mod.rs"),
    ),
    (
        "fn render_product_streamer_dependency_readiness_change_invalidates_material_cache()",
        include_str!("../../../../graphics/scene/render_product_streamer_tests/readiness_diagnostics.rs"),
    ),
    (
        "mod render_product_material_property_tests;",
        include_str!("../../../../graphics/scene/mod.rs"),
    ),
    (
        "fn render_product_material_properties_prepare_uniform_payload()",
        include_str!("../../../../graphics/scene/render_product_material_property_tests.rs"),
    ),
    (
        "mod render_pass_executor_registry;",
        include_str!("../../../../graphics/scene/scene_renderer/graph_execution/mod.rs"),
    ),
    (
        "pub use render_pass_executor_registry::{RenderPassExecutorFn, RenderPassExecutorRegistry};",
        include_str!("../../../../graphics/scene/scene_renderer/graph_execution/mod.rs"),
    ),
    (
        "fn deferred_lighting_shader_matches_scene_uniform_layout()",
        include_str!("../../../../graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs"),
    ),
];

const SCENE_ASSET_PROJECT_IO_DIAGNOSTIC_SOURCE_ANCHORS: &[(&str, &str)] = &[
    (
        "fn scene_asset_toml_roundtrip_preserves_entities_and_bindings()",
        include_str!("../../../../asset/tests/assets/scene/foundation.rs"),
    ),
    (
        "fn scene_assets_roundtrip_primitive_mesh_material_bindings()",
        include_str!("../../../../scene/tests/asset_scene/mesh_bindings.rs"),
    ),
    (
        "fn scene_project_serialization_sources_do_not_store_editor_authoring_state()",
        include_str!("../../../../scene/tests/component_structure/project_serialization.rs"),
    ),
];

const ECS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS: &[(&str, &str)] = &[
    (
        "mod ecs_query;",
        include_str!("../../../../scene/tests/mod.rs"),
    ),
    (
        "fn first_stage_updates_all_registered_event_channels()",
        include_str!("../../../../scene/tests/ecs_events_messages.rs"),
    ),
    (
        "fn render_extract_prepare_flushes_parent_reorder_and_active_changes()",
        include_str!("../../../../scene/tests/ecs_schedule/render_extract.rs"),
    ),
    (
        "fn world_bootstraps_with_renderable_defaults()",
        include_str!("../../../../scene/tests/world_basics/world_state.rs"),
    ),
    (
        "fn world_resolves_entity_paths_and_mutates_component_properties()",
        include_str!("../../../../scene/tests/property_paths/runtime_mutation.rs"),
    ),
];
