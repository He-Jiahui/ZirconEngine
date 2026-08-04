use super::*;

#[test]
fn runtime_15_ui_shared_core_input_visibility_children_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/shared_core/input_visibility.rs");
    let collapsed_layout =
        read_runtime_src("ui/tests/shared_core/input_visibility/collapsed_layout.rs");
    let focus_candidates =
        read_runtime_src("ui/tests/shared_core/input_visibility/focus_candidates.rs");
    let hit_visibility =
        read_runtime_src("ui/tests/shared_core/input_visibility/hit_visibility.rs");
    let pointer_routes =
        read_runtime_src("ui/tests/shared_core/input_visibility/pointer_routes.rs");

    assert_contains_all(
        "UI shared core input visibility parent mounts folder-backed children",
        &parent,
        &[
            "mod collapsed_layout;",
            "mod focus_candidates;",
            "mod hit_visibility;",
            "mod pointer_routes;",
            "use super::*;",
        ],
    );
    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count(),
        0,
        "ui/tests/shared_core/input_visibility.rs should only mount child test owners"
    );
    for moved_test in [
        "pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers",
        "hit_grid_respects_slate_visibility_and_clip_semantics",
        "taffy_vertical_layout_skips_collapsed_child_without_fallback",
        "focus_navigation_and_scroll_candidates_use_effective_visibility",
        "pointer_capture_routes_move_and_up_to_the_captured_node",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI shared-core input visibility test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI shared core pointer-route child owns pointer route and capture tests",
        &pointer_routes,
        &[
            "fn pointer_dispatcher_exposes_pointer_button_to_shared_route_handlers",
            "fn pointer_capture_routes_move_and_up_to_the_captured_node",
        ],
    );
    assert_contains_all(
        "UI shared core hit-visibility child owns hit/render visibility tests",
        &hit_visibility,
        &[
            "fn hit_testing_respects_z_order_input_policy_and_clip_chain",
            "fn surface_rebuild_derives_render_and_hit_from_same_arranged_geometry",
            "fn hit_grid_respects_slate_visibility_and_clip_semantics",
            "fn legacy_visible_false_is_normalized_into_hidden_visibility_for_surface_outputs",
        ],
    );
    assert_contains_all(
        "UI shared core collapsed-layout child owns collapse layout tests",
        &collapsed_layout,
        &[
            "fn explicit_collapsed_visibility_preserves_layout_collapse_with_legacy_visible_false",
            "fn taffy_vertical_layout_skips_collapsed_child_without_fallback",
        ],
    );
    assert_contains_all(
        "UI shared core focus-candidate child owns effective-visibility focus tests",
        &focus_candidates,
        &["fn focus_navigation_and_scroll_candidates_use_effective_visibility"],
    );

    let child_test_total = [
        collapsed_layout.as_str(),
        focus_candidates.as_str(),
        hit_visibility.as_str(),
        pointer_routes.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 9,
        "UI shared-core input visibility children should preserve all 9 parent tests"
    );

    for (path, source) in [
        ("ui/tests/shared_core/input_visibility.rs", parent.as_str()),
        (
            "ui/tests/shared_core/input_visibility/collapsed_layout.rs",
            collapsed_layout.as_str(),
        ),
        (
            "ui/tests/shared_core/input_visibility/focus_candidates.rs",
            focus_candidates.as_str(),
        ),
        (
            "ui/tests/shared_core/input_visibility/hit_visibility.rs",
            hit_visibility.as_str(),
        ),
        (
            "ui/tests/shared_core/input_visibility/pointer_routes.rs",
            pointer_routes.as_str(),
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
}
