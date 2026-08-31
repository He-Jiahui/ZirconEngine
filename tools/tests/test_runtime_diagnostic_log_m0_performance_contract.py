from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RuntimeDiagnosticLogM0PerformanceContract(unittest.TestCase):
    def test_known_full_best_effort_gate_precedes_lazy_message_evaluation(self) -> None:
        worker = source("zircon_runtime/src/diagnostic_log/sink/worker.rs")
        start = worker.index("pub(super) fn enqueue_lazy")
        end = worker.index("pub(super) fn flush", start)
        enqueue = worker[start:end]

        self.assertIn("self.sender.is_full()", enqueue)
        self.assertLess(enqueue.index("self.sender.is_full()"), enqueue.index("let message = message();"))

    def test_empty_compiled_filter_returns_without_a_hash_probe(self) -> None:
        compiled = source("zircon_runtime/src/diagnostic_log/level/compiled.rs")
        start = compiled.index("fn filter_for_scope")
        body = compiled[start:]

        self.assertIn("self.nodes.len() == 1", body)
        self.assertLess(body.index("self.nodes.len() == 1"), body.index("for byte in scope.as_bytes()"))

    def test_batch_formats_one_wall_clock_timestamp(self) -> None:
        worker = source("zircon_runtime/src/diagnostic_log/sink/worker.rs")
        start = worker.index("fn flush_pending")
        end = worker.index("fn sync_outputs", start)
        flush_pending = worker[start:end]

        self.assertEqual(flush_pending.count("current_log_timestamp()"), 1)
        self.assertLess(
            flush_pending.index("current_log_timestamp()"),
            flush_pending.index("for record in pending.drain(..)"),
        )

    def test_control_admission_uses_bounded_channel_wait_without_busy_yield(self) -> None:
        worker = source("zircon_runtime/src/diagnostic_log/sink/worker.rs")
        start = worker.index("fn send_control_until")
        end = worker.index("fn run_sink_worker", start)
        admission = worker[start:end]

        self.assertIn("send_timeout", admission)
        self.assertNotIn("yield_now", admission)


if __name__ == "__main__":
    unittest.main()
