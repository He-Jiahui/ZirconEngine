import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_PLAN_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py"
)
BUILD_PLAN_COMMAND_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_commands.py"
)
BUILD_PLAN_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema_helpers.py"
)
BUILD_PLAN_PACKAGE_DETAILS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_package_details.py"
)


class ZirconExportNativeDynamicBuildPlanOwnerBoundaryTests(unittest.TestCase):
    def test_build_plan_command_semantics_live_in_command_owner(self):
        self.assertTrue(
            BUILD_PLAN_COMMAND_SEMANTICS.exists(),
            "NativeDynamic build-plan command semantics need a dedicated owner",
        )
        schema_text = BUILD_PLAN_SCHEMA.read_text(encoding="utf-8")
        command_text = BUILD_PLAN_COMMAND_SEMANTICS.read_text(encoding="utf-8")
        helper_text = BUILD_PLAN_SCHEMA_HELPERS.read_text(encoding="utf-8")

        for function_name in (
            "native_dynamic_build_plan_package_command_semantics_diagnostics",
            "command_forbidden_flag_diagnostics",
            "command_option_string_value_match_diagnostics",
            "command_alias_string_value_match_diagnostics",
            "command_flag_presence_diagnostics",
            "command_option_absence_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the build-plan command owner",
            )
            self.assertIn(
                f"def {function_name}(",
                command_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_build_plan_commands import",
            helper_text,
            "build-plan package schema helpers should consume command semantics",
        )
        self.assertIn(
            "from .pipeline_report_native_dynamic_build_plan_schema_helpers import",
            schema_text,
            "build-plan schema should consume package schema helpers",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_build_plan_schema import",
            command_text,
            "command semantics must not import the build-plan schema owner",
        )

    def test_build_plan_package_details_live_in_package_details_owner(self):
        self.assertTrue(
            BUILD_PLAN_PACKAGE_DETAILS.exists(),
            "NativeDynamic build-plan package details need a dedicated owner",
        )
        schema_text = BUILD_PLAN_SCHEMA.read_text(encoding="utf-8")
        package_details_text = (
            BUILD_PLAN_PACKAGE_DETAILS.read_text(encoding="utf-8")
            if BUILD_PLAN_PACKAGE_DETAILS.exists()
            else ""
        )

        for function_name in (
            "native_dynamic_build_plan_package_header_diagnostics",
            "native_dynamic_build_plan_package_expected_artifact_diagnostics",
            "native_dynamic_normalized_path",
            "native_dynamic_build_plan_package_header_field_diagnostics",
            "native_dynamic_build_plan_header_value_is_comparable",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                schema_text,
                f"{function_name} belongs in the build-plan package details owner",
            )
            self.assertIn(
                f"def {function_name}(",
                package_details_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_build_plan_package_details import",
            schema_text,
            "build-plan schema should consume the package details owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_build_plan_schema import",
            package_details_text,
            "package details owner must not import the build-plan schema owner",
        )

    def test_build_plan_schema_owner_stays_under_large_file_threshold(self):
        line_count = len(BUILD_PLAN_SCHEMA.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            360,
            "NativeDynamic build-plan schema owner should stay below 360 lines",
        )
        self.assertTrue(
            BUILD_PLAN_PACKAGE_DETAILS.exists(),
            "NativeDynamic build-plan package details owner file is missing",
        )
        package_details_line_count = len(
            BUILD_PLAN_PACKAGE_DETAILS.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            package_details_line_count,
            180,
            "NativeDynamic build-plan package details owner should stay below 180 lines",
        )


if __name__ == "__main__":
    unittest.main()
