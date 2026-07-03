import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_extension_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/"
    "test_plugin_validate_feature_provider_extension_schema.py"
)


class PluginValidateFeatureProviderExtensionSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_extension_schema_stays_in_schema_leaf(self):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_SCHEMA.read_text(
            encoding="utf-8"
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "def validate_plugin_feature_provider_extension_schema(",
            schema_text,
        )
        for root_helper in (
            "validate_plugin_feature_extension_id",
            "validate_plugin_feature_extension_owner_package_token",
        ):
            self.assertIn(root_helper, schema_text)
            self.assertNotIn(root_helper, parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_extension_schema import",
            extension_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_extension_schema import",
            parent_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_extension_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(extension_text.splitlines()), 130)
        self.assertLessEqual(len(schema_text.splitlines()), 70)


if __name__ == "__main__":
    unittest.main()
