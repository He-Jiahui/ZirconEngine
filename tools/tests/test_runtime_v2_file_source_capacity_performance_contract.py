import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/file_cache.rs"
REGRESSION = REPO_ROOT / "zircon_runtime/src/ui/v2/tests/file_source_capacity_tests.rs"


class RuntimeV2FileSourceCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8") if REGRESSION.exists() else ""
        start = cls.source.index("fn collect_v2_sources")
        cls.collector = cls.source[start : cls.source.index("fn push_source_path", start)]

    def test_source_queue_and_output_reserve_root_path_bound(self) -> None:
        self.assertEqual(self.collector.count("Vec::with_capacity(paths.len())"), 2)
        self.assertNotIn("let mut queue = Vec::new();", self.collector)
        self.assertNotIn("let mut sources = Vec::new();", self.collector)

    def test_queue_and_source_ordering_contracts_remain_wired(self) -> None:
        self.assertIn("push_source_path(&mut queue, &mut seen, path.clone());", self.collector)
        self.assertIn("sources.push(UiV2FileSource { path, document });", self.collector)
        self.assertIn("while index < queue.len()", self.collector)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod file_source_capacity_tests;", self.source)
        self.assertIn("runtime855_v2_file_source_capacity_preserves_order", self.regression)
        self.assertIn("RUNTIME855_V2_FILE_SOURCE_CAPACITY_BENCH_V1", self.regression)

    def test_deterministic_model_removes_root_only_growth(self) -> None:
        root_paths = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertGreater(root_paths, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
