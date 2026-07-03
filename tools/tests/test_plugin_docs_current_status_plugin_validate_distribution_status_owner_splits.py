import unittest
from pathlib import Path

from tools.tests.plugin_docs_current_status_plugin_validate_support import (
    assert_required_phrases,
    current_doc_sections,
    plugin_validate_status_requirements,
)


class PluginDocsCurrentStatusPluginValidateDistributionOwnerTests(unittest.TestCase):
    def setUp(self) -> None:
        self.sections = current_doc_sections(Path(__file__).resolve().parents[2])

    def test_current_plugin_docs_reflect_dist_crate_dependency_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_dist_crate_dependency_owner_split",
                "plugin_validate_dist_crate_dependency.py",
                "PluginValidate dist crate dependency owner",
                "dist crate SDK dependency/ABI helper diagnostics",
            ),
            "Current plugin docs do not reflect dist crate dependency owner split",
        )

    def test_current_plugin_docs_reflect_distribution_assets_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_assets_owner_split",
                "plugin_validate_distribution_assets.py",
                "distribution.assets glob validation owner",
                "distribution.assets glob diagnostics",
            ),
            "Current plugin docs do not reflect distribution.assets owner split",
        )

    def test_current_plugin_docs_reflect_distribution_engine_compat_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_engine_compat_owner_split",
                "plugin_validate_distribution_engine_compat.py",
                "distribution.engine_compat owner",
                "engine compatibility range diagnostics",
            ),
            "Current plugin docs do not reflect distribution.engine_compat owner split",
        )

    def test_current_plugin_docs_reflect_distribution_descriptor_symbol_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_descriptor_symbol_owner_split",
                "plugin_validate_distribution_descriptor_symbol.py",
                "distribution descriptor symbol owner",
                "descriptor symbol diagnostics",
            ),
            "Current plugin docs do not reflect distribution descriptor symbol owner split",
        )

    def test_current_plugin_docs_reflect_distribution_entries_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_entries_owner_split",
                "plugin_validate_distribution_entries.py",
                "distribution entries owner",
                "runtime/editor entry diagnostics",
            ),
            "Current plugin docs do not reflect distribution entries owner split",
        )

    def test_current_plugin_docs_reflect_distribution_packaging_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_packaging_owner_split",
                "plugin_validate_distribution_packaging.py",
                "distribution packaging owner",
                "forms/default_packaging diagnostics",
            ),
            "Current plugin docs do not reflect distribution packaging owner split",
        )

    def test_current_plugin_docs_reflect_distribution_scalars_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_scalars_owner_split",
                "plugin_validate_distribution_scalars.py",
                "distribution scalars owner",
                "dist_crate/abi_version diagnostics",
            ),
            "Current plugin docs do not reflect distribution scalars owner split",
        )

    def test_current_plugin_docs_reflect_distribution_module_target_modes_owner_split(self):
        assert_required_phrases(
            self,
            self.sections,
            plugin_validate_status_requirements(
                "plugins_13_m5_t1_plugin_validate_distribution_module_target_modes_owner_split",
                "plugin_validate_distribution_module_target_modes.py",
                "distribution module target_modes owner",
                "entry/target-mode diagnostics",
            ),
            "Current plugin docs do not reflect distribution module target_modes owner split",
        )


if __name__ == "__main__":
    unittest.main()
