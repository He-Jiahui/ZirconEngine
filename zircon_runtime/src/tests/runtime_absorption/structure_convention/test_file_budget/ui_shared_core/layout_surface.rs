use super::*;

#[test]
fn runtime_15_ui_shared_core_layout_surface_children_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/shared_core/layout_surface.rs");
    let container_overlays =
        read_runtime_src("ui/tests/shared_core/layout_surface/container_overlays.rs");
    let layout_measurement =
        read_runtime_src("ui/tests/shared_core/layout_surface/layout_measurement.rs");
    let render_extract = read_runtime_src("ui/tests/shared_core/layout_surface/render_extract.rs");

    assert_contains_all(
        "UI shared core layout surface parent mounts folder-backed children",
        &parent,
        &[
            "mod container_overlays;",
            "mod layout_measurement;",
            "mod render_extract;",
            "use super::*;",
        ],
    );
    assert_eq!(
        parent.matches(TEST_ATTRIBUTE).count(),
        0,
        "ui/tests/shared_core/layout_surface.rs should only mount child test owners"
    );
    for moved_test in [
        "shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes",
        "layout_pass_measures_content_driven_roots_and_arranges_anchored_children",
        "render_extract_uses_label_when_schema_text_default_is_placeholder",
        "overlay_deserializes_and_measures_to_the_largest_child_extent",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI shared-core layout surface test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI shared core layout measurement child owns layout and container tests",
        &layout_measurement,
        &[
            "fn shared_axis_solver_grows_high_priority_axes_before_lower_priority_axes",
            "fn layout_invalidation_bubbles_until_parent_directed_boundary",
            "fn layout_pass_measures_content_driven_roots_and_arranges_anchored_children",
            "fn layout_pass_measures_label_leaf_from_text_intrinsic_size",
            "fn layout_pass_measures_button_leaf_as_text_plus_padding",
            "fn container_deserializes_and_arranges_anchored_children_like_shared_free_layout",
        ],
    );
    assert_contains_all(
        "UI shared core render-extract child owns visual contract tests",
        &render_extract,
        &[
            "fn render_extract_carries_visual_contract_fields_for_visible_nodes",
            "fn render_extract_accepts_flat_style_color_aliases",
            "fn render_extract_uses_label_when_schema_text_default_is_placeholder",
        ],
    );
    assert_contains_all(
        "UI shared core container-overlay child owns overlay and spacer tests",
        &container_overlays,
        &[
            "fn overlay_deserializes_and_measures_to_the_largest_child_extent",
            "fn space_ignores_child_content_and_behaves_as_layout_spacer",
        ],
    );

    let child_test_total = [
        container_overlays.as_str(),
        layout_measurement.as_str(),
        render_extract.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches(TEST_ATTRIBUTE).count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 11,
        "UI shared-core layout surface children should preserve all 11 parent tests"
    );

    for (path, source) in [
        ("ui/tests/shared_core/layout_surface.rs", parent.as_str()),
        (
            "ui/tests/shared_core/layout_surface/container_overlays.rs",
            container_overlays.as_str(),
        ),
        (
            "ui/tests/shared_core/layout_surface/layout_measurement.rs",
            layout_measurement.as_str(),
        ),
        (
            "ui/tests/shared_core/layout_surface/render_extract.rs",
            render_extract.as_str(),
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
                "Runtime 15 M3 UI shared core layout surface child folder split",
                "runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred",
                "ui/tests/shared_core/layout_surface.rs",
                "ui/tests/shared_core/layout_surface/layout_measurement.rs",
                "ui/tests/shared_core/layout_surface/render_extract.rs",
                "runtime_15_ui_shared_core_layout_surface_children_are_folder_backed",
            ],
        );
    }
}
