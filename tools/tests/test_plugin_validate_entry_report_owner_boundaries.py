import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_TARGET_DISCOVERY = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_target_discovery.py"
)
PLUGIN_VALIDATE_REPORT = REPO_ROOT / "tools/zircon_export/plugin_validate_report.py"
PLUGIN_VALIDATE_ENGINE_VERSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_engine_version.py"
)

ENTRY_REPORT_BOUNDARY_METHODS = (
    "test_all_target_discovery_lives_in_target_discovery_owner",
    "test_report_rendering_lives_in_report_owner",
    "test_all_target_report_assembly_lives_in_report_owner",
    "test_engine_version_resolution_lives_in_engine_version_owner",
    "test_single_target_validation_lives_in_single_target_owner",
)


class PluginValidateEntryReportOwnerBoundaryTests(unittest.TestCase):
    def test_entry_report_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in ENTRY_REPORT_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_entry_report_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            3150,
            "general PluginValidate owner boundary tests should shrink after entry/report split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            220,
            "focused PluginValidate entry/report owner boundary file should stay narrow",
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

    def test_single_target_validation_lives_in_single_target_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_SINGLE_TARGET.exists(),
            "single plugin target validation orchestration belongs in plugin_validate_single_target.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")

        self.assertNotIn(
            "def plugin_validate_single_report(",
            validate_text,
            "plugin_validate.py should stay as CLI/run dispatch and must not own single-target validation orchestration",
        )
        self.assertIn(
            "def plugin_validate_single_report(",
            single_target_text,
        )
        for imported_owner in (
            "validate_plugin_distribution",
            "validate_plugin_distribution_modules",
            "validate_plugin_dist_crate_workspace_member",
            "validate_plugin_feature_provider_package_projection",
        ):
            self.assertNotIn(
                imported_owner,
                validate_text,
                f"{imported_owner} belongs behind plugin_validate_single_target.py",
            )
            self.assertIn(imported_owner, single_target_text)
        self.assertIn(
            "from .plugin_validate_single_target import plugin_validate_single_report",
            validate_text,
            "plugin validate entry owner should dispatch single-target validation through the single-target owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                single_target_text,
                "single-target validation owner must not borrow build or entry orchestration owners",
            )
        self.assertLessEqual(
            len(validate_text.splitlines()),
            170,
            "plugin validate entry owner should shrink after single-target owner split",
        )
        self.assertLessEqual(
            len(single_target_text.splitlines()),
            150,
            "single-target validation owner should stay focused on one-target orchestration",
        )


if __name__ == "__main__":
    unittest.main()
