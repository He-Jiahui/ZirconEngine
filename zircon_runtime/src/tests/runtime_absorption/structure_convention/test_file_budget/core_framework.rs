use super::*;

#[test]
fn runtime_15_core_framework_tests_are_folder_backed() {
    let parent = read_runtime_src("core/framework/tests.rs");
    let framework_surfaces = read_runtime_src("core/framework/tests/framework_surfaces.rs");
    let render_product_surface = read_runtime_src("core/framework/tests/render_product_surface.rs");
    let phase_queue_summary = read_runtime_src("core/framework/tests/phase_queue_summary.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/foundation_guards.rs",
    );

    assert_contains_all(
        "core framework parent test module mounts",
        &parent,
        &[
            "mod phase_queue_summary;",
            "mod framework_surfaces;",
            "mod render_product_surface;",
        ],
    );

    for moved_guard in [
        "fn time_framework_tracks_real_virtual_and_fixed_clocks",
        "fn task_framework_contracts_describe_pools_status_and_poll_budget",
        "fn render_profile_validation_rejects_unsatisfied_advanced_capabilities",
        "fn render_product_post_process_graph_elides_disabled_effects",
        "fn render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "core/framework/tests.rs should mount child test owners instead of defining {moved_guard}"
        );
    }
    assert_contains_all(
        "framework surfaces child owns moved framework contracts",
        &framework_surfaces,
        &[
            "fn time_framework_tracks_real_virtual_and_fixed_clocks",
            "fn task_framework_contracts_describe_pools_status_and_poll_budget",
            "fn render_profile_validation_rejects_unsatisfied_advanced_capabilities",
            "include_str!(\"../tasks/mod.rs\")",
            "include_str!(\"../time/mod.rs\")",
        ],
    );
    assert_contains_all(
        "render product child owns moved render contracts",
        &render_product_surface,
        &[
            "fn render_product_post_process_graph_elides_disabled_effects",
            "fn render_camera_ordering_reports_ambiguities_and_skips_inactive_cameras",
            "super::super::render::sort_render_cameras",
        ],
    );
    assert_contains_all(
        "phase queue summary child remains folder backed",
        &phase_queue_summary,
        &["render_phase_queue_summary_reports_phase_counts_and_ordering_bounds"],
    );

    for (path, source) in [
        ("core/framework/tests.rs", parent.as_str()),
        (
            "core/framework/tests/framework_surfaces.rs",
            framework_surfaces.as_str(),
        ),
        (
            "core/framework/tests/render_product_surface.rs",
            render_product_surface.as_str(),
        ),
        (
            "core/framework/tests/phase_queue_summary.rs",
            phase_queue_summary.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
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
                "Runtime 15 M3 core framework test folder split",
                "runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked",
                "core/framework/tests/framework_surfaces.rs",
                "core/framework/tests/render_product_surface.rs",
                "runtime_15_core_framework_tests_are_folder_backed",
            ],
        );
    }

    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 core framework test folder split",
            "runtime_15_core_framework_tests_folder_split_static_passed_cargo_lock_blocked",
            "core/framework/tests/framework_surfaces.rs",
            "core/framework/tests/render_product_surface.rs",
            "runtime_15_core_framework_tests_are_folder_backed",
        ],
    );
}
