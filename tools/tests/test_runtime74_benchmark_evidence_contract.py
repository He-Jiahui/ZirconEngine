import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


CASES = (
    (
        REPO_ROOT / "zircon_runtime/src/ui/template/asset/component_contract/validation/tests/hash_index_tests.rs",
        "SAMPLE_PAIRS",
        "RUNTIME74_COMPONENT_CONTRACT_HASH_INDEX_BENCH_V1",
        "sample_pairs={SAMPLE_PAIRS}",
    ),
    (
        REPO_ROOT / "zircon_runtime" / "src" / "ui" / "tests" / "asset_dependency_index.rs",
        "CASCADE_SAMPLE_COUNT",
        "RUNTIME74_DEPENDENCY_CASCADE_HASH_VISITED_BENCH_V1",
        "sample_count={CASCADE_SAMPLE_COUNT}",
    ),
    (
        REPO_ROOT / "zircon_runtime/src/ui/template/asset/tests/hot_reload_plan_optimization_tests.rs",
        "SAMPLE_COUNT",
        "RUNTIME74_HOT_RELOAD_HASH_ADMISSION_BENCH_V1",
        "sample_pairs={SAMPLE_COUNT}",
    ),
    (
        REPO_ROOT / "zircon_runtime/src/ui/template/asset/prototype_store/tests/hash_index_tests.rs",
        "SAMPLE_COUNT",
        "RUNTIME74_UI_PROTOTYPE_HASH_INDEX_BENCH_V1",
        "samples={SAMPLE_COUNT}",
    ),
    (
        REPO_ROOT / "zircon_runtime/src/ui/v2/cache/tests/hash_lookup_tests.rs",
        "SAMPLE_COUNT",
        "RUNTIME74_UI_V2_PROTOTYPE_HASH_INDEX_BENCH_V1",
        "samples={SAMPLE_COUNT}",
    ),
    (
        REPO_ROOT / "zircon_runtime/src/ui/template/asset/tests/watch_invalidation_optimization_tests.rs",
        "SAMPLE_COUNT",
        "RUNTIME74_WATCH_INVALIDATION_HASH_ADMISSION_BENCH_V1",
        "sample_pairs={SAMPLE_COUNT}",
    ),
)


class Runtime74BenchmarkEvidenceContractTests(unittest.TestCase):
    def test_release_probes_use_101_alternating_samples(self) -> None:
        for path, constant, marker, sample_field in CASES:
            with self.subTest(path=path):
                source = path.read_text(encoding="utf-8")
                self.assertIn(f"const {constant}: usize = 101;", source)
                self.assertIn(marker, source)
                self.assertIn(sample_field, source)

    def test_release_probes_report_p50_p95_p99_and_balanced_order(self) -> None:
        for path, _, marker, _ in CASES:
            with self.subTest(path=path):
                source = path.read_text(encoding="utf-8")
                marker_start = source.index(marker)
                marker_block = source[marker_start : source.find(");", marker_start)]
                self.assertIn("p50", marker_block)
                self.assertIn("p95", marker_block)
                self.assertIn("p99", marker_block)
                self.assertIn("pair_order=alternating_", marker_block)
                self.assertRegex(
                    marker_block,
                    r"(?:legacy|ordered|baseline)_first_(?:pairs|samples)=51",
                )
                self.assertRegex(
                    marker_block,
                    r"(?:optimized|hash)_first_(?:pairs|samples)=50",
                )


if __name__ == "__main__":
    unittest.main()
