use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_visibility_virtual_geometry_tests_are_child_owners() {
    let parent = read_runtime_src("graphics/tests/visibility.rs");
    let page_plan = read_runtime_src("graphics/tests/visibility/virtual_geometry_page_plan.rs");
    let frontier = read_runtime_src("graphics/tests/visibility/virtual_geometry_frontier.rs");
    let priority = read_runtime_src("graphics/tests/visibility/virtual_geometry_priority.rs");

    let plan_04 = read_repo("docs/plans/zircon_runtime/render/04/2026-07-09-visibility-culling-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let visibility_doc = read_repo("docs/zircon_runtime/graphics/visibility.md");
    let render_submit_doc = read_repo("docs/zircon_runtime/graphics/render-product-submit.md");

    assert_contains_all(
        "visibility parent keeps base visibility tests, shared fixtures, and virtual-geometry child mounts",
        &parent,
        &[
            "mod virtual_geometry_frontier;",
            "mod virtual_geometry_page_plan;",
            "mod virtual_geometry_priority;",
            "fn visibility_context_partitions_static_and_dynamic_meshes(",
            "fn visibility_context_builds_deterministic_batches_and_instancing_candidates(",
            "fn visibility_context_filters_visible_batches_through_camera_frustum(",
            "fn visibility_context_without_history_marks_bvh_full_rebuild(",
            "fn visibility_context_with_history_tracks_bvh_dirty_entities(",
            "fn visibility_context_without_history_marks_particle_emitters_dirty(",
            "fn visibility_context_with_history_tracks_particle_upload_changes(",
            "fn crate_batch(",
            "fn draw_commands_for_batches(",
            "fn remove_default_meshes(",
            "fn virtual_cluster(",
            "fn virtual_page(",
        ],
    );

    for (moved_anchor, owner_name, owner_source) in [
        (
            "fn visibility_context_builds_virtual_geometry_visibility_feedback_and_page_plan(",
            "virtual_geometry_page_plan.rs",
            page_plan.as_str(),
        ),
        (
            "fn visibility_context_with_history_tracks_virtual_geometry_requested_pages(",
            "virtual_geometry_page_plan.rs",
            page_plan.as_str(),
        ),
        (
            "fn visibility_context_refines_virtual_geometry_parent_cluster_into_visible_children_when_budget_allows(",
            "virtual_geometry_page_plan.rs",
            page_plan.as_str(),
        ),
        (
            "fn visibility_context_holds_resident_child_page_one_frame_when_frontier_merges_back_to_parent(",
            "virtual_geometry_frontier.rs",
            frontier.as_str(),
        ),
        (
            "fn visibility_context_keeps_resident_child_frontier_hot_across_repeated_budget_collapse_without_pending_requests(",
            "virtual_geometry_frontier.rs",
            frontier.as_str(),
        ),
        (
            "fn visibility_context_keeps_intermediate_virtual_geometry_lineage_pages_hot_while_ancestor_request_remains_pending(",
            "virtual_geometry_frontier.rs",
            frontier.as_str(),
        ),
        (
            "fn visibility_context_only_holds_requested_virtual_geometry_lineage_when_frontier_budget_collapses(",
            "virtual_geometry_priority.rs",
            priority.as_str(),
        ),
        (
            "fn visibility_context_splits_virtual_geometry_draw_segments_across_parent_lineages_even_when_page_matches(",
            "virtual_geometry_priority.rs",
            priority.as_str(),
        ),
        (
            "fn visibility_context_uses_aggregate_screen_space_error_to_break_virtual_geometry_page_priority_ties(",
            "virtual_geometry_priority.rs",
            priority.as_str(),
        ),
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "visibility.rs should delegate `{moved_anchor}` to {owner_name}"
        );
        assert!(
            owner_source.contains(moved_anchor),
            "{owner_name} should contain `{moved_anchor}`"
        );
    }

    assert_contains_all(
        "virtual-geometry page-plan child keeps page request and refinement coverage",
        &page_plan,
        &[
            "use super::{",
            "RenderVirtualGeometryExtract",
            "VisibilityVirtualGeometryCluster",
            "VisibilityVirtualGeometryFeedback",
            "VisibilityVirtualGeometryPageUploadPlan",
            "virtual_cluster",
            "virtual_page",
        ],
    );
    assert_contains_all(
        "virtual-geometry frontier child keeps split-merge hysteresis coverage",
        &frontier,
        &[
            "use super::{",
            "visibility_context_requests_nonresident_ancestor_page_and_holds_descendants_when_frontier_collapses_multiple_levels",
            "VisibilityVirtualGeometryFeedback",
            "hot_resident_pages",
            "virtual_cluster",
            "virtual_page",
        ],
    );
    assert_contains_all(
        "virtual-geometry priority child keeps draw-segment and page-priority coverage",
        &priority,
        &[
            "use super::{",
            "VisibilityVirtualGeometryDrawSegment",
            "visibility_context_prioritizes_virtual_geometry_pages_backing_more_visible_clusters_when_page_budget_is_tight",
            "VisibilityVirtualGeometryPageUploadPlan",
            "virtual_cluster",
            "virtual_page",
        ],
    );

    for (path, source) in [
        ("graphics/tests/visibility.rs", parent.as_str()),
        (
            "graphics/tests/visibility/virtual_geometry_page_plan.rs",
            page_plan.as_str(),
        ),
        (
            "graphics/tests/visibility/virtual_geometry_frontier.rs",
            frontier.as_str(),
        ),
        (
            "graphics/tests/visibility/virtual_geometry_priority.rs",
            priority.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the R4.3 visibility test budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 04", plan_04.as_str()),
        ("render index", render_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("visibility docs", visibility_doc.as_str()),
        ("render submit docs", render_submit_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Visibility virtual-geometry tests owner split",
                "render_plan04_visibility_virtual_geometry_tests_owner_split_static_passed_cargo_deferred_implementation_cadence",
                "graphics/tests/visibility.rs",
                "graphics/tests/visibility/virtual_geometry_page_plan.rs",
                "graphics/tests/visibility/virtual_geometry_frontier.rs",
                "graphics/tests/visibility/virtual_geometry_priority.rs",
                "runtime_15_visibility_virtual_geometry_tests_are_child_owners",
            ],
        );
    }
}
