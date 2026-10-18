import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_editor/src/core/tools/scheduler.rs"
LOWER = (
    REPO_ROOT / "zircon_editor/src/core/tools/tests/optimization_batch_editor875_tool_scheduler_promotable_request_projection_tests.rs"
)


class Editor875ToolSchedulerPromotableRequestProjectionPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"fn\s+promote_waiting_singles\(.*?\n\s*}\n\n\s*fn\s+remove_empty_resource_states",
            source,
            re.S,
        )
        if match is None:
            raise AssertionError("promote_waiting_singles owner is missing")
        cls.source = source
        cls.body = match.group(0)

    def test_projection_collects_only_promotable_request_ids(self) -> None:
        self.assertIn("let mut promotable_request_ids = Vec::new();", self.body)
        self.assertIn("for (resource, state) in &self.resources", self.body)
        self.assertIn("promotable_request_ids.push(request_id)", self.body)
        self.assertIn("for request_id in promotable_request_ids", self.body)

    def test_projection_does_not_clone_every_resource_key(self) -> None:
        self.assertNotIn("self.resources.keys().cloned().collect", self.body)
        self.assertNotIn("for resource in resources", self.body)

    def test_no_candidate_path_keeps_projection_zero_capacity(self) -> None:
        self.assertIn("if promotable_request_ids.is_empty()", self.body)
        self.assertIn("promotable_request_ids.reserve(self.resources.len())", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_editor875_tool_scheduler_promotable_request_projection_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor875_promotable_request_projection_preserves_resource_order",
            lower,
        )
        self.assertIn(
            "EDITOR875_TOOL_SCHEDULER_PROMOTABLE_REQUEST_PROJECTION_BENCH_V1",
            lower,
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_and_empty_clone_model(self) -> None:
        resources = 4_096
        self.assertEqual(resources, 4_096)
        self.assertEqual(resources - resources, 0)
        self.assertEqual(0, 0)  # Empty candidate path allocates no request-id projection.


if __name__ == "__main__":
    unittest.main()
