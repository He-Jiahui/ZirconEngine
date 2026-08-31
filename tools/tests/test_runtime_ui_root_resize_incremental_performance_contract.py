from pathlib import Path
import unittest

from tools.ui_root_resize_incremental_pressure import run


ROOT = Path(__file__).resolve().parents[2]
SURFACE_REBUILD = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
INCREMENTAL_LAYOUT = ROOT / "zircon_runtime/src/ui/layout/pass/incremental.rs"
ARRANGE = ROOT / "zircon_runtime/src/ui/layout/pass/arrange.rs"
SURFACE_ARRANGED = ROOT / "zircon_runtime/src/ui/surface/arranged.rs"
RUNTIME_REGRESSION = (
    ROOT / "zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs"
)


class RuntimeUiRootResizeIncrementalPerformanceContractTests(unittest.TestCase):
    def test_product_resize_routes_to_the_existing_incremental_authority(self) -> None:
        rebuild = SURFACE_REBUILD.read_text(encoding="utf-8")
        incremental = INCREMENTAL_LAYOUT.read_text(encoding="utf-8")
        arrange = ARRANGE.read_text(encoding="utf-8")
        surface_arranged = SURFACE_ARRANGED.read_text(encoding="utf-8")
        regression = RUNTIME_REGRESSION.read_text(encoding="utf-8")

        self.assertIn("layout_dirty_before_resize", rebuild)
        self.assertIn("layout_dirty_node_ids", rebuild)
        self.assertIn("layout_input_patch_node_ids", rebuild)
        self.assertIn("arranged_geometry_patch_node_ids", rebuild)
        self.assertIn("render_payload_dirty_node_ids", rebuild)
        self.assertIn("node_dirty.visible_range", rebuild)
        self.assertNotIn("return root_size_changed", rebuild)
        self.assertIn("apply_mui_responsive_layout_indexed", incremental)
        self.assertIn("pure_root_resize", incremental)
        self.assertIn("arrange_resized_root", incremental)
        self.assertIn("copy_parent_size_dependent_children", arrange)
        self.assertIn("pending_geometry_node_ids", surface_arranged)
        self.assertIn(
            "input_patch_allows_geometry_committed_by_the_following_patch",
            surface_arranged,
        )
        self.assertIn("root_resize_reports_early_out_probe_work", regression)
        self.assertIn(
            "root_resize_excludes_non_layout_dirty_nodes_from_the_layout_budget",
            regression,
        )
        self.assertIn(
            "root_resize_combines_input_and_geometry_patches_without_full_rebuild",
            regression,
        )
        self.assertIn(
            "root_resize_dependency_index_tracks_a_child_that_becomes_stretched",
            regression,
        )
        self.assertIn(
            "clipped_root_resize_uses_the_conservative_clip_propagation_path",
            regression,
        )

    def test_pure_resize_work_scales_with_roots_and_size_dependencies(self) -> None:
        result = run(
            total_node_count=10_000,
            resize_step_count=200,
            root_count=1,
            parent_size_dependent_child_count=1,
        )

        self.assertEqual(result["full_measure_probe_work"], 2_000_000)
        self.assertEqual(result["full_arrange_probe_work"], 2_000_000)
        self.assertEqual(result["incremental_measure_probe_work"], 0)
        self.assertEqual(result["incremental_arrange_probes_per_step"], 2)
        self.assertEqual(result["incremental_arrange_probe_work"], 400)
        self.assertEqual(result["eliminated_measure_probe_work"], 2_000_000)
        self.assertEqual(result["eliminated_arrange_probe_work"], 1_999_600)
        self.assertEqual(result["arrange_probe_reduction_ratio"], 5_000)
        self.assertEqual(result["combined_patch_nodes_per_step"], 3)
        self.assertEqual(result["full_arranged_patch_work"], 2_000_000)
        self.assertEqual(result["full_hit_patch_work"], 2_000_000)
        self.assertEqual(result["full_render_patch_work"], 2_000_000)
        self.assertEqual(result["incremental_arranged_patch_work"], 600)
        self.assertEqual(result["incremental_hit_patch_work"], 600)
        self.assertEqual(result["incremental_render_patch_work"], 600)
        self.assertAlmostEqual(
            result["combined_post_layout_reduction_ratio"],
            10_000 / 3,
        )

    def test_model_rejects_invalid_dimensions(self) -> None:
        with self.assertRaises(ValueError):
            run(0, 200, 1, 0)
        with self.assertRaises(ValueError):
            run(10, 200, 11, 0)
        with self.assertRaises(ValueError):
            run(10, 200, 1, -1)
        with self.assertRaises(ValueError):
            run(10, 200, 1, 0, -1)


if __name__ == "__main__":
    unittest.main()
