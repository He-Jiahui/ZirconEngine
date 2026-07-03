"""Boundary tests for Validate CompileHost command semantic diagnostics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATE_COMPILE_HOST_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_validate_compile_host_semantics.py"
)
VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_validate_compile_host_command_semantics.py"
)
VALIDATE_COMPILE_HOST_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_validate_compile_host_schema.py"
)
COMPILE_HOST_PLAN = REPO_ROOT / "tools/zircon_export/compile_host_plan.py"
COMPILE_HOST_PLAN_COMMAND_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/compile_host_plan_command_semantics.py"
)

COMMAND_FUNCTIONS = (
    "library_embed_compile_host_command_schema_diagnostics",
    "command_flag_diagnostics",
    "command_forbidden_flag_diagnostics",
    "compile_host_command_forbidden_target_diagnostics",
    "compile_host_command_forbidden_target_triple_diagnostics",
    "compile_host_command_forbidden_package_diagnostics",
    "compile_host_command_forbidden_profile_diagnostics",
    "compile_host_command_forbidden_wrapper_policy_diagnostics",
)
COMMAND_CONSTANTS = (
    "COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_FLAGS",
    "COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS",
    "COMPILE_HOST_COMMAND_FORBIDDEN_PACKAGE_FLAGS",
    "COMPILE_HOST_COMMAND_FORBIDDEN_PROFILE_FLAGS",
    "COMPILE_HOST_COMMAND_FORBIDDEN_WRAPPER_POLICY_FLAGS",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class ValidateCompileHostCommandSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_command_semantics_owner_exists(self):
        self.assertTrue(
            VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.exists(),
            "Validate CompileHost command semantics owner file is missing",
        )

    def test_command_semantics_live_in_command_owner(self):
        semantics_text = VALIDATE_COMPILE_HOST_SEMANTICS.read_text(encoding="utf-8")
        command_text = (
            VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.read_text(encoding="utf-8")
            if VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in COMMAND_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in semantics_text:
                failures.append(f"semantics owner still owns {function_name}")
            if definition not in command_text:
                failures.append(f"command semantics owner missing {function_name}")
        for constant_name in COMMAND_CONSTANTS:
            assignment = f"{constant_name} ="
            if assignment in semantics_text:
                failures.append(f"semantics owner still owns {constant_name}")
            if assignment not in command_text:
                failures.append(f"command semantics owner missing {constant_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_consumers_import_command_semantics_directly(self):
        schema_text = VALIDATE_COMPILE_HOST_SCHEMA.read_text(encoding="utf-8")
        compile_host_plan_command_text = (
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS.read_text(encoding="utf-8")
            if COMPILE_HOST_PLAN_COMMAND_SEMANTICS.exists()
            else ""
        )
        command_text = (
            VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.read_text(encoding="utf-8")
            if VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_validate_compile_host_command_semantics import (",
            schema_text,
        )
        self.assertIn(
            "from .pipeline_report_validate_compile_host_command_semantics import (",
            compile_host_plan_command_text,
        )
        self.assertNotIn(
            ".pipeline_report_validate_compile_host_semantics",
            command_text,
        )

    def test_validate_compile_host_semantics_owners_stay_small(self):
        self.assertLess(_line_count(VALIDATE_COMPILE_HOST_SEMANTICS), 170)
        self.assertTrue(
            VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS.exists(),
            "Validate CompileHost command semantics owner file is missing",
        )
        self.assertLess(_line_count(VALIDATE_COMPILE_HOST_COMMAND_SEMANTICS), 430)


if __name__ == "__main__":
    unittest.main()
