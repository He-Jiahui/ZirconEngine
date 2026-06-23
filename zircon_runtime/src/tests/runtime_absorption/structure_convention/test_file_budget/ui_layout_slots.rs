use super::*;

#[test]
fn runtime_15_ui_layout_slots_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/layout_slots.rs");
    let linear_free = read_runtime_src("ui/tests/layout_slots/linear_free.rs");
    let overlay_scroll = read_runtime_src("ui/tests/layout_slots/overlay_scroll.rs");
    let flow_grid_masonry = read_runtime_src("ui/tests/layout_slots/flow_grid_masonry.rs");

    assert_contains_all(
        "UI layout slots parent mounts folder-backed children",
        &parent,
        &[
            "mod flow_grid_masonry;",
            "mod linear_free;",
            "mod overlay_scroll;",
            "fn fixed_constraint(",
            "fn pointer_node(",
            "fn render_frame_for(",
            "fn hit_frame_for(",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/layout_slots.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "linear_layout_consumes_slot_padding_order_and_alignment",
        "free_layout_consumes_canvas_slot_placement_before_child_default_anchor",
        "overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame",
        "scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries",
        "grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame",
        "masonry_shortest_column_layout_feeds_arranged_render_hit_from_one_surface_frame",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI layout slot test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI layout slots linear/free child owns linear and free layout slot tests",
        &linear_free,
        &[
            "fn linear_layout_consumes_slot_padding_order_and_alignment",
            "fn free_layout_consumes_explicit_slot_padding_alignment_and_preserves_default_anchor_fallback",
            "fn free_layout_consumes_canvas_slot_placement_before_child_default_anchor",
        ],
    );
    assert_contains_all(
        "UI layout slots overlay/scroll child owns overlay and scroll frame tests",
        &overlay_scroll,
        &[
            "fn overlay_layout_consumes_slot_padding_alignment",
            "fn overlay_slot_geometry_feeds_arranged_render_hit_and_z_order_from_one_surface_frame",
            "fn scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries",
        ],
    );
    assert_contains_all(
        "UI layout slots flow/grid/masonry child owns flow, grid, and masonry tests",
        &flow_grid_masonry,
        &[
            "fn wrap_flow_slot_padding_alignment_feeds_shared_surface_frame",
            "fn grid_slot_cell_placement_feeds_arranged_render_hit_from_one_surface_frame",
            "fn masonry_shortest_column_layout_feeds_arranged_render_hit_from_one_surface_frame",
            "fn masonry_sequential_layout_preserves_ordered_column_assignment",
        ],
    );

    let child_test_total = [
        linear_free.as_str(),
        overlay_scroll.as_str(),
        flow_grid_masonry.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 10,
        "UI layout slots children should preserve all 10 parent tests"
    );

    for (path, source) in [
        ("ui/tests/layout_slots.rs", parent.as_str()),
        ("ui/tests/layout_slots/linear_free.rs", linear_free.as_str()),
        (
            "ui/tests/layout_slots/overlay_scroll.rs",
            overlay_scroll.as_str(),
        ),
        (
            "ui/tests/layout_slots/flow_grid_masonry.rs",
            flow_grid_masonry.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
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
                "Runtime 15 M3 UI layout slots test folder split",
                "runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/layout_slots.rs",
                "ui/tests/layout_slots/linear_free.rs",
                "ui/tests/layout_slots/flow_grid_masonry.rs",
                "runtime_15_ui_layout_slots_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI layout slots test folder split",
            "runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/layout_slots.rs",
            "ui/tests/layout_slots/linear_free.rs",
            "runtime_15_ui_layout_slots_tests_are_folder_backed",
        ],
    );
}
