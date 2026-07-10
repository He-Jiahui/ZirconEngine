use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_pipeline_compile_monolith_tests_are_child_owners() {
    let root = read_runtime_src("graphics/tests/pipeline_compile.rs");
    let default_pipelines =
        read_runtime_src("graphics/tests/pipeline_compile/default_pipelines.rs");
    let dynamic_resolution =
        read_runtime_src("graphics/tests/pipeline_compile/dynamic_resolution.rs");
    let plugin_features = read_runtime_src("graphics/tests/pipeline_compile/plugin_features.rs");
    let temporal = read_runtime_src("graphics/tests/pipeline_compile/temporal_and_ops.rs");
    let compile_options = read_runtime_src("graphics/tests/pipeline_compile/compile_options.rs");
    let feature_descriptors =
        read_runtime_src("graphics/tests/pipeline_compile/feature_descriptors.rs");
    let validation_core = read_runtime_src("graphics/tests/pipeline_compile/validation_core.rs");
    let validation_descriptors =
        read_runtime_src("graphics/tests/pipeline_compile/validation_descriptors.rs");

    let plan_01 = read_repo("docs/plans/zircon_runtime/render/01/2026-07-09-render-graph-rdg-alignment-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let pass_authoring_doc =
        read_repo("docs/zircon_runtime/graphics/pipeline/render_pipeline_asset/pass_authoring.md");
    let resource_descriptors_doc = read_repo(
        "docs/zircon_runtime/graphics/pipeline/render_pipeline_asset/resource_descriptors.md",
    );
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "pipeline compile root keeps shared fixtures and child mounts",
        &root,
        &[
            "mod compile_options;",
            "mod default_pipelines;",
            "mod dynamic_resolution;",
            "mod feature_descriptors;",
            "mod plugin_features;",
            "mod temporal_and_ops;",
            "mod validation_core;",
            "mod validation_descriptors;",
            "fn default_rendering_feature_descriptors(",
            "fn rendering_post_process_descriptor(",
            "fn test_extract(",
            "fn extract_with_camera(",
            "fn pass_resource_access<",
            "fn graph_resource_lifetime<",
            "fn orthographic_extract(",
        ],
    );

    for (moved_anchor, owner_name, owner_source) in [
        (
            "fn default_forward_plus_pipeline_compiles_expected_stage_order_and_passes(",
            "default_pipelines.rs",
            default_pipelines.as_str(),
        ),
        (
            "fn default_deferred_pipeline_compiles_expected_stage_order_and_passes(",
            "default_pipelines.rs",
            default_pipelines.as_str(),
        ),
        (
            "fn dynamic_resolution_scales_internal_graph_resources_without_resizing_viewport_output(",
            "dynamic_resolution.rs",
            dynamic_resolution.as_str(),
        ),
        (
            "fn default_core2d_pipeline_compiles_expected_stage_order_and_passes(",
            "dynamic_resolution.rs",
            dynamic_resolution.as_str(),
        ),
        (
            "fn rendering_plugin_default_features_restore_legacy_forward_plus_pass_order(",
            "plugin_features.rs",
            plugin_features.as_str(),
        ),
        (
            "fn rendering_plugin_post_process_routes_output_transfer_through_terminal_anti_alias_input(",
            "plugin_features.rs",
            plugin_features.as_str(),
        ),
        (
            "fn taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective(",
            "temporal_and_ops.rs",
            temporal.as_str(),
        ),
        (
            "fn pipeline_compile_assigns_attachment_ops_from_resource_write_order(",
            "temporal_and_ops.rs",
            temporal.as_str(),
        ),
        (
            "fn compile_options_can_disable_clustered_history_and_rendering_plugin_features(",
            "compile_options.rs",
            compile_options.as_str(),
        ),
        (
            "fn compile_options_fallback_async_compute_passes_to_graphics_queue(",
            "compile_options.rs",
            compile_options.as_str(),
        ),
        (
            "fn feature_pass_descriptors_drive_executor_ids_and_resource_graph(",
            "feature_descriptors.rs",
            feature_descriptors.as_str(),
        ),
        (
            "fn compiled_pipeline_resources_use_extract_viewport_hdr_and_msaa_descriptors(",
            "feature_descriptors.rs",
            feature_descriptors.as_str(),
        ),
        (
            "fn pipeline_compile_rejects_duplicate_stage_entries(",
            "validation_core.rs",
            validation_core.as_str(),
        ),
        (
            "fn renderer_feature_asset_descriptor_override_changes_compiled_graph(",
            "validation_core.rs",
            validation_core.as_str(),
        ),
        (
            "fn pipeline_compile_rejects_descriptor_passes_for_undeclared_stages(",
            "validation_descriptors.rs",
            validation_descriptors.as_str(),
        ),
        (
            "fn pipeline_compile_rejects_duplicate_history_bindings_in_one_descriptor(",
            "validation_descriptors.rs",
            validation_descriptors.as_str(),
        ),
    ] {
        assert!(
            !root.contains(moved_anchor),
            "pipeline_compile.rs should delegate `{moved_anchor}` to {owner_name}"
        );
        assert!(
            owner_source.contains(moved_anchor),
            "{owner_name} should contain `{moved_anchor}`"
        );
    }

    for (label, source) in [
        ("default pipeline child", default_pipelines.as_str()),
        ("dynamic resolution child", dynamic_resolution.as_str()),
        ("plugin feature child", plugin_features.as_str()),
        ("temporal/ops child", temporal.as_str()),
        ("compile options child", compile_options.as_str()),
        ("feature descriptors child", feature_descriptors.as_str()),
        ("validation core child", validation_core.as_str()),
        (
            "validation descriptors child",
            validation_descriptors.as_str(),
        ),
    ] {
        assert!(
            source.contains("use super::*;"),
            "{label} should import shared pipeline compile fixtures from the root owner"
        );
    }

    for (path, source) in [
        ("graphics/tests/pipeline_compile.rs", root.as_str()),
        (
            "graphics/tests/pipeline_compile/default_pipelines.rs",
            default_pipelines.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/dynamic_resolution.rs",
            dynamic_resolution.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/plugin_features.rs",
            plugin_features.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/temporal_and_ops.rs",
            temporal.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/compile_options.rs",
            compile_options.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/feature_descriptors.rs",
            feature_descriptors.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/validation_core.rs",
            validation_core.as_str(),
        ),
        (
            "graphics/tests/pipeline_compile/validation_descriptors.rs",
            validation_descriptors.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R1.4 pipeline compile test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 01", plan_01.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("pass authoring docs", pass_authoring_doc.as_str()),
        (
            "resource descriptor docs",
            resource_descriptors_doc.as_str(),
        ),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Pipeline compile monolith tests owner split",
                "render_pipeline_compile_monolith_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/pipeline_compile.rs",
                "graphics/tests/pipeline_compile/default_pipelines.rs",
                "graphics/tests/pipeline_compile/dynamic_resolution.rs",
                "graphics/tests/pipeline_compile/plugin_features.rs",
                "graphics/tests/pipeline_compile/temporal_and_ops.rs",
                "graphics/tests/pipeline_compile/compile_options.rs",
                "graphics/tests/pipeline_compile/feature_descriptors.rs",
                "graphics/tests/pipeline_compile/validation_core.rs",
                "graphics/tests/pipeline_compile/validation_descriptors.rs",
                "runtime_15_pipeline_compile_monolith_tests_are_child_owners",
            ],
        );
    }
}
