from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SERVICE = ROOT / "zircon_editor/src/ui/control/service.rs"
REGRESSION = (
    ROOT / "zircon_editor/src/ui/control/service/tests/activity_registry_hash_tests.rs"
)


class EditorActivityRegistrySnapshotCapacityPerformanceContract(unittest.TestCase):
    def test_sorted_activity_snapshots_reserve_registry_counts(self) -> None:
        source = SERVICE.read_text(encoding="utf-8")
        views = source.split("pub fn activity_views", 1)[1].split(
            "pub fn activity_windows", 1
        )[0]
        windows = source.split("pub fn activity_windows", 1)[1].split(
            "pub fn register_route", 1
        )[0]

        self.assertIn("Vec::with_capacity(self.activity_views.len())", views)
        self.assertIn("views.extend(self.activity_views.values().cloned())", views)
        self.assertIn("Vec::with_capacity(self.activity_windows.len())", windows)
        self.assertIn("windows.extend(self.activity_windows.values().cloned())", windows)
        self.assertIn("views.sort_by", views)
        self.assertIn("windows.sort_by", windows)

    def test_snapshot_capacity_regression_keeps_deterministic_order(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "optimization_batch_20260912_activity_registry_snapshots_reserve_known_counts",
            source,
        )
        self.assertIn("assert!(views.capacity() >= views.len());", source)
        self.assertIn("assert!(windows.capacity() >= windows.len());", source)


if __name__ == "__main__":
    unittest.main()
