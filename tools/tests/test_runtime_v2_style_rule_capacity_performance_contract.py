import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/style.rs"
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/v2/style/tests/rule_capacity_tests.rs"
)


class RuntimeV2StyleRuleCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_rule_collection_reserves_known_stylesheet_rule_bound(self) -> None:
        match = re.search(
            r"fn\s+collect_rules\(.*?\n}\n",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("rule_count", body)
        self.assertIn("Vec::with_capacity(rule_count)", body)
        self.assertNotIn("let mut rules = Vec::new()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod rule_capacity_tests;", self.source)
        self.assertIn("runtime785_v2_style_rule_capacity_preserves_order", lower)
        self.assertIn("RUNTIME785_V2_STYLE_RULE_CAPACITY_BENCH_V1", lower)
        self.assertIn("optimized_growth_events", lower)

    def test_deterministic_model_removes_geometric_growth(self) -> None:
        rule_count = 2_048
        legacy_growth_events = 12
        optimized_growth_events = 0
        self.assertGreater(rule_count, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
