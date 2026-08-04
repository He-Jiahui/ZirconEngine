use super::*;

#[test]
fn runtime_15_ui_shared_core_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/shared_core.rs");
    let layout_surface = read_runtime_src("ui/tests/shared_core/layout_surface.rs");
    let layout_surface_layout =
        read_runtime_src("ui/tests/shared_core/layout_surface/layout_measurement.rs");
    let layout_surface_render =
        read_runtime_src("ui/tests/shared_core/layout_surface/render_extract.rs");
    let box_flow = read_runtime_src("ui/tests/shared_core/box_flow.rs");
    let input_visibility = read_runtime_src("ui/tests/shared_core/input_visibility.rs");
    let input_visibility_pointer =
        read_runtime_src("ui/tests/shared_core/input_visibility/pointer_routes.rs");
    let navigation = read_runtime_src("ui/tests/shared_core/navigation.rs");
    let scroll_mutation = read_runtime_src("ui/tests/shared_core/scroll_mutation.rs");
    let scroll_mutation_property =
        read_runtime_src("ui/tests/shared_core/scroll_mutation/property_mutation.rs");
    let scroll_mutation_virtual =
        read_runtime_src("ui/tests/shared_core/scroll_mutation/virtual_scroll.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "UI shared core parent test module mounts",
        &parent,
        &[
            "mod box_flow;",
            "mod input_visibility;",
            "mod layout_surface;",
            "mod navigation;",
            "mod scroll_mutation;",
            "fn stretch_constraint",
            "fn fixed_constraint",
        ],
    );

    for moved_guard in [
        "fn shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes",
        "fn horizontal_box_deserializes_and_arranges_children_with_gap_and_cross_axis_stretch",
        "fn pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers",
        "fn navigation_routes_from_focus_and_falls_back_to_roots",
        "fn virtual_list_window_tracks_visible_range_with_overscan",
    ] {
        assert!(
            !parent.contains(moved_guard),
            "ui/tests/shared_core.rs should mount child test owners instead of defining {moved_guard}"
        );
    }

    assert_contains_all(
        "UI shared core layout surface child mounts layout/render contracts",
        &layout_surface,
        &[
            "mod container_overlays;",
            "mod layout_measurement;",
            "mod render_extract;",
        ],
    );
    assert_contains_all(
        "UI shared core layout surface layout child owns layout contracts",
        &layout_surface_layout,
        &["fn shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes"],
    );
    assert_contains_all(
        "UI shared core layout surface render child owns render contracts",
        &layout_surface_render,
        &["fn render_extract_uses_label_when_schema_text_default_is_placeholder"],
    );
    for moved_test in [
        "fn shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes",
        "fn render_extract_uses_label_when_schema_text_default_is_placeholder",
    ] {
        assert!(
            !layout_surface.contains(moved_test),
            "ui/tests/shared_core/layout_surface.rs should mount child test owners instead of defining {moved_test}"
        );
    }
    assert_contains_all(
        "UI shared core box flow child owns box contracts",
        &box_flow,
        &[
            "fn horizontal_box_deserializes_and_arranges_children_with_gap_and_cross_axis_stretch",
            "fn wrap_box_measurement_uses_width_bounds_before_root_arrange",
        ],
    );
    assert_contains_all(
        "UI shared core input visibility child owns pointer/visibility contracts",
        &input_visibility_pointer,
        &[
            "fn pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers",
            "fn pointer_capture_routes_move_and_up_to_the_captured_node",
        ],
    );
    assert_contains_all(
        "UI shared core navigation child owns navigation contracts",
        &navigation,
        &[
            "fn navigation_routes_from_focus_and_falls_back_to_roots",
            "fn navigation_dispatcher_keeps_focus_when_activate_or_cancel_is_unhandled",
        ],
    );
    assert_contains_all(
        "UI shared core scroll mutation child mounts scroll/mutation contracts",
        &scroll_mutation,
        &[
            "mod pointer_routes;",
            "mod property_mutation;",
            "mod virtual_scroll;",
        ],
    );
    assert_contains_all(
        "UI shared core scroll mutation virtual child owns scroll contracts",
        &scroll_mutation_virtual,
        &["fn virtual_list_window_tracks_visible_range_with_overscan"],
    );
    assert_contains_all(
        "UI shared core scroll mutation property child owns mutation contracts",
        &scroll_mutation_property,
        &["fn surface_property_mutation_updates_authored_metadata_and_reflector_snapshot"],
    );
    for moved_test in [
        "fn virtual_list_window_tracks_visible_range_with_overscan",
        "fn surface_property_mutation_updates_authored_metadata_and_reflector_snapshot",
    ] {
        assert!(
            !scroll_mutation.contains(moved_test),
            "ui/tests/shared_core/scroll_mutation.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    for (path, source) in [
        (
            "ui/tests/shared_core/scroll_mutation/property_mutation.rs",
            scroll_mutation_property.as_str(),
        ),
        (
            "ui/tests/shared_core/scroll_mutation/virtual_scroll.rs",
            scroll_mutation_virtual.as_str(),
        ),
        (
            "ui/tests/shared_core/layout_surface/layout_measurement.rs",
            layout_surface_layout.as_str(),
        ),
        (
            "ui/tests/shared_core/layout_surface/render_extract.rs",
            layout_surface_render.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }

    for (path, source) in [
        ("ui/tests/shared_core.rs", parent.as_str()),
        (
            "ui/tests/shared_core/layout_surface.rs",
            layout_surface.as_str(),
        ),
        ("ui/tests/shared_core/box_flow.rs", box_flow.as_str()),
        (
            "ui/tests/shared_core/input_visibility.rs",
            input_visibility.as_str(),
        ),
        (
            "ui/tests/shared_core/input_visibility/pointer_routes.rs",
            input_visibility_pointer.as_str(),
        ),
        ("ui/tests/shared_core/navigation.rs", navigation.as_str()),
        (
            "ui/tests/shared_core/scroll_mutation.rs",
            scroll_mutation.as_str(),
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
                "Runtime 15 M3 UI shared core test folder split",
                "runtime_15_ui_shared_core_tests_folder_split_static_passed_cargo_lock_blocked",
                "ui/tests/shared_core/layout_surface.rs",
                "ui/tests/shared_core/scroll_mutation.rs",
                "runtime_15_ui_shared_core_tests_are_folder_backed",
            ],
        );
    }
}
