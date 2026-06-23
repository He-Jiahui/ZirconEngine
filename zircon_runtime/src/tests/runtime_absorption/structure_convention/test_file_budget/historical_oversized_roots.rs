use super::*;

#[test]
fn runtime_15_historical_oversized_test_roots_are_folder_backed() {
    let test_file_budget_parent =
        read_runtime_src("tests/runtime_absorption/structure_convention/test_file_budget/mod.rs");
    let core_framework_parent = read_runtime_src("core/framework/tests.rs");
    let core_framework_surface = read_runtime_src("core/framework/tests/framework_surfaces.rs");
    let core_render_product = read_runtime_src("core/framework/tests/render_product_surface.rs");
    let core_phase_queue = read_runtime_src("core/framework/tests/phase_queue_summary.rs");
    let ui_v2_parent = read_runtime_src("ui/tests/v2_asset.rs");
    let ui_v2_style = read_runtime_src("ui/tests/v2_asset/style_runtime.rs");
    let ui_v2_file_cache = read_runtime_src("ui/tests/v2_asset/file_cache.rs");
    let ui_shared_parent = read_runtime_src("ui/tests/shared_core.rs");
    let ui_shared_layout = read_runtime_src("ui/tests/shared_core/layout_surface.rs");
    let ui_shared_input = read_runtime_src("ui/tests/shared_core/input_visibility.rs");
    let ui_shared_scroll = read_runtime_src("ui/tests/shared_core/scroll_mutation.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
    );

    assert_contains_all(
        "test-file budget parent mounts historical oversized roots guard",
        &test_file_budget_parent,
        &[
            "mod historical_oversized_roots;",
            "fn runtime_15_core_framework_tests_are_folder_backed",
            "fn runtime_15_ui_v2_asset_tests_are_folder_backed",
            "fn runtime_15_ui_shared_core_tests_are_folder_backed",
        ],
    );
    assert_contains_all(
        "core framework historical root delegates test owners",
        &core_framework_parent,
        &[
            "mod framework_surfaces;",
            "mod phase_queue_summary;",
            "mod render_product_surface;",
        ],
    );
    assert_contains_all(
        "UI v2 historical root delegates test owners",
        &ui_v2_parent,
        &[
            "mod asset_loading;",
            "mod composite_components;",
            "mod default_controls;",
            "mod demo_and_builder;",
            "mod file_cache;",
            "mod range_controls;",
            "mod style_runtime;",
        ],
    );
    assert_contains_all(
        "UI shared core historical root delegates test owners",
        &ui_shared_parent,
        &[
            "mod box_flow;",
            "mod input_visibility;",
            "mod layout_surface;",
            "mod navigation;",
            "mod scroll_mutation;",
        ],
    );

    for moved_guard in [
        "time_framework_tracks_real_virtual_and_fixed_clocks",
        "render_product_post_process_graph_rejects_cycles",
        "ui_v2_style_specificity_and_pseudo_state_are_resolved",
        "ui_v2_file_cache_rebuilds_when_persistent_cache_dependency_changes",
        "hit_testing_respects_z_order_input_policy_and_clip_chain",
        "virtual_list_window_tracks_visible_range_with_overscan",
    ] {
        assert!(
            !core_framework_parent.contains(moved_guard)
                && !ui_v2_parent.contains(moved_guard)
                && !ui_shared_parent.contains(moved_guard),
            "historical oversized test root should delegate moved test {moved_guard}"
        );
    }
    assert_contains_all(
        "historical oversized child owners keep representative tests",
        &format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            core_framework_surface,
            core_render_product,
            core_phase_queue,
            ui_v2_style,
            ui_v2_file_cache,
            ui_shared_layout,
            ui_shared_scroll
        ),
        &[
            "time_framework_tracks_real_virtual_and_fixed_clocks",
            "render_product_post_process_graph_rejects_cycles",
            "render_phase_queue_summary_reports_phase_counts_and_ordering_bounds",
            "ui_v2_style_specificity_and_pseudo_state_are_resolved",
            "ui_v2_file_cache_rebuilds_when_persistent_cache_dependency_changes",
            "hit_testing_respects_z_order_input_policy_and_clip_chain",
            "virtual_list_window_tracks_visible_range_with_overscan",
        ],
    );

    for path in [
        "core/framework/tests.rs",
        "core/framework/tests/framework_surfaces.rs",
        "core/framework/tests/render_product_surface.rs",
        "core/framework/tests/phase_queue_summary.rs",
        "ui/tests/v2_asset.rs",
        "ui/tests/v2_asset/asset_loading.rs",
        "ui/tests/v2_asset/composite_components.rs",
        "ui/tests/v2_asset/default_controls.rs",
        "ui/tests/v2_asset/demo_and_builder.rs",
        "ui/tests/v2_asset/file_cache.rs",
        "ui/tests/v2_asset/range_controls.rs",
        "ui/tests/v2_asset/style_runtime.rs",
        "ui/tests/shared_core.rs",
        "ui/tests/shared_core/box_flow.rs",
        "ui/tests/shared_core/input_visibility.rs",
        "ui/tests/shared_core/layout_surface.rs",
        "ui/tests/shared_core/navigation.rs",
        "ui/tests/shared_core/scroll_mutation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/historical_oversized_roots.rs",
    ] {
        let source = read_runtime_src(path);
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the historical oversized test-root budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 historical oversized test roots closeout",
                "runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred",
                "core/framework/tests.rs",
                "ui/tests/v2_asset.rs",
                "ui/tests/shared_core.rs",
                "runtime_15_historical_oversized_test_roots_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 historical oversized test roots closeout",
            "runtime_15_historical_oversized_test_roots_closeout_static_passed_cargo_deferred",
            "core/framework/tests.rs",
            "ui/tests/v2_asset.rs",
            "runtime_15_historical_oversized_test_roots_are_folder_backed",
        ],
    );
}
