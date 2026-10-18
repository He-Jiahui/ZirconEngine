import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
DEBUG_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/debug.rs"
DEBUG_TEST_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/debug/tests/performance_tests.rs"
)
CACHE_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/cache.rs"
CACHE_TEST_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/cache/tests/performance_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03RenderDebugCachePerformanceContract(unittest.TestCase):
    def test_debug_snapshot_moves_final_batch_plan_payloads(self) -> None:
        source = DEBUG_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn from_paint_elements(")
        compact_body = "".join(body.split())

        self.assertIn("plan.batches.into_iter()", compact_body)
        self.assertNotIn("plan\n                .batches\n                .iter()", body)
        self.assertIn("key: batch.key", body)
        self.assertIn("source_indices: batch.source_indices", body)
        self.assertIn("node_ids: batch.node_ids", body)
        self.assertIn("mod performance_tests;", source)
        benchmark = DEBUG_TEST_SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "RUNTIME_INTERFACE03_MOVED_RENDER_DEBUG_BATCHES_BENCH_V1",
            benchmark,
        )

    def test_cache_stats_are_counted_while_entries_are_built(self) -> None:
        source = CACHE_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_paint_elements_and_batches(")

        self.assertIn("let mut reused_paint_count = 0;", body)
        self.assertIn("let mut reused_batch_count = 0;", body)
        self.assertIn("reused_paint_count += usize::from", body)
        self.assertIn("reused_batch_count += usize::from", body)
        self.assertNotIn("UiRenderCacheStats::from_entries", body)
        self.assertNotIn("fn from_entries(", source)
        self.assertIn("mod performance_tests;", source)
        benchmark = CACHE_TEST_SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "RUNTIME_INTERFACE03_FUSED_RENDER_CACHE_STATS_BENCH_V1",
            benchmark,
        )


if __name__ == "__main__":
    unittest.main()
