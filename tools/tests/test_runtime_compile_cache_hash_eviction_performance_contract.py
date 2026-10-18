import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache/tests/hash_eviction_tests.rs"
)


class RuntimeCompileCacheHashEvictionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_eviction_uses_borrowed_hash_membership(self) -> None:
        body = self.source.split("pub fn evict_assets", 1)[1].split("pub fn get", 1)[0]
        self.assertIn("collect::<HashSet<_>>()", body)
        self.assertIn("contains(compiled.asset.id.as_str())", body)
        self.assertNotIn("map(str::to_string)", body)
        self.assertNotIn("BTreeSet", body)

    def test_semantics_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "optimization_batch_20260826ah_runtime74_hash_eviction_preserves_multi_asset_semantics",
            self.regression,
        )
        self.assertIn(
            "optimization_batch_20260826ah_runtime74_compile_cache_hash_eviction_performance_evidence",
            self.regression,
        )
        self.assertIn("RUNTIME74_COMPILE_CACHE_HASH_EVICTION_BENCH_V1", self.regression)
        self.assertIn("const SAMPLE_COUNT: usize = 101;", self.regression)
        self.assertIn("sample_count={SAMPLE_COUNT}", self.regression)

    def test_cache_authority_remains_ordered_while_membership_is_hashed(self) -> None:
        self.assertIn("entries: BTreeMap<UiCompileCacheKey, UiCompiledDocument>", self.source)
        self.assertIn("last_snapshots: BTreeMap<String, UiInvalidationSnapshot>", self.source)
        self.assertIn("use std::collections::{BTreeMap, HashSet};", self.source)


if __name__ == "__main__":
    unittest.main()
