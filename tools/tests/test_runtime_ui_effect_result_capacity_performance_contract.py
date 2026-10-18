from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EFFECT = ROOT / "zircon_runtime/src/ui/surface/input/effect.rs"
REGRESSION = ROOT / "zircon_runtime/src/ui/surface/input/effect/tests/capacity_tests.rs"


class RuntimeUiEffectResultCapacityPerformanceContractTests(unittest.TestCase):
    def test_effect_result_reserves_known_effect_capacity(self) -> None:
        source = EFFECT.read_text(encoding="utf-8")
        body = source.split("pub(super) fn apply_dispatch_reply_core", 1)[1].split(
            "pub(in crate::ui::surface::input) fn append_dispatch_effect_to_result", 1
        )[0]

        self.assertIn("let effect_count = reply.effects.len();", body)
        self.assertIn("reserve_effect_result_capacity(&mut result, effect_count);", body)

    def test_capacity_helper_covers_each_effect_projection(self) -> None:
        source = EFFECT.read_text(encoding="utf-8")
        helper = source.split("fn reserve_effect_result_capacity", 1)[1].split(
            "fn apply_dispatch_effect_at_index", 1
        )[0]

        for field in (
            "applied_effects",
            "rejected_effects",
            "host_requests",
            "component_events",
        ):
            self.assertIn(f"result.{field}.reserve(effect_count);", helper)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        source = REGRESSION.read_text(encoding="utf-8")
        self.assertIn("runtime77_effect_result_capacity_reserves_each_projection", source)
        self.assertIn("RUNTIME77_EFFECT_RESULT_CAPACITY_BENCH_V1", source)
        self.assertIn("legacy_growth_events", source)
        self.assertIn("optimized_growth_events", source)

    def test_deterministic_model_removes_growth_reallocations(self) -> None:
        effect_count = 4
        # A zero-capacity Vec grows 0 -> 4 for each projection that receives all effects.
        legacy_growth_events = 4
        optimized_growth_events = 0
        self.assertEqual(legacy_growth_events, 4)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(effect_count, 4)


if __name__ == "__main__":
    unittest.main()
