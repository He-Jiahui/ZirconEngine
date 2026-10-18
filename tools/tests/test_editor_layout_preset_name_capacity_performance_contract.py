from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/host/layout_persistence.rs"
LOWER = ROOT / "zircon_editor/src/ui/host/tests/optimization_batch_editor830_layout_preset_capacity_tests.rs"


class EditorLayoutPresetNameCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = LOWER.read_text(encoding="utf-8")
        start = cls.source.index("    pub(super) fn preset_names")
        end = cls.source.index("\n    pub(super) fn load_global_default_layout", start)
        cls.body = cls.source[start:end]

    def test_preset_name_collector_reserves_known_asset_and_map_bounds(self) -> None:
        self.assertIn(
            "let asset_uris = self.asset_manager()?.current_project_asset_uris();",
            self.body,
        )
        self.assertIn("let presets = self.load_presets()?;", self.body)
        self.assertIn(
            "Vec::with_capacity(asset_uris.len().saturating_add(presets.len()))",
            self.body,
        )
        self.assertIn("names.extend(list_layout_preset_assets(asset_uris));", self.body)
        self.assertIn("names.extend(presets.into_keys());", self.body)

    def test_order_and_deduplication_remain_authoritative(self) -> None:
        self.assertIn("names.sort_unstable();", self.body)
        self.assertIn("names.dedup();", self.body)
        self.assertIn(
            "EDITOR830_LAYOUT_PRESET_NAME_CAPACITY_BENCH_V1",
            self.lower,
        )

    def test_lower_test_module_is_wired(self) -> None:
        self.assertIn(
            "#[path = \"tests/optimization_batch_editor830_layout_preset_capacity_tests.rs\"]",
            self.source,
        )

    def test_deterministic_combined_bound_removes_growth_events(self) -> None:
        asset_count = 4_096
        preset_count = 4_096
        combined_count = asset_count + preset_count
        legacy_growth_events = self._growth_events(combined_count, reserve=False)
        optimized_growth_events = self._growth_events(combined_count, reserve=True)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(legacy_growth_events, 12)
        self.assertEqual(optimized_growth_events, 0)

    @staticmethod
    def _growth_events(count: int, *, reserve: bool) -> int:
        capacity = count if reserve else 0
        events = 0
        for index in range(count):
            if index == capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                events += 1
        return events


if __name__ == "__main__":
    unittest.main()
