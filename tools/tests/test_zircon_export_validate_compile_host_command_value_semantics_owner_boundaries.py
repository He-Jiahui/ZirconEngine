"""Boundary tests for Validate CompileHost command value semantics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMMAND_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_validate_compile_host_command_semantics.py"
)
COMMAND_VALUE_SEMANTICS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_validate_compile_host_command_value_semantics.py"
)
COMPILE_HOST_PLAN_COMMAND_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/compile_host_plan_command_semantics.py"
)

VALUE_SEMANTIC_FUNCTIONS = (
    "command_option_value_match_diagnostics",
    "command_features_match_diagnostics",
    "command_option_path_value_match_diagnostics",
    "command_alias_value_match_diagnostics",
    "command_option_value",
    "cargo_feature_list",
    "compile_host_release_flag_schema_diagnostics",
    "compile_host_cargo_profile_is_schema_clean",
)
VALUE_SEMANTIC_CONSTANTS = (
    "VALIDATE_COMPILE_HOST_COMMAND_CARGO_PROFILES",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class ValidateCompileHostCommandValueSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_command_value_semantics_owner_exists(self) -> None:
        self.assertTrue(
            COMMAND_VALUE_SEMANTICS.exists(),
            "Validate CompileHost command value semantics owner file is missing",
        )

    def test_command_value_semantics_live_in_value_owner(self) -> None:
        command_text = COMMAND_SEMANTICS.read_text(encoding="utf-8")
        value_text = (
            COMMAND_VALUE_SEMANTICS.read_text(encoding="utf-8")
            if COMMAND_VALUE_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in VALUE_SEMANTIC_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in command_text:
                failures.append(f"command semantics owner still owns {function_name}")
            if definition not in value_text:
                failures.append(
                    f"command value semantics owner missing {function_name}"
                )
        for constant_name in VALUE_SEMANTIC_CONSTANTS:
            assignment = f"{constant_name} ="
            if assignment in command_text:
                failures.append(f"command semantics owner still owns {constant_name}")
            if assignment not in value_text:
                failures.append(f"command value semantics owner missing {constant_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_consumers_import_command_value_semantics_directly(self) -> None:
        command_text = COMMAND_SEMANTICS.read_text(encoding="utf-8")
        plan_command_text = COMPILE_HOST_PLAN_COMMAND_SEMANTICS.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "from .pipeline_report_validate_compile_host_command_value_semantics import (",
            command_text,
        )
        self.assertIn(
            "from .pipeline_report_validate_compile_host_command_value_semantics import (",
            plan_command_text,
        )

    def test_command_semantics_owner_budget_stays_tight(self) -> None:
        self.assertLess(_line_count(COMMAND_SEMANTICS), 300)
        self.assertTrue(
            COMMAND_VALUE_SEMANTICS.exists(),
            "Validate CompileHost command value semantics owner file is missing",
        )
        self.assertLess(_line_count(COMMAND_VALUE_SEMANTICS), 220)


if __name__ == "__main__":
    unittest.main()
