import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPILE_HOST_STAGE = REPO_ROOT / "tools/zircon_export/compile_host.py"
COMPILE_HOST_PLAN = REPO_ROOT / "tools/zircon_export/compile_host_plan.py"
COMPILE_HOST_PLAN_COMMAND_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/compile_host_plan_command_semantics.py"
)


class ZirconExportCompileHostOwnerBoundaryTests(unittest.TestCase):
    def test_compile_host_plan_evidence_lives_in_plan_owner(self):
        self.assertTrue(
            COMPILE_HOST_PLAN.exists(),
            "CompileHost plan/evidence diagnostics need a dedicated owner",
        )
        stage_text = COMPILE_HOST_STAGE.read_text(encoding="utf-8")
        plan_text = COMPILE_HOST_PLAN.read_text(encoding="utf-8")

        for function_name in (
            "load_compile_host_plan",
            "compile_host_plan_string_evidence_diagnostics",
            "compile_host_plan_array_evidence_diagnostics",
            "validate_report_requires_compile_host_strategy",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the CompileHost plan owner",
            )
            self.assertIn(f"def {function_name}(", plan_text)

        self.assertIn(
            "from .compile_host_plan import",
            stage_text,
            "CompileHost stage runner should consume the plan owner",
        )
        self.assertNotIn(
            "from .compile_host import",
            plan_text,
            "CompileHost plan owner must not import the stage runner",
        )

    def test_compile_host_stage_runner_stays_under_large_file_threshold(self):
        line_count = len(COMPILE_HOST_STAGE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "CompileHost stage runner should stay below 560 lines after plan split",
        )

    def test_compile_host_plan_owner_stays_leaf_sized(self):
        self.assertTrue(
            COMPILE_HOST_PLAN.exists(),
            "CompileHost plan owner should exist before its size can be checked",
        )
        line_count = len(COMPILE_HOST_PLAN.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            340,
            "CompileHost plan owner should stay below 340 lines",
        )


if __name__ == "__main__":
    unittest.main()
