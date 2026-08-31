import unittest
from pathlib import Path

from tools.runtime25_watch_error_pressure import run


ROOT = Path(__file__).resolve().parents[2]
WATCH_DISPATCH = ROOT / (
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs"
)
BATCH_VALIDATOR = ROOT / "tools/zircon-validation-runtime25-watch-error-batch.ps1"


class Runtime25WatchErrorPressureTests(unittest.TestCase):
    def test_tail_admission_eliminates_prefix_record_moves(self) -> None:
        admission = run()["tail_admission"]

        self.assertEqual(admission["overflow_admission_count"], 199_936)
        self.assertEqual(admission["baseline_prefix_record_move_count"], 12_595_968)
        self.assertEqual(admission["candidate_prefix_record_move_count"], 0)
        self.assertEqual(admission["candidate_retained_error_count"], 64)

    def test_source_uses_bounded_vecdeque_admission(self) -> None:
        source = WATCH_DISPATCH.read_text(encoding="utf-8")
        helper = source.split("fn push_bounded_error", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]

        self.assertIn("&mut std::collections::VecDeque<T>", helper)
        self.assertIn("errors.pop_front()", helper)
        self.assertIn("errors.push_back(error)", helper)
        self.assertNotIn("remove(0)", helper)
        self.assertIn("const WATCH_ACTIVATION_ERROR_CAPACITY: usize = 64", source)

    def test_source_keeps_correctness_and_release_contracts(self) -> None:
        source = WATCH_DISPATCH.read_text(encoding="utf-8")

        self.assertIn(
            "activation_error_overflow_discards_oldest_and_preserves_fifo_order",
            source,
        )
        self.assertIn("watch_error_tail_queue_release_benchmark_evidence", source)
        self.assertIn("WATCH_ERROR_TAIL_QUEUE_BENCH_V1", source)
        self.assertIn("const ITEMS: usize = 200_000", source)
        self.assertIn("const SAMPLE_PAIRS: usize = 21", source)
        self.assertIn(
            "optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns.saturating_mul(3)",
            source,
        )

    def test_batch_validator_runs_correctness_then_release_benchmark(self) -> None:
        source = BATCH_VALIDATOR.read_text(encoding="utf-8")

        self.assertEqual(source.count("[pscustomobject]@{"), 2)
        self.assertIn(
            "activation_error_overflow_discards_oldest_and_preserves_fifo_order",
            source,
        )
        self.assertIn("watch_error_tail_queue_release_benchmark_evidence", source)
        self.assertIn('"--exact"', source)
        self.assertIn('"--ignored"', source)
        self.assertIn("RUNTIME25_BATCH_PASS", source)

    def test_release_acceptance_is_explicit_and_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertEqual(acceptance["optimized_p95_maximum_legacy_ratio"], 0.75)
        self.assertEqual(acceptance["percentile_method"], "nearest_rank")
        self.assertTrue(acceptance["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f",
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(items=0)
        with self.assertRaises(ValueError):
            run(capacity=0)
        with self.assertRaises(ValueError):
            run(items=63, capacity=64)


if __name__ == "__main__":
    unittest.main()
