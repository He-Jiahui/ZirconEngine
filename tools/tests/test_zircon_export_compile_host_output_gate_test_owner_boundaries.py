"""Boundary tests for CompileHost output-gate test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPILE_HOST_OUTPUT_GATE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_compile_host_output_gate.py"
)
COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_compile_host_plan_command_semantics.py"
)

COMMAND_SEMANTICS_TEST_METHODS = (
    "test_compile_host_rejects_plan_with_empty_command",
    "test_compile_host_rejects_plan_with_blank_command_entry",
    "test_compile_host_rejects_plan_with_non_cargo_command",
    "test_compile_host_rejects_plan_command_metadata_mismatch",
    "test_compile_host_rejects_plan_command_target_broadening",
    "test_compile_host_rejects_plan_command_target_triple_override",
    "test_compile_host_rejects_plan_command_package_broadening",
    "test_compile_host_rejects_plan_command_profile_override",
    "test_compile_host_rejects_plan_command_wrapper_policy_override",
    "test_compile_host_rejects_plan_forbidden_command_equals_form",
    "test_compile_host_rejects_plan_with_dangling_target_dir_option",
    "test_compile_host_rejects_target_dir_option_with_option_value",
    "test_compile_host_rejects_plan_with_duplicate_target_dir_option",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class CompileHostOutputGateTestOwnerBoundaryTests(unittest.TestCase):
    def test_compile_host_plan_command_semantics_tests_have_dedicated_owner(self):
        self.assertTrue(
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST.exists(),
            "CompileHost plan command semantics test owner is missing",
        )

        output_gate_text = COMPILE_HOST_OUTPUT_GATE_TEST.read_text(encoding="utf-8")
        command_semantics_text = COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in COMMAND_SEMANTICS_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    output_gate_text,
                    "CompileHost output gate test should not own command semantics",
                )
                self.assertIn(
                    f"def {method_name}(",
                    command_semantics_text,
                    "CompileHost command semantics test owner is missing coverage",
                )

    def test_compile_host_output_gate_keeps_plan_and_host_output_tests(self):
        output_gate_text = COMPILE_HOST_OUTPUT_GATE_TEST.read_text(encoding="utf-8")
        command_semantics_text = (
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST.read_text(encoding="utf-8")
            if COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST.exists()
            else ""
        )

        for method_name in (
            "test_compile_host_rejects_plan_without_binary",
            "test_compile_host_rejects_plan_missing_required_evidence_field",
            "test_compile_host_rejects_empty_host_output",
            "test_compile_host_rejects_directory_host_output",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", output_gate_text)
                self.assertNotIn(f"def {method_name}(", command_semantics_text)

    def test_compile_host_output_gate_test_owners_stay_small(self):
        self.assertLess(
            _line_count(COMPILE_HOST_OUTPUT_GATE_TEST),
            800,
            "CompileHost output gate test should stay focused on plan and host output gates",
        )
        self.assertTrue(
            COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST.exists(),
            "CompileHost plan command semantics test owner is missing",
        )
        self.assertLess(
            _line_count(COMPILE_HOST_PLAN_COMMAND_SEMANTICS_TEST),
            900,
            "CompileHost plan command semantics test owner should stay below split threshold",
        )


if __name__ == "__main__":
    unittest.main()
