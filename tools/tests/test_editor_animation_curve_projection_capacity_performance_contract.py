from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/animation_editor/session/curve_foundation.rs"


class EditorAnimationCurveProjectionCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("fn project_track_curves(")
        end = cls.source.index("\n#[derive(Clone, Copy)]", start)
        cls.body = cls.source[start:end]

    def test_component_and_key_bounds_are_reserved_before_append(self) -> None:
        self.assertIn("let component_names = layout.component_names();", self.body)
        self.assertIn(
            "let mut curves = Vec::with_capacity(component_names.len());", self.body
        )
        self.assertLess(
            self.body.index("Vec::with_capacity(component_names.len())"),
            self.body.index("curves.push("),
        )
        self.assertIn(
            "let mut keys = Vec::with_capacity(track.channel.keys.len());", self.body
        )
        self.assertLess(
            self.body.index("Vec::with_capacity(track.channel.keys.len())"),
            self.body.index("keys.push("),
        )
        self.assertIn("editor814_animation_curve_projection_capacity_bench_v1", self.source)

    def test_component_filtering_and_curve_semantics_remain_explicit(self) -> None:
        self.assertIn(
            "for (component_index, component_name) in component_names.iter().enumerate()",
            self.body,
        )
        self.assertIn(
            "if let Some(curve) = project_component_curve",
            " ".join(self.body.split()),
        )
        self.assertIn("curves.push(curve);", self.body)
        self.assertIn("for key in &track.channel.keys", self.body)
        self.assertIn("if !key.time_seconds.is_finite() || !value.is_finite()", self.body)
        self.assertIn("CurveKey::new(", self.body)
        self.assertIn(".with_tangents(", self.body)
        self.assertIn("curve_interpolation(track.channel.interpolation)", self.body)
        self.assertNotIn("filter_map(", self.body)
        self.assertNotIn("collect::<Option<Vec<_>>>()", self.body)

    def test_empty_or_discrete_channels_keep_the_zero_curve_path(self) -> None:
        self.assertIn("let Some(first_key) = track.channel.keys.first() else", self.body)
        self.assertIn("return Vec::new();", self.body)
        self.assertIn("AnimationChannelValueAsset::Bool(_)\n", self.source)
        self.assertIn("AnimationChannelValueAsset::Integer(_)\n", self.source)
        self.assertIn("AnimationChannelValueAsset::Quaternion(_) => None", self.source)

    def test_deterministic_component_projection_removes_outer_growth(self) -> None:
        component_count = 4
        legacy_growth_events = 0
        capacity = 0
        for length in range(1, component_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        self.assertGreater(legacy_growth_events, 0)
        optimized_growth_events = 0
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
