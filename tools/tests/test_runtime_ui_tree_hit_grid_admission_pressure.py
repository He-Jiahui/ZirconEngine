import tempfile
import unittest
from pathlib import Path

from tools.runtime_ui_tree_hit_grid_admission_pressure import (
    SourceContractError,
    pressure_report,
    source_binding_report,
    validate_output_path,
)


ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiTreeHitGridAdmissionPressureTests(unittest.TestCase):
    def test_sequential_paint_order_cursor_removes_quadratic_rescans(self) -> None:
        report = pressure_report(node_count=10_000)
        paint = report["paint_order_admission"]

        self.assertEqual(paint["legacy_rescan_node_visits"], 49_995_000)
        self.assertEqual(paint["cursor_sequential_rebuild_node_visits"], 0)
        self.assertEqual(paint["cursor_invalidated_rebuild_node_visits"], 10_000)

    def test_huge_projected_extent_has_bounded_cell_admission(self) -> None:
        report = pressure_report(node_count=10_000, huge_extent=1_000_000)
        grid = report["hit_grid_admission"]

        self.assertEqual(grid["legacy_columns"], 15_625)
        self.assertEqual(grid["legacy_cell_count"], 244_140_625)
        self.assertEqual(grid["bounded_max_cell_count"], 16_384)
        self.assertEqual(grid["huge_entry_projected_cell_count"], 4_096)
        self.assertEqual(grid["wide_entry_membership_count"], 4_096)
        self.assertFalse(grid["global_grid_collapsed"])
        self.assertEqual(grid["non_finite_entry_cell_memberships"], 0)

    def test_adaptive_coarsening_is_not_presented_as_product_timing(self) -> None:
        report = pressure_report(node_count=10_000)

        self.assertFalse(report["is_product_timing"])
        self.assertEqual(
            report["hit_grid_admission"][
                "full_bounds_entry_candidate_contribution_per_query"
            ],
            1,
        )
        self.assertIn(
            "product input latency",
            report["interpretation"]["dynamic_acceptance_pending"],
        )

    def test_current_source_contract_is_ready_and_binds_dirty_sources(self) -> None:
        binding = source_binding_report(ROOT)

        self.assertTrue(binding["ready"], binding)
        self.assertEqual(len(binding["git_revision"]), 40)
        self.assertTrue(binding["critical_sources_dirty"])
        self.assertGreaterEqual(binding["critical_source_dirty_entry_count"], 4)
        self.assertTrue(binding["contracts"]["shared_bounded_grid_helpers"])
        self.assertTrue(binding["contracts"]["projected_grid_reuses_shared_helpers"])
        self.assertTrue(binding["contracts"]["non_finite_membership_rejected"])
        self.assertTrue(binding["contracts"]["no_duplicate_projected_cell_mapper"])
        critical_paths = {
            source["relative_path"] for source in binding["critical_sources"]
        }
        self.assertIn(
            "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs",
            critical_paths,
        )

    def test_source_contract_fails_closed_when_projected_helper_reuse_disappears(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative_path in (
                "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
                "zircon_runtime/src/ui/tree/hit_test.rs",
                "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs",
                "zircon_runtime/src/ui/tree/mod.rs",
                "zircon_runtime/src/ui/surface/frame_hit_test.rs",
                "tools/runtime_ui_tree_hit_grid_admission_pressure.py",
            ):
                source = ROOT / relative_path
                target = root / relative_path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(source.read_bytes())
            projected = root / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
            projected.write_text(
                projected.read_text(encoding="utf-8").replace(
                    "bounded_hit_grid_dimensions(bounds, &entries, cell_size)",
                    "legacy_projected_grid_dimensions(bounds, &entries, cell_size)",
                ),
                encoding="utf-8",
            )

            with self.assertRaises(SourceContractError):
                source_binding_report(root)

    def test_rejects_invalid_inputs_and_c_drive_outputs(self) -> None:
        with self.assertRaises(ValueError):
            pressure_report(node_count=0)
        with self.assertRaises(ValueError):
            pressure_report(huge_extent=float("nan"))
        with self.assertRaises(ValueError):
            pressure_report(cell_size=0)
        with self.assertRaises(ValueError):
            validate_output_path(Path("C:/profiles/tree-hit-grid.json"))
        self.assertEqual(
            validate_output_path(Path("E:/profiles/tree-hit-grid.json")),
            Path("E:/profiles/tree-hit-grid.json"),
        )


if __name__ == "__main__":
    unittest.main()
