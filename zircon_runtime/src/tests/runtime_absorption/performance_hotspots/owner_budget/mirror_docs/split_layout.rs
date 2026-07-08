use super::sources::{assert_contains_all, MirrorDocsSources};

pub(super) fn assert_mirror_docs_split_layout(sources: &MirrorDocsSources) {
    assert_contains_all(
        "owner-budget mirror-docs route",
        sources.owner_budget_mirror_docs_guard,
        &[
            "#[path = \"mirror_docs/audit_wiring.rs\"]",
            "#[path = \"mirror_docs/doc_mirrors.rs\"]",
            "#[path = \"mirror_docs/performance_guard.rs\"]",
            "#[path = \"mirror_docs/source_inventory.rs\"]",
            "#[path = \"mirror_docs/sources.rs\"]",
            "#[path = \"mirror_docs/split_layout.rs\"]",
            "fn runtime_07_performance_hotpath_mirror_docs_match_structure_audit_counts()",
            "fn runtime_15_runtime_07_owner_budget_mirror_docs_sources_guard_folder_backed_split()",
            "performance_guard::assert_performance_guard_anchors(&sources);",
            "source_inventory::assert_source_inventory_anchors(&sources);",
            "audit_wiring::assert_audit_wiring_anchors(&sources);",
            "doc_mirrors::assert_runtime_07_mirror_docs(&sources);",
            "split_layout::assert_mirror_docs_split_layout(&sources);",
        ],
    );

    for moved_anchor in [
        "let runtime_07_plan = include_str!",
        "let performance_guard_sources = [",
        "for source_inventory_anchor in [",
        "for boundary_anchor in [",
        "for expected_anchor in [",
    ] {
        assert!(
            !sources.owner_budget_mirror_docs_guard.contains(moved_anchor),
            "owner_budget/mirror_docs.rs should route instead of owning assertion block `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "mirror-docs child sources",
        sources.owner_budget_mirror_docs_sources_guard,
        &[
            "#[path = \"sources/assertions.rs\"]",
            "#[path = \"sources/load.rs\"]",
            "#[path = \"sources/views.rs\"]",
            "pub(super) fn load() -> MirrorDocsSources",
            "pub(super) fn assert_contains_all",
            "pub(super) struct MirrorDocsSources",
            "owner_budget_mirror_docs_sources_guard",
            "owner_budget_mirror_docs_sources_assertions_guard",
            "owner_budget_mirror_docs_sources_load_guard",
            "owner_budget_mirror_docs_sources_views_guard",
            "owner_budget_sources_load_guard",
        ],
    );
    for moved_anchor in [
        "include_str!(",
        "pub(super) fn performance_guard_sources",
        "pub(super) fn mirror_docs",
    ] {
        assert!(
            !sources
                .owner_budget_mirror_docs_sources_guard
                .contains(moved_anchor),
            "owner_budget/mirror_docs/sources.rs should route instead of owning `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "mirror-docs source-loading children",
        &format!(
            "{}\n{}\n{}",
            sources.owner_budget_mirror_docs_sources_assertions_guard,
            sources.owner_budget_mirror_docs_sources_load_guard,
            sources.owner_budget_mirror_docs_sources_views_guard
        ),
        &[
            "pub(super) fn assert_contains_all",
            "pub(super) fn load() -> MirrorDocsSources",
            "performance_hotpath_source_inventory.py",
            "owner_budget_sources_load_guard",
            "../../sources/load.rs",
            "artifact_render_diagnostics_split_layout_route_guard",
            "artifact_render_diagnostics_split_layout_source_inventory_guard",
            "artifact_render_diagnostics_split_layout_sources_guard",
            "artifact_render_diagnostics_split_layout_status_docs_guard",
            "scene_project_splits_split_layout_route_guard",
            "scene_project_splits_split_layout_source_inventory_guard",
            "scene_project_splits_split_layout_sources_guard",
            "scene_project_splits_split_layout_status_docs_guard",
            "fn performance_guard_sources",
            "fn mirror_docs",
        ],
    );
    assert_contains_all(
        "mirror-docs performance guard child",
        sources.owner_budget_mirror_docs_performance_guard,
        &[
            "assert_performance_guard_anchors",
            "runtime_15_runtime_07_owner_budget_mirror_docs_guard_folder_backed_split",
            "runtime_15_runtime_07_owner_budget_sources_guard_folder_backed_split",
            "AnimationSceneFrameDiagnostics",
        ],
    );
    assert_contains_all(
        "mirror-docs source inventory child",
        sources.owner_budget_mirror_docs_source_inventory_guard,
        &[
            "assert_source_inventory_anchors",
            "EXPECTED_TEST_FILE_COUNT = 91",
            "owner_budget/child_routes/submit_context.rs",
            "owner_budget/line_budgets/owner_budget.rs",
            "owner_budget/line_budgets/root.rs",
            "owner_budget/split_layout/route/parent_route.rs",
            "owner_budget/split_layout/route/split_route.rs",
            "owner_budget/split_layout/route/support_routes.rs",
            "owner_budget/mirror_docs/sources/assertions.rs",
            "owner_budget/mirror_docs/sources/load.rs",
            "owner_budget/mirror_docs/sources/views.rs",
            "owner_budget/sources/load.rs",
            "hotspot_inventory/ecs_extract_counters/split_layout/status_docs.rs",
            "artifact_render_diagnostics_splits/split_layout/status_docs.rs",
            "scene_project_splits/split_layout/status_docs.rs",
            "owner_budget/mirror_docs/split_layout.rs",
        ],
    );
    assert_contains_all(
        "mirror-docs audit wiring child",
        sources.owner_budget_mirror_docs_audit_wiring_guard,
        &[
            "assert_audit_wiring_anchors",
            "MIRROR_DOCS_GUARD",
            "performance_hotpath_source_inventory",
        ],
    );
    assert_contains_all(
        "mirror-docs doc mirror child",
        sources.owner_budget_mirror_docs_doc_mirrors_guard,
        &[
            "assert_runtime_07_mirror_docs",
            "expected_test_file_count = 91",
            "risks = []",
        ],
    );

    for (path, source) in [
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs.rs",
            sources.owner_budget_mirror_docs_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/audit_wiring.rs",
            sources.owner_budget_mirror_docs_audit_wiring_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/doc_mirrors.rs",
            sources.owner_budget_mirror_docs_doc_mirrors_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/performance_guard.rs",
            sources.owner_budget_mirror_docs_performance_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/source_inventory.rs",
            sources.owner_budget_mirror_docs_source_inventory_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources.rs",
            sources.owner_budget_mirror_docs_sources_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/assertions.rs",
            sources.owner_budget_mirror_docs_sources_assertions_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/load.rs",
            sources.owner_budget_mirror_docs_sources_load_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/sources/views.rs",
            sources.owner_budget_mirror_docs_sources_views_guard,
        ),
        (
            "tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/split_layout.rs",
            sources.owner_budget_mirror_docs_split_layout_guard,
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 320,
            "{path} should stay below the focused mirror-docs split guard budget; got {line_count} lines"
        );
    }
}
