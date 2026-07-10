use super::*;

#[test]
fn runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/shared_core/scroll_mutation.rs");
    let pointer_routes = read_runtime_src("ui/tests/shared_core/scroll_mutation/pointer_routes.rs");
    let property_mutation =
        read_runtime_src("ui/tests/shared_core/scroll_mutation/property_mutation.rs");
    let virtual_scroll = read_runtime_src("ui/tests/shared_core/scroll_mutation/virtual_scroll.rs");

    assert_contains_all(
        "UI shared core scroll mutation parent mounts folder-backed children",
        &parent,
        &[
            "mod pointer_routes;",
            "mod property_mutation;",
            "mod virtual_scroll;",
            "use super::*;",
        ],
    );
    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count(),
        0,
        "ui/tests/shared_core/scroll_mutation.rs should only mount child test owners"
    );
    for moved_test in [
        "virtual_list_window_tracks_visible_range_with_overscan",
        "scroll_pointer_event_scrolls_the_nearest_scrollable_box_when_unhandled",
        "surface_property_mutation_updates_authored_metadata_and_reflector_snapshot",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI shared-core scroll mutation test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI shared core scroll mutation virtual-scroll child owns scroll tests",
        &virtual_scroll,
        &[
            "fn virtual_list_window_tracks_visible_range_with_overscan",
            "fn scrollable_box_tracks_content_metrics_virtual_window_and_local_scroll_invalidation",
        ],
    );
    assert_contains_all(
        "UI shared core scroll mutation pointer child owns pointer and scroll-route tests",
        &pointer_routes,
        &[
            "fn pointer_dispatcher_applies_block_passthrough_and_capture_semantics",
            "fn captured_pointer_dispatch_keeps_move_and_up_targeting_the_captured_node_outside_hit_bounds",
            "fn scroll_pointer_event_scrolls_the_nearest_scrollable_box_when_unhandled",
        ],
    );
    assert_contains_all(
        "UI shared core scroll mutation property child owns mutation/reflection tests",
        &property_mutation,
        &[
            "fn surface_property_mutation_marks_dirty_only_when_values_change",
            "fn surface_property_mutation_restores_collapsed_visibility_with_layout_dirty",
            "fn surface_property_mutation_keeps_template_visibility_metadata_in_sync",
            "fn surface_property_mutation_marks_material_layout_metadata_as_layout_dirty",
            "fn surface_property_mutation_updates_authored_metadata_and_reflector_snapshot",
        ],
    );

    let child_test_total = [
        pointer_routes.as_str(),
        property_mutation.as_str(),
        virtual_scroll.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 10,
        "UI shared-core scroll mutation children should preserve all 10 parent tests"
    );

    for (path, source) in [
        ("ui/tests/shared_core/scroll_mutation.rs", parent.as_str()),
        (
            "ui/tests/shared_core/scroll_mutation/pointer_routes.rs",
            pointer_routes.as_str(),
        ),
        (
            "ui/tests/shared_core/scroll_mutation/property_mutation.rs",
            property_mutation.as_str(),
        ),
        (
            "ui/tests/shared_core/scroll_mutation/virtual_scroll.rs",
            virtual_scroll.as_str(),
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
    let status_rows = ui_tests_first_status_row_source();
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("UI architecture doc", ui_doc.as_str()),
        ("status-output row data", status_rows.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Runtime 15 M3 UI shared core scroll mutation child folder split",
                "runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred",
                "ui/tests/shared_core/scroll_mutation.rs",
                "ui/tests/shared_core/scroll_mutation/property_mutation.rs",
                "ui/tests/shared_core/scroll_mutation/virtual_scroll.rs",
                "runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed",
            ],
        );
    }
}
