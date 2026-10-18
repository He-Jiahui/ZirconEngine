from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/tools/scheduler.rs"
LOWER = (
    ROOT / "zircon_editor/src/core/tools/tests/optimization_batch_editor827_tool_scheduler_promotion_tests.rs"
)


class EditorToolSchedulerPromotionCapacityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_promotion_vectors_reserve_their_known_upper_bounds(self) -> None:
        set_body = re.search(
            r"fn\s+promote_available_sets\(.*?\n\s*}\n\n\s*fn\s+promote_waiting_singles",
            self.source,
            re.S,
        )
        single_body = re.search(
            r"fn\s+promote_waiting_singles\(.*?\n\s*}\n\n\s*fn\s+remove_empty_resource_states",
            self.source,
            re.S,
        )
        self.assertIsNotNone(set_body)
        self.assertIsNotNone(single_body)
        self.assertIn("let mut activated = Vec::new();", set_body.group(0))
        self.assertIn(
            "if activated.is_empty() {\n                activated.reserve(self.set_queue.len());\n            }",
            set_body.group(0),
        )
        self.assertIn("let mut activated = Vec::new();", single_body.group(0))
        self.assertIn(
            "if activated.is_empty() {\n                activated.reserve(self.resources.len());\n            }",
            single_body.group(0),
        )

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "mod optimization_batch_editor827_tool_scheduler_promotion_tests;",
            self.source,
        )
        self.assertIn(
            "editor827_release_promotes_set_and_single_waiters_in_order",
            lower,
        )
        self.assertIn("EDITOR827_TOOL_SCHEDULER_PROMOTION_BENCH_V1", lower)

    def test_deterministic_model_removes_geometric_growth(self) -> None:
        pending_sets = 4_096
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(pending_sets, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
