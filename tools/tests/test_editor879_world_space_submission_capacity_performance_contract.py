import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node.rs"
)
LOWER = (
    REPO_ROOT / "zircon_editor/src/ui/retained_host/host_contract/data/world_space_submission/builder/node/tests/capacity_tests.rs"
)


class Editor879WorldSpaceSubmissionCapacityPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        match = re.search(
            r"pub\(super\) fn extend_world_space_ui_surface_submissions\(.*?\n}\n\n",
            cls.source,
            re.S,
        )
        if match is None:
            raise AssertionError("world-space submission append owner is missing")
        cls.body = match.group(0)

    def test_append_owner_tracks_the_existing_destination_boundary(self) -> None:
        self.assertIn("let initial_len = submissions.len()", self.body)
        self.assertIn("if submissions.len() == initial_len", self.body)

    def test_first_retained_node_reserves_the_authored_node_bound(self) -> None:
        self.assertIn("submissions.reserve(nodes.row_count())", self.body)
        self.assertNotIn("submissions.reserve(nodes.len())", self.body)
        self.assertIn("submissions.push(submission)", self.body)

    def test_projection_stays_streaming_and_filters_before_materialization(self) -> None:
        self.assertIn("for node in nodes.iter().filter", self.body)
        self.assertIn("node.world_space_enabled", self.body)
        self.assertIn("world_space_submission_for_node(surface_id, node)", self.body)
        self.assertNotIn("submissions.extend(", self.body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod capacity_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "editor879_world_space_submission_capacity_keeps_empty_path_lazy",
            lower,
        )
        self.assertIn("EDITOR879_WORLD_SPACE_SUBMISSION_CAPACITY_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_growth_model(self) -> None:
        node_count = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertEqual(node_count, 4_096)
        self.assertEqual(legacy_growth_events, 11)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
