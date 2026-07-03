import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_FEATURE_PROVIDER = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_extension.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_CAPABILITIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_capabilities.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_dependencies.py"
)
PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_optional_feature_dependencies.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/"
    "test_plugin_validate_feature_provider_capability_dependency_schema.py"
)


class PluginValidateFeatureProviderCapabilityDependencySchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_feature_provider_capability_dependency_schema_stays_in_leaf_owners(
        self,
    ) -> None:
        parent_text = PLUGIN_VALIDATE_FEATURE_PROVIDER.read_text(encoding="utf-8")
        extension_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION.read_text(
            encoding="utf-8"
        )
        capabilities_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_CAPABILITIES.read_text(
            encoding="utf-8"
        )
        dependencies_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCIES.read_text(
            encoding="utf-8"
        )
        optional_dependencies_text = (
            PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCIES.read_text(
                encoding="utf-8"
            )
        )
        test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        self.assertIn("validate_plugin_capability_values", capabilities_text)
        self.assertIn(
            "validate_plugin_optional_feature_dependency_rows_at_label",
            dependencies_text,
        )
        self.assertIn(
            "def validate_plugin_optional_feature_dependency_rows_at_label(",
            optional_dependencies_text,
        )
        for root_helper in (
            "validate_plugin_capability_values",
            "validate_plugin_optional_feature_dependency_rows_at_label",
        ):
            self.assertNotIn(root_helper, parent_text)
        self.assertIn("owner_package_capabilities", extension_text)
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_capability_schema_drift",
            test_text,
        )
        self.assertIn(
            "test_plugin_validate_rejects_generated_feature_provider_dependency_schema_drift",
            test_text,
        )
        self.assertLessEqual(len(capabilities_text.splitlines()), 70)
        self.assertLessEqual(len(dependencies_text.splitlines()), 130)
        self.assertLessEqual(len(extension_text.splitlines()), 145)


if __name__ == "__main__":
    unittest.main()
