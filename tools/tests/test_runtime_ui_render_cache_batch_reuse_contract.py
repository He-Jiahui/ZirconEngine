from pathlib import Path
import unittest

from tools.runtime_ui_render_cache_batch_reuse_pressure import run


ROOT = Path(__file__).resolve().parents[2]
CACHE_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/cache.rs"


class RuntimeUiRenderCacheBatchReuseContractTests(unittest.TestCase):
    def test_batch_reuse_checks_sources_without_collecting_a_temporary_vec(self) -> None:
        source = CACHE_SOURCE.read_text(encoding="utf-8")
        self.assertIn("fn batch_cache_status(", source)
        self.assertIn(
            "if reason != UiRenderCacheInvalidationReason::Unchanged",
            source,
        )
        self.assertIn(".all(|&source_index|", source)
        self.assertIn(
            "is_some_and(|element| element.cache_generation.is_some())", source
        )
        self.assertNotIn("collect::<Option<Vec<_>>>()", source)

    def test_pressure_model_removes_only_temporary_allocation_work(self) -> None:
        result = run(batch_count=4096, elements_per_batch=8)
        self.assertEqual(
            result["legacy_collect"]["source_index_visits"],
            result["borrowed_all_check"]["source_index_visits"],
        )
        self.assertEqual(
            result["borrowed_all_check"]["temporary_source_vec_allocations"], 0
        )
        self.assertEqual(
            result["delta"]["avoided_temporary_source_vec_allocations"], 4096
        )
        self.assertEqual(
            result["reason_first_short_circuit"]["dirty_frame_source_index_visits"],
            0,
        )
        self.assertEqual(
            result["delta"]["avoided_dirty_frame_source_index_visits"],
            4096 * 8,
        )

    def test_pressure_model_rejects_invalid_dimensions(self) -> None:
        with self.assertRaises(ValueError):
            run(batch_count=0)
        with self.assertRaises(ValueError):
            run(elements_per_batch=0)


if __name__ == "__main__":
    unittest.main()
