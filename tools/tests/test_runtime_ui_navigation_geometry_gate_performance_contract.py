from pathlib import Path
import unittest

from tools.ui_navigation_geometry_gate_pressure import run, run_frame_patch


ROOT = Path(__file__).resolve().parents[2]
NAVIGATION = ROOT / "zircon_runtime/src/ui/surface/navigation_index.rs"
GEOMETRY_PATCH = ROOT / "zircon_runtime/src/ui/surface/navigation_index/geometry_patch.rs"
NAVIGATION_TESTS = ROOT / "zircon_runtime/src/ui/surface/navigation_index/tests.rs"
SURFACE_NAVIGATION_TESTS = (
    ROOT / "zircon_runtime/src/ui/tests/focus_navigation/tab_directional.rs"
)


def read_surface_rebuild_source() -> str:
    rebuild_root = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild"
    return (
        (rebuild_root.with_suffix(".rs")).read_text(encoding="utf-8")
        + (rebuild_root / "incremental.rs").read_text(encoding="utf-8")
    )


class RuntimeUiNavigationGeometryGatePerformanceContractTests(unittest.TestCase):
    def test_geometry_patch_uses_navigation_authorities_not_all_indexed_nodes(self) -> None:
        source = NAVIGATION.read_text(encoding="utf-8") + GEOMETRY_PATCH.read_text(
            encoding="utf-8"
        )

        self.assertIn("patch_changed_geometry", source)
        self.assertIn("geometry_authority_node_ids", source)
        self.assertIn("referenced_modal_root_node_ids", source)
        self.assertIn("is_navigation_geometry_authority", source)
        self.assertNotIn("self.nodes.contains_key(node_id)", source)
        self.assertNotIn("fn needs_geometry_rebuild(", source)

    def test_lower_regression_covers_non_candidate_and_candidate_changes(self) -> None:
        tests = NAVIGATION_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "geometry_patch_skips_non_candidates_and_updates_focus_candidate_frames",
            tests,
        )
        self.assertIn("index.patch_changed_geometry(", tests)
        self.assertIn(
            "projected_geometry_patch_updates_candidate_frames_but_rebuilds_for_order_changes",
            tests,
        )
        self.assertIn(
            "externally_referenced_modal_root_patches_frames_but_rebuilds_for_order_changes",
            tests,
        )

    def test_surface_regression_keeps_patch_and_fallback_generations_observable(self) -> None:
        tests = SURFACE_NAVIGATION_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "surface_geometry_patch_preserves_navigation_generation_and_ordering_rebuilds",
            tests,
        )
        self.assertIn("navigation_index_build_generation", tests)
        self.assertIn("frame-only candidate movement", tests)
        self.assertIn("ordering changes must fail closed", tests)

    def test_projected_hit_change_is_filtered_by_navigation_geometry(self) -> None:
        source = NAVIGATION.read_text(encoding="utf-8")
        rebuild = read_surface_rebuild_source()

        self.assertIn("patch_projected_geometry", source)
        self.assertIn("navigation_index_patch_projected_geometry", rebuild)
        self.assertNotIn("|| projected_hit_changed\n", rebuild)

    def test_pressure_model_eliminates_non_candidate_full_rebuilds(self) -> None:
        result = run(
            total_node_count=1_024,
            focus_candidate_count=64,
            non_candidate_update_count=128,
        )

        self.assertEqual(result["old_full_rebuild_count"], 128)
        self.assertEqual(result["new_full_rebuild_count"], 0)
        self.assertEqual(result["authority_gate_check_count"], 128)
        self.assertGreater(result["work_reduction_ratio"], 1_000)
        self.assertTrue(result["focus_candidate_change_detected"])
        self.assertTrue(result["removed_focus_candidate_detected"])

    def test_pressure_model_patches_candidate_frames_without_tree_rebuild(self) -> None:
        result = run_frame_patch(
            total_node_count=1_024,
            focus_candidate_count=64,
            candidate_frame_update_count=128,
        )

        self.assertEqual(result["old_full_rebuild_count"], 128)
        self.assertEqual(result["new_full_rebuild_count"], 0)
        self.assertEqual(result["changed_authority_check_count"], 128)
        self.assertEqual(result["candidate_geometry_lookup_count"], 128)
        self.assertEqual(result["candidate_frame_write_count"], 128)
        self.assertGreater(result["work_reduction_ratio"], 400)
        self.assertTrue(result["ordering_change_forces_rebuild"])


if __name__ == "__main__":
    unittest.main()
