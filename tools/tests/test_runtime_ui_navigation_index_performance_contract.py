from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface"
TREE_FOCUS = ROOT / "zircon_runtime/src/ui/tree/node/focus.rs"


def read_surface_rebuild_source() -> str:
    rebuild_root = SURFACE / "surface/rebuild"
    return (
        (rebuild_root.with_suffix(".rs")).read_text(encoding="utf-8")
        + (rebuild_root / "incremental.rs").read_text(encoding="utf-8")
    )


class RuntimeUiNavigationIndexPerformanceContractTests(unittest.TestCase):
    def test_navigation_index_is_registered_and_owns_product_queries(self) -> None:
        module = (SURFACE / "mod.rs").read_text(encoding="utf-8")
        routing = (SURFACE / "surface/event_routing.rs").read_text(encoding="utf-8")
        effect = (SURFACE / "input/effect/navigation.rs").read_text(encoding="utf-8")

        self.assertIn("mod navigation_index;", module)
        self.assertIn("self.next_navigation_target(route_target, route_kind)?", routing)
        self.assertIn(".next_navigation_target(route.target, *kind)", effect)
        self.assertNotIn("self.tree.next_navigation_target", routing)
        self.assertNotIn(".tree\n                .next_navigation_target", effect)

    def test_event_query_uses_prebuilt_projected_geometry(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        tests = (SURFACE / "navigation_index/tests/cases.rs").read_text(encoding="utf-8")
        query_start = source.index("pub(super) fn next_navigation_target(")
        query_end = source.index("fn clear_for_rebuild", query_start)
        query = source[query_start:query_end]

        self.assertIn("authoritative_entry(base_hit_grid, node_id)", source)
        self.assertIn("published_hit_geometry_is_the_directional_navigation_authority", tests)
        self.assertNotIn("tree:", query)
        self.assertNotIn("collect_node(", query)
        self.assertNotIn(".sort", query)

    def test_navigation_index_production_module_stays_below_large_file_threshold(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        semantics = (SURFACE / "navigation_index/semantics.rs").read_text(encoding="utf-8")

        self.assertLess(len(source.splitlines()), 1000)
        self.assertLess(len(semantics.splitlines()), 300)
        self.assertIn("mod semantics;", source)
        self.assertIn("#[cfg(test)]", source)
        self.assertIn('#[path = "navigation_index/tests/cases.rs"]', source)
        self.assertIn("mod tests;", source)

    def test_tree_local_rebuild_and_geometry_authority_is_removed(self) -> None:
        source = TREE_FOCUS.read_text(encoding="utf-8")

        self.assertNotIn("fn next_navigation_target", source)
        self.assertNotIn("fn navigation_candidates", source)
        self.assertNotIn("NavigationCandidate", source)
        self.assertNotIn("nearest_navigation_candidate_in_direction", source)

    def test_rebuild_boundary_skips_stable_render_only_navigation_work(self) -> None:
        source = read_surface_rebuild_source()

        self.assertIn("let projected_geometry_changed =", source)
        self.assertIn("let navigation_semantics_changed =", source)
        self.assertIn("navigation_index_patch_projected_geometry", source)
        self.assertIn("navigation_projected_geometry_requires_rebuild", source)
        self.assertNotIn("|| dirty.render;", source)

    def test_layout_geometry_rebuild_is_scoped_to_navigation_candidates(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        geometry_patch = (SURFACE / "navigation_index/geometry_patch.rs").read_text(
            encoding="utf-8"
        )
        tests = (SURFACE / "navigation_index/tests/cases.rs").read_text(encoding="utf-8")
        rebuild = read_surface_rebuild_source()

        self.assertIn("patch_changed_geometry", geometry_patch)
        self.assertIn("tree: &UiTree", geometry_patch)
        self.assertIn("removed_node_ids", geometry_patch)
        self.assertIn("geometry_authority_node_ids", geometry_patch)
        self.assertIn("is_navigation_geometry_authority", geometry_patch)
        self.assertNotIn("self.nodes.contains_key(node_id)", geometry_patch)
        self.assertIn(
            "geometry_patch_skips_non_candidates_and_updates_focus_candidate_frames",
            tests,
        )
        self.assertIn(
            "navigation_index_patch_changed_geometry",
            rebuild,
        )
        self.assertIn("navigation_geometry_requires_rebuild", rebuild)
        self.assertIn("navigation_index_needs_semantics_rebuild", rebuild)
        self.assertNotIn(
            "let navigation_semantics_changed = dirty.style || dirty.text || dirty.visible_range;",
            rebuild,
        )
        self.assertIn("|| navigation_semantics_changed", rebuild)
        self.assertIn("popup_dependency_impact.stack_reconciliation", rebuild)
        self.assertIn("navigation_index_patch_projected_geometry", rebuild)
        self.assertNotIn("|| projected_hit_changed\n", rebuild)
        self.assertIn('"ui.navigation_index.geometry_skip_count"', rebuild)

    def test_tab_position_indexes_use_reusable_hash_maps(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")

        self.assertIn("collections::{BTreeMap, BTreeSet, HashMap}", source)
        self.assertIn("tab_base_positions: HashMap<UiNodeId, usize>", source)
        self.assertIn(
            "tab_group_positions: HashMap<UiNavigationGroupId, HashMap<UiNodeId, usize>>",
            source,
        )
        self.assertIn(
            "tab_mui_root_positions: HashMap<UiNodeId, HashMap<UiNodeId, usize>>",
            source,
        )
        self.assertNotIn("tab_base_positions: BTreeMap", source)
        self.assertNotIn("tab_group_positions: BTreeMap", source)
        self.assertNotIn("tab_mui_root_positions: BTreeMap", source)
        self.assertNotIn(
            "self.tab_group_positions.clear();",
            source,
            "nested group buckets must retain their capacity across rebuilds",
        )
        self.assertNotIn(
            "self.tab_mui_root_positions.clear();",
            source,
            "nested MUI-root buckets must retain their capacity across rebuilds",
        )
        self.assertIn("rebuild_position_map(", source)
        self.assertIn("rebuild_group_position_maps(", source)
        self.assertIn("rebuild_root_position_maps(", source)
        self.assertNotIn(
            "fn position_map(candidates: &[UiNodeId]) -> BTreeMap<UiNodeId, usize>",
            source,
        )

    def test_stable_group_keys_use_lookup_before_owned_insert(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")

        self.assertIn(
            "if let Some(position_map) = positions.get_mut(group_id)",
            source,
            "stable navigation groups should update their bucket without cloning the String key",
        )
        self.assertIn(
            "positions.insert(group_id.clone(), position_map)",
            source,
            "only a newly admitted group should pay the owned key clone",
        )
        self.assertNotIn(
            "positions.entry(group_id.clone())",
            source,
            "HashMap::entry would clone every stable group key before discovering occupancy",
        )

    def test_first_group_target_avoids_temporary_candidate_vectors(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        finish_lists = source.split("    fn finish_lists(&mut self) {", 1)[1].split(
            "    fn next_tab_target(", 1
        )[0]

        self.assertIn(
            "first_candidate_by_group.get_mut(group_id)",
            finish_lists,
            "group targets should be reduced while the retained node stream is already hot",
        )
        self.assertIn(
            "self.first_candidate_by_group.insert(\n                        group_id.clone(),",
            finish_lists,
            "only the first group admission should clone an owned group key",
        )
        self.assertNotIn(
            "let mut group_candidates",
            finish_lists,
            "group-first navigation targets must not materialize a temporary candidate map",
        )

    def test_navigation_scope_candidate_buckets_retain_capacity(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        bucket_module = (SURFACE / "navigation_index/candidate_buckets.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("mod candidate_buckets;", source)
        self.assertIn("pub(super) fn clear_candidate_buckets", bucket_module)
        self.assertIn("pub(super) fn prune_empty_candidate_buckets", bucket_module)
        self.assertIn("pub(super) fn push_group_candidate", bucket_module)
        self.assertIn(
            "clear_candidate_buckets(&mut self.tab_groups);",
            source,
            "tab group vectors should be cleared in place before a rebuild",
        )
        self.assertIn(
            "clear_candidate_buckets(&mut self.spatial_groups);",
            source,
            "spatial group vectors should be cleared in place before a rebuild",
        )
        self.assertIn(
            "clear_candidate_buckets(&mut self.tab_mui_roots);",
            source,
            "tab MUI-root vectors should be cleared in place before a rebuild",
        )
        self.assertIn(
            "clear_candidate_buckets(&mut self.spatial_mui_roots);",
            source,
            "spatial MUI-root vectors should be cleared in place before a rebuild",
        )
        self.assertNotIn(
            "self.tab_groups.clear();",
            source,
            "dropping stable tab group buckets forfeits their Vec capacity",
        )
        self.assertNotIn(
            "self.spatial_groups.clear();",
            source,
            "dropping stable spatial group buckets forfeits their Vec capacity",
        )
        self.assertIn("prune_empty_candidate_buckets(&mut self.tab_groups);", source)
        self.assertIn("prune_empty_candidate_buckets(&mut self.spatial_groups);", source)
        self.assertIn("prune_empty_candidate_buckets(&mut self.tab_mui_roots);", source)
        self.assertIn("prune_empty_candidate_buckets(&mut self.spatial_mui_roots);", source)

    def test_navigation_scope_candidates_insert_owned_group_keys_only_on_new_scope(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        finish_lists = source.split("    fn finish_lists(&mut self) {", 1)[1].split(
            "    fn next_tab_target(", 1
        )[0]

        self.assertIn(
            "push_group_candidate(&mut self.spatial_groups, group_id, *node_id);",
            finish_lists,
        )
        self.assertIn(
            "push_group_candidate(&mut self.tab_groups, group_id, *node_id);",
            finish_lists,
        )
        self.assertNotIn(
            ".entry(group_id.clone())",
            finish_lists,
            "stable scope candidates should not clone group keys before probing occupancy",
        )

    def test_first_group_candidate_map_reuses_stable_keys_across_rebuilds(self) -> None:
        source = (SURFACE / "navigation_index.rs").read_text(encoding="utf-8")
        bucket_module = (SURFACE / "navigation_index/candidate_buckets.rs").read_text(
            encoding="utf-8"
        )
        finish_lists = source.split("    fn finish_lists(&mut self) {", 1)[1].split(
            "    fn next_tab_target(", 1
        )[0]

        self.assertIn(
            "first_candidate_by_group: HashMap<UiNavigationGroupId, FirstGroupCandidate>",
            source,
        )
        self.assertIn("pub(super) struct FirstGroupCandidate", bucket_module)
        self.assertIn(
            "reset_first_group_candidates(&mut self.first_candidate_by_group);",
            source,
        )
        self.assertIn(
            "prune_first_group_candidates(&mut self.first_candidate_by_group);",
            finish_lists,
        )
        self.assertNotIn(
            "self.first_candidate_by_group.clear();",
            source,
            "stable first-candidate group keys must survive rebuilds",
        )
        self.assertIn("candidate.seen", finish_lists)


if __name__ == "__main__":
    unittest.main()
