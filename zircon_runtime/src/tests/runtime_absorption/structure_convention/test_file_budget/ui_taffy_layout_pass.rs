use super::*;

#[test]
fn runtime_15_ui_taffy_layout_pass_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/taffy_layout_pass.rs");
    let arrangement = read_runtime_src("ui/tests/taffy_layout_pass/arrangement.rs");
    let fallback_policy = read_runtime_src("ui/tests/taffy_layout_pass/fallback_policy.rs");
    let grid_slots = read_runtime_src("ui/tests/taffy_layout_pass/grid_slots.rs");
    let linear_slots = read_runtime_src("ui/tests/taffy_layout_pass/linear_slots.rs");
    let routing_diagnostics = read_runtime_src("ui/tests/taffy_layout_pass/routing_diagnostics.rs");

    assert_contains_all(
        "UI taffy layout pass parent mounts folder-backed children",
        &parent,
        &[
            "mod arrangement;",
            "mod fallback_policy;",
            "mod grid_slots;",
            "mod linear_slots;",
            "mod routing_diagnostics;",
            "fn tree_with_root(",
            "fn assert_taffy_native_family(",
            "fn assert_fallback_route_reason(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/taffy_layout_pass.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "layout_pass_reports_taffy_native_and_zircon_fallback_routes",
        "taffy_layout_pass_arranges_linear_wrap_and_grid_containers",
        "taffy_layout_pass_maps_linear_slot_padding_without_fallback",
        "taffy_layout_pass_reports_non_finite_container_config_fallback",
        "taffy_layout_pass_maps_grid_slot_placement_without_fallback",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI taffy layout pass test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI taffy routing child owns route diagnostics",
        &routing_diagnostics,
        &[
            "fn layout_pass_routes_supported_containers_through_taffy_arrange",
            "fn layout_pass_reports_taffy_native_and_zircon_fallback_routes",
            "fn taffy_layout_pass_aggregates_distinct_fallback_reason_counts",
        ],
    );
    assert_contains_all(
        "UI taffy arrangement child owns native layout outputs",
        &arrangement,
        &[
            "fn taffy_layout_pass_arranges_linear_wrap_and_grid_containers",
            "fn taffy_layout_pass_preserves_fractional_fixed_extents",
            "fn taffy_layout_pass_uses_measured_text_and_image_desired_sizes",
        ],
    );
    assert_contains_all(
        "UI taffy linear-slot child owns linear and wrap slot mapping",
        &linear_slots,
        &[
            "fn taffy_layout_pass_maps_linear_slot_padding_without_fallback",
            "fn taffy_layout_pass_maps_wrap_slot_padding_and_cross_axis_alignment_without_fallback",
            "fn taffy_layout_pass_maps_linear_stretch_content_slot_sizing_without_fallback",
            "fn taffy_layout_pass_maps_vertical_linear_slot_sizing_bounds_without_fallback",
        ],
    );
    assert_contains_all(
        "UI taffy fallback-policy child owns unsupported route policies",
        &fallback_policy,
        &[
            "fn taffy_layout_pass_rejects_unsupported_slot_padding_values",
            "fn taffy_layout_pass_reports_non_finite_axis_constraint_fallback",
            "fn taffy_layout_pass_reports_child_placement_policy_fallback",
            "fn size_box_contain_aspect_ratio_stays_zircon_owned",
        ],
    );
    assert_contains_all(
        "UI taffy grid-slot child owns grid placement and span mapping",
        &grid_slots,
        &[
            "fn taffy_layout_pass_maps_grid_slot_placement_without_fallback",
            "fn taffy_layout_pass_expands_grid_tracks_for_out_of_bounds_slot_span_without_fallback",
            "fn taffy_layout_pass_reports_grid_slot_alignment_without_fixed_extent_fallback",
        ],
    );

    let child_test_total = [
        arrangement.as_str(),
        fallback_policy.as_str(),
        grid_slots.as_str(),
        linear_slots.as_str(),
        routing_diagnostics.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 35,
        "UI taffy layout pass children should preserve all 35 parent tests"
    );

    for (path, source) in [
        ("ui/tests/taffy_layout_pass.rs", parent.as_str()),
        (
            "ui/tests/taffy_layout_pass/arrangement.rs",
            arrangement.as_str(),
        ),
        (
            "ui/tests/taffy_layout_pass/fallback_policy.rs",
            fallback_policy.as_str(),
        ),
        (
            "ui/tests/taffy_layout_pass/grid_slots.rs",
            grid_slots.as_str(),
        ),
        (
            "ui/tests/taffy_layout_pass/linear_slots.rs",
            linear_slots.as_str(),
        ),
        (
            "ui/tests/taffy_layout_pass/routing_diagnostics.rs",
            routing_diagnostics.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ui_doc = read_repo("docs/zircon_runtime/ui/architecture.md");
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_second.rs",
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI taffy layout pass test folder split",
                "runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/taffy_layout_pass.rs",
                "ui/tests/taffy_layout_pass/routing_diagnostics.rs",
                "ui/tests/taffy_layout_pass/linear_slots.rs",
                "runtime_15_ui_taffy_layout_pass_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI taffy layout pass test folder split",
            "runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/taffy_layout_pass.rs",
            "ui/tests/taffy_layout_pass/routing_diagnostics.rs",
            "runtime_15_ui_taffy_layout_pass_tests_are_folder_backed",
        ],
    );
}
