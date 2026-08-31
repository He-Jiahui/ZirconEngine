from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/extension_registry/register/event_registration.rs"
)


class ExactEventCatalogNamespacePerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.production = cls.source.split("#[cfg(test)]", maxsplit=1)[0]

    def test_namespace_preallocates_plugin_id_and_suffix(self) -> None:
        self.assertIn('plugin_id.len() + ".events".len()', self.source)
        self.assertIn("String::with_capacity(capacity)", self.source)
        self.assertIn('namespace.push_str(".events")', self.source)

    def test_namespace_path_does_not_use_formatter_growth(self) -> None:
        self.assertNotIn('format!("{plugin_id}.events")', self.production)
        self.assertIn("Some(namespace)", self.source)

    def test_rust_guard_preserves_namespace_and_invalid_empty_owner(self) -> None:
        self.assertIn(
            "exact_event_catalog_namespace_preserves_module_identity",
            self.source,
        )
        self.assertIn('Some("weather.events".to_string())', self.source)
        self.assertIn(
            'plugin_event_catalog_namespace_from_module(".runtime")',
            self.source,
        )
        self.assertIn("None", self.source)


if __name__ == "__main__":
    unittest.main()
