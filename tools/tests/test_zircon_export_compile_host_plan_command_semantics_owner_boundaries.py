"""Boundary tests for CompileHost plan command semantic diagnostics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPILE_HOST_PLAN = REPO_ROOT / "tools/zircon_export/compile_host_plan.py"
COMPILE_HOST_PLAN_COMMAND_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/compile_host_plan_command_semantics.py"
)
VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_validate_compile_host_command_semantics.py"
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class CompileHostPlanCommandSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_plan_command_semantics_owner_exists(self):
        self.assertTrue(
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS.exists(),
            "CompileHost plan command semantics owner file is missing",
        )

    def test_plan_command_semantics_live_in_dedicated_owner(self):
        plan_text = COMPILE_HOST_PLAN.read_text(encoding="utf-8")
        command_text = (
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS.read_text(encoding="utf-8")
            if COMPILE_HOST_PLAN_COMMAND_SEMANTICS.exists()
            else ""
        )

        function_name = "compile_host_plan_command_semantic_diagnostics"
        self.assertNotIn(
            f"def {function_name}(",
            plan_text,
            "CompileHost plan owner should not own plan-side Cargo command semantics",
        )
        self.assertIn(
            f"def {function_name}(",
            command_text,
            "CompileHost plan command semantics owner is missing diagnostics",
        )

    def test_compile_host_plan_consumes_plan_command_owner_only(self):
        plan_text = COMPILE_HOST_PLAN.read_text(encoding="utf-8")
        command_text = (
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS.read_text(encoding="utf-8")
            if COMPILE_HOST_PLAN_COMMAND_SEMANTICS.exists()
            else ""
        )

        self.assertIn(
            "from .compile_host_plan_command_semantics import (",
            plan_text,
        )
        self.assertNotIn(
            "from .pipeline_report_validate_compile_host_command_semantics import (",
            plan_text,
            "CompileHost plan owner should consume the plan command semantics owner",
        )
        self.assertIn(
            "from .pipeline_report_validate_compile_host_command_semantics import (",
            command_text,
            "Plan command semantics owner should adapt Validate command helpers",
        )
        self.assertNotIn(
            "from .compile_host_plan import",
            command_text,
            "Plan command semantics owner must stay leaf-like and avoid plan owner cycles",
        )

    def test_compile_host_plan_command_semantics_owners_stay_small(self):
        self.assertLess(_line_count(COMPILE_HOST_PLAN), 340)
        self.assertTrue(
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS.exists(),
            "CompileHost plan command semantics owner file is missing",
        )
        self.assertLess(_line_count(COMPILE_HOST_PLAN_COMMAND_SEMANTICS), 190)


if __name__ == "__main__":
    unittest.main()
