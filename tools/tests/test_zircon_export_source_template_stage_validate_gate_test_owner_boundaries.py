"""Boundary tests for SourceTemplate stage validate-gate test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMPILE_HOST_SOURCE_TEMPLATE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_compile_host_source_template.py"
)
SOURCE_TEMPLATE_VALIDATE_GATES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_source_template_stage_validate_gates.py"
)

VALIDATE_GATE_TEST_METHODS = (
    "test_source_template_stage_rejects_empty_build_command",
    "test_source_template_stage_rejects_invalid_validate_metadata",
    "test_source_template_stage_rejects_validate_report_directory",
    "test_source_template_stage_requires_source_template_strategy",
    "test_source_template_stage_rejects_invalid_strategy_metadata",
    "test_source_template_stage_rejects_escaped_manifest_path",
    "test_source_template_stage_marks_invalid_generated_file_fatal",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class SourceTemplateStageValidateGateTestOwnerBoundaryTests(unittest.TestCase):
    def test_source_template_validate_gate_tests_have_dedicated_owner(self):
        self.assertTrue(
            SOURCE_TEMPLATE_VALIDATE_GATES_TEST.exists(),
            "SourceTemplate stage validate-gate test owner is missing",
        )

        root_text = COMPILE_HOST_SOURCE_TEMPLATE_TEST.read_text(encoding="utf-8")
        validate_gate_text = SOURCE_TEMPLATE_VALIDATE_GATES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in VALIDATE_GATE_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "CompileHost/SourceTemplate root test should not own validate gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    validate_gate_text,
                    "SourceTemplate validate-gate test owner is missing coverage",
                )

    def test_compile_host_source_template_keeps_materialization_and_build_tests(self):
        root_text = COMPILE_HOST_SOURCE_TEMPLATE_TEST.read_text(encoding="utf-8")
        validate_gate_text = (
            SOURCE_TEMPLATE_VALIDATE_GATES_TEST.read_text(encoding="utf-8")
            if SOURCE_TEMPLATE_VALIDATE_GATES_TEST.exists()
            else ""
        )

        for method_name in (
            "test_compile_host_report_preserves_library_embed_link_plan",
            "test_source_template_stage_materializes_generated_project_without_build",
            "test_source_template_stage_reports_successful_build_validation",
            "test_source_template_stage_reports_failed_build_validation",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", validate_gate_text)

    def test_source_template_stage_validate_gate_test_owners_stay_small(self):
        self.assertLess(
            _line_count(COMPILE_HOST_SOURCE_TEMPLATE_TEST),
            950,
            "CompileHost/SourceTemplate root test should stay below the split budget",
        )
        self.assertTrue(
            SOURCE_TEMPLATE_VALIDATE_GATES_TEST.exists(),
            "SourceTemplate stage validate-gate test owner is missing",
        )
        self.assertLess(
            _line_count(SOURCE_TEMPLATE_VALIDATE_GATES_TEST),
            330,
            "SourceTemplate stage validate-gate test owner should stay leaf-sized",
        )


if __name__ == "__main__":
    unittest.main()
