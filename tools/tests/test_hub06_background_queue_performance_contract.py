from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
HUB_ROOT = ROOT / "zircon_hub/src"
ACTION_TASKS = HUB_ROOT / "tauri_app/runtime_state/action_tasks.rs"
QUEUE_ADMISSION = HUB_ROOT / "tauri_app/runtime_state/action_tasks/queue_admission.rs"
ERRORS = HUB_ROOT / "error.rs"


class Hub06BackgroundQueuePerformanceContractTests(unittest.TestCase):
    def test_background_queue_is_bounded_before_request_clone(self) -> None:
        action_tasks = ACTION_TASKS.read_text(encoding="utf-8")
        queue_admission = QUEUE_ADMISSION.read_text(encoding="utf-8")

        self.assertIn("mod queue_admission;", action_tasks)
        self.assertIn("BACKGROUND_ACTION_QUEUE_CAPACITY: usize = 64", queue_admission)
        capacity_check = queue_admission.index(
            "queue.len() >= BACKGROUND_ACTION_QUEUE_CAPACITY"
        )
        request_clone = queue_admission.index("queue.push_back(request.clone())")
        self.assertLess(capacity_check, request_clone)
        self.assertIn("HubError::BackgroundActionQueueFull", queue_admission)

    def test_queue_contract_has_behavior_and_release_performance_evidence(self) -> None:
        source = QUEUE_ADMISSION.read_text(encoding="utf-8")

        self.assertIn("background_queue_rejects_request_at_capacity", source)
        self.assertIn("background_queue_preserves_fifo_below_capacity", source)
        self.assertIn("hub06_background_queue_admission_release_benchmark_evidence", source)
        self.assertIn("HUB06_BACKGROUND_QUEUE_BENCH_V1", source)
        self.assertIn("legacy_retained_requests=10000", source)
        self.assertIn("optimized_retained_requests={BACKGROUND_ACTION_QUEUE_CAPACITY}", source)
        self.assertIn(".div_ceil(100)", source)
        self.assertIn(
            "optimized_p95_ns.saturating_mul(100)"
            " <= legacy_p95_ns.saturating_mul(35)",
            source,
        )

    def test_queue_overload_uses_a_typed_error(self) -> None:
        errors = ERRORS.read_text(encoding="utf-8")

        self.assertIn("BackgroundActionQueueFull { capacity: usize }", errors)
        self.assertIn("background action queue is full", errors)


if __name__ == "__main__":
    unittest.main()
