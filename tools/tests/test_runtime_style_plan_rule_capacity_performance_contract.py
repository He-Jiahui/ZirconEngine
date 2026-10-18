import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PRODUCTION = (
    ROOT
    / "zircon_runtime/src/ui/template/asset/compiler/style_apply.rs"
)
LOWER = (
    ROOT / "zircon_runtime/src/ui/template/asset/compiler/style_apply/tests/token_map_sharing_tests.rs"
)


class RuntimeStylePlanRuleCapacityContract(unittest.TestCase):
    def test_build_style_plan_reserves_total_authored_rule_bound(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        body = source.split("pub(super) fn apply_styles_to_tree", 1)[0]

        self.assertIn("stylesheet.rules.len()", body)
        self.assertIn("saturating_add", body)
        self.assertIn("Vec::with_capacity(rule_count)", body)
        self.assertNotIn("let mut rules = Vec::new()", body)

    def test_rule_order_and_token_sharing_paths_remain_present(self):
        source = PRODUCTION.read_text(encoding="utf-8")
        self.assertIn("tokens: Arc::clone(&tokens)", source)
        self.assertIn("order += 1", source)
        self.assertIn("UiSelector::parse(&rule.selector)?", source)

    def test_lower_capacity_regression_and_release_marker_are_wired(self):
        source = LOWER.read_text(encoding="utf-8")
        self.assertIn("style_plan_rule_capacity_is_bounded", source)
        self.assertIn("style_plan_empty_sheets_keep_zero_capacity", source)
        self.assertIn("RUNTIME845_STYLE_PLAN_RULE_CAPACITY_BENCH_V1", source)

    def test_deterministic_model_removes_geometric_growth(self):
        rule_count = 4_096

        def growth_events(initial_capacity):
            capacity = initial_capacity
            length = 0
            growths = 0
            for _ in range(rule_count):
                if length == capacity:
                    capacity = max(4, capacity * 2)
                    growths += 1
                length += 1
            return growths

        legacy = growth_events(0)
        optimized = growth_events(rule_count)
        self.assertGreater(legacy, optimized)
        self.assertEqual(optimized, 0)


if __name__ == "__main__":
    unittest.main()
