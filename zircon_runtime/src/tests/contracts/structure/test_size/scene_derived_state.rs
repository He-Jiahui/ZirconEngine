use super::*;

#[test]
fn runtime_15_scene_derived_state_tests_are_folder_backed() {
    let parent = read_runtime_src("scene/tests/derived_state.rs");
    let hierarchy_behavior = read_runtime_src("scene/tests/derived_state/hierarchy_behavior.rs");
    let hierarchy_rebuild = read_runtime_src("scene/tests/derived_state/hierarchy_rebuild.rs");
    let projected_reads = read_runtime_src("scene/tests/derived_state/projected_reads.rs");
    let runtime_freshness = read_runtime_src("scene/tests/derived_state/runtime_freshness.rs");
    let spawn_paths = read_runtime_src("scene/tests/derived_state/spawn_paths.rs");
    let checked_parent_chain =
        read_runtime_src("scene/tests/derived_state/checked_parent_chain.rs");
    let work_counters = read_runtime_src("scene/tests/derived_state/work_counters.rs");
    let subtree_cycle_walk = read_runtime_src("scene/tests/derived_state/subtree_cycle_walk.rs");
    let subtree_cycle_profile =
        read_runtime_src("scene/tests/derived_state/subtree_cycle_profile.rs");
    let derived_state_owner = read_runtime_src("scene/world/derived_state.rs");
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let ecs_doc = read_repo("docs/zircon_runtime/scene/ecs.md");

    assert_contains_all(
        "scene derived-state parent test module mounts",
        &parent,
        &[
            "mod checked_parent_chain;",
            "mod hierarchy_behavior;",
            "mod hierarchy_rebuild;",
            "mod projected_reads;",
            "mod runtime_freshness;",
            "mod spawn_paths;",
            "mod subtree_cycle_walk;",
            "mod work_counters;",
            "fn detached_node_record",
            "fn pending_reparented_world",
            "fn read_source",
        ],
    );
    assert_eq!(
        parent.matches("#[test]").count(),
        0,
        "scene/tests/derived_state.rs should mount child owners instead of keeping executable tests"
    );

    for moved_test in [
        "fn spawn_node_kind_ordinals_compare_kinds_without_candidate_clones",
        "fn spawn_node_kind_ordinals_use_cached_kind_counts",
        "fn derived_state_rebuilds_use_single_hierarchy_traversal_index",
        "fn derived_state_rebuilds_reuse_the_mutation_hierarchy_index",
        "fn derived_state_projected_reads_use_direct_parent_branches",
        "fn projected_reads_stay_fresh_until_post_update_refreshes_retained_cache",
        "fn imported_records_validate_missing_parents_and_preserve_out_of_order_links",
        "fn checked_parent_chain_rejects_pending_two_node_cycle_without_mutation",
        "fn direct_child_hierarchy_world",
        "fn derived_state_full_rebuild_baseline_is_deterministic_for_one_and_one_thousand_nodes",
        "fn derived_state_full_rebuild_baseline_is_deterministic_for_one_hundred_thousand_nodes",
        "fn derived_state_leaf_transform_change_rebuilds_only_affected_rows",
        "fn derived_state_parent_transform_change_rebuilds_the_affected_subtree",
        "fn derived_state_leaf_active_change_rebuilds_only_the_affected_row",
        "fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_thousand_nodes",
        "fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_hundred_thousand_nodes",
    ] {
        assert!(
            !parent.contains(moved_test),
            "scene/tests/derived_state.rs should mount child test owners instead of defining {moved_test}"
        );
    }

    assert_contains_all(
        "spawn-paths child owns spawn-node hot-path guards",
        &spawn_paths,
        &[
            "use super::*;",
            "fn spawn_node_kind_ordinals_use_cached_kind_counts",
            "fn spawn_node_reuses_copy_node_kind_without_spawn_path_clones",
        ],
    );
    assert_contains_all(
        "hierarchy rebuild child owns traversal and schedule source guards",
        &hierarchy_rebuild,
        &[
            "use super::*;",
            "fn derived_state_rebuilds_reuse_the_mutation_hierarchy_index",
            "fn hierarchy_validity_rebuild_uses_pre_sized_parent_snapshot",
            "fn subtree_record_collection_reuses_hierarchy_traversal_index",
            "fn mobility_static_parent_preflight_uses_direct_parent_branch",
            "fn internal_scene_system_flushes_reuse_schedule_stage_plan",
        ],
    );
    assert_contains_all(
        "projected reads child owns direct lookup and retained-cache source guards",
        &projected_reads,
        &[
            "use super::*;",
            "fn derived_state_projected_reads_use_direct_parent_branches",
            "fn derived_state_projected_value_reads_use_direct_branches",
            "fn derived_state_default_component_reads_use_direct_branches",
            "fn node_records_projection_uses_pre_sized_push_snapshot",
            "fn world_query_scalar_accessors_use_direct_lookup_branches",
            "fn retained_node_cache_refresh_reuses_pre_sized_storage",
        ],
    );
    assert_contains_all(
        "runtime freshness child owns pending-system and render-extract freshness behavior",
        &runtime_freshness,
        &[
            "use super::*;",
            "fn projected_reads_stay_fresh_until_post_update_refreshes_retained_cache",
            "fn no_op_mutators_do_not_mark_derived_state_dirty",
            "fn render_extract_prepare_flushes_direct_frame_and_legacy_viewport_paths",
            "fn property_path_node_cache_changes_mark_dirty_and_zero_morph_extension_is_not_noop",
            "fn active_camera_selection_marks_render_extract_freshness_without_rebuilding_scheduler",
        ],
    );
    assert_contains_all(
        "hierarchy behavior child owns import, cycle, active-state, and mobility behavior",
        &hierarchy_behavior,
        &[
            "use super::*;",
            "fn imported_records_validate_missing_parents_and_preserve_out_of_order_links",
            "fn hierarchy_cycle_rejection_preserves_existing_parent_state",
            "fn active_hierarchy_propagates_inactive_and_reactivated_ancestors",
            "fn post_update_propagates_large_hierarchy_transform_and_active_state",
            "fn mobility_changes_refresh_visibility_buckets_without_transform_rebuild",
        ],
    );

    assert_contains_all(
        "checked parent-chain child owns rejection, pending-state, and error-priority regressions",
        &checked_parent_chain,
        &[
            "fn checked_parent_chain_rejects_pending_two_node_cycle_without_mutation",
            "fn checked_parent_chain_rejects_pending_three_node_cycle_without_mutation",
            "fn checked_parent_chain_rejects_pending_self_cycle_without_mutation",
            "fn checked_parent_chain_rejects_tail_entering_a_cycle_without_mutation",
            "fn checked_parent_chain_rejects_direct_serde_cycle_before_first_flush",
            "fn checked_parent_chain_keeps_ancestor_hit_a_prospective_cycle_error",
            "fn checked_parent_chain_preserves_valid_reparent_noop_and_detach",
            "fn checked_parent_chain_preserves_existing_error_priority",
        ],
    );
    assert_contains_all(
        "work-counter child owns existing derived-state baseline and incremental-work assertions",
        &work_counters,
        &[
            "use super::*;",
            "fn direct_child_hierarchy_world",
            "fn assert_full_derived_state_baseline",
            "fn assert_structured_reparent_avoids_global_hierarchy_work",
            "fn derived_state_full_rebuild_baseline_is_deterministic_for_one_and_one_thousand_nodes",
            "fn derived_state_full_rebuild_baseline_is_deterministic_for_one_hundred_thousand_nodes",
            "fn derived_state_leaf_transform_change_rebuilds_only_affected_rows",
            "fn derived_state_parent_transform_change_rebuilds_the_affected_subtree",
            "fn derived_state_leaf_active_change_rebuilds_only_the_affected_row",
            "fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_thousand_nodes",
            "fn derived_state_structured_reparent_avoids_global_hierarchy_work_at_one_hundred_thousand_nodes",
        ],
    );
    assert_contains_all(
        "subtree-cycle child owns the actual public traversal and deferred-flush regressions",
        &subtree_cycle_walk,
        &[
            "fn subtree_walk_dirty_index_terminates_self_two_and_three_node_cycles_with_branches",
            "fn subtree_walk_current_index_terminates_self_two_and_three_node_cycles_with_branches",
            "fn subtree_walk_tail_start_does_not_enter_an_ancestor_cycle",
            "fn subtree_walk_pending_active_query_flush_reaches_typed_parent_guard",
            "fn subtree_walk_valid_forest_preserves_preorder_counts_and_prepare_coverage",
        ],
    );
    assert_contains_all(
        "subtree-cycle Release profile keeps complete Q public API bodies and paired samples",
        &subtree_cycle_profile,
        &[
            "fn frozen_subtree_records",
            "fn frozen_collect_subtree_records_with_traversal",
            "fn frozen_subtree_component_count",
            "fn runtime62_subtree_cycle_guard_public_api_release_profile",
            "baseline_raw_ns={baseline:?} current_raw_ns={current:?}",
        ],
    );
    assert!(derived_state_owner.contains("mod subtree_cycle_profile;"));
    assert!(derived_state_owner.contains("../tests/derived_state/subtree_cycle_profile.rs"));
    assert_eq!(subtree_cycle_walk.matches("#[test]").count(), 5);
    assert_eq!(subtree_cycle_profile.matches("#[test]").count(), 1);
    assert_eq!(subtree_cycle_profile.matches("#[ignore").count(), 1);

    let enumerated_owner_test_count = [
        spawn_paths.as_str(),
        hierarchy_rebuild.as_str(),
        projected_reads.as_str(),
        runtime_freshness.as_str(),
        hierarchy_behavior.as_str(),
        checked_parent_chain.as_str(),
        work_counters.as_str(),
    ]
    .iter()
    .map(|source| source.matches("#[test]").count())
    .sum::<usize>();
    assert_eq!(
        enumerated_owner_test_count, 48,
        "the seven enumerated scene derived-state child-owner files should contain 48 test declarations"
    );

    for (path, source) in [
        ("scene/tests/derived_state.rs", parent.as_str()),
        (
            "scene/tests/derived_state/checked_parent_chain.rs",
            checked_parent_chain.as_str(),
        ),
        (
            "scene/tests/derived_state/work_counters.rs",
            work_counters.as_str(),
        ),
        (
            "scene/tests/derived_state/hierarchy_behavior.rs",
            hierarchy_behavior.as_str(),
        ),
        (
            "scene/tests/derived_state/hierarchy_rebuild.rs",
            hierarchy_rebuild.as_str(),
        ),
        (
            "scene/tests/derived_state/projected_reads.rs",
            projected_reads.as_str(),
        ),
        (
            "scene/tests/derived_state/runtime_freshness.rs",
            runtime_freshness.as_str(),
        ),
        (
            "scene/tests/derived_state/spawn_paths.rs",
            spawn_paths.as_str(),
        ),
        (
            "scene/tests/derived_state/subtree_cycle_walk.rs",
            subtree_cycle_walk.as_str(),
        ),
        (
            "scene/tests/derived_state/subtree_cycle_profile.rs",
            subtree_cycle_profile.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
