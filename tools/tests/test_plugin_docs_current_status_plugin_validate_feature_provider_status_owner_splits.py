import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_plugin_validate_support import (
    assert_required_phrases,
    current_doc_sections,
    plugin_validate_status_requirements,
)


class PluginDocsCurrentStatusPluginValidateFeatureProviderOwnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = current_doc_sections(Path(__file__).resolve().parents[2])

    def test_current_export_plan_reflects_feature_provider_projection_compare_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_projection_compare_owner_split",
                "plugin_validate_feature_provider_projection_compare.py",
                "PluginValidate feature-provider projection compare owner",
                "feature-provider distribution projection field comparison",
            ),
            "Current export/plugin docs do not reflect PluginValidate feature-provider projection compare owner split",
        )

    def test_current_plugin_docs_reflect_feature_provider_dependencies_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_dependencies_owner_split",
                "plugin_validate_feature_provider_dependencies.py",
                "PluginValidate feature-provider dependencies owner",
                "feature-provider dependency projection diagnostics",
            ),
            "Current plugin docs do not reflect feature-provider dependencies owner split",
        )

    def test_current_plugin_docs_reflect_projection_optional_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_projection_optional_owner_split",
                "plugin_validate_feature_provider_projection_optional.py",
                "projection optional owner",
                "optional projection diagnostics",
            ),
            "Current plugin docs do not reflect projection optional owner split",
        )

    def test_current_plugin_docs_reflect_feature_provider_capabilities_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_capabilities_owner_split",
                "plugin_validate_feature_provider_capabilities.py",
                "feature-provider capabilities owner",
                "capability projection diagnostics",
            ),
            "Current plugin docs do not reflect feature-provider capabilities owner split",
        )

    def test_current_plugin_docs_reflect_feature_provider_distribution_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_distribution_owner_split",
                "plugin_validate_feature_provider_distribution.py",
                "feature-provider distribution owner",
                "distribution projection diagnostics",
            ),
            "Current plugin docs do not reflect feature-provider distribution owner split",
        )

    def test_current_plugin_docs_reflect_feature_provider_extension_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_feature_provider_extension_owner_split",
                "plugin_validate_feature_provider_extension.py",
                "feature-provider extension owner",
                "feature extension projection diagnostics",
            ),
            "Current plugin docs do not reflect feature-provider extension owner split",
        )


if __name__ == "__main__":
    unittest.main()
