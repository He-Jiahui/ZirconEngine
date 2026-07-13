use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_surface_targets_texture_target_tests_are_child_owner() {
    let parent = read_runtime_src("graphics/tests/surface_targets.rs");
    let texture_target = read_runtime_src("graphics/tests/surface_targets/texture_target.rs");

    let plan_09 = read_repo(
        "docs/plans/zircon_runtime/render/09/2026-07-09-camera-render-ordering-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "surface-target parent keeps surface/headless tests, shared fixtures, and texture-target child mount",
        &parent,
        &[
            "mod texture_target;",
            "fn graphics_surface_default_contract_reports_unsupported_present_and_noop_unbind(",
            "fn graphics_surface_offscreen_submit_and_capture_survive_unbind_noop(",
            "fn graphics_camera_target_headless_size_controls_offscreen_capture_size(",
            "fn graphics_camera_target_headless_present_reports_unsupported_surface_fallback(",
            "fn graphics_surface_present_path_source_uses_swapchain_present_without_readback_fallback(",
            "fn empty_extract_with_target(",
            "fn render_target_texture_asset(",
            "trait CameraDescriptorTestExt",
        ],
    );

    for moved_anchor in [
        "fn graphics_camera_target_texture_missing_asset_reports_unsupported_without_primary_fallback_capture(",
        "fn graphics_camera_target_texture_requires_render_target_usage(",
        "fn graphics_camera_target_texture_requires_renderable_render_target_format(",
        "fn graphics_camera_target_texture_render_target_metadata_controls_offscreen_capture_size(",
        "fn graphics_camera_target_texture_srgb_target_imports_direct_graph_final_target(",
        "fn graphics_camera_target_texture_overlay_stack_preserves_base_composite(",
        "fn graphics_camera_target_texture_base_stacks_write_independent_texture_targets(",
        "fn graphics_camera_target_texture_present_reports_unsupported_surface_fallback(",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "surface_targets.rs should delegate `{moved_anchor}` to texture_target.rs"
        );
        assert!(
            texture_target.contains(moved_anchor),
            "surface_targets texture-target child should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "texture-target child keeps texture asset setup, graph-import telemetry, and shared parent fixtures",
        &texture_target,
        &[
            "use super::{",
            "AssetUri",
            "TextureAsset",
            "RenderCameraTargetGraphImportStatus",
            "RenderCameraTargetWritebackStatus",
            "CameraDescriptorTestExt as _",
            "read_output_target_texture_rgba_for_tests",
        ],
    );

    for (path, source) in [
        ("graphics/tests/surface_targets.rs", parent.as_str()),
        (
            "graphics/tests/surface_targets/texture_target.rs",
            texture_target.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 render surface target test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 09", plan_09.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Surface targets texture-target tests owner split",
                "render_plan09_surface_targets_texture_target_test_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/surface_targets.rs",
                "graphics/tests/surface_targets/texture_target.rs",
                "runtime_15_surface_targets_texture_target_tests_are_child_owner",
            ],
        );
    }
}
