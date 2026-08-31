from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STORE = ROOT / (
    "zircon_runtime/src/core/framework/render/viewport_highlight_store.rs"
)
BENCHMARK = ROOT / (
    "zircon_runtime/src/core/framework/render/viewport_highlight_store/"
    "hash_index_tests.rs"
)


class Runtime62ViewportHighlightHashIndexPerformanceContract(unittest.TestCase):
    def test_store_uses_an_unordered_viewport_owner(self) -> None:
        source = STORE.read_text(encoding="utf-8")

        self.assertIn("use std::collections::HashMap;", source)
        self.assertIn("by_viewport: HashMap<u64, ViewportHighlightSet>", source)
        self.assertNotIn("BTreeMap", source)
        self.assertNotIn("pub fn iter", source)
        self.assertNotIn("by_viewport.values", source)

    def test_behavior_contract_preserves_generation_and_viewport_isolation(self) -> None:
        source = BENCHMARK.read_text(encoding="utf-8")

        self.assertIn(
            "viewport_highlight_hash_index_preserves_generation_isolation", source
        )
        self.assertIn("assert!(!store.submit(3, 6, set([99])));", source)
        self.assertIn("assert_eq!(store.get(4).unwrap().generation(), 1);", source)

    def test_benchmark_uses_warm_alternating_tail_samples(self) -> None:
        source = BENCHMARK.read_text(encoding="utf-8")

        self.assertIn("const WARMUP_COUNT: usize = 4;", source)
        self.assertIn("const SAMPLE_COUNT: usize = 17;", source)
        self.assertIn("sample_index % 2 == 0", source)
        self.assertIn("ordered_ns", source)
        self.assertIn("hash_ns", source)

    def test_benchmark_gates_p95_and_emits_machine_data(self) -> None:
        source = BENCHMARK.read_text(encoding="utf-8")

        self.assertIn("hash_p95.saturating_mul(10)", source)
        self.assertIn("ordered_p95.saturating_mul(7)", source)
        self.assertIn(
            "RUNTIME62_VIEWPORT_HIGHLIGHT_HASH_INDEX_BENCH_V1", source
        )
        self.assertIn("ordered_p50_ns=", source)
        self.assertIn("hash_p95_ns=", source)


if __name__ == "__main__":
    unittest.main()
