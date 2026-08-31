from pathlib import Path
import unittest

from tools.runtime_ui_scroll_geometry_patch_pressure import run


ROOT = Path(__file__).resolve().parents[2]
ARRANGED = ROOT / "zircon_runtime/src/ui/surface/arranged.rs"
REBUILD = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
SCROLL = ROOT / "zircon_runtime/src/ui/tree/node/scroll.rs"
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"
HIT_GEOMETRY_PATCH = ROOT / "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs"
SCROLL_TESTS = ROOT / "zircon_runtime/src/ui/tests/scroll_virtualization.rs"


class RuntimeUiScrollGeometryPatchPerformanceContractTests(unittest.TestCase):
    def test_scroll_offset_uses_layout_hit_and_render_domains_only(self) -> None:
        source = SCROLL.read_text(encoding="utf-8")
        implementation = source.split("impl UiRuntimeTreeScrollExt for UiTree", 1)[1]
        setter = implementation.split("fn set_scroll_offset", 1)[1].split(
            "fn scroll_by", 1
        )[0]

        self.assertIn("node.dirty.layout = true", setter)
        self.assertIn("node.dirty.hit_test = true", setter)
        self.assertIn("node.dirty.render = true", setter)
        self.assertNotIn("node.dirty.input = true", setter)
        self.assertNotIn("node.state_flags.dirty = true", setter)

    def test_arranged_geometry_patch_returns_exact_clip_affected_set(self) -> None:
        source = ARRANGED.read_text(encoding="utf-8")
        patch = source.split("pub(crate) fn patch_arranged_tree_geometry", 1)[1].split(
            "pub(crate) fn patch_arranged_tree_input", 1
        )[0]

        self.assertIn("Option<BTreeSet<UiNodeId>>", patch)
        self.assertIn("collect_tree_descendants", patch)
        self.assertIn("affected_node_ids", patch)
        self.assertNotIn("has_clip_ancestor", patch)

    def test_hit_geometry_patch_owns_stable_zero_area_entries(self) -> None:
        root = HIT_TEST.read_text(encoding="utf-8")
        geometry = HIT_GEOMETRY_PATCH.read_text(encoding="utf-8")
        build = root.split("fn build_hit_grid", 1)[1].split(
            "fn cell_bounds_for_query", 1
        )[0]

        self.assertIn("geometry_patch", root)
        self.assertIn("stable_geometry_entry", build)
        self.assertIn("unwrap_or_default", build)
        self.assertIn("geometry_patch_activates_and_deactivates_stable_entry_cells", geometry)
        self.assertIn("match (entry_index, next_entry)", geometry)
        self.assertIn("Vec::new()", geometry)

    def test_rebuild_routes_visible_range_through_actual_arranged_patch_set(self) -> None:
        source = REBUILD.read_text(encoding="utf-8")
        layout_branch = source.split(
            "if dirty.layout || dirty.style || dirty.text || dirty.visible_range", 1
        )[1].split("let mut report = UiSurfaceRebuildReport", 1)[0]

        self.assertIn("arranged_patch_node_ids", layout_branch)
        self.assertIn("dirty.visible_range", layout_branch)
        self.assertIn("patch_arranged_tree_geometry", layout_branch)
        self.assertNotIn("&& !dirty.visible_range", layout_branch)
        self.assertIn("virtual_scroll_patches_arranged_and_hit_without_index_fallback", SCROLL_TESTS.read_text(encoding="utf-8"))

    def test_pressure_model_reports_post_layout_savings_and_stable_entry_cost(self) -> None:
        result = run(
            total_node_count=1_024,
            hit_entry_count=768,
            scroll_update_count=128,
            affected_node_count=16,
            inactive_stable_entry_count=752,
            modeled_entry_bytes=64,
        )

        self.assertEqual(result["scope"], "post_layout_arranged_hit_only")
        self.assertEqual(result["full_arranged_rebuilds_modeled_after_patch"], 0)
        self.assertEqual(result["full_hit_rebuilds_modeled_after_patch"], 0)
        self.assertGreater(result["post_layout_work_reduction_ratio"], 50)
        self.assertEqual(result["modeled_stable_entry_payload_bytes"], 48_128)
        self.assertFalse(result["layout_child_iteration_modeled"])
        self.assertFalse(result["variable_render_command_patch_modeled"])
        self.assertFalse(result["true_instance_virtualization_modeled"])
        self.assertFalse(result["cpu_or_rss_measured"])


if __name__ == "__main__":
    unittest.main()
