import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_contract.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ASSETS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_assets.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_MODULES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_modules.py"
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_feature_provider.py"
)


class PluginValidateDistributionTestOwnerBoundaryTests(unittest.TestCase):
    def test_distribution_contract_tests_live_in_distribution_contract_test_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST.exists(),
            "distribution manifest contract tests belong in test_plugin_validate_distribution_contract.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        distribution_test_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_reports_missing_dist_form",
            "test_plugin_validate_reports_unknown_distribution_form",
            "test_plugin_validate_reports_unknown_default_packaging",
            "test_plugin_validate_reports_duplicate_distribution_packaging_values",
            "test_plugin_validate_reports_malformed_distribution_forms_entry",
            "test_plugin_validate_reports_invalid_engine_compat_version_shape",
            "test_plugin_validate_reports_engine_compat_range_excludes_current_engine",
            "test_plugin_validate_reports_descriptor_symbol_mismatch",
            "test_plugin_validate_reports_distribution_assets_not_array",
            "test_plugin_validate_reports_distribution_assets_untrimmed_entry",
            "test_plugin_validate_reports_distribution_assets_plugin_relative_glob",
            "test_plugin_validate_reports_distribution_assets_empty_glob",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the distribution contract test owner",
            )
            self.assertIn(
                f"def {test_name}(",
                distribution_test_text,
            )

    def test_distribution_asset_tests_live_in_distribution_assets_test_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ASSETS_TEST.exists(),
            "distribution asset file checks belong in test_plugin_validate_distribution_assets.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        distribution_contract_text = (
            PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST.read_text(encoding="utf-8")
        )
        distribution_assets_text = PLUGIN_VALIDATE_DISTRIBUTION_ASSETS_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_distribution_assets_rejects_malformed_zui_documents",
            "test_distribution_assets_rejects_zui_documents_without_known_kind",
            "test_distribution_assets_accepts_current_zui_document_kinds",
            "test_plugin_validate_reports_distribution_assets_zui_kind_drift",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                (
                    "test_plugin_validate_distribution_contract.py",
                    distribution_contract_text,
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the distribution assets test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", distribution_assets_text)

    def test_distribution_module_tests_live_in_distribution_modules_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_MODULES_TEST.exists(),
            "distribution module binding tests belong in test_plugin_validate_distribution_modules.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        feature_provider_test_text = PLUGIN_VALIDATE_FEATURE_PROVIDER_TEST.read_text(
            encoding="utf-8"
        )
        module_test_text = PLUGIN_VALIDATE_DISTRIBUTION_MODULES_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_reports_dist_crate_not_declared_by_root_module",
            "test_plugin_validate_reports_runtime_entry_without_runtime_target_mode",
            "test_plugin_validate_reports_editor_entry_without_editor_target_mode",
            "test_plugin_validate_reports_unknown_dist_module_target_mode",
            "test_plugin_validate_reports_dist_crate_not_declared_by_feature_module",
            "test_plugin_validate_reports_feature_runtime_entry_without_runtime_target_mode",
            "test_plugin_validate_reports_feature_unknown_dist_module_target_mode",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                (
                    "test_plugin_validate_feature_provider.py",
                    feature_provider_test_text,
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the distribution modules test owner, not {source_name}",
                )
            self.assertIn(
                f"def {test_name}(",
                module_test_text,
            )


if __name__ == "__main__":
    unittest.main()
