from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"
HIT_GEOMETRY_PATCH = ROOT / "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs"
ROUTE_INDEX = ROOT / "zircon_runtime/src/ui/tree/hit_test/route_index.rs"
FRAME_HIT_TEST = ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
HIT_CONTRACT = ROOT / "zircon_runtime_interface/src/ui/surface/hit.rs"
REBUILD = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"


class RuntimeUiHitRouteIndexPerformanceContract(unittest.TestCase):
    def test_grid_owns_one_shared_route_table_and_entries_store_only_an_index(self) -> None:
        source = HIT_CONTRACT.read_text(encoding="utf-8")
        entry = source.split("pub struct UiHitTestEntry", 1)[1].split("pub struct UiHitTestCell", 1)[0]
        grid = source.split("pub struct UiHitTestGrid", 1)[1].split("impl Default for UiHitTestGrid", 1)[0]

        self.assertIn("pub struct UiHitRouteNode", source)
        self.assertIn("pub route_node_index: u32", entry)
        self.assertNotIn("effective_input_policy", entry)
        self.assertNotIn("bubble_route", entry)
        self.assertIn("pub route_nodes: Arc<Vec<UiHitRouteNode>>", grid)

    def test_route_publication_is_iterative_fail_closed_and_depth_independent(self) -> None:
        source = ROUTE_INDEX.read_text(encoding="utf-8")

        self.assertIn("pub(super) fn build_route_nodes", source)
        self.assertIn("Vec::with_capacity(arranged_tree.nodes.len())", source)
        self.assertIn("route_publication_preallocates_node_table", source)
        self.assertIn("UiHitRouteNode::invalid", source)
        self.assertIn("while let Some", source)
        self.assertNotIn("fn resolve_route_node", source)
        self.assertIn("deep_chain_builds_without_recursion", source)
        self.assertIn("missing_parent_and_cycle_fail_closed", source)

    def test_full_route_build_reuses_parent_index_from_validation_walk(self) -> None:
        source = ROUTE_INDEX.read_text(encoding="utf-8")
        build = source.split("pub(super) fn build_route_nodes", 1)[1].split(
            "pub(super) fn patch_route_nodes", 1
        )[0]
        self.assertIn("let mut chain: Vec<(usize, Option<usize>)> = Vec::new();", build)
        self.assertIn("chain.push((index, parent_index));", build)
        self.assertRegex(
            build,
            r"compose_route_node\(\s*node,\s*node_indices,\s*&route_nodes,\s*None,\s*parent_index,?\s*\)",
        )
        self.assertNotRegex(
            build,
            r"compose_route_node\(\s*node,\s*node_indices,\s*&route_nodes,\s*None\s*\)",
        )

    def test_full_build_does_not_repeat_ancestor_walk_helpers(self) -> None:
        source = HIT_TEST.read_text(encoding="utf-8")
        build = source.split("fn build_hit_grid", 1)[1].split("fn cell_bounds_for_query", 1)[0]

        self.assertIn("build_route_nodes", build)
        self.assertNotIn("arranged_bubble_route_indexed", build)
        self.assertNotIn("arranged_effective_input_policy_indexed", build)
        self.assertNotIn("is_arranged_child_hit_path_visible_indexed", build)

    def test_hit_grid_entry_projection_reserves_draw_order_upper_bound(self) -> None:
        source_text = HIT_TEST.read_text(encoding="utf-8")
        build = source_text.split("fn build_hit_grid", 1)[1].split(
            "fn cell_bounds_for_query", 1
        )[0]

        self.assertIn(
            "let mut entries = Vec::with_capacity(arranged_tree.draw_order.len());",
            build,
        )
        self.assertIn("for node_id in arranged_tree.draw_order.iter().copied()", build)
        self.assertIn("let Some(node_index) = node_indices.get(&node_id).copied()", build)
        self.assertRegex(
            build,
            r"arranged_tree\s*\.\s*nodes\s*\.\s*get\(node_index\)",
        )
        self.assertNotIn(
            "route_node_index_for_node(node_indices, node.node_id)",
            build,
        )
        self.assertNotIn(".filter_map(|node_id|", build)

    def test_geometry_and_input_patches_have_separate_route_costs(self) -> None:
        source = HIT_TEST.read_text(encoding="utf-8")
        geometry = HIT_GEOMETRY_PATCH.read_text(encoding="utf-8")
        input_patch = source.split("pub(crate) fn patch_arranged_input", 1)[1].split(
            "fn entry_index_by_node_id", 1
        )[0]

        geometry_production = geometry.split("#[cfg(test)]", 1)[0]
        self.assertNotIn("patch_route_nodes", geometry_production)
        self.assertNotIn("arranged_bubble_route", geometry_production)
        self.assertIn("patch_route_nodes", input_patch)
        self.assertIn("geometry_patch_reuses_route_table", source)
        self.assertIn(
            "input_patch_updates_descendant_route_semantics",
            ROUTE_INDEX.read_text(encoding="utf-8"),
        )
        self.assertIn(
            "input_patch_without_route_semantic_change_keeps_shared_allocation",
            ROUTE_INDEX.read_text(encoding="utf-8"),
        )

    def test_event_path_materializes_only_the_selected_route(self) -> None:
        source = HIT_TEST.read_text(encoding="utf-8")
        result = source.split("fn hit_result_from_stacked", 1)[1].split(
            "fn entry_frame_and_input_policy", 1
        )[0]

        self.assertIn("bubble_route_for_entry", result)
        self.assertIn("UiHitPath::from_bubble_route", result)
        self.assertNotIn("arranged_bubble_route", result)
        self.assertNotIn("unwrap_or_default", result)
        self.assertIn("malformed_parent_route_fails_closed", source)

    def test_popup_projection_shares_and_queries_the_same_route_authority(self) -> None:
        source = FRAME_HIT_TEST.read_text(encoding="utf-8")

        self.assertIn("route_nodes: base_grid.route_nodes.clone()", source)
        self.assertIn("find_bubble_route_value", source)
        self.assertNotIn("entry.bubble_route", source)

    def test_rebuild_routes_input_descendants_to_input_patch_only(self) -> None:
        source = REBUILD.read_text(encoding="utf-8")
        input_branch = source.split("if dirty.hit_test || dirty.input", 1)[1].split(
            "if dirty.render", 1
        )[0]

        self.assertIn("patch_arranged_input", input_branch)
        self.assertNotIn("patch_arranged_geometry", input_branch)


if __name__ == "__main__":
    unittest.main()
