import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_distribution.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_distribution_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_distribution_schema.py"
)


class PluginValidateFeatureProviderDistributionSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_distribution_schema_stays_in_schema_leaf(self):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_SCHEMA.read_text(
            encoding="utf-8"
        )
        test_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_SCHEMA_TEST.read_text(
                encoding="utf-8"
            )
        )

        self.assertIn(
            "def validate_plugin_feature_provider_distribution_schema(",
            schema_text,
        )
        for root_helper in (
            "plugin_validate_distribution_packaging",
            "plugin_validate_distribution_scalars",
            "plugin_validate_descriptor_symbol",
            "plugin_validate_distribution_entries",
            "plugin_validate_distribution_assets",
        ):
            self.assertIn(root_helper, schema_text)
            self.assertNotIn(root_helper, parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_distribution_schema import",
            distribution_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_distribution_schema import",
            parent_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_distribution_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(distribution_text.splitlines()), 100)
        self.assertLessEqual(len(schema_text.splitlines()), 70)


if __name__ == "__main__":
    unittest.main()
