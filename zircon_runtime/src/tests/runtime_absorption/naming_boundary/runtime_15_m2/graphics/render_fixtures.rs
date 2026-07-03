use super::*;

#[test]
fn runtime_15_render_feature_fallback_capability_fixtures_use_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let submission_features = read_text(
        &manifest_root.join(
            "src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs",
        ),
        "render submission enabled-feature fixture source should be readable",
    );
    let runtime_features = read_text(
        &manifest_root.join(
            "src/graphics/scene/scene_renderer/core/runtime_features/runtime_features_from_pipeline.rs",
        ),
        "scene renderer runtime-feature fixture source should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let render_product_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary date slice should be readable",
    );

    assert_contains_all(
        "render feature fallback capability fixture ids",
        &(submission_features.clone() + "\n" + &runtime_features),
        &[
            "fallback-virtual-geometry-without-submission-capability",
            "fallback.hybrid-gi.without-submission-capability",
            "fallback-virtual-geometry-without-capability",
            "fallback.hybrid-gi.without-capability",
        ],
    );
    for retired_id in [
        "legacy-virtual-geometry-without-submission-capability",
        "legacy-hybrid-gi-without-submission-capability",
        "legacy.virtual-geometry.without-submission-capability",
        "legacy.hybrid-gi.without-submission-capability",
        "legacy-virtual-geometry-without-capability",
        "legacy-hybrid-gi-without-capability",
        "legacy.virtual-geometry.without-capability",
        "legacy.hybrid-gi.without-capability",
    ] {
        assert!(
            !submission_features.contains(retired_id) && !runtime_features.contains(retired_id),
            "render feature fallback capability fixtures should not retain retired `{retired_id}` IDs"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render product submit doc", render_product_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 render feature fallback capability naming hard cutover",
                "runtime_15_render_feature_fallback_capability_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_enabled_features.rs",
                "fallback-virtual-geometry-without-capability",
                "runtime_15_render_feature_fallback_capability_fixtures_use_current_names",
            ],
        );
    }
}

#[test]
fn runtime_15_render_material_stale_texture_fixtures_use_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let material_runtime = read_text(
        &manifest_root.join("src/graphics/scene/render_product_streamer_tests/material_runtime.rs"),
        "render product material runtime fixture source should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let render_assets_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/asset/render-assets.md");
    let zmeta_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/asset/zmeta-shader-material.md",
    );
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary date slice should be readable",
    );

    assert_contains_all(
        "render material stale texture fixture names",
        &material_runtime,
        &[
            "let stale_texture_id =",
            "res://textures/stale-base.png",
            "render_product_streamer_shader_standard_alias_shadows_unresolved_stale_texture",
            "res://textures/missing-stale-base.png",
            "shader standard texture alias shadows stale schema texture",
        ],
    );
    for retired_name in [
        "legacy_texture_id",
        "legacy-base.png",
        "missing-legacy-base.png",
        "unresolved_legacy_texture",
        "stale legacy texture",
    ] {
        assert!(
            !material_runtime.contains(retired_name),
            "render material stale texture fixtures should not retain retired `{retired_name}` wording"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render assets doc", render_assets_doc),
        ("zmeta shader material doc", zmeta_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 render material stale texture fixture naming hard cutover",
                "runtime_15_render_material_stale_texture_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/render_product_streamer_tests/material_runtime.rs",
                "unresolved_stale_texture",
                "runtime_15_render_material_stale_texture_fixtures_use_current_names",
            ],
        );
    }
}

#[test]
fn runtime_15_render_graph_fallback_fixtures_use_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let advanced_resources = read_text(
        &manifest_root.join(
            "src/graphics/scene/scene_renderer/core/scene_renderer_core/advanced_plugin_resources/scene_renderer_advanced_plugin_resources.rs",
        ),
        "advanced plugin resources fixture source should be readable",
    );
    let compute_workload = read_text(
        &manifest_root.join(
            "src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
        ),
        "render graph compute workload fixture source should be readable",
    );
    let runtime_15_plan = read_repo_text(
        manifest_root,
        "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
    );
    let runtime_index = read_repo_text(manifest_root, "docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-review-findings-2026-06.md",
    );
    let structure_convention = read_repo_text(
        manifest_root,
        "docs/plans/engine-code-structure-convention.md",
    );
    let module_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/structure/module-convention.md",
    );
    let render_product_doc = read_repo_text(
        manifest_root,
        "docs/zircon_runtime/graphics/render-product-submit.md",
    );
    let render_graph_doc =
        read_repo_text(manifest_root, "docs/zircon_runtime/render_graph/builder.md");
    let status_rows = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
        ),
        "Runtime 15 status rows should be readable",
    );
    let status_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/naming_boundary.rs",
        ),
        "Runtime 15 naming-boundary date slice should be readable",
    );

    assert_contains_all(
        "render graph fallback fixture names",
        &(advanced_resources.clone() + "\n" + &compute_workload),
        &[
            "fallback-virtual-geometry-without-resource-capability",
            "unexpected-compute",
            "unexpected.executor",
            "unexpected-pipeline",
        ],
    );
    for retired_name in [
        "legacy-virtual-geometry-without-resource-capability",
        "legacy-compute",
        "legacy.executor",
        "legacy-pipeline",
    ] {
        assert!(
            !advanced_resources.contains(retired_name) && !compute_workload.contains(retired_name),
            "render graph fallback fixtures should not retain retired `{retired_name}` wording"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render product submit doc", render_product_doc),
        ("render graph builder doc", render_graph_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 render graph fallback fixture naming hard cutover",
                "runtime_15_render_graph_fallback_fixture_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/compute_workload.rs",
                "unexpected-compute",
                "runtime_15_render_graph_fallback_fixtures_use_current_names",
            ],
        );
    }
}
