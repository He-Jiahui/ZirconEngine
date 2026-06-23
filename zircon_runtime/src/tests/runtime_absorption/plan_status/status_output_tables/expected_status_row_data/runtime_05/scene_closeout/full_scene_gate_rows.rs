use super::ExpectedStatusOutputSlice;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[ExpectedStatusOutputSlice] = &[
    (
        "Runtime 05 full scene closeout failed evidence",
        &[
            "cargo test -p zircon_runtime --lib scene:: --locked",
            "880 passed",
            "31 failed",
            "pending_full_scene_cargo",
        ],
    ),
    (
        "Runtime 05 full scene compile-pass graphics-scene blocker",
        &[
            "zircon_runtime_scene_closeout_20260620_040700",
            "running 1074 tests",
            "graphics::scene::*",
            "STATUS_ACCESS_VIOLATION",
        ],
    ),
    (
        "Runtime 05 full scene closeout no-result recheck",
        &[
            "zircon_runtime_scene_closeout_20260615_1806.log",
            "SCENE_CLOSEOUT_EXIT=-1",
            "无测试结果",
            "zircon-editor-ui-command-registry-0615",
        ],
    ),
    (
        "Runtime 05 scene:: failure support-first triage",
        &[
            "runtime_05_full_scene_failure_clusters_keep_support_first_triage_visible",
            "graphics-scene-lower-layer-candidate",
            "scene-asset-project-io-lower-layer-candidate",
            "ecs-scene-lower-layer-candidate",
        ],
    ),
    (
        "Runtime 05 scene:: lower-layer diagnostic matrix",
        &[
            "runtime_05_scene_failure_triage_records_minimum_lower_layer_diagnostics",
            "support-first-minimal-diagnostics-matrix",
            "graphics-scene-diagnostics",
            "scene-asset-project-io-diagnostics",
        ],
    ),
    (
        "Runtime 05 scene:: diagnostic matrix source anchors",
        &[
            "runtime_05_scene_failure_diagnostic_matrix_source_anchors_exist",
            "render_product_streamer_tests",
            "scene_asset_toml_roundtrip_preserves_entities_and_bindings",
            "world_resolves_entity_paths_and_mutates_component_properties",
        ],
    ),
    (
        "Runtime 05 render product streamer 2026-06-21 no-result diagnostic",
        &[
            "runtime_05_render_product_streamer_20260621_no_result_residual_stopped",
            "424s",
            "6 cargo and 3 rustc",
            "scene_asset` / `ecs_query",
        ],
    ),
    (
        "Runtime 05 scene_asset 2026-06-21 no-result diagnostic",
        &[
            "runtime_05_scene_asset_20260621_no_result_residual_stopped",
            "scene_asset --locked",
            "604s",
            "4 cargo and 2 rustc",
        ],
    ),
    (
        "Runtime 05 ecs_query 2026-06-21 no-result diagnostic",
        &[
            "runtime_05_ecs_query_20260621_no_result_residual_stopped",
            "ecs_query --locked",
            "604s",
            "no owner-file repair",
        ],
    ),
];
