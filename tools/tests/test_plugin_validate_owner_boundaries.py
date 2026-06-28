import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_DIST_CRATE = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dist_crate.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_contract.py"
)
PLUGIN_VALIDATE_TARGET_DISCOVERY = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_target_discovery.py"
)
PLUGIN_VALIDATE_REPORT = REPO_ROOT / "tools/zircon_export/plugin_validate_report.py"
PLUGIN_VALIDATE_ENGINE_VERSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_engine_version.py"
)


class PluginValidateOwnerBoundaryTests(unittest.TestCase):
    def test_distribution_enum_helper_lives_in_common_owner(self):
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")

        self.assertNotIn(
            "def plugin_validate_allowed_string_values(",
            validate_text,
            "closed-set distribution helper belongs in plugin_validate_common.py",
        )
        self.assertIn(
            "def plugin_validate_allowed_string_values(",
            common_text,
        )

    def test_distribution_allowed_value_constants_live_in_common_owner(self):
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")

        for constant_name in (
            "PLUGIN_VALIDATE_DIST_PACKAGING",
            "PLUGIN_VALIDATE_DISTRIBUTION_FORMS",
            "PLUGIN_VALIDATE_DEFAULT_PACKAGING",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                validate_text,
                f"{constant_name} belongs in plugin_validate_common.py",
            )
            self.assertIn(
                f"{constant_name} =",
                common_text,
            )

    def test_dist_crate_cargo_preflight_lives_in_dist_crate_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE.exists(),
            "dist crate Cargo preflight belongs in plugin_validate_dist_crate.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")

        for function_name in (
            "validate_plugin_dist_crate_feature",
            "validate_plugin_dist_crate_sdk_dependency",
            "plugin_validate_find_dependency",
            "plugin_validate_dependency_tables",
            "plugin_validate_sdk_dependency_enables_abi_helpers",
            "plugin_validate_feature_enables_dependency",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_dist_crate.py",
            )
            self.assertIn(
                f"def {function_name}(",
                dist_crate_text,
            )

        for constant_name in (
            "PLUGIN_VALIDATE_SDK_DEPENDENCY",
            "PLUGIN_VALIDATE_SDK_ABI_FEATURES",
            "PLUGIN_VALIDATE_FORBIDDEN_DIST_FEATURE_DEPENDENCIES",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                validate_text,
                f"{constant_name} belongs in plugin_validate_dist_crate.py",
            )
            self.assertIn(
                f"{constant_name} =",
                dist_crate_text,
            )

    def test_dist_crate_workspace_member_resolution_lives_in_dist_crate_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE.exists(),
            "dist crate workspace-member resolution belongs in plugin_validate_dist_crate.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")

        for function_name in (
            "plugin_validate_dist_crate_manifest",
            "validate_plugin_dist_crate_workspace_member",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_dist_crate.py",
            )
            self.assertIn(
                f"def {function_name}(",
                dist_crate_text,
            )

        self.assertNotIn(
            "is not a cdylib workspace member",
            validate_text,
            "dist crate workspace-member diagnostics belong in plugin_validate_dist_crate.py",
        )
        self.assertIn(
            "is not a cdylib workspace member",
            dist_crate_text,
        )

    def test_distribution_contract_lives_in_distribution_contract_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.exists(),
            "distribution manifest contract belongs in plugin_validate_distribution_contract.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "validate_plugin_distribution",
            "plugin_validate_descriptor_symbol",
            "plugin_validate_engine_compat",
            "plugin_validate_engine_compat_matches",
            "plugin_validate_parse_engine_comparator",
            "plugin_validate_parse_engine_version",
            "plugin_validate_distribution_assets",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_distribution_contract.py",
            )
            self.assertIn(
                f"def {function_name}(",
                distribution_text,
            )

        self.assertNotIn(
            "PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 =",
            validate_text,
            "descriptor symbol contract belongs in plugin_validate_distribution_contract.py",
        )
        self.assertIn(
            "PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 =",
            distribution_text,
        )

    def test_all_target_discovery_lives_in_target_discovery_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_TARGET_DISCOVERY.exists(),
            "all-target discovery belongs in plugin_validate_target_discovery.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        discovery_text = PLUGIN_VALIDATE_TARGET_DISCOVERY.read_text(encoding="utf-8")

        for function_name in (
            "plugin_validate_discover_target_ids",
            "plugin_validate_append_target",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_target_discovery.py",
            )
            self.assertIn(
                f"def {function_name}(",
                discovery_text,
            )

    def test_report_rendering_lives_in_report_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_REPORT.exists(),
            "report assembly and rendering belongs in plugin_validate_report.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        report_text = PLUGIN_VALIDATE_REPORT.read_text(encoding="utf-8")

        for function_name in (
            "plugin_validate_report",
            "render_plugin_validate_report",
            "render_plugin_validate_all_report",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_report.py",
            )
            self.assertIn(
                f"def {function_name}(",
                report_text,
            )

    def test_all_target_report_assembly_lives_in_report_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_REPORT.exists(),
            "all-target report assembly belongs in plugin_validate_report.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        report_text = PLUGIN_VALIDATE_REPORT.read_text(encoding="utf-8")

        self.assertNotIn(
            "def plugin_validate_all_report(",
            validate_text,
            "all-target report assembly belongs in plugin_validate_report.py",
        )
        self.assertIn(
            "def plugin_validate_all_report(",
            report_text,
        )

        for field_name in (
            '"target_count":',
            '"failed_count":',
            '"items":',
        ):
            self.assertNotIn(
                field_name,
                validate_text,
                f"{field_name} assembly belongs in plugin_validate_report.py",
            )
            self.assertIn(
                field_name,
                report_text,
            )

    def test_engine_version_resolution_lives_in_engine_version_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ENGINE_VERSION.exists(),
            "engine version resolution belongs in plugin_validate_engine_version.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        engine_version_text = PLUGIN_VALIDATE_ENGINE_VERSION.read_text(
            encoding="utf-8"
        )

        self.assertNotIn(
            "PLUGIN_VALIDATE_ENGINE_VERSION_FIELD =",
            validate_text,
            "engine version field path belongs in plugin_validate_engine_version.py",
        )
        self.assertIn(
            "PLUGIN_VALIDATE_ENGINE_VERSION_FIELD =",
            engine_version_text,
        )
        self.assertNotIn(
            "def plugin_validate_engine_version(",
            validate_text,
            "engine version resolution belongs in plugin_validate_engine_version.py",
        )
        self.assertIn(
            "def plugin_validate_engine_version(",
            engine_version_text,
        )


if __name__ == "__main__":
    unittest.main()
