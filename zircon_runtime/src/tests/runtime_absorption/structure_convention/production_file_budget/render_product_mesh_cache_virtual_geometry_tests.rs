use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_product_mesh_cache_virtual_geometry_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/render_product_mesh_cache.rs");
    let virtual_geometry =
        read_runtime_src("graphics/tests/render_product_mesh_cache/virtual_geometry.rs");

    let plan_02 = read_repo(
        "docs/plans/zircon_runtime/render/02/2026-07-09-mesh-draw-command-pipeline-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let mesh_pass_doc =
        read_repo("docs/zircon_runtime/graphics/scene/scene_renderer/mesh/mesh_pass.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "mesh-cache product parent keeps compact cache product tests, shared fixtures, and child mount",
        &parent,
        &[
            "mod virtual_geometry;",
            "fn render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse(",
            "fn render_product_static_mesh_material_revision_invalidates_pre_mesh_cache(",
            "fn render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path(",
            "fn render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache(",
            "fn render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache(",
            "fn static_cache_extract(",
            "fn static_command_cache_mesh(",
        ],
    );

    for moved_anchor in [
        "fn render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache(",
        "fn static_cache_virtual_geometry_extract(",
        "fn static_cache_virtual_geometry_visibility_mesh(",
        "fn static_cache_virtual_geometry_cluster(",
        "fn static_cache_virtual_geometry_page(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "render_product_mesh_cache.rs should delegate `{moved_anchor}` to virtual_geometry.rs"
        );
        assert!(
            virtual_geometry.contains(moved_anchor),
            "virtual geometry child owner should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "mesh-cache virtual-geometry child keeps authored VG and indirect execution product guards",
        &virtual_geometry,
        &[
            "pluginized_wgpu_render_framework_with_advanced_providers",
            "with_virtual_geometry(true)",
            "RenderVirtualGeometryPayloadSource::Authored",
            "RenderVirtualGeometryExtract",
            "last_virtual_geometry_indirect_draw_count",
            "last_mesh_pending_static_command_cache_draw_candidate_count",
        ],
    );

    for (path, source) in [
        (
            "graphics/tests/render_product_mesh_cache.rs",
            parent.as_str(),
        ),
        (
            "graphics/tests/render_product_mesh_cache/virtual_geometry.rs",
            virtual_geometry.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render product test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 02", plan_02.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("mesh pass docs", mesh_pass_doc.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Render product mesh-cache virtual-geometry test owner split",
                "render_plan02_product_mesh_cache_virtual_geometry_test_owner_split_static_passed_cargo_deferred_active_compile_lane",
                "graphics/tests/render_product_mesh_cache.rs",
                "graphics/tests/render_product_mesh_cache/virtual_geometry.rs",
                "runtime_15_render_product_mesh_cache_virtual_geometry_tests_are_child_owner",
            ],
        );
    }
}
