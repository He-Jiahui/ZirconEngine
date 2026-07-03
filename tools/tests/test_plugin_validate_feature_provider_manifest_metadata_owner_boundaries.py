import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_manifest_schema.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_DESCRIPTION = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_manifest_description.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_OWNER_METADATA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_manifest_owner_metadata.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_REQUIRED_METADATA = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_feature_provider_manifest_required_metadata.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_manifest_metadata.py"
)


class PluginValidateFeatureProviderManifestMetadataOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_manifest_display_name_projection_stays_in_schema_owner(
        self,
    ):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        manifest_schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "plugin_validate_feature_provider_manifest_display_name_projection",
            "manifest.display_name must equal",
            "feature_extensions[0].display_name + Provider",
        ):
            self.assertIn(symbol, manifest_schema_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
        self.assertIn(
            "plugin_validate_feature_provider_manifest_projection_consistency",
            manifest_schema_text,
        )
        self.assertIn(
            "from .plugin_validate_feature_provider_manifest_schema import",
            parent_text,
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_manifest_display_name_drift(",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(manifest_schema_text.splitlines()), 150)

    def test_feature_provider_manifest_description_projection_stays_in_description_owner(
        self,
    ):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        manifest_schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        description_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_DESCRIPTION.read_text(
            encoding="utf-8"
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "plugin_validate_feature_provider_manifest_description_projection",
            "manifest.description must equal",
            "Native dynamic provider for optional feature",
            "feature_extensions[0].id",
        ):
            self.assertIn(symbol, description_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
            self.assertNotIn(f"def {symbol}(", manifest_schema_text)
        self.assertIn(
            "plugin_validate_feature_provider_manifest_description_projection",
            manifest_schema_text,
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_manifest_description_drift(",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(manifest_schema_text.splitlines()), 150)
        self.assertLessEqual(len(description_text.splitlines()), 60)

    def test_feature_provider_manifest_owner_metadata_projection_stays_in_owner_metadata_leaf(
        self,
    ):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        manifest_schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        owner_metadata_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_OWNER_METADATA.read_text(
                encoding="utf-8"
            )
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_feature_provider_manifest_owner_metadata",
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_OWNER_STRING_FIELDS",
            '("version", "0.1.0")',
            '("sdk_api_version", "0.1.0")',
            '("category", "runtime")',
            '("maturity", "beta")',
            'f"must equal owner manifest.{field}"',
            "generated manifest.supported_platforms",
            "must match owner manifest.supported_platforms",
        ):
            self.assertIn(symbol, owner_metadata_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
            self.assertNotIn(f"def {symbol}(", manifest_schema_text)
        self.assertIn(
            "validate_plugin_feature_provider_manifest_owner_metadata",
            (
                REPO_ROOT
                / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
            ).read_text(encoding="utf-8"),
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_manifest_owner_metadata_drift(",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(manifest_schema_text.splitlines()), 150)
        self.assertLessEqual(len(owner_metadata_text.splitlines()), 90)

    def test_feature_provider_manifest_required_metadata_stays_in_required_metadata_leaf(
        self,
    ):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        manifest_schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        required_metadata_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_REQUIRED_METADATA.read_text(
                encoding="utf-8"
            )
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_REQUIRED_FIELDS",
            "validate_plugin_feature_provider_manifest_required_metadata",
            "is required",
            "supported_platforms",
            "default_packaging",
        ):
            self.assertIn(symbol, required_metadata_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
        self.assertIn(
            "validate_plugin_feature_provider_manifest_required_metadata",
            manifest_schema_text,
        )
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_manifest_missing_metadata(",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(manifest_schema_text.splitlines()), 150)
        self.assertLessEqual(len(required_metadata_text.splitlines()), 60)

    def test_feature_provider_manifest_supported_targets_projection_stays_in_schema_owner(
        self,
    ):
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        manifest_schema_text = (
            PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_SCHEMA.read_text(
                encoding="utf-8"
            )
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_METADATA_TEST.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "plugin_validate_feature_provider_manifest_supported_targets_projection",
            "manifest.supported_targets must match",
            "feature_extensions[0].modules[0].target_modes",
        ):
            self.assertIn(symbol, manifest_schema_text)
            self.assertNotIn(f"def {symbol}(", parent_text)
        self.assertIn(
            "def test_plugin_validate_rejects_generated_feature_provider_manifest_supported_targets_drift(",
            test_text,
        )
        self.assertLessEqual(len(parent_text.splitlines()), 90)
        self.assertLessEqual(len(manifest_schema_text.splitlines()), 150)


if __name__ == "__main__":
    unittest.main()
