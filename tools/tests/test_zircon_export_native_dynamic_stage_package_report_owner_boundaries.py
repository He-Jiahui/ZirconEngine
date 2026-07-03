import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_PAYLOAD = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_native_dynamic_stage_payload.py"
)
STAGE_PACKAGE_REPORT = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_stage_package_report.py"
)


class ZirconExportNativeDynamicStagePackageReportOwnerBoundaryTests(
    unittest.TestCase
):
    def test_stage_package_report_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            STAGE_PACKAGE_REPORT.exists(),
            "NativeDynamic materialized package report diagnostics need a dedicated owner",
        )
        stage_text = STAGE_PAYLOAD.read_text(encoding="utf-8")
        package_report_text = STAGE_PACKAGE_REPORT.read_text(encoding="utf-8")

        for function_name in (
            "native_dynamic_package_report_diagnostics",
            "native_dynamic_source_manifest_id",
            "native_dynamic_trimmed_non_empty_string_is_schema_clean",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the stage package-report owner",
            )
            self.assertIn(f"def {function_name}(", package_report_text)

        self.assertIn(
            "from .pipeline_report_native_dynamic_stage_package_report import",
            stage_text,
            "stage payload owner should consume the stage package-report owner",
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_stage_payload import",
            package_report_text,
            "stage package-report owner must not import stage payload orchestration",
        )

    def test_stage_payload_owner_stays_under_large_file_threshold(self):
        line_count = len(STAGE_PAYLOAD.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "NativeDynamic stage payload owner should stay below 560 lines "
            "after package-report diagnostics split",
        )


if __name__ == "__main__":
    unittest.main()
