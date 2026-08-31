from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/net_features/manifest.rs"
)


class ExactNetFeatureIdentifiersPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_identifier_join_preallocates_borrowed_parts(self) -> None:
        self.assertIn("parts.iter().map(|part| part.len()).sum()", self.source)
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn("joined.push_str(part)", self.source)
        self.assertNotIn("format!(", self.production)

    def test_runtime_id_is_built_before_feature_ownership_transfer(self) -> None:
        runtime_line = self.source.index("let runtime_module_id =")
        manifest_line = self.source.index("PluginFeatureBundleManifest::new(feature_id,")
        self.assertLess(runtime_line, manifest_line)
        self.assertIn(
            'join_string_parts(&[&feature_id, ".runtime"])',
            self.source,
        )
        self.assertIn("PluginModuleManifest::runtime(runtime_module_id,", self.source)
        self.assertNotIn("feature_id.clone()", self.production)

    def test_rust_guard_preserves_feature_and_runtime_ids(self) -> None:
        self.assertIn(
            "exact_net_identifier_join_preserves_feature_and_runtime_ids",
            self.source,
        )
        self.assertIn('"net.reliable_udp"', self.source)
        self.assertIn('"net.reliable_udp.runtime"', self.source)


if __name__ == "__main__":
    unittest.main()
