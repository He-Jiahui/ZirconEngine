import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
FEATURE_PROVIDER_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_provider.py"
)
CAPABILITY_DEPENDENCY_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_capability_dependency_schema.py"
)
DISTRIBUTION_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_distribution_schema.py"
)
EXTENSION_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_extension_schema.py"
)
EXTENSION_METADATA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_extension_metadata.py"
)
EXTENSION_METADATA_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_extension_metadata_schema.py"
)
MANIFEST_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_manifest_schema.py"
)
MODULE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_feature_provider_module_schema.py"
)


class PluginValidateFeatureProviderTestOwnerBoundaryTests(unittest.TestCase):
    def test_generated_projection_cases_live_in_focused_test_owners(self):
        main_text = FEATURE_PROVIDER_TEST.read_text(encoding="utf-8")

        moved_tests = (
            "test_plugin_validate_rejects_generated_feature_provider_dependency_fields",
            "test_plugin_validate_rejects_generated_feature_provider_distribution_fields",
            "test_plugin_validate_rejects_generated_feature_provider_extension_fields",
            "test_plugin_validate_rejects_generated_feature_provider_enabled_default_drift",
            "test_plugin_validate_rejects_generated_feature_provider_enabled_default_type",
            "test_plugin_validate_rejects_generated_feature_provider_default_packaging_drift",
            "test_plugin_validate_rejects_generated_feature_provider_manifest_fields",
            "test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_schema",
            "test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_projection_drift",
            "test_plugin_validate_rejects_generated_feature_provider_module_projection_drift",
        )
        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_text,
                f"{test_name} belongs in a focused feature-provider schema test owner",
            )

        expectations = {
            CAPABILITY_DEPENDENCY_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_dependency_fields",
            ),
            DISTRIBUTION_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_distribution_fields",
            ),
            EXTENSION_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_extension_fields",
            ),
            EXTENSION_METADATA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_enabled_default_drift",
                "test_plugin_validate_rejects_generated_feature_provider_default_packaging_drift",
            ),
            EXTENSION_METADATA_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_enabled_default_type",
            ),
            MANIFEST_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_manifest_fields",
                "test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_schema",
                "test_plugin_validate_rejects_generated_feature_provider_manifest_metadata_projection_drift",
            ),
            MODULE_SCHEMA_TEST: (
                "test_plugin_validate_rejects_generated_feature_provider_module_projection_drift",
            ),
        }
        for owner_file, test_names in expectations.items():
            self.assertTrue(owner_file.exists(), f"{owner_file} should exist")
            owner_text = owner_file.read_text(encoding="utf-8")
            for test_name in test_names:
                self.assertIn(
                    f"def {test_name}(",
                    owner_text,
                    f"{test_name} should live in {owner_file.name}",
                )

        self.assertLessEqual(
            len(main_text.splitlines()),
            380,
            "feature-provider entry behavior tests should stay below the split budget",
        )


if __name__ == "__main__":
    unittest.main()
