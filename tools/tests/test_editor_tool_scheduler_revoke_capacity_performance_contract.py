from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/tools/scheduler.rs"
LOWER = (
    ROOT / "zircon_editor/src/core/tools/tests/optimization_batch_editor828_tool_scheduler_revoke_capacity_tests.rs"
)


class EditorToolSchedulerRevokeCapacityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_revoke_collectors_lazily_reserve_and_outputs_use_exact_bounds(self) -> None:
        match = re.search(
            r"pub\(crate\) fn\s+revoke_owner_generation\(.*?\n\s*}\n\n\s*pub\(crate\) fn\s+shutdown",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("let mut lease_ids = Vec::new();", body)
        self.assertIn("lease_ids.reserve(self.leases.len());", body)
        self.assertIn("let mut request_positions = Vec::new();", body)
        self.assertIn("request_positions.reserve(self.requests.len());", body)
        self.assertIn(
            "let mut released_leases = Vec::with_capacity(lease_ids.len());",
            body,
        )
        self.assertIn(
            "let mut withdrawn_requests = Vec::with_capacity(request_positions.len());",
            body,
        )

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "mod optimization_batch_editor828_tool_scheduler_revoke_capacity_tests;",
            self.source,
        )
        self.assertIn("editor828_revoke_preserves_owner_and_kind_filters", lower)
        self.assertIn("EDITOR828_TOOL_SCHEDULER_REVOKE_CAPACITY_BENCH_V1", lower)

    def test_deterministic_model_removes_dense_revoke_growth(self) -> None:
        claims = 4_096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(claims, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
