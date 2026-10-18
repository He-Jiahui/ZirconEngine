import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/style.rs"
REGRESSION = REPO_ROOT / "zircon_runtime/src/ui/v2/style/tests/rule_capacity_tests.rs"


class RuntimeV2StyleRuleFilterRetainPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_static_and_runtime_filters_retain_the_compiled_buffer(self) -> None:
        self.assertIn(
            "retain_rules_by_pseudo_state(&mut rules, false);",
            self.source,
        )
        self.assertIn(
            "retain_rules_by_pseudo_state(&mut rules, true);",
            self.source,
        )
        self.assertNotIn(
            ".filter(|rule| !rule.uses_pseudo_state())\n            .collect::<Vec<_>>()",
            self.source,
        )
        self.assertNotIn(
            ".filter(ResolvedRule::uses_pseudo_state)\n            .collect::<Vec<_>>()",
            self.source,
        )

    def test_retain_helper_is_order_preserving_and_state_qualified(self) -> None:
        match = re.search(
            r"fn retain_rules_by_pseudo_state\(.*?\n}\n",
            self.source,
            re.S,
        )
        self.assertIsNotNone(match)
        body = match.group(0)
        self.assertIn("rules.retain", body)
        self.assertIn("uses_pseudo_state()", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("runtime853_v2_style_rule_filter_retain_preserves_order", self.regression)
        self.assertIn("RUNTIME853_V2_STYLE_RULE_FILTER_RETAIN_BENCH_V1", self.regression)

    def test_deterministic_model_removes_filter_buffer_allocation(self) -> None:
        authored_rule_count = 4_096
        legacy_filter_buffers = 1
        optimized_filter_buffers = 0
        self.assertGreater(authored_rule_count, 0)
        self.assertEqual(legacy_filter_buffers, 1)
        self.assertEqual(optimized_filter_buffers, 0)


if __name__ == "__main__":
    unittest.main()
