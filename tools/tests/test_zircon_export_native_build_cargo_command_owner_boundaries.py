import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_BUILD = REPO_ROOT / "tools/zircon_export/native_build.py"
NATIVE_BUILD_COMMAND = REPO_ROOT / "tools/zircon_export/native_build_command.py"


class ZirconExportNativeBuildCargoCommandOwnerBoundaryTests(unittest.TestCase):
    def test_cargo_command_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_BUILD_COMMAND.exists(),
            "NativeBuild Cargo profile/features/command/artifact naming needs a dedicated owner",
        )
        native_build_text = NATIVE_BUILD.read_text(encoding="utf-8")
        command_text = NATIVE_BUILD_COMMAND.read_text(encoding="utf-8")

        for function_name in (
            "native_dynamic_cargo_profile",
            "native_dynamic_cargo_build_command",
            "native_dynamic_expected_loadable_artifact",
            "platform_dynamic_library_name",
            "normalized_native_dynamic_build_features",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                native_build_text,
                f"{function_name} belongs in the NativeBuild Cargo command owner",
            )
            self.assertIn(f"def {function_name}(", command_text)

        self.assertNotIn(
            "NATIVE_BUILD_DEFAULT_MODE =",
            native_build_text,
            "NativeBuild default Cargo mode belongs with Cargo profile resolution",
        )
        self.assertIn("NATIVE_BUILD_DEFAULT_MODE =", command_text)
        self.assertIn(
            "from .native_build_command import",
            native_build_text,
            "native build execution owner should consume the Cargo command owner",
        )
        self.assertNotIn(
            "from .native_build import",
            command_text,
            "Cargo command owner must not import native build execution owner",
        )

    def test_cargo_command_consumers_import_owner_directly(self):
        for relative_path in (
            "tools/zircon_export/pipeline_report_native_dynamic_build_plan_package_details.py",
            "tools/zircon_export/plugin_build_package.py",
        ):
            text = (REPO_ROOT / relative_path).read_text(encoding="utf-8")
            self.assertIn(
                "from .native_build_command import",
                text,
                f"{relative_path} should consume NativeBuild command helpers directly",
            )
            self.assertNotIn(
                "from .native_build import platform_dynamic_library_name",
                text,
                f"{relative_path} should not borrow command helpers through native_build.py",
            )
        build_plan_schema_text = (
            REPO_ROOT
            / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "from .pipeline_report_native_dynamic_build_plan_package_details import",
            build_plan_schema_text,
            "build-plan schema parent should consume package-detail owner directly",
        )
        self.assertNotIn(
            "from .native_build_command import",
            build_plan_schema_text,
            "build-plan schema parent should not bypass package-detail owner",
        )

    def test_native_build_execution_owner_stays_below_split_threshold(self):
        native_build_line_count = len(
            NATIVE_BUILD.read_text(encoding="utf-8").splitlines()
        )
        command_line_count = len(
            NATIVE_BUILD_COMMAND.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            native_build_line_count,
            380,
            "NativeBuild execution owner should stay below 380 lines after Cargo command split",
        )
        self.assertLess(
            command_line_count,
            160,
            "NativeBuild Cargo command owner should stay focused below 160 lines",
        )


if __name__ == "__main__":
    unittest.main()
