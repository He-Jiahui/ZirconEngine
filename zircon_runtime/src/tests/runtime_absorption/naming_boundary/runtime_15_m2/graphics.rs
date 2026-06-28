use std::fs;
use std::path::{Path, PathBuf};

use super::super::{assert_contains_all, read_repo_text, read_text};

#[path = "graphics/hybrid_gi.rs"]
mod hybrid_gi;

#[test]
fn runtime_15_render_framework_receiver_uses_framework_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_framework_dir = manifest_root.join("src/graphics/runtime/render_framework");
    let render_framework_files = rust_files(&render_framework_dir);
    let representative_files = [
        "src/graphics/runtime/render_framework/capture_frame/capture_frame.rs",
        "src/graphics/runtime/render_framework/create_viewport/create.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs",
        "src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs",
    ];
    let naming_boundary = read_text(
        &manifest_root.join("src/tests/runtime_absorption/naming_boundary.rs"),
        "runtime naming boundary guard should be readable",
    );
    let audit_script = read_repo_text(
        manifest_root,
        ".codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py",
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
    let graphics_doc = read_repo_text(
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !render_framework_files.is_empty(),
        "render framework source owner should contain Rust files"
    );
    for file in &render_framework_files {
        let source = read_text(file, "render framework source should be readable");
        assert!(
            !source.to_ascii_lowercase().contains("server"),
            "{} should not use non-network server naming for render-framework receiver/context variables",
            file.strip_prefix(manifest_root)
                .expect("render framework file should be under manifest root")
                .display()
        );
    }
    for relative_path in representative_files {
        let source = read_text(
            &manifest_root.join(relative_path),
            "representative render framework source should be readable",
        );
        assert_contains_all(
            relative_path,
            &source,
            &["framework: &WgpuRenderFramework", "framework.lock_"],
        );
    }
    assert!(
        !naming_boundary.contains("graphics-render-framework-debt"),
        "runtime naming boundary should not require the retired graphics render-framework server debt bucket"
    );
    assert!(
        !audit_script.contains("graphics-render-framework-debt"),
        "runtime structure audit should not retain the retired graphics render-framework server debt bucket"
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 graphics render-framework receiver naming hard cutover",
                "runtime_15_graphics_render_framework_receiver_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/runtime/render_framework",
                "runtime_15_render_framework_receiver_uses_framework_name",
            ],
        );
    }
}

#[test]
fn runtime_15_resource_streamer_construction_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let resource_streamer_dir =
        manifest_root.join("src/graphics/scene/resources/resource_streamer");
    let retired_resource_streamer_new = resource_streamer_dir.join("resource_streamer_new.rs");
    let resource_streamer_mod = read_text(
        &resource_streamer_dir.join("mod.rs"),
        "resource streamer module entry should be readable",
    );
    let resource_streamer_construction = read_text(
        &resource_streamer_dir.join("resource_streamer_construction.rs"),
        "resource streamer construction owner should be readable",
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
    let graphics_doc = read_repo_text(
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_resource_streamer_new.exists(),
        "resource streamer should not keep *_new construction owner file {:?}",
        retired_resource_streamer_new
    );
    assert_contains_all(
        "resource streamer module entry",
        &resource_streamer_mod,
        &["mod resource_streamer_construction;"],
    );
    assert!(
        !resource_streamer_mod.contains("mod resource_streamer_new;"),
        "resource_streamer/mod.rs should not preserve the retired resource_streamer_new module name"
    );
    assert_contains_all(
        "resource streamer construction owner",
        &resource_streamer_construction,
        &[
            "impl ResourceStreamer",
            "pub(crate) fn new(",
            "fallback_texture: Arc::new",
            "OutputTargetWritebackConverter::new(device)",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 resource streamer construction module naming hard cutover",
                "runtime_15_resource_streamer_construction_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/resources/resource_streamer/resource_streamer_construction.rs",
                "runtime_15_resource_streamer_construction_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_offscreen_target_construct_uses_owner_name() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let render_backend_dir = manifest_root.join("src/graphics/backend/render_backend");
    let retired_offscreen_target_new = render_backend_dir.join("offscreen_target_new");
    let offscreen_target_construct_dir = render_backend_dir.join("offscreen_target_construct");
    let render_backend_mod = read_text(
        &render_backend_dir.join("mod.rs"),
        "render backend module entry should be readable",
    );
    let construct_mod = read_text(
        &offscreen_target_construct_dir.join("mod.rs"),
        "offscreen target construct module entry should be readable",
    );
    let construct_owner = read_text(
        &offscreen_target_construct_dir.join("construct.rs"),
        "offscreen target construct owner should be readable",
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
    let graphics_doc = read_repo_text(
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
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15.rs",
        ),
        "Runtime 15 expected status slice should be readable",
    );
    let date_slice = read_text(
        &manifest_root.join(
            "src/tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15.rs",
        ),
        "Runtime 15 expected date slice should be readable",
    );

    assert!(
        !retired_offscreen_target_new.exists(),
        "render backend should not keep *_new construction owner directory {:?}",
        retired_offscreen_target_new
    );
    assert!(
        offscreen_target_construct_dir.is_dir(),
        "render backend should keep offscreen target construction in a construct-named directory"
    );
    assert_contains_all(
        "render backend module entry",
        &render_backend_mod,
        &["mod offscreen_target_construct;"],
    );
    assert!(
        !render_backend_mod.contains("mod offscreen_target_new;"),
        "render_backend/mod.rs should not preserve the retired offscreen_target_new module name"
    );
    assert_contains_all(
        "offscreen target construct module entry",
        &construct_mod,
        &[
            "mod construct;",
            "mod create_cluster_buffer;",
            "mod create_texture_bundle;",
            "mod texture_bundle;",
        ],
    );
    assert_contains_all(
        "offscreen target construct owner",
        &construct_owner,
        &[
            "impl OffscreenTarget",
            "pub(crate) fn new(",
            "zircon-offscreen-final-color",
            "create_cluster_buffer(device, cluster_buffer_bytes)",
        ],
    );

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("graphics render-product doc", graphics_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 offscreen target construct directory naming hard cutover",
                "runtime_15_offscreen_target_construct_naming_hard_cutover_static_passed_cargo_timeout_no_result",
                "graphics/backend/render_backend/offscreen_target_construct/construct.rs",
                "runtime_15_offscreen_target_construct_uses_owner_name",
            ],
        );
    }
}

#[test]
fn runtime_15_gpu_model_embedded_primitive_uses_current_names() {
    let manifest_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let gpu_model_source = read_text(
        &manifest_root
            .join("src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs"),
        "GPU model resource from asset owner should be readable",
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
        "GPU model embedded primitive source names",
        &gpu_model_source,
        &[
            "model_primitives_preferring_mesh_assets",
            "model_render_primitives_keep_embedded_payload_when_mesh_reference_unresolved",
            "let embedded = embedded_primitive(",
            "fn embedded_primitive(",
        ],
    );
    for retired_name in [
        "legacy_primitive",
        "keep_legacy_payload",
        "let legacy =",
        "vec![legacy]",
    ] {
        assert!(
            !gpu_model_source.contains(retired_name),
            "GPU model primitive fallback owner should not retain retired `{retired_name}` naming"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan),
        ("runtime index", runtime_index),
        ("review findings", review_findings),
        ("structure convention", structure_convention),
        ("module convention doc", module_doc),
        ("render assets doc", render_assets_doc),
        ("status row data", status_rows),
        ("status slice", status_slice),
        ("date slice", date_slice),
    ] {
        assert_contains_all(
            label,
            &source,
            &[
                "Runtime 15 M2 GPU model embedded primitive naming hard cutover",
                "runtime_15_gpu_model_embedded_primitive_naming_hard_cutover_static_passed_cargo_deferred",
                "graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs",
                "embedded primitive",
                "runtime_15_gpu_model_embedded_primitive_uses_current_names",
            ],
        );
    }
}

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

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("render framework directory should be readable") {
        let entry = entry.expect("render framework entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
