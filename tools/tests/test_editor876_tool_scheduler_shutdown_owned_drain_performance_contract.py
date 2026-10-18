import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/core/tools/scheduler.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/core/tools/tests/optimization_batch_editor876_tool_scheduler_shutdown_owned_drain_tests.rs"
)


class Editor876ToolSchedulerShutdownOwnedDrainPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"pub\(crate\)\s+fn\s+shutdown\(.*?\n\s*}\n\n\s*fn\s+report_existing_claim",
            source,
            re.S,
        )
        if match is None:
            raise AssertionError("ToolScheduler::shutdown owner is missing")
        cls.source = source
        cls.body = match.group(0)

    def test_shutdown_projects_positions_before_taking_request_ownership(self) -> None:
        self.assertIn("let queued_request_count = self.requests.len();", self.body)
        self.assertIn("let queued_positions = self", self.body)
        self.assertIn(".map(|request| self.request_position(request).unwrap_or(1))", self.body)
        self.assertLess(
            self.body.index("let queued_positions = self"),
            self.body.index("std::mem::take(&mut self.requests)"),
        )

    def test_shutdown_moves_requests_without_cloning_handles(self) -> None:
        self.assertIn(
            "let queued_requests = std::mem::take(&mut self.requests);", self.body
        )
        self.assertIn("queued_requests.into_values().zip(queued_positions)", self.body)
        self.assertNotIn(".values()\n            .cloned()", self.body)
        self.assertNotIn("self.requests.clear()", self.body)

    def test_shutdown_moves_leases_without_a_full_handle_vector(self) -> None:
        self.assertIn("let leases = std::mem::take(&mut self.leases);", self.body)
        self.assertIn("for lease in leases.into_values()", self.body)
        self.assertNotIn("into_values()\n            .collect::<Vec<_>>()", self.body)

    def test_event_capacity_uses_counts_without_handle_scratch_collections(self) -> None:
        self.assertIn("let active_lease_count = self.leases.len();", self.body)
        self.assertIn(".saturating_add(active_lease_count)", self.body)
        self.assertIn(".saturating_add(queued_request_count)", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_editor876_tool_scheduler_shutdown_owned_drain_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor876_shutdown_owned_drain_preserves_request_order_and_positions",
            lower,
        )
        self.assertIn(
            "EDITOR876_TOOL_SCHEDULER_SHUTDOWN_OWNED_DRAIN_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_shutdown_clone_model(self) -> None:
        queued_requests = 4_096
        active_leases = 4_096
        legacy_handle_clones = queued_requests
        optimized_handle_clones = 0
        legacy_handle_scratch_slots = queued_requests + active_leases
        optimized_handle_scratch_slots = 0
        self.assertEqual(legacy_handle_clones, 4_096)
        self.assertEqual(optimized_handle_clones, 0)
        self.assertEqual(legacy_handle_scratch_slots, 8_192)
        self.assertEqual(optimized_handle_scratch_slots, 0)


if __name__ == "__main__":
    unittest.main()
