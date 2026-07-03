import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_manifest_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_VALUES = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_manifest_metadata_values.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_VALUES_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/"
    "test_plugin_validate_feature_provider_manifest_metadata_value_schema.py"
)


class PluginValidateFeatureProviderManifestMetadataValuesOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_manifest_metadata_values_stays_in_schema_leaf(
        self,
    ) -> None:
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        schema_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
            encoding="utf-8"
        )
        values_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_VALUES.read_text(
                encoding="utf-8"
            )
        )
        test_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_VALUES_TEST.read_text(
                encoding="utf-8"
            )
        )

        self.assertIn(
            "def plugin_validate_feature_provider_manifest_metadata_values(",
            values_text,
        )
        for root_helper in (
            "validate_plugin_capability_values",
            "validate_plugin_default_packaging_values",
        ):
            self.assertIn(root_helper, values_text)
            self.assertNotIn(root_helper, parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_manifest_metadata_values import",
            schema_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_manifest_metadata_values import",
            parent_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_value_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(schema_text.splitlines()), 155)
        self.assertLessEqual(len(values_text.splitlines()), 70)


if __name__ == "__main__":
    unittest.main()
