from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def source(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RuntimeOperationInflightRetentionPerformanceContract(unittest.TestCase):
    def test_blocking_worker_behavior_gate_covers_pressure_and_later_admission(self) -> None:
        behavior = source(
            "zircon_runtime/src/operation/tests/inflight_retention.rs"
        )

        for marker in [
            "cancelled_preparing_task_remains_non_evictable_until_worker_completion",
            "service.cancel(first)",
            "pressure_admission = service.submit",
            "TaskCapacityReached { maximum: 1 }",
            "post_completion_admission.is_some()",
        ]:
            self.assertIn(marker, behavior)

    def test_tombstone_eviction_retains_worker_completion_lease(self) -> None:
        admission = source("zircon_runtime/src/operation/service/admission.rs")
        start = admission.index("fn evict_tombstones_until_admissible")
        eviction = admission[start:]

        self.assertIn("task.prepare_in_flight", eviction)
        self.assertIn("!task.prepare_in_flight", "".join(eviction.split()))

    def test_maintenance_and_completion_spatial_scale_is_observable(self) -> None:
        maintenance = source("zircon_runtime/src/operation/maintenance.rs")
        completion = source("zircon_runtime/src/operation/service/completion.rs")

        for counter in [
            "operation.deadline_scan_rows",
            "operation.deadline_expired_rows",
            "operation.terminal_ttl_scan_rows",
            "operation.terminal_ttl_expired_rows",
            "operation.maintenance_select_scan_rows",
        ]:
            self.assertIn(counter, maintenance)
        for counter in [
            "operation.completion_receiver_rows",
            "operation.completion_receiver_probe",
            "operation.completion_rows",
            "operation.completion_lost_rows",
        ]:
            self.assertIn(counter, completion)


if __name__ == "__main__":
    unittest.main()
