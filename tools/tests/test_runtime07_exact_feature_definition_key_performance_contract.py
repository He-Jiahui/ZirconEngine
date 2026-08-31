from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/feature_definitions/key.rs"
)


class ExactFeatureDefinitionKeyPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_key_preallocates_both_identities_and_separator(self) -> None:
        self.assertIn(
            "feature_id.len() + 1 + provider_package_id.len()",
            self.source,
        )
        self.assertIn("String::with_capacity(capacity)", self.source)

    def test_key_appends_borrowed_parts_without_formatter_growth(self) -> None:
        self.assertIn("key.push_str(feature_id)", self.source)
        self.assertIn("key.push('@')", self.source)
        self.assertIn("key.push_str(provider_package_id)", self.source)
        self.assertNotIn(
            'format!("{feature_id}@{provider_package_id}")',
            self.production,
        )

    def test_rust_guard_preserves_exact_and_empty_part_keys(self) -> None:
        self.assertIn(
            "exact_feature_definition_key_preserves_both_identities",
            self.source,
        )
        self.assertIn('"sound.timeline@sound_core"', self.source)
        self.assertIn('feature_definition_key("", "")', self.source)
        self.assertIn('"@"', self.source)


if __name__ == "__main__":
    unittest.main()
