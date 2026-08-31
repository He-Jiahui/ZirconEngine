from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PRESENT_STATS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs"
)
EDITOR_BATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/ui_perf/counter_batch.rs"
)
RUNTIME_SCOPE = ROOT / (
    "zircon_runtime/src/core/runtime/diagnostics/profiling/scope.rs"
)


class EditorGpuPresentStatsBatchPerformanceContractTests(unittest.TestCase):
    def test_one_present_publishes_one_counter_batch(self) -> None:
        source = PRESENT_STATS.read_text(encoding="utf-8")
        entry = source.split("pub(super) fn record_present_stats", 1)[1]
        entry = entry.split("fn append_present_stats", 1)[0]

        self.assertEqual(entry.count("record_current_ui_perf_counter_batch"), 1)
        self.assertNotIn("record_current_ui_perf_counter(", entry)
        self.assertIn("append_present_stats(counters, stats, region_present)", entry)

    def test_inactive_capture_returns_before_counter_allocation(self) -> None:
        source = EDITOR_BATCH.read_text(encoding="utf-8")
        helper = source.split("pub(crate) fn record_current_ui_perf_counter_batch", 1)[1]
        helper = helper.split("#[cfg(feature = \"profiling\")]\nfn named_counter_batch", 1)[0]

        inactive_return = helper.index("return;")
        allocation = helper.index("Vec::with_capacity")
        submission = helper.index("record_counter_batch(\"editor\", &named)")

        self.assertLess(inactive_return, allocation)
        self.assertLess(allocation, submission)

    def test_runtime_batch_uses_one_lock_scope_and_shared_timestamp(self) -> None:
        source = RUNTIME_SCOPE.read_text(encoding="utf-8")
        batch = source.split("pub(crate) fn record_counter_batch", 1)[1]

        self.assertEqual(batch.count("with_recorder(|recorder|"), 1)
        self.assertEqual(batch.count("let timestamp_us = recorder.now_us();"), 1)
        self.assertIn("for &(name, value) in counters", batch)
        self.assertIn("timestamp_us,", batch)
        self.assertNotIn("timestamp_us: recorder.now_us()", batch)


if __name__ == "__main__":
    unittest.main()
