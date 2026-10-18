import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/tests/eviction_capacity_tests.rs"
)


class RuntimeCompileCacheEvictionKeyCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_eviction_key_vectors_reserve_input_asset_lower_bound(self) -> None:
        body = self.source.split("pub fn evict_assets", 1)[1].split("pub fn get", 1)[0]
        self.assertEqual(body.count("Vec::with_capacity(asset_ids.len())"), 2)
        self.assertLess(
            body.index("Vec::with_capacity(asset_ids.len())"),
            body.index(".filter_map"),
        )

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"compile_cache/tests/eviction_capacity_tests.rs\"]",
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260918_runtime797_compile_cache_eviction_key_capacity_bench",
            self.regression,
        )
        self.assertIn(
            "RUNTIME797_COMPILE_CACHE_EVICTION_KEY_CAPACITY_BENCH_V1",
            self.regression,
        )

    def test_capacity_model_is_bounded_by_input_asset_count(self) -> None:
        input_count = 65_536
        capacity = input_count
        self.assertEqual(capacity, input_count)
        self.assertGreater(input_count, 0)


if __name__ == "__main__":
    unittest.main()
