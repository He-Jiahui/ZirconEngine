use super::support::frontmatter_status;

#[test]
fn runtime_05_closeout_status_waits_for_full_scene_cargo_gate() {
    let source = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );

    assert_eq!(
        frontmatter_status(source),
        Some("in_progress"),
        "Runtime 05 should not be completed until the full scene:: Cargo gate closes"
    );
    for required_anchor in [
        "pending_full_scene_cargo",
        "cargo test -p zircon_runtime --lib scene:: --locked",
        "frontmatter 从 `completed` 修正为 `in_progress`",
    ] {
        assert!(
            source.contains(required_anchor),
            "Runtime 05 closeout plan should record `{required_anchor}`"
        );
    }
}

#[test]
fn runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible() {
    let runtime_05_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let convergence =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");

    let required_anchors = [
        "Runtime 05 scene:: failure support-first triage",
        "graphics-scene-lower-layer-candidate",
        "scene-asset-project-io-lower-layer-candidate",
        "ecs-scene-lower-layer-candidate",
        "support-first-scene-closeout-triage-before-owner-edits",
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
                "{label} should keep Runtime 05 support-first triage anchor `{required_anchor}`"
            );
        }
    }
}

#[test]
fn runtime_05_scene_failure_triage_records_minimum_lower_layer_diagnostics() {
    let runtime_05_plan = include_str!(
        "../../../../../docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md"
    );
    let runtime_index = include_str!("../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let review =
        include_str!("../../../../../docs/engine-architecture/runtime-architecture-review-m0.md");
    let convergence =
        include_str!("../../../../../docs/engine-architecture/runtime-interface-convergence.md");

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
        include_str!("../../../graphics/scene/mod.rs"),
    ),
    (
        "fn render_product_streamer_dependency_readiness_change_invalidates_material_cache()",
        include_str!("../../../graphics/scene/render_product_streamer_tests/readiness_diagnostics.rs"),
    ),
    (
        "mod render_product_material_property_tests;",
        include_str!("../../../graphics/scene/mod.rs"),
    ),
    (
        "fn render_product_material_properties_prepare_uniform_payload()",
        include_str!("../../../graphics/scene/render_product_material_property_tests.rs"),
    ),
    (
        "mod render_pass_executor_registry;",
        include_str!("../../../graphics/scene/scene_renderer/graph_execution/mod.rs"),
    ),
    (
        "pub use render_pass_executor_registry::{RenderPassExecutorFn, RenderPassExecutorRegistry};",
        include_str!("../../../graphics/scene/scene_renderer/graph_execution/mod.rs"),
    ),
    (
        "fn deferred_lighting_shader_matches_scene_uniform_layout()",
        include_str!("../../../graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs"),
    ),
];

const SCENE_ASSET_PROJECT_IO_DIAGNOSTIC_SOURCE_ANCHORS: &[(&str, &str)] = &[
    (
        "fn scene_asset_toml_roundtrip_preserves_entities_and_bindings()",
        include_str!("../../../asset/tests/assets/scene.rs"),
    ),
    (
        "fn scene_assets_roundtrip_primitive_mesh_material_bindings()",
        include_str!("../../../scene/tests/asset_scene.rs"),
    ),
    (
        "fn scene_project_serialization_sources_do_not_store_editor_authoring_state()",
        include_str!("../../../scene/tests/component_structure.rs"),
    ),
];

const ECS_SCENE_DIAGNOSTIC_SOURCE_ANCHORS: &[(&str, &str)] = &[
    (
        "mod ecs_query;",
        include_str!("../../../scene/tests/mod.rs"),
    ),
    (
        "fn first_stage_updates_all_registered_event_channels()",
        include_str!("../../../scene/tests/ecs_events_messages.rs"),
    ),
    (
        "fn render_extract_prepare_flushes_parent_reorder_and_active_changes()",
        include_str!("../../../scene/tests/ecs_schedule.rs"),
    ),
    (
        "fn world_bootstraps_with_renderable_defaults()",
        include_str!("../../../scene/tests/world_basics.rs"),
    ),
    (
        "fn world_resolves_entity_paths_and_mutates_component_properties()",
        include_str!("../../../scene/tests/property_paths.rs"),
    ),
];
