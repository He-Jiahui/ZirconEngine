import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_extension_metadata.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_extension_metadata_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/"
    "test_plugin_validate_feature_provider_extension_metadata_schema.py"
)


class PluginValidateFeatureProviderExtensionMetadataSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_extension_metadata_schema_stays_in_schema_leaf(
        self,
    ) -> None:
        metadata_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA.read_text(
                encoding="utf-8"
            )
        )
        schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        test_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_SCHEMA_TEST.read_text(
                encoding="utf-8"
            )
        )

        self.assertIn(
            "def validate_plugin_feature_provider_extension_metadata_schema(",
            schema_text,
        )
        self.assertIn("validate_plugin_default_packaging_values", schema_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_extension_metadata_schema import",
            metadata_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_default_packaging_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(metadata_text.splitlines()), 120)
        self.assertLessEqual(len(schema_text.splitlines()), 50)


if __name__ == "__main__":
    unittest.main()
