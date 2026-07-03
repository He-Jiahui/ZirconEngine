use super::*;

#[test]
fn runtime_15_ui_surface_dirty_domains_tests_are_folder_backed() {
    let parent = read_runtime_src("ui/tests/surface_dirty_domains.rs");
    let rebuild_domains = read_runtime_src("ui/tests/surface_dirty_domains/rebuild_domains.rs");
    let incremental_layout =
        read_runtime_src("ui/tests/surface_dirty_domains/incremental_layout.rs");
    let render_domains = read_runtime_src("ui/tests/surface_dirty_domains/render_domains.rs");
    let mutation_state = read_runtime_src("ui/tests/surface_dirty_domains/mutation_state.rs");

    assert_contains_all(
        "UI surface dirty-domain parent mounts folder-backed children",
        &parent,
        &[
            "mod incremental_layout;",
            "mod mutation_state;",
            "mod rebuild_domains;",
            "mod render_domains;",
            "fn test_surface()",
            "fn mark_structured_dirty(",
            "fn assert_report_phases(",
            "fn sibling_surface(",
            "fn layout_route_merge_surface()",
            "fn keyboard_event()",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "ui/tests/surface_dirty_domains.rs should only mount child test owners and shared helpers"
    );
    for moved_test in [
        "surface_dirty_rebuild_separates_hit_input_render_and_legacy_state_flags",
        "surface_dirty_layout_skips_siblings_under_non_auto_parent",
        "surface_dirty_render_only_metadata_does_not_trigger_hit_or_input_rebuild",
        "surface_dirty_route_state_mutations_keep_legacy_dirty_for_bridges",
    ] {
        assert!(
            !parent.contains(moved_test),
            "moved UI surface dirty-domain test `{moved_test}` should not return to the parent"
        );
    }

    assert_contains_all(
        "UI surface dirty-domain rebuild child owns rebuild phase contracts",
        &rebuild_domains,
        &[
            "fn surface_dirty_rebuild_separates_hit_input_render_and_legacy_state_flags",
            "fn surface_dirty_rebuild_recomputes_layout_for_structural_domains",
        ],
    );
    assert_contains_all(
        "UI surface dirty-domain incremental layout child owns incremental layout contracts",
        &incremental_layout,
        &[
            "fn surface_dirty_layout_skips_siblings_under_non_auto_parent",
            "fn surface_dirty_layout_preserves_unvisited_layout_engine_routes",
            "fn surface_dirty_layout_replaces_visited_layout_engine_routes",
            "fn surface_dirty_layout_drops_removed_layout_engine_routes",
            "fn surface_dirty_layout_revisits_auto_parent_when_child_size_changes",
        ],
    );
    assert_contains_all(
        "UI surface dirty-domain render child owns render-only contracts",
        &render_domains,
        &[
            "fn surface_dirty_render_reuses_unchanged_commands_without_damage",
            "fn surface_dirty_render_only_metadata_does_not_trigger_hit_or_input_rebuild",
            "fn surface_dirty_text_edit_visual_metadata_stays_render_only",
            "fn surface_dirty_render_only_dispatch_effect_does_not_trigger_hit_or_input_rebuild",
        ],
    );
    assert_contains_all(
        "UI surface dirty-domain mutation child owns state mutation contracts",
        &mutation_state,
        &[
            "fn surface_dirty_route_state_mutations_keep_legacy_dirty_for_bridges",
            "fn surface_dirty_layout_marking_keeps_structured_domains_precise",
        ],
    );

    let child_test_total = [
        rebuild_domains.as_str(),
        incremental_layout.as_str(),
        render_domains.as_str(),
        mutation_state.as_str(),
    ]
    .into_iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        child_test_total, 13,
        "UI surface dirty-domain children should preserve all 13 parent tests"
    );

    for (path, source) in [
        ("ui/tests/surface_dirty_domains.rs", parent.as_str()),
        (
            "ui/tests/surface_dirty_domains/rebuild_domains.rs",
            rebuild_domains.as_str(),
        ),
        (
            "ui/tests/surface_dirty_domains/incremental_layout.rs",
            incremental_layout.as_str(),
        ),
        (
            "ui/tests/surface_dirty_domains/render_domains.rs",
            render_domains.as_str(),
        ),
        (
            "ui/tests/surface_dirty_domains/mutation_state.rs",
            mutation_state.as_str(),
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
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/ui_tests_first.rs",
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
                "Runtime 15 M3 UI surface dirty domains test folder split",
                "runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred",
                "ui/tests/surface_dirty_domains.rs",
                "ui/tests/surface_dirty_domains/incremental_layout.rs",
                "ui/tests/surface_dirty_domains/render_domains.rs",
                "runtime_15_ui_surface_dirty_domains_tests_are_folder_backed",
            ],
        );
    }
    assert_contains_all(
        "status-output row data",
        &status_rows,
        &[
            "Runtime 15 M3 UI surface dirty domains test folder split",
            "runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred",
            "ui/tests/surface_dirty_domains.rs",
            "ui/tests/surface_dirty_domains/rebuild_domains.rs",
            "runtime_15_ui_surface_dirty_domains_tests_are_folder_backed",
        ],
    );
}
