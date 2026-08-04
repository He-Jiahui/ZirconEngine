use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner() {
    let parent = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs");
    let tests = read_repo("zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry/tests.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let rhi_ui_doc = read_repo("docs/zircon_runtime/rhi/ui_surface.md");

    assert_contains_all(
        "WGPU UI surface geometry parent keeps geometry production entry points and child test mount",
        &parent,
        &[
            "#[cfg(test)]",
            "mod tests;",
            "pub(super) fn draw_items(",
            "pub(super) fn ordered_commands(",
            "pub(super) fn command_effective_rect(",
            "pub(super) fn text_bounds_from_rect(",
            "fn solid_vertices(",
            "fn rounded_rect_vertices(",
            "fn rounded_border_vertices(",
            "fn image_vertices(",
            "fn primitive_effective_rect(",
        ],
    );
    for moved_owner in [
        "fn solid_items(",
        "fn wgpu_ui_surface_generates_border_items_inside_damage(",
        "fn wgpu_ui_surface_damage_and_clip_trim_solid_item_geometry(",
        "fn wgpu_ui_surface_draw_items_sort_by_stable_z_order(",
        "fn wgpu_ui_surface_image_uvs_compose_clipped_rect_with_atlas_uv(",
        "fn wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip(",
        "UiSurfaceTextStyle",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "rhi_wgpu/ui_surface/geometry.rs should delegate geometry test owner `{moved_owner}` to geometry/tests.rs"
        );
    }

    assert_contains_all(
        "WGPU UI surface geometry test child owns geometry coverage and test-only helpers",
        &tests,
        &[
            "fn solid_items(",
            "fn wgpu_ui_surface_generates_border_items_inside_damage(",
            "fn wgpu_ui_surface_damage_and_clip_trim_solid_item_geometry(",
            "fn wgpu_ui_surface_draw_items_sort_by_stable_z_order(",
            "fn wgpu_ui_surface_generates_rounded_solid_vertices_for_quad_and_border(",
            "fn wgpu_ui_surface_image_uvs_follow_clipped_rect(",
            "fn wgpu_ui_surface_image_uvs_compose_clipped_rect_with_atlas_uv(",
            "fn wgpu_ui_surface_skips_image_with_invalid_atlas_uv(",
            "fn wgpu_ui_surface_text_bounds_clip_to_damage_and_command_clip(",
            "fn wgpu_ui_surface_text_skips_disjoint_damage(",
            "UiSurfaceImagePayload",
            "UiSurfaceTextStyle",
        ],
    );

    for (path, source) in [
        ("rhi_wgpu/ui_surface/geometry.rs", parent.as_str()),
        ("rhi_wgpu/ui_surface/geometry/tests.rs", tests.as_str()),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("RHI UI surface doc", rhi_ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M4 RHI WGPU UI surface geometry test owner split",
                "runtime_15_rhi_wgpu_ui_surface_geometry_tests_owner_split_static_passed_cargo_timeout_no_result",
                "rhi_wgpu/ui_surface/geometry.rs",
                "rhi_wgpu/ui_surface/geometry/tests.rs",
                "runtime_15_rhi_wgpu_ui_surface_geometry_tests_are_child_owner",
            ],
        );
    }
}
