use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_stats_product_diagnostics_tests_are_child_owners() {
    let parent = read_runtime_src("core/runtime/diagnostics/render_stats_store/product.rs");
    let test_parent =
        read_runtime_src("core/runtime/diagnostics/render_stats_store/product/tests.rs");
    let camera_targets = read_runtime_src(
        "core/runtime/diagnostics/render_stats_store/product/tests/camera_targets.rs",
    );
    let visibility_hzb_light = read_runtime_src(
        "core/runtime/diagnostics/render_stats_store/product/tests/visibility_hzb_light.rs",
    );
    let mesh_gpu_scene = read_runtime_src(
        "core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs",
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let large_file_doc = read_repo("docs/engine-architecture/large-file-ownership-m1.md");

    assert_contains_all(
        "render-stats product parent mounts test child owner",
        &parent,
        &["#[cfg(test)]", "mod tests;", "pub(super) fn record("],
    );
    for moved_test in [
        "fn render_product_diagnostics_record_texture_conversion_writeback_marker",
        "fn render_product_diagnostics_record_visibility_stats",
        "fn render_product_diagnostics_record_mesh_command_cache_counts",
    ] {
        assert!(
            !parent.contains(moved_test),
            "render_stats_store/product.rs should delegate {moved_test} to product/tests child owners"
        );
        assert!(
            !test_parent.contains(moved_test),
            "render_stats_store/product/tests.rs should mount child owners instead of defining {moved_test}"
        );
    }
    assert_contains_all(
        "render-stats product test parent mounts child owners",
        &test_parent,
        &[
            "mod camera_targets;",
            "mod mesh_gpu_scene;",
            "mod visibility_hzb_light;",
            "fn assert_series(",
        ],
    );
    assert_contains_all(
        "camera/capture diagnostics tests live in camera target child",
        &camera_targets,
        &[
            "render_product_diagnostics_record_texture_conversion_writeback_marker",
            "render_product_diagnostics_record_capture_source_report",
        ],
    );
    assert_contains_all(
        "visibility/HZB/light diagnostics tests live in visibility child",
        &visibility_hzb_light,
        &[
            "render_product_diagnostics_record_visibility_stats",
            "render_product_diagnostics_record_hzb_stats",
            "render_product_diagnostics_record_light_grid_stats",
        ],
    );
    assert_contains_all(
        "mesh/GPU diagnostics tests live in mesh child",
        &mesh_gpu_scene,
        &[
            "render_product_diagnostics_record_skinned_mesh_queue_count",
            "render_product_diagnostics_record_gpu_scene_upload_stats",
            "render_product_diagnostics_record_mesh_command_cache_counts",
        ],
    );

    for relative in [
        "core/runtime/diagnostics/render_stats_store/product.rs",
        "core/runtime/diagnostics/render_stats_store/product/tests.rs",
        "core/runtime/diagnostics/render_stats_store/product/tests/camera_targets.rs",
        "core/runtime/diagnostics/render_stats_store/product/tests/visibility_hzb_light.rs",
        "core/runtime/diagnostics/render_stats_store/product/tests/mesh_gpu_scene.rs",
    ] {
        let source = read_runtime_src(relative);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{relative} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }
}
