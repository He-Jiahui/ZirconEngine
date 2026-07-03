import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_STAGE = REPO_ROOT / "tools/zircon_export/native_dynamic.py"
NATIVE_DYNAMIC_MATERIALIZE = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_materialize.py"
)
NATIVE_DYNAMIC_MATERIALIZE_IO = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_materialize_io.py"
)
NATIVE_DYNAMIC_CLI_OPTIONS = (
    REPO_ROOT / "tools/zircon_export/native_dynamic_cli_options.py"
)
NATIVE_DYNAMIC_STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)
NATIVE_DYNAMIC_BUILD_EXECUTION = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_execution.py"
)
NATIVE_DYNAMIC_PACKAGE_EXPORTS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_package_exports.py"
)


class ZirconExportNativeDynamicOwnerBoundaryTests(unittest.TestCase):
    def test_native_dynamic_package_materialization_lives_in_materialize_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_MATERIALIZE.exists(),
            "NativeDynamic package materialization needs a dedicated owner",
        )
        stage_text = NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8")
        materialize_text = NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8")

        for function_name in (
            "materialize_native_dynamic_packages",
            "find_native_package_dir",
            "read_package_manifest_id",
            "copy_native_dynamic_package",
            "copy_native_artifacts",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the materialize owner",
            )
            self.assertIn(
                f"def {function_name}(",
                materialize_text,
            )

        self.assertNotIn(
            "class PackageManifestRead",
            stage_text,
            "Package manifest read state belongs in the materialize owner",
        )
        self.assertIn("class PackageManifestRead", materialize_text)
        self.assertIn(
            "from .native_dynamic_materialize import",
            stage_text,
            "NativeDynamic stage orchestration should consume the materialize owner",
        )
        self.assertNotIn(
            "from .native_dynamic import",
            materialize_text,
            "materialize helpers must not import the stage orchestration owner",
        )

    def test_native_dynamic_materialization_io_helpers_live_in_io_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_MATERIALIZE_IO.exists(),
            "NativeDynamic materialization IO/path helpers need a dedicated owner",
        )
        stage_text = NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8")
        materialize_text = NATIVE_DYNAMIC_MATERIALIZE.read_text(encoding="utf-8")
        materialize_io_text = NATIVE_DYNAMIC_MATERIALIZE_IO.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "reset_native_dynamic_plugins_dir",
            "remove_native_dynamic_dir",
            "list_native_dynamic_dir",
            "copy_native_dynamic_file",
            "copy_native_dynamic_tree",
            "resolve_stage_child",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the materialize IO/path owner",
            )
            self.assertNotIn(
                f"def {function_name}(",
                materialize_text,
                f"{function_name} belongs in the materialize IO/path owner",
            )
            self.assertIn(f"def {function_name}(", materialize_io_text)

        self.assertIn(
            "from .native_dynamic_materialize_io import",
            stage_text,
            "NativeDynamic stage orchestration should consume IO/path owner directly",
        )
        self.assertIn(
            "from .native_dynamic_materialize_io import",
            materialize_text,
            "NativeDynamic materialization should consume IO/path owner directly",
        )
        self.assertNotIn(
            "from .native_dynamic_materialize import",
            materialize_io_text,
            "IO/path owner must not import package materialization",
        )
        self.assertNotIn(
            "from .native_dynamic import",
            materialize_io_text,
            "IO/path owner must not import stage orchestration",
        )

    def test_native_dynamic_stage_owner_stays_under_large_file_threshold(self):
        line_count = len(NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            700,
            "NativeDynamic stage orchestration should stay below the split threshold",
        )

    def test_native_dynamic_cli_options_live_in_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_CLI_OPTIONS.exists(),
            "NativeDynamic CLI option normalization needs a dedicated owner",
        )
        stage_text = NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8")
        cli_options_text = NATIVE_DYNAMIC_CLI_OPTIONS.read_text(encoding="utf-8")

        for function_name in (
            "native_dynamic_cli_optional_trimmed_string",
            "native_dynamic_cli_string_array",
            "native_dynamic_signing_profile",
            "native_dynamic_signing_platforms",
            "default_repo_root",
            "resolve_user_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the NativeDynamic CLI options owner",
            )
            self.assertIn(f"def {function_name}(", cli_options_text)

        self.assertIn(
            "from .native_dynamic_cli_options import",
            stage_text,
            "NativeDynamic stage orchestration should consume the CLI options owner",
        )
        self.assertNotIn(
            "from .native_dynamic import",
            cli_options_text,
            "CLI options owner must not import stage orchestration",
        )

    def test_native_dynamic_stage_runner_stays_focused_after_cli_option_split(self):
        stage_line_count = len(
            NATIVE_DYNAMIC_STAGE.read_text(encoding="utf-8").splitlines()
        )
        cli_options_line_count = len(
            NATIVE_DYNAMIC_CLI_OPTIONS.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            stage_line_count,
            390,
            "NativeDynamic stage orchestration should stay below 390 lines after CLI option split",
        )
        self.assertLess(
            cli_options_line_count,
            90,
            "NativeDynamic CLI options owner should stay below 90 lines",
        )

    def test_native_dynamic_build_execution_diagnostics_live_in_build_execution_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_BUILD_EXECUTION.exists(),
            "NativeDynamic build execution report diagnostics belong in their own owner",
        )
        stage_payload_text = NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8")
        build_execution_text = NATIVE_DYNAMIC_BUILD_EXECUTION.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "native_dynamic_build_execution_plan_diagnostics",
            "native_dynamic_build_execution_artifact_diagnostics",
            "native_dynamic_build_execution_plan_field_diagnostics",
            "native_dynamic_copied_artifact_bundle_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_payload_text,
                f"{function_name} belongs in the build execution owner",
            )
            self.assertIn(
                f"def {function_name}(",
                build_execution_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_build_execution import",
            stage_payload_text,
            "stage payload orchestration should consume the build execution owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            build_execution_text,
            "build execution diagnostics must not import the stage payload owner",
        )

    def test_native_dynamic_stage_payload_owner_stays_under_large_file_threshold(self):
        line_count = len(
            NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            1000,
            "NativeDynamic stage payload owner should stay below the large-file threshold",
        )

    def test_native_dynamic_package_exports_live_in_package_exports_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_PACKAGE_EXPORTS.exists(),
            "NativeDynamic package export projection needs a dedicated owner",
        )
        stage_payload_text = NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8")
        package_exports_text = NATIVE_DYNAMIC_PACKAGE_EXPORTS.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "validate_native_dynamic_package_export_ids",
            "validate_native_dynamic_package_exports",
            "schema_clean_native_dynamic_package_exports",
            "normalized_native_dynamic_package_exports",
            "native_dynamic_package_export_materialization_diagnostics",
            "materialized_package_exports_by_id",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_payload_text,
                f"{function_name} belongs in the package exports owner",
            )
            self.assertIn(
                f"def {function_name}(",
                package_exports_text,
            )

        self.assertIn(
            "from .pipeline_report_native_dynamic_package_exports import",
            stage_payload_text,
            "stage payload orchestration should consume the package exports owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            package_exports_text,
            "package exports diagnostics must not import the stage payload owner",
        )


if __name__ == "__main__":
    unittest.main()
