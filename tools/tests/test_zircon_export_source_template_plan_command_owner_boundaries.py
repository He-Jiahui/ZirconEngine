import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_TEMPLATE_STAGE = REPO_ROOT / "tools/zircon_export/source_template.py"
SOURCE_TEMPLATE_PLAN_COMMAND = (
    REPO_ROOT / "tools/zircon_export/source_template_plan_command.py"
)
COMPILE_HOST_SOURCE_TEMPLATE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_compile_host_source_template.py"
)


class ZirconExportSourceTemplatePlanCommandOwnerBoundaryTests(unittest.TestCase):
    def test_plan_command_diagnostics_live_in_dedicated_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_PLAN_COMMAND.exists(),
            "SourceTemplate plan/command diagnostics need a dedicated owner",
        )
        stage_text = SOURCE_TEMPLATE_STAGE.read_text(encoding="utf-8")
        plan_command_text = SOURCE_TEMPLATE_PLAN_COMMAND.read_text(encoding="utf-8")
        compile_host_test_text = COMPILE_HOST_SOURCE_TEMPLATE_TEST.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "source_template_command",
            "load_validate_report",
            "validate_report_requires_strategy",
            "source_template_plan",
            "source_template_command_array_is_valid",
            "source_template_command_array_has_entry_type_errors",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the SourceTemplate plan/command owner",
            )
            self.assertIn(f"def {function_name}(", plan_command_text)

        for marker in (
            "SourceTemplate build plan command must be a non-empty string array",
            "SourceTemplate build plan manifest_path must be a non-empty string",
            "SourceTemplate build plan target_dir must be a non-empty string",
        ):
            self.assertNotIn(
                marker,
                stage_text,
                f"{marker} diagnostics belong in the SourceTemplate plan/command owner",
            )
            self.assertIn(marker, plan_command_text)

        self.assertIn(
            "from .source_template_plan_command import",
            stage_text,
            "SourceTemplate stage runner should consume the plan/command owner",
        )
        self.assertIn(
            "from tools.zircon_export.source_template_plan_command import source_template_command",
            compile_host_test_text,
            "SourceTemplate command unit tests should consume the plan/command owner directly",
        )
        self.assertNotIn(
            "from .source_template import",
            plan_command_text,
            "SourceTemplate plan/command owner must not import stage orchestration",
        )

    def test_source_template_stage_and_plan_command_owners_stay_small(self):
        stage_lines = len(SOURCE_TEMPLATE_STAGE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            stage_lines,
            300,
            "SourceTemplate stage runner should stay below 300 lines "
            "after plan/command split",
        )
        self.assertTrue(SOURCE_TEMPLATE_PLAN_COMMAND.exists())
        plan_command_lines = len(
            SOURCE_TEMPLATE_PLAN_COMMAND.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            plan_command_lines,
            340,
            "SourceTemplate plan/command owner should stay below 340 lines",
        )


if __name__ == "__main__":
    unittest.main()
