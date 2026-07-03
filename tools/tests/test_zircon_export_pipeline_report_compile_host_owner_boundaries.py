import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PIPELINE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report.py"
PIPELINE_REPORT_COMPILE_HOST = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_compile_host.py"
)


class ZirconExportPipelineReportCompileHostOwnerBoundaryTests(unittest.TestCase):
    def test_compile_host_report_diagnostics_live_in_compile_host_owner(self):
        self.assertTrue(
            PIPELINE_REPORT_COMPILE_HOST.exists(),
            "CompileHost final Report diagnostics need a dedicated owner",
        )
        report_text = PIPELINE_REPORT.read_text(encoding="utf-8")
        compile_host_text = PIPELINE_REPORT_COMPILE_HOST.read_text(encoding="utf-8")

        self.assertNotIn(
            "COMPILE_HOST_LINK_PLAN_FIELDS =",
            report_text,
            "CompileHost link-plan constants belong in the CompileHost owner",
        )
        self.assertIn("COMPILE_HOST_LINK_PLAN_FIELDS =", compile_host_text)

        for function_name in (
            "compile_host_link_plan_diagnostics",
            "compile_host_command_diagnostics",
            "compile_host_command_alias_match_diagnostics",
            "compile_host_command_option_match_diagnostics",
            "compile_host_command_target_dir_match_diagnostics",
            "command_target_dir_matches_out_root",
            "compile_host_command_features_match_diagnostics",
            "compile_host_command_release_flag_diagnostics",
            "compile_host_host_executable_diagnostics",
            "validate_library_embed_compile_host_plan",
            "compile_host_stage_link_plan",
            "stage_report_payload",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the CompileHost final Report owner",
            )
            self.assertIn(f"def {function_name}(", compile_host_text)

        self.assertIn(
            "from .pipeline_report_compile_host import",
            report_text,
            "final Report orchestration should consume the CompileHost owner",
        )
        self.assertNotIn(
            "from .pipeline_report import",
            compile_host_text,
            "CompileHost final Report owner must not import Report orchestration",
        )

    def test_pipeline_report_orchestration_stays_under_large_file_threshold(self):
        line_count = len(PIPELINE_REPORT.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            520,
            "pipeline_report.py should stay below 520 lines after owner split",
        )

    def test_pipeline_report_compile_host_owner_stays_leaf_sized(self):
        self.assertTrue(
            PIPELINE_REPORT_COMPILE_HOST.exists(),
            "CompileHost final Report owner should exist before size check",
        )
        line_count = len(
            PIPELINE_REPORT_COMPILE_HOST.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            430,
            "CompileHost final Report owner should stay below 430 lines",
        )


if __name__ == "__main__":
    unittest.main()
