import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE_REPORT = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle.py"
)
PLATFORM_BUNDLE_FILE_EVIDENCE = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_file_evidence.py"
)
PLATFORM_BUNDLE_STAGE_HANDOFF = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_platform_bundle_stage_handoff.py"
)
PIPELINE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report.py"


class ZirconExportPlatformBundleReportOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_file_evidence_lives_in_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_FILE_EVIDENCE.exists(),
            "PlatformBundle report file/path evidence needs a dedicated owner",
        )
        report_text = PLATFORM_BUNDLE_REPORT.read_text(encoding="utf-8")
        evidence_text = PLATFORM_BUNDLE_FILE_EVIDENCE.read_text(encoding="utf-8")

        for function_name in (
            "resolve_user_path",
            "resolve_user_path_or_diagnostic",
            "platform_bundle_manifest_path_diagnostics",
            "platform_bundle_report_bundle_path",
            "platform_bundle_expected_bundle_path",
            "platform_bundle_payload_path_diagnostics",
            "platform_bundle_template_file_path_diagnostics",
            "path_relative_to_diagnostics",
            "path_is_relative_to",
            "load_platform_bundle_manifest",
            "platform_bundle_manifest_field_diagnostics",
            "platform_bundle_manifest_values_match",
            "platform_bundle_output_file_diagnostics",
            "platform_bundle_file_sha256",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the PlatformBundle file evidence owner",
            )
            self.assertIn(f"def {function_name}(", evidence_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_file_evidence import",
            report_text,
            "PlatformBundle report orchestration should consume the file evidence owner",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle import",
            evidence_text,
            "file evidence owner must not import report orchestration",
        )

    def test_platform_bundle_stage_handoff_lives_in_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_STAGE_HANDOFF.exists(),
            "PlatformBundle final report stage handoff diagnostics need a dedicated owner",
        )
        report_text = PLATFORM_BUNDLE_REPORT.read_text(encoding="utf-8")
        handoff_text = PLATFORM_BUNDLE_STAGE_HANDOFF.read_text(encoding="utf-8")
        pipeline_report_text = PIPELINE_REPORT.read_text(encoding="utf-8")

        for function_name in (
            "delta_verification_diagnostics",
            "platform_bundle_host_diagnostics",
            "compile_host_stage_report_failed",
            "compile_host_report_host_path",
            "platform_bundle_pack_diagnostics",
            "pack_stage_report_failed",
            "pack_report_pack_path",
            "platform_bundle_delta_diagnostics",
            "native_dynamic_stage_report_failed",
            "pack_report_has_verified_delta",
            "pack_report_delta_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the PlatformBundle stage handoff owner",
            )
            self.assertIn(f"def {function_name}(", handoff_text)

        self.assertIn(
            "from .pipeline_report_platform_bundle_stage_handoff import",
            pipeline_report_text,
            "Final report aggregation should consume PlatformBundle stage handoff diagnostics directly",
        )
        self.assertIn(
            "from .pipeline_report_platform_bundle_stage_handoff import",
            report_text,
            "PlatformBundle manifest diagnostics should consume native stage failure helpers directly",
        )
        self.assertNotIn(
            "from .pipeline_report_platform_bundle import",
            handoff_text,
            "stage handoff owner must not import PlatformBundle manifest orchestration",
        )

    def test_platform_bundle_report_orchestration_stays_under_large_file_threshold(self):
        line_count = len(PLATFORM_BUNDLE_REPORT.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            650,
            "PlatformBundle report orchestration should stay below 650 lines after file evidence split",
        )

    def test_platform_bundle_stage_handoff_line_budgets(self):
        self.assertTrue(
            PLATFORM_BUNDLE_STAGE_HANDOFF.exists(),
            "PlatformBundle stage handoff owner should exist before line-budget checks",
        )
        report_line_count = len(
            PLATFORM_BUNDLE_REPORT.read_text(encoding="utf-8").splitlines()
        )
        handoff_line_count = len(
            PLATFORM_BUNDLE_STAGE_HANDOFF.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            report_line_count,
            260,
            "PlatformBundle manifest orchestration owner should stay below 260 lines",
        )
        self.assertLess(
            handoff_line_count,
            380,
            "PlatformBundle stage handoff owner should stay below 380 lines",
        )


if __name__ == "__main__":
    unittest.main()
