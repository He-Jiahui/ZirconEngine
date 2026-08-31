import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
INCREMENTAL_LAYOUT = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/incremental.rs"
)
LAYOUT_PASS_MOD = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/mod.rs"
LAYOUT_CONSTRAINTS = REPO_ROOT / "zircon_runtime/src/ui/layout/constraints.rs"
LAYOUT_AXIS = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/axis.rs"
LAYOUT_MEASURE = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/measure.rs"
LAYOUT_MEASURE_TRAVERSAL = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/measure/traversal.rs"
)
LAYOUT_MEASURE_TESTS = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/measure/tests.rs"
)
LAYOUT_ARRANGE = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/arrange.rs"
LAYOUT_ENGINE = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/engine.rs"
TREE_NODES = REPO_ROOT / "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs"
TREE_LAYOUT = REPO_ROOT / "zircon_runtime/src/ui/tree/node/layout.rs"
LAYOUT_CACHE = REPO_ROOT / "zircon_runtime_interface/src/ui/tree/node/layout_cache.rs"
LAYOUT_ARRANGE_TESTS = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/arrange/tests.rs"
)
LAYOUT_TAFFY_ARRANGE = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/taffy_arrange.rs"
)
LAYOUT_TAFFY_BRIDGE_COMPUTE = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/taffy_bridge/compute.rs"
)
LAYOUT_GRID_MASONRY_ARRANGE = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/arrange/grid_masonry.rs"
)
LAYOUT_RESPONSIVE_MUI = (
    REPO_ROOT / "zircon_runtime/src/ui/layout/pass/responsive_mui.rs"
)
LAYOUT_SLOT = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/slot.rs"
LAYOUT_WORKSPACE = REPO_ROOT / "zircon_runtime/src/ui/layout/pass/workspace.rs"
RUNTIME_REGRESSION = (
    REPO_ROOT
    / "zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs"
)
REBUILD_REPORT = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/report.rs"
)
SURFACE_REBUILD = REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
SURFACE_REBUILD_INCREMENTAL = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
)
INTERFACE_DIAGNOSTICS = (
    REPO_ROOT / "zircon_runtime_interface/src/ui/surface/diagnostics.rs"
)
EDITOR_PROJECTION = (
    REPO_ROOT / "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs"
)
PROFILE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"


def surface_rebuild_source() -> str:
    return "\n".join(
        path.read_text(encoding="utf-8")
        for path in (SURFACE_REBUILD, SURFACE_REBUILD_INCREMENTAL)
    )


class RuntimeIncrementalLayoutSnapshotPerformanceContract(unittest.TestCase):
    def test_measurement_workspace_is_owned_by_the_retained_layout_index(self) -> None:
        module = LAYOUT_PASS_MOD.read_text(encoding="utf-8")
        slot = LAYOUT_SLOT.read_text(encoding="utf-8")
        workspace = LAYOUT_WORKSPACE.read_text(encoding="utf-8")

        self.assertIn("mod workspace;", module)
        self.assertRegex(
            slot,
            re.compile(
                r"workspace:\s*RefCell<UiLayoutPassWorkspace>", re.DOTALL
            ),
        )
        self.assertIn("with_measure_workspace", slot)
        self.assertIn("impl Clone for UiLayoutSlotIndex", slot)
        self.assertIn("workspace: RefCell::default()", slot)
        self.assertIn("pub(crate) struct UiLayoutPassWorkspace", workspace)
        self.assertIn("clear_transient_lengths", workspace)

    def test_measurement_uses_one_post_order_plan_without_per_node_child_clones(self) -> None:
        source = LAYOUT_MEASURE_TRAVERSAL.read_text(encoding="utf-8")

        self.assertIn("plan_measurement_post_order", source)
        self.assertIn("workspace.post_order.clear()", source)
        self.assertIn("workspace.child_desired.clear()", source)
        self.assertIn("workspace.clear_transient_lengths()", source)
        self.assertIn("ordered_children_for_container", source)
        self.assertNotIn("node.children.clone()", source)
        self.assertNotIn("Vec::with_capacity(children.len())", source)
        self.assertNotIn("fn collapse_node_measure", source)
        self.assertEqual(source.count("fn measure_node_with_profile("), 1)

    def test_container_measurement_consumes_cached_order_and_reuses_column_buffers(self) -> None:
        source = LAYOUT_MEASURE.read_text(encoding="utf-8")

        self.assertIn("container_scratch.primary_extents.resize(", source)
        self.assertIn("container_scratch.secondary_extents.resize(", source)
        self.assertIn("container_scratch.column_counts.resize(", source)
        self.assertNotIn("prepare_ordered_child_desired", source)
        self.assertNotIn("ordered_child_desired", source)
        self.assertNotIn(".collect::<Vec<_>>()", source)
        self.assertNotIn("vec![0.0_f32;", source)
        self.assertNotIn("vec![0usize;", source)

    def test_measurement_workspace_has_collapsed_and_ten_thousand_node_regressions(self) -> None:
        source = LAYOUT_MEASURE_TESTS.read_text(encoding="utf-8")

        self.assertIn("post_order_measurement_collapses_the_entire_hidden_subtree", source)
        self.assertIn(
            "ten_thousand_sibling_measurement_reuses_the_retained_workspace_capacity",
            source,
        )
        self.assertIn(
            "grid_after_masonry_reuses_extents_and_preserves_hidden_vs_collapsed_measurement",
            source,
        )
        self.assertIn(
            "equal_slot_order_preserves_original_child_order_in_cached_generation",
            source,
        )
        self.assertIn(
            "post_order_measurement_calls_the_text_cache_once_per_leaf_and_reuses_it",
            source,
        )
        self.assertIn("const CHILD_COUNT: u64 = 10_000", source)
        self.assertIn("assert_eq!(workspace_capacity(&slot_index), first_capacity)", source)

    def test_arrangement_reuses_depth_scoped_scratch_and_generation_owned_order(self) -> None:
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        slot = LAYOUT_SLOT.read_text(encoding="utf-8")
        workspace = LAYOUT_WORKSPACE.read_text(encoding="utf-8")
        tests = LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct UiArrangeChildScratch", workspace)
        self.assertIn("arrange_child_pool: Vec<UiArrangeChildScratch>", workspace)
        self.assertIn("hidden_subtree_stack: Vec<UiNodeId>", workspace)
        self.assertIn("take_arrange_child_scratch", slot)
        self.assertIn("recycle_arrange_child_scratch", slot)
        self.assertIn("ordered_children_by_parent", slot)
        self.assertIn("ordered_children_for_container", slot)
        self.assertIn("sort_unstable_by_key", slot)
        self.assertIn("let mut child_scratch = slot_index.take_arrange_child_scratch()", arrange)
        self.assertIn("slot_index.recycle_arrange_child_scratch(child_scratch)", arrange)
        self.assertNotIn("node.children.clone()", arrange)
        self.assertIn("ordered_children_for_container", arrange)
        self.assertNotIn("order_children_for_container", arrange)
        self.assertIn(
            "ten_thousand_sibling_arrangement_reuses_depth_scoped_child_scratch",
            tests,
        )
        self.assertIn(
            "equal_slot_order_reuses_the_cached_generation",
            tests,
        )
        self.assertIn(
            "free_arrangement_preserves_slot_order",
            tests,
        )
        self.assertIn(
            "scrollable_arrangement_preserves_tree_order_despite_slot_order_metadata",
            tests,
        )
        self.assertIn(
            "wrap_content_size_uses_the_same_cached_order_as_arrangement",
            tests,
        )
        self.assertIn("hidden_subtree_arrangement_reuses_one_iterative_stack", tests)
        self.assertIn("failed_arrangement_recycles_child_scratch", tests)

    def test_child_order_is_not_rebuilt_in_nested_arrange_helpers(self) -> None:
        taffy = LAYOUT_TAFFY_ARRANGE.read_text(encoding="utf-8")
        grid_masonry = LAYOUT_GRID_MASONRY_ARRANGE.read_text(encoding="utf-8")

        self.assertNotIn("ordered_children_for_container", taffy)
        self.assertNotIn("ordered_children_for_container", grid_masonry)

    def test_taffy_bridge_reuses_backend_tree_and_depth_scoped_buffers(self) -> None:
        compute = LAYOUT_TAFFY_BRIDGE_COMPUTE.read_text(encoding="utf-8")
        taffy_arrange = LAYOUT_TAFFY_ARRANGE.read_text(encoding="utf-8")
        workspace = LAYOUT_WORKSPACE.read_text(encoding="utf-8")
        slot = LAYOUT_SLOT.read_text(encoding="utf-8")
        arrange_tests = LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct TaffyLayoutBridgeScratch", compute)
        self.assertIn("taffy: TaffyTree<()>", compute)
        self.assertIn("child_node_ids: Vec<UiNodeId>", compute)
        self.assertIn("taffy_children: Vec<NodeId>", compute)
        self.assertIn("child_frames: Vec<TaffyLayoutChildFrame>", compute)
        self.assertNotIn("Vec<TaffyPreparedChild>", compute)
        self.assertIn("begin_children", compute)
        self.assertIn("push_child", compute)
        self.assertIn("self.taffy.clear()", compute)
        self.assertNotIn(
            "let mut taffy: TaffyTree<()> = TaffyTree::new()", compute
        )
        self.assertIn("pub(crate) struct UiTaffyArrangeScratch", workspace)
        self.assertIn(
            "static TAFFY_ARRANGE_SCRATCH_POOL: RefCell<Vec<UiTaffyArrangeScratch>>",
            workspace,
        )
        self.assertIn("take_taffy_arrange_scratch", workspace)
        self.assertIn("recycle_taffy_arrange_scratch", workspace)
        self.assertNotIn("let mut child_inputs = Vec::with_capacity", taffy_arrange)
        self.assertNotIn("let mut layout_children = Vec::with_capacity", taffy_arrange)
        self.assertNotIn("let mut hidden_children = Vec::new()", taffy_arrange)
        self.assertIn(
            "taffy_bridge_reuses_tree_and_buffers_across_repeated_arrangement",
            arrange_tests,
        )

    def test_linear_axis_solver_reuses_all_transient_buffers(self) -> None:
        constraints = LAYOUT_CONSTRAINTS.read_text(encoding="utf-8")
        axis = LAYOUT_AXIS.read_text(encoding="utf-8")
        workspace = LAYOUT_WORKSPACE.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct UiLinearArrangeScratch", workspace)
        for field in (
            "constraints: Vec<AxisConstraint>",
            "resolved: Vec<ResolvedAxisConstraint>",
            "priorities: Vec<i32>",
            "active_indices: Vec<usize>",
        ):
            self.assertIn(field, workspace)
        self.assertIn("pub(crate) fn solve_axis_constraints_into", constraints)
        self.assertIn("priorities.clear()", constraints)
        self.assertIn("active_indices.clear()", constraints)
        self.assertNotIn("let indices: Vec<_>", constraints)
        self.assertIn("solve_axis_constraints_into(", axis)
        self.assertIn("scratch.constraints.clear()", axis)
        self.assertNotIn("let mut constraints = Vec::with_capacity", axis)
        self.assertNotIn(".collect()", axis)
        self.assertIn(
            "reusable_solver_workspace_matches_owned_results_and_preserves_capacity",
            constraints,
        )
        for oracle in (
            "reusable_solver_growth_respects_priority_and_max_saturation",
            "reusable_solver_growth_with_zero_weights_shares_evenly",
            "reusable_solver_shrink_respects_ascending_priority",
            "reusable_solver_exact_fit_and_minimum_floor_are_explicit",
        ):
            self.assertIn(oracle, constraints)
        arrange_tests = LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "linear_arrangement_solver_reuses_constraints_and_active_indices",
            arrange_tests,
        )

    def test_scroll_wrap_and_masonry_avoid_per_pass_vec_builds(self) -> None:
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        grid_masonry = LAYOUT_GRID_MASONRY_ARRANGE.read_text(encoding="utf-8")
        workspace = LAYOUT_WORKSPACE.read_text(encoding="utf-8")

        for field in (
            "wrap_row_items: Vec<(UiNodeId, f32)>",
            "wrap_content_desired: Vec<(UiNodeId, DesiredSize)>",
            "masonry: UiMasonryArrangeScratch",
            "column_heights: Vec<f32>",
            "column_counts: Vec<usize>",
        ):
            self.assertIn(field, workspace)
        self.assertNotIn("fn child_positions(", arrange)
        self.assertNotIn("let positions = child_positions", arrange)
        self.assertIn("validate_scrollable_children(tree, children)?", arrange)
        self.assertNotIn("let mut row_items: Vec", arrange)
        self.assertNotIn("collect::<Option<Vec<_>>>()", arrange)
        self.assertIn("wrap_row_items.clear()", arrange)
        self.assertIn("wrap_content_desired.clear()", arrange)
        self.assertIn("column_heights.resize(columns, 0.0)", grid_masonry)
        self.assertIn("column_counts.resize(columns, 0)", grid_masonry)
        self.assertNotIn("vec![0.0_f32;", grid_masonry)
        self.assertNotIn("vec![0usize;", grid_masonry)
        tests = LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8")
        for regression in (
            "scrollable_arrangement_streams_positions_across_collapsed_children",
            "scrollable_arrangement_validates_all_direct_children_before_mutating_subtrees",
            "masonry_arrangement_reuses_column_buffers",
            "container_arrange_capacities",
        ):
            self.assertIn(regression, tests)

    def test_geometry_delta_is_recorded_during_arrangement_without_a_prefetch_scan(self) -> None:
        incremental = INCREMENTAL_LAYOUT.read_text(encoding="utf-8")
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")

        self.assertNotIn("BTreeMap", incremental)
        self.assertNotIn("snapshot_geometry", incremental)
        self.assertNotIn("collect_subtree_nodes", incremental)
        self.assertIn("record_geometry", arrange)

    def test_independent_layout_parents_use_provenance_aware_sparse_arrangement(self) -> None:
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        engine = LAYOUT_ENGINE.read_text(encoding="utf-8")
        tree_nodes = TREE_NODES.read_text(encoding="utf-8")
        tree_layout = TREE_LAYOUT.read_text(encoding="utf-8")

        self.assertIn("layout_source_node_ids", tree_nodes)
        self.assertIn("pending_layout_source_node_ids", tree_nodes)
        self.assertIn("mark_layout_dirty_source", tree_nodes)
        self.assertIn("mark_layout_dirty_source", tree_layout)
        self.assertIn("copy_required_children", engine)
        self.assertIn("can_sparse_arrange_independent_children", engine)
        self.assertIn("layout_source_node_ids", engine)
        self.assertIn("copy_required_children", arrange)
        self.assertIn("can_sparse_arrange_independent_children", arrange)
        self.assertIn("UiContainerKind::Free", arrange)
        self.assertIn("UiContainerKind::SizeBox", arrange)
        sparse_branch = arrange.split("if sparse_children {", 1)[1].split("} else {", 1)[0]
        self.assertNotIn("original_children.extend_from_slice", sparse_branch)
        arrange_tests = LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8")
        self.assertIn(
            "incremental_free_parent_arranges_only_required_direct_children",
            arrange_tests,
        )
        self.assertIn(
            "incremental_free_parent_source_keeps_full_child_arrangement",
            arrange_tests,
        )

    def test_incremental_measurement_uses_explicit_validity_for_zero_geometry(self) -> None:
        cache = LAYOUT_CACHE.read_text(encoding="utf-8")
        traversal = LAYOUT_MEASURE_TRAVERSAL.read_text(encoding="utf-8")
        measure_tests = LAYOUT_MEASURE_TESTS.read_text(encoding="utf-8")

        self.assertIn("pub measure_valid: bool", cache)
        self.assertIn("invalidate_measure", cache)
        self.assertIn("complete_measure", cache)
        self.assertIn("#[serde(default)]\n    pub measure_valid", cache)
        self.assertIn("&& node.layout_cache.measure_valid", traversal)
        self.assertIn("let force_children = force_subtree || collapsed;", traversal)
        self.assertNotIn("node.dirty.input || !node.layout_cache.measure_valid", traversal)
        self.assertNotIn("node.layout_cache.frame == UiFrame::default()", traversal)
        tree_layout = TREE_LAYOUT.read_text(encoding="utf-8")
        rebuild = surface_rebuild_source()
        node_pool = (REPO_ROOT / "zircon_runtime/src/ui/surface/node_pool.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("node.layout_cache.invalidate_measure()", tree_layout)
        self.assertIn("node.layout_cache.invalidate_measure()", rebuild)
        self.assertIn("pooled.layout_cache.invalidate_measure()", node_pool)
        self.assertIn("reset_reinserted_node", node_pool)
        tree_nodes_tests = TREE_NODES.read_text(encoding="utf-8")
        self.assertIn(
            "child_structure_changes_invalidate_the_parent_measurement_cache",
            tree_nodes_tests,
        )
        self.assertIn(
            "incremental_measurement_reuses_valid_zero_frame_parent_without_forcing_clean_descendants",
            measure_tests,
        )
        self.assertIn(
            "invalid_parent_measurement_does_not_force_valid_sibling_subtrees",
            measure_tests,
        )

    def test_incremental_layout_uses_exact_dependency_and_geometry_reuse_authority(self) -> None:
        incremental = INCREMENTAL_LAYOUT.read_text(encoding="utf-8")
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        runtime = RUNTIME_REGRESSION.read_text(encoding="utf-8")
        surface_rebuild = surface_rebuild_source()
        responsive = LAYOUT_RESPONSIVE_MUI.read_text(encoding="utf-8")
        slot = LAYOUT_SLOT.read_text(encoding="utf-8")

        self.assertIn("layout_dependency_paths", incremental)
        self.assertIn("measure_node_incremental", incremental)
        self.assertIn("UiLayoutPassEngineContext::incremental", incremental)
        self.assertIn("layout_engine_route_node_ids", incremental)
        self.assertIn("removed_node_ids", incremental)
        self.assertNotIn("collect_subtree_nodes", incremental)
        self.assertNotIn("snapshot_geometry", incremental)
        self.assertIn("can_reuse_geometry", arrange)
        self.assertIn("record_geometry", arrange)
        hide_body = arrange.split("pub(super) fn hide_subtree_layout", 1)[1]
        self.assertIn("can_reuse_geometry", hide_body)
        self.assertIn("apply_mui_responsive_layout_indexed", incremental)
        self.assertIn("synchronize_responsive_candidates", incremental)
        self.assertIn("with_responsive_candidates", responsive)
        self.assertIn("responsive_candidates: MuiResponsiveCandidates", slot)
        self.assertIn("patch_nodes(tree, node_ids)", slot)
        self.assertIn("&layout_stats.removed_node_ids", surface_rebuild)
        self.assertGreaterEqual(
            surface_rebuild.count("&layout_stats.layout_engine_route_node_ids"),
            2,
        )
        self.assertRegex(
            surface_rebuild,
            re.compile(
                r"removed_node_ids\s*\.iter\(\)\s*"
                r"\.any\(\|node_id\| previous_indices\.contains_key\(node_id\)\)",
                re.DOTALL,
            ),
        )
        self.assertIn(
            "surface_dirty_layout_drops_removed_layout_engine_routes", runtime
        )
        self.assertIn(
            "measured_but_geometry_reused_container_preserves_layout_engine_route",
            runtime,
        )
        self.assertIn(
            "incremental_measurement_rebuilds_a_subtree_restored_from_collapsed",
            LAYOUT_MEASURE_TESTS.read_text(encoding="utf-8"),
        )
        self.assertIn(
            "candidate_patch_tracks_same_cardinality_metadata_changes", responsive
        )
        self.assertIn(
            "unchanged_responsive_values_do_not_create_mutation_candidates",
            responsive,
        )
        grid_slots = responsive.split("fn apply_responsive_grid_slots", 1)[1]
        self.assertLess(
            grid_slots.index("implicit_grid_parent_ids.is_empty()"),
            grid_slots.index("for index in 0..tree.layout_slots().len()"),
        )
        self.assertIn(
            "root_resize_reuses_clean_descendant_measurement_and_arrangement",
            runtime,
        )
        self.assertIn(
            "incremental_hidden_subtree_rejects_redundant_zero_geometry_walks",
            LAYOUT_ARRANGE_TESTS.read_text(encoding="utf-8"),
        )

    def test_runtime_regression_guards_arrange_time_geometry_deltas(self) -> None:
        source = RUNTIME_REGRESSION.read_text(encoding="utf-8")

        self.assertIn(
            "incremental_layout_records_geometry_deltas_during_arrangement", source
        )
        self.assertIn('assert!(!source.contains("BTreeMap"));', source)
        self.assertIn('assert!(!source.contains("snapshot_geometry"));', source)
        self.assertIn('assert!(!source.contains("collect_subtree_nodes"));', source)

    def test_runtime_regression_guards_current_pass_taffy_work(self) -> None:
        source = RUNTIME_REGRESSION.read_text(encoding="utf-8")

        self.assertIn(
            "incremental_layout_reports_only_current_taffy_tree_build_work", source
        )
        self.assertIn("report.layout_taffy_tree_build_count, 1", source)
        self.assertIn("report.layout_taffy_tree_node_build_count, 3", source)
        self.assertIn("stable.layout_taffy_tree_build_count, 0", source)
        self.assertIn("stable.layout_taffy_tree_node_build_count, 0", source)

    def test_rebuild_report_carries_current_pass_taffy_work(self) -> None:
        report = REBUILD_REPORT.read_text(encoding="utf-8")
        rebuild = surface_rebuild_source()
        diagnostics = INTERFACE_DIAGNOSTICS.read_text(encoding="utf-8")

        for field, report_field in (
            ("layout_taffy_tree_build_count", "taffy_tree_build_count"),
            ("layout_taffy_tree_node_build_count", "taffy_tree_node_count"),
        ):
            self.assertIn(f"pub {field}: u64", report)
            self.assertIn(f"pub {field}: u64", diagnostics)
            self.assertRegex(
                rebuild,
                re.compile(
                    rf"{field}:\s*layout_stats\s*\.layout_engine_report\s*"
                    rf"\.{report_field}",
                    re.DOTALL,
                ),
            )
            self.assertRegex(
                rebuild,
                re.compile(
                    rf"{field}:\s*self\s*\.layout_engine_report\s*"
                    rf"\.{report_field}",
                    re.DOTALL,
                ),
            )

    def test_incremental_layout_reports_probe_work_before_early_out(self) -> None:
        incremental = INCREMENTAL_LAYOUT.read_text(encoding="utf-8")
        measure = LAYOUT_MEASURE_TRAVERSAL.read_text(encoding="utf-8")
        arrange = LAYOUT_ARRANGE.read_text(encoding="utf-8")
        report = REBUILD_REPORT.read_text(encoding="utf-8")
        diagnostics = INTERFACE_DIAGNOSTICS.read_text(encoding="utf-8")
        runtime = RUNTIME_REGRESSION.read_text(encoding="utf-8")

        for field in (
            "layout_measure_probe_node_count",
            "layout_arrange_probe_node_count",
        ):
            self.assertIn(f"pub {field}: usize", report)
            self.assertIn(f"pub {field}: usize", diagnostics)
            self.assertIn(field, incremental)

        self.assertIn("measurement_probe_node_count", measure)
        self.assertIn("record_arrangement_probe", arrange)
        self.assertIn("parent_size_dependent_children", LAYOUT_SLOT.read_text(encoding="utf-8"))
        self.assertIn("arrange_resized_root", arrange)
        self.assertNotIn(
            "self.invalidate_node(root_id, UiInvalidationReason::Layout)",
            surface_rebuild_source(),
        )
        self.assertIn(
            "root_resize_reports_early_out_probe_work",
            runtime,
        )
        self.assertIn(
            "root_resize_dependency_index_tracks_a_child_that_becomes_stretched",
            runtime,
        )
        self.assertIn(
            "clipped_root_resize_uses_the_conservative_clip_propagation_path",
            runtime,
        )

    def test_editor_capture_publishes_current_pass_taffy_work(self) -> None:
        projection = EDITOR_PROJECTION.read_text(encoding="utf-8")
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8").replace("\\", "/")

        self.assertIn("ui.template_projection.taffy_tree_build_count", projection)
        self.assertIn("ui.template_projection.taffy_tree_node_build_count", projection)
        for path in (
            "zircon_runtime/src/ui/surface/surface/rebuild/report.rs",
            "zircon_runtime_interface/src/ui/surface/diagnostics.rs",
            "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs",
        ):
            self.assertIn(path, manifest)

    def test_editor_capture_and_resize_gate_publish_layout_probe_work(self) -> None:
        projection = EDITOR_PROJECTION.read_text(encoding="utf-8")
        catalog = (
            REPO_ROOT
            / "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs"
        ).read_text(encoding="utf-8")
        ui_perf = (
            REPO_ROOT / "zircon_editor/src/ui/retained_host/ui_perf.rs"
        ).read_text(encoding="utf-8")
        gate = (
            REPO_ROOT / "tools/ui-profile-counter-evidence.ps1"
        ).read_text(encoding="utf-8")

        for field, variant in (
            (
                "layout_measure_probe_node_count",
                "TemplateProjectionLayoutMeasureProbeNodeCount",
            ),
            (
                "layout_arrange_probe_node_count",
                "TemplateProjectionLayoutArrangeProbeNodeCount",
            ),
        ):
            self.assertIn(field, projection)
            self.assertIn(variant, catalog)
            self.assertIn(f"UiPerfCounter::{variant}", ui_perf)
        self.assertIn(
            "ui.window_resize.template_projection_layout_measure_probe_node_count",
            gate,
        )
        self.assertIn(
            "ui.window_resize.template_projection_layout_arrange_probe_node_count",
            gate,
        )
        self.assertIn("$layoutMeasureProbeCount -eq 0", gate)
        self.assertIn("$layoutArrangeProbeCount -le ($completedSteps * 64)", gate)


if __name__ == "__main__":
    unittest.main()
