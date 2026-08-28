import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_payload.py"
)
NATIVE_DYNAMIC_PAYLOAD_PLATFORM_BUNDLE = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_platform_bundle.py"
)
NATIVE_DYNAMIC_STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)
NATIVE_DYNAMIC_PAYLOAD_PACKAGE_REPORT = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_package_report.py"
)
NATIVE_DYNAMIC_PAYLOAD_PACKAGE_PATH = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_payload_package_path.py"
)
NATIVE_DYNAMIC_STAGE_PACKAGE_REPORT = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_stage_package_report.py"
)


class ZirconExportNativeDynamicPayloadOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_package_report_diagnostics_live_in_payload_package_report_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_PAYLOAD_PACKAGE_REPORT.exists(),
            "PlatformBundle NativeDynamic payload package-report diagnostics need a dedicated owner",
        )
        payload_text = NATIVE_DYNAMIC_PAYLOAD.read_text(encoding="utf-8")
        handoff_text = NATIVE_DYNAMIC_PAYLOAD_PLATFORM_BUNDLE.read_text(
            encoding="utf-8"
        )
        stage_payload_text = NATIVE_DYNAMIC_STAGE_PAYLOAD.read_text(encoding="utf-8")
        package_report_text = NATIVE_DYNAMIC_PAYLOAD_PACKAGE_REPORT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "platform_bundle_native_plugins_package_report_content_diagnostics",
            "platform_bundle_native_plugins_package_report_abi_diagnostics",
            "platform_bundle_native_plugins_package_report_payload_diagnostics",
            "is_non_empty_safe_relative_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                payload_text,
                f"{function_name} belongs in the payload package-report owner",
            )
            self.assertNotIn(
                f"def {function_name}(",
                handoff_text,
                f"{function_name} belongs in the payload package-report owner",
            )
            self.assertIn(
                f"def {function_name}(",
                package_report_text,
            )

        import_statement = (
            "from .pipeline_report_native_dynamic_payload_package_report import"
        )
        for consumer_path in (
            NATIVE_DYNAMIC_PAYLOAD_PACKAGE_PATH,
            NATIVE_DYNAMIC_STAGE_PACKAGE_REPORT,
        ):
            self.assertIn(
                import_statement,
                consumer_path.read_text(encoding="utf-8"),
                f"{consumer_path.name} should consume the package-report owner",
            )
        self.assertNotIn(
            import_statement,
            handoff_text,
            "PlatformBundle payload orchestration should delegate package-report checks",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_payload import",
            package_report_text,
            "package-report diagnostics must not import the payload owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            package_report_text,
            "package-report diagnostics must not import the stage payload owner",
        )

    def test_native_dynamic_payload_owner_stays_under_large_file_threshold(self):
        line_count = len(NATIVE_DYNAMIC_PAYLOAD.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            750,
            "NativeDynamic payload owner should stay below 750 lines after package-report split",
        )


if __name__ == "__main__":
    unittest.main()
