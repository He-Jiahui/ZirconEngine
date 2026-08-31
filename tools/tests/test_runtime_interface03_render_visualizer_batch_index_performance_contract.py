from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
VISUALIZER = ROOT / "zircon_runtime_interface/src/ui/surface/render/visualizer.rs"


class RuntimeInterface03RenderVisualizerBatchIndexPerformanceContractTests(unittest.TestCase):
    def test_visualizer_builds_shared_batch_and_cache_indexes(self) -> None:
        source = VISUALIZER.read_text(encoding="utf-8")

        self.assertIn(
            "let batch_indices = batch_indices_by_source_index(&plan.batches",
            source,
        )
        self.assertIn(
            "paint_cache_statuses_by_index(&cache.paint_entries",
            source,
        )
        self.assertIn(
            "batch_cache_statuses_by_index(&cache.batch_entries",
            source,
        )
        self.assertIn("RUNTIME_INTERFACE03_RENDER_VISUALIZER_BATCH_INDEX_BENCH_V1", source)
        self.assertNotIn("fn batch_index_for_paint_index(", source)
        self.assertNotIn("cache.paint_entries\n                        .iter()\n                        .find(", source)
        self.assertNotIn("cache.batch_entries\n                    .iter()\n                    .find(", source)


if __name__ == "__main__":
    unittest.main()
