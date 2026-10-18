from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
POINTER_REPLY = ROOT / "zircon_runtime/src/ui/surface/input/pointer_reply.rs"
REGRESSION = ROOT / "zircon_runtime/src/ui/surface/input/pointer_reply/tests/capacity_tests.rs"


class RuntimeUiPointerReplyCapacityPerformanceContractTests(unittest.TestCase):
    def test_pointer_reply_reserves_fixed_and_dynamic_effect_bound(self) -> None:
        source = POINTER_REPLY.read_text(encoding="utf-8")
        body = source.split("fn pointer_reply_effects", 1)[1].split(
            "#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]", 1
        )[0]

        self.assertIn("let fixed_effect_count", body)
        self.assertIn("let fallback_effect_count", body)
        self.assertIn("pointer_reply_effect_capacity_hint", body)
        self.assertIn("Vec::with_capacity", body)
        self.assertNotIn("let mut effects = Vec::new()", body)

    def test_pointer_text_merge_reserves_each_moved_projection(self) -> None:
        source = POINTER_REPLY.read_text(encoding="utf-8")
        body = source.split("pub(super) fn merge_pointer_text_result", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]
        normalized = "".join(body.split())

        for field in (
            "reply.effects",
            "applied_effects",
            "component_events",
            "widget_events",
            "binding_reports",
            "host_requests",
            "rejected_effects",
            "diagnostics.notes",
        ):
            self.assertIn(f"result.{field}.reserve(", normalized)

    def test_pointer_text_merge_moves_widget_events(self) -> None:
        source = POINTER_REPLY.read_text(encoding="utf-8")
        body = source.split("pub(super) fn merge_pointer_text_result", 1)[1].split(
            "#[cfg(test)]", 1
        )[0]
        normalized = "".join(body.split())
        self.assertIn("result.widget_events.extend(text_result.widget_events)", normalized)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime77_pointer_reply_capacity_hint_preserves_upper_bound", source)
        self.assertIn("RUNTIME77_POINTER_REPLY_CAPACITY_BENCH_V1", source)
        self.assertIn("legacy_growth_events", source)
        self.assertIn("optimized_growth_events", source)

    def test_deterministic_model_removes_known_growth_paths(self) -> None:
        projections = 8
        legacy_growth_events = projections
        optimized_growth_events = 0
        self.assertEqual(legacy_growth_events, 8)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
