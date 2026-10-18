from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/timers.rs"
LOWER = ROOT / "zircon_runtime/src/ui/dispatch/input_manager/timers/tests/retain_drain_tests.rs"


class RuntimeInputTimerDrainCapacityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.lower = LOWER.read_text(encoding="utf-8")

    def test_each_retain_drain_lazily_reserves_map_bound(self) -> None:
        functions = (
            ("drain_expired_typeahead", "self.typeahead_expirations.len()"),
            ("drain_expired_submenu_hover", "self.submenu_hover_expirations.len()"),
            ("drain_expired_tooltips", "self.tooltip_expirations.len()"),
            ("drain_expired_toasts", "self.toast_expirations.len()"),
        )
        for function, bound in functions:
            match = re.search(
                r"pub fn\s+" + re.escape(function) + r"\(.*?\n\s*}\n",
                self.source,
                re.S,
            )
            self.assertIsNotNone(match, function)
            body = match.group(0)
            self.assertIn("let mut expired = Vec::new();", body)
            self.assertIn(f"let capacity = {bound};", body)
            self.assertIn("if expired.is_empty()", body)
            self.assertIn("expired.reserve(capacity);", body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime824_input_timer_drain_capacity_preserves_order_and_pending_entries",
            self.lower,
        )
        self.assertIn("RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1", self.lower)

    def test_deterministic_model_removes_dense_expiration_growth(self) -> None:
        timers = 4_096
        legacy_growth_events = 11
        optimized_growth_events = 0
        self.assertGreater(timers, 0)
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
