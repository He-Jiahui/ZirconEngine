import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_TEMPLATE_REPORT = REPO_ROOT / "tools/zircon_export/pipeline_report_source_template.py"
SOURCE_TEMPLATE_BUILD_HANDOFF = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_source_template_build_handoff.py"
)
SOURCE_TEMPLATE_BUILD_STATUS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_source_template_build_status.py"
)
SOURCE_TEMPLATE_BUILD_VALIDATION_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_source_template_build_validation.py"
)
SOURCE_TEMPLATE_BUILD_VALIDATION_STATUS_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_source_template_build_validation_status.py"
)


class ZirconExportSourceTemplateBuildHandoffOwnerBoundaryTests(unittest.TestCase):
    def test_build_handoff_diagnostics_live_in_build_handoff_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_BUILD_HANDOFF.exists(),
            "SourceTemplate build handoff diagnostics need a dedicated owner",
        )
        report_text = SOURCE_TEMPLATE_REPORT.read_text(encoding="utf-8")
        build_handoff_text = SOURCE_TEMPLATE_BUILD_HANDOFF.read_text(encoding="utf-8")

        for function_name in (
            "source_template_validate_build_plan_diagnostics",
            "source_template_validate_build_plan_target_dir_diagnostics",
            "source_template_build_validation_diagnostics",
            "source_template_command_manifest_path_diagnostics",
            "source_template_report_target_dir_diagnostics",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                report_text,
                f"{function_name} belongs in the SourceTemplate build handoff owner",
            )
            self.assertIn(f"def {function_name}(", build_handoff_text)

        self.assertIn(
            "from .pipeline_report_source_template_build_handoff import",
            report_text,
            "SourceTemplate report owner should consume the build handoff owner",
        )
        self.assertNotIn(
            "from .pipeline_report_source_template import",
            build_handoff_text,
            "SourceTemplate build handoff owner must not import report orchestration",
        )

    def test_source_template_report_owner_stays_under_build_handoff_threshold(self):
        line_count = len(SOURCE_TEMPLATE_REPORT.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            360,
            "SourceTemplate report owner should stay below 360 lines after build handoff split",
        )

    def test_source_template_build_handoff_owner_stays_leaf_sized(self):
        self.assertTrue(
            SOURCE_TEMPLATE_BUILD_HANDOFF.exists(),
            "SourceTemplate build handoff owner should exist before size check",
        )
        line_count = len(
            SOURCE_TEMPLATE_BUILD_HANDOFF.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            420,
            "SourceTemplate build handoff owner should stay below 420 lines",
        )

    def test_source_template_build_status_diagnostics_live_in_status_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_BUILD_STATUS.exists(),
            "SourceTemplate build_validation status diagnostics need a dedicated owner",
        )
        build_handoff_text = SOURCE_TEMPLATE_BUILD_HANDOFF.read_text(encoding="utf-8")
        build_status_text = SOURCE_TEMPLATE_BUILD_STATUS.read_text(encoding="utf-8")

        self.assertIn(
            "from .pipeline_report_source_template_build_status import",
            build_handoff_text,
            "SourceTemplate build handoff owner should consume the status owner",
        )
        self.assertNotIn(
            "from .pipeline_report_source_template_build_handoff import",
            build_status_text,
            "SourceTemplate build status owner must not import handoff orchestration",
        )
        self.assertNotIn(
            "def source_template_build_status_diagnostics(",
            build_handoff_text,
            "SourceTemplate build status diagnostics belong in the status owner",
        )
        self.assertIn(
            "def source_template_build_status_diagnostics(",
            build_status_text,
        )

    def test_source_template_build_status_owner_stays_leaf_sized(self):
        self.assertTrue(
            SOURCE_TEMPLATE_BUILD_STATUS.exists(),
            "SourceTemplate build status owner should exist before size check",
        )
        line_count = len(
            SOURCE_TEMPLATE_BUILD_STATUS.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            line_count,
            120,
            "SourceTemplate build status owner should stay below 120 lines",
        )

    def test_source_template_build_validation_status_tests_live_in_status_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_BUILD_VALIDATION_STATUS_TEST.exists(),
            "SourceTemplate build_validation status semantics need a dedicated test owner",
        )
        main_test_text = SOURCE_TEMPLATE_BUILD_VALIDATION_TEST.read_text(
            encoding="utf-8"
        )
        status_test_text = SOURCE_TEMPLATE_BUILD_VALIDATION_STATUS_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_report_rejects_failed_source_template_build_validation",
            "test_report_rejects_requested_source_template_build_validation_skip",
            "test_report_rejects_unrequested_source_template_build_validation_skip",
            "test_report_rejects_skipped_source_template_build_validation_exit_code",
            "test_report_rejects_unrequested_source_template_build_validation_execution",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                main_test_text,
                f"{test_name} belongs in the SourceTemplate build_validation status test owner",
            )
            self.assertIn(f"def {test_name}(", status_test_text)

    def test_source_template_build_validation_test_owners_stay_under_line_budgets(
        self,
    ):
        budgets = {
            SOURCE_TEMPLATE_BUILD_VALIDATION_TEST: 850,
            SOURCE_TEMPLATE_BUILD_VALIDATION_STATUS_TEST: 320,
        }
        failures: list[str] = []
        for path, budget in budgets.items():
            line_count = len(path.read_text(encoding="utf-8").splitlines())
            if line_count > budget:
                failures.append(
                    f"{path.relative_to(REPO_ROOT).as_posix()} has {line_count} lines; budget {budget}"
                )
        if failures:
            self.fail(
                "SourceTemplate build_validation test owners exceeded line budgets:\n"
                + "\n".join(failures)
            )


if __name__ == "__main__":
    unittest.main()
