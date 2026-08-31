from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/sound_features/manifest.rs"
)


class ExactSoundFeatureIdentifiersPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_identifier_join_preallocates_the_exact_output_length(self) -> None:
        self.assertIn("fn join_string_parts(parts: &[&str]) -> String", self.source)
        self.assertIn(
            "parts.iter().map(|part| part.len()).sum()",
            self.source,
        )
        self.assertIn("String::with_capacity(capacity)", self.source)

    def test_all_sound_feature_identifiers_use_the_exact_join(self) -> None:
        self.assertGreaterEqual(self.production.count("join_string_parts("), 5)
        self.assertNotIn("format!", self.production)

    def test_rust_guard_preserves_identifier_parts(self) -> None:
        self.assertIn("exact_sound_identifier_join_preserves_parts", self.source)
        self.assertIn(
            'join_string_parts(&["sound.", "timeline_animation_track", ".runtime"])',
            self.source,
        )
        self.assertIn('"sound.timeline_animation_track.runtime"', self.source)


if __name__ == "__main__":
    unittest.main()
