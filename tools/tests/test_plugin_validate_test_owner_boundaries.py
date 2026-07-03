import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_ALL_TARGETS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_all_targets.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTERS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_asset_importers.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_CONTRACT_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_asset_importer_contract.py"
)
PLUGIN_VALIDATE_DIST_CRATE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_dist_crate.py"
)
PLUGIN_VALIDATE_OPTIONS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_options.py"
)
PLUGIN_VALIDATE_DEPENDENCIES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_dependencies.py"
)

RECENT_TEST_OWNER_BOUNDARY_METHODS = (
    "test_all_target_tests_live_in_all_target_test_owner",
    "test_asset_importer_schema_tests_live_in_asset_importer_test_owner",
    "test_asset_importer_contract_tests_live_in_asset_importer_contract_owner",
    "test_dist_crate_tests_live_in_dist_crate_test_owner",
    "test_option_tests_live_in_option_test_owner",
    "test_dependency_tests_live_in_dependency_test_owner",
)


class PluginValidateTestOwnerBoundaryTests(unittest.TestCase):
    def test_recent_test_owner_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in RECENT_TEST_OWNER_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_test_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            120,
            "general PluginValidate owner boundary tests should stay as a thin common-owner guard",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            360,
            "focused PluginValidate test-owner boundary file should stay narrow",
        )

    def test_all_target_tests_live_in_all_target_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ALL_TARGETS_TEST.exists(),
            "PluginValidate --all aggregate tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        all_target_test_text = PLUGIN_VALIDATE_ALL_TARGETS_TEST.read_text(
            encoding="utf-8"
        )
        moved_tests = (
            "test_plugin_validate_all_reports_malformed_root_distribution",
            "test_plugin_validate_all_reports_failed_target_diagnostics",
            "test_plugin_validate_all_rejects_duplicate_option_keys",
            "test_plugin_validate_all_rejects_duplicate_asset_importer_ids",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in test_plugin_validate_all_targets.py",
            )
            self.assertIn(
                f"def {test_name}(",
                all_target_test_text,
                f"{test_name} must stay covered by the focused --all test owner",
            )

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(all_target_test_text.splitlines()),
            380,
            "PluginValidate all-target test owner should stay focused",
        )

    def test_asset_importer_schema_tests_live_in_asset_importer_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTERS_TEST.exists(),
            "PluginValidate asset_importer schema tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        asset_importer_test_text = PLUGIN_VALIDATE_ASSET_IMPORTERS_TEST.read_text(
            encoding="utf-8"
        )
        moved_tests = (
            "test_plugin_validate_accepts_asset_importer_zui_suffix",
            "test_plugin_validate_rejects_asset_importers_with_retired_ui_suffixes",
            "test_plugin_validate_rejects_malformed_asset_importer_schema",
            "test_plugin_validate_rejects_malformed_asset_importer_ids",
            "test_plugin_validate_rejects_asset_importer_numeric_range_overflow",
            "test_plugin_validate_rejects_malformed_asset_importer_string_arrays",
            "test_plugin_validate_rejects_unknown_asset_importer_output_kinds",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in test_plugin_validate_asset_importers.py",
            )
            self.assertIn(
                f"def {test_name}(",
                asset_importer_test_text,
                f"{test_name} must stay covered by the focused asset_importer test owner",
            )

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(asset_importer_test_text.splitlines()),
            450,
            "PluginValidate asset_importer schema test owner should stay focused",
        )

    def test_asset_importer_contract_tests_live_in_asset_importer_contract_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_CONTRACT_TEST.exists(),
            "PluginValidate asset_importer contract tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        contract_test_text = PLUGIN_VALIDATE_ASSET_IMPORTER_CONTRACT_TEST.read_text(
            encoding="utf-8"
        )
        moved_tests = (
            "test_plugin_validate_rejects_malformed_source_extensions",
            "test_plugin_validate_rejects_duplicate_source_extensions",
            "test_plugin_validate_rejects_duplicate_full_suffixes",
            "test_plugin_validate_rejects_duplicate_asset_importer_metadata_arrays",
            "test_plugin_validate_rejects_empty_asset_importer_metadata_arrays",
            "test_plugin_validate_rejects_asset_importer_required_capability_namespace",
            "test_plugin_validate_rejects_asset_importer_undeclared_required_capability",
            "test_plugin_validate_accepts_asset_importer_optional_feature_capability",
            "test_plugin_validate_rejects_malformed_full_suffixes",
            "test_plugin_validate_rejects_asset_importer_plugin_id_mismatch",
            "test_plugin_validate_rejects_asset_importer_without_source_selector",
            "test_plugin_validate_rejects_empty_asset_importer_selector_arrays",
            "test_plugin_validate_rejects_empty_asset_importers_array",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in test_plugin_validate_asset_importer_contract.py",
            )
            self.assertIn(
                f"def {test_name}(",
                contract_test_text,
                f"{test_name} must stay covered by the focused asset_importer contract test owner",
            )

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(contract_test_text.splitlines()),
            760,
            "PluginValidate asset_importer contract test owner should stay focused",
        )

    def test_dist_crate_tests_live_in_dist_crate_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE_TEST.exists(),
            "PluginValidate dist crate preflight tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        dist_crate_test_text = PLUGIN_VALIDATE_DIST_CRATE_TEST.read_text(
            encoding="utf-8"
        )
        moved_tests = (
            "test_plugin_validate_accepts_dist_package_contract",
            "test_plugin_validate_reports_missing_dist_crate",
            "test_plugin_validate_reports_dist_crate_missing_dist_feature",
            "test_plugin_validate_reports_malformed_dist_feature_entry",
            "test_plugin_validate_reports_dist_crate_missing_sdk_native_dependency",
            "test_plugin_validate_reports_dist_feature_forbidden_zircon_runtime_feature_route",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in the dist crate test owner",
            )
            self.assertIn(f"def {test_name}(", dist_crate_test_text)

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(dist_crate_test_text.splitlines()),
            320,
            "PluginValidate dist crate test owner should stay focused",
        )

    def test_option_tests_live_in_option_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_OPTIONS_TEST.exists(),
            "PluginValidate option schema/gate tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        option_test_text = PLUGIN_VALIDATE_OPTIONS_TEST.read_text(encoding="utf-8")
        moved_tests = (
            "test_plugin_validate_rejects_option_undeclared_required_capability",
            "test_plugin_validate_accepts_option_optional_feature_required_capability",
            "test_plugin_validate_rejects_malformed_options_schema",
            "test_plugin_validate_rejects_malformed_option_default_values",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in test_plugin_validate_options.py",
            )
            self.assertIn(
                f"def {test_name}(",
                option_test_text,
                f"{test_name} must stay covered by the focused options test owner",
            )

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(option_test_text.splitlines()),
            340,
            "PluginValidate option test owner should stay focused",
        )

    def test_dependency_tests_live_in_dependency_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DEPENDENCIES_TEST.exists(),
            "PluginValidate dependency behavior tests need a focused test owner",
        )
        main_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        dependency_test_text = PLUGIN_VALIDATE_DEPENDENCIES_TEST.read_text(
            encoding="utf-8"
        )
        moved_tests = (
            "test_plugin_validate_rejects_malformed_dependencies",
            "test_plugin_validate_rejects_empty_dependencies_array",
            "test_plugin_validate_rejects_duplicate_dependency_rows",
            "test_plugin_validate_rejects_dependency_capability_not_declared_by_package",
            "test_plugin_validate_rejects_external_dependency_capability_namespace",
            "test_plugin_validate_rejects_optional_feature_dependency_capability_not_declared_by_package",
            "test_plugin_validate_rejects_optional_feature_external_dependency_capability_namespace",
            "test_plugin_validate_rejects_optional_feature_without_dependencies",
            "test_plugin_validate_rejects_malformed_optional_feature_dependencies",
            "test_plugin_validate_rejects_invalid_optional_feature_primary_dependency",
            "test_plugin_validate_rejects_duplicate_optional_feature_dependency_rows",
        )

        for test_name in moved_tests:
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in test_plugin_validate_dependencies.py",
            )
            self.assertIn(
                f"def {test_name}(",
                dependency_test_text,
                f"{test_name} must stay covered by the focused dependency test owner",
            )

        self.assertLessEqual(
            len(main_test_text.splitlines()),
            40,
            "PluginValidate root test should stay as a retired entry marker after all-target split",
        )
        self.assertLessEqual(
            len(dependency_test_text.splitlines()),
            640,
            "PluginValidate dependency test owner should stay focused",
        )
