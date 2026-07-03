import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_extension_metadata.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_extension_metadata.py"
)


class PluginValidateFeatureProviderExtensionMetadataOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_extension_metadata_lives_in_metadata_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA.exists(),
            "feature-provider extension metadata projection belongs in its own owner",
        )
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        metadata_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA.read_text(
            encoding="utf-8"
        )
        test_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST.read_text(encoding="utf-8")
            + PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_METADATA_TEST.read_text(
                encoding="utf-8"
            )
        )

        for symbol in (
            "validate_plugin_feature_provider_extension_metadata",
            "validate_plugin_feature_provider_extension_display_name",
            "validate_plugin_feature_provider_enabled_by_default",
            "validate_plugin_feature_provider_extension_default_packaging",
            "feature_extensions[0].display_name must match",
            "owner optional feature display_name",
            "must match owner optional feature enabled_by_default",
            "feature_extensions[0].default_packaging must match",
            "generated distribution.default_packaging",
        ):
            self.assertIn(symbol, metadata_text)
            self.assertNotIn(f"def {symbol}(", extension_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
        self.assertIn(
            "from .plugin_validate_feature_provider_extension_metadata import",
            extension_text,
        )
        self.assertNotIn(
            "from .plugin_validate_feature_provider_extension_metadata import",
            parent_text,
            "feature-provider parent should dispatch through the extension owner",
        )
        self.assertNotIn(
            "generated_enabled = generated_feature.get",
            extension_text,
            "extension metadata parsing should stay behind the metadata owner",
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_default_packaging_drift(",
            test_text,
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_display_name_drift(",
            test_text,
        )
        self.assertLessEqual(len(extension_text.splitlines()), 130)
        self.assertLessEqual(len(metadata_text.splitlines()), 120)


if __name__ == "__main__":
    unittest.main()
