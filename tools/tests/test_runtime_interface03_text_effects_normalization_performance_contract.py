import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_effects.rs"
BENCH = ROOT / "zircon_runtime_interface/src/ui/surface/render/text_effects/tests/performance_tests.rs"


class RuntimeInterface03TextEffectsNormalizationPerformanceContract(unittest.TestCase):
    def test_normalization_prefilters_before_allocating_effect_colors(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        start = source.index("    pub fn normalized(&self) -> Self {")
        end = source.index("\n    pub fn requires_distance_field", start)
        body = source[start:end]

        self.assertIn("and_then(normalized_active_outline)", body)
        self.assertIn("and_then(normalized_active_shadow)", body)
        self.assertIn("and_then(normalized_active_glow)", body)
        self.assertNotIn(".map(UiTextOutlineEffect::normalized)", body)
        self.assertNotIn(".map(UiTextShadowEffect::normalized)", body)
        self.assertNotIn(".map(UiTextGlowEffect::normalized)", body)

        for helper in (
            "fn normalized_active_outline",
            "fn normalized_active_shadow",
            "fn normalized_active_glow",
        ):
            helper_start = source.index(helper)
            helper_end = source.index("\n}\n", helper_start) + 2
            helper_body = source[helper_start:helper_end]
            self.assertIn("color_has_visible_alpha", helper_body)
            self.assertIn("Some(value.normalized())", helper_body)

    def test_behavior_and_release_benchmark_are_present(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        bench = BENCH.read_text(encoding="utf-8")

        self.assertIn("mod performance_tests", source)
        self.assertIn("legacy_normalized", bench)
        self.assertIn(
            "runtime_interface03_batch55_61_prefiltered_text_effects_preserve_legacy_normalization",
            bench,
        )
        self.assertIn("RUNTIME_INTERFACE03_PREFILTER_TEXT_EFFECTS_BENCH_V1", bench)
        self.assertIn("saturating_mul(5)", bench)


if __name__ == "__main__":
    unittest.main()
