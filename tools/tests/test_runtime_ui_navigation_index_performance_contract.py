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
        tests = (SURFACE / "navigation_index/tests.rs").read_text(encoding="utf-8")
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
        self.assertIn("#[cfg(test)]\nmod tests;", source)

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
        tests = (SURFACE / "navigation_index/tests.rs").read_text(encoding="utf-8")
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


if __name__ == "__main__":
    unittest.main()
