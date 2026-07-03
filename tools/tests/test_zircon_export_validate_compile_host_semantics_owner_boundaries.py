"""Boundary tests for Validate CompileHost semantic diagnostics ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
VALIDATE_COMPILE_HOST_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_validate_compile_host_schema.py"
)
VALIDATE_COMPILE_HOST_SEMANTICS = (
    REPO_ROOT / "tools/zircon_export/pipeline_report_validate_compile_host_semantics.py"
)

MOVED_FUNCTIONS = (
    "library_embed_compile_host_profile_release_diagnostics",
    "compile_host_cargo_profile_is_schema_clean",
    "compile_host_target_selector_schema_diagnostics",
)
MOVED_CONSTANTS = (
    "VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES",
    "VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PACKAGES",
    "VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BINARIES",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class ValidateCompileHostSemanticsOwnerBoundaryTests(unittest.TestCase):
    def test_validate_compile_host_semantics_owner_exists(self):
        self.assertTrue(
            VALIDATE_COMPILE_HOST_SEMANTICS.exists(),
            "Validate CompileHost semantics owner file is missing",
        )

    def test_semantic_functions_are_owned_by_semantics_module(self):
        schema_text = VALIDATE_COMPILE_HOST_SCHEMA.read_text(encoding="utf-8")
        semantics_text = (
            VALIDATE_COMPILE_HOST_SEMANTICS.read_text(encoding="utf-8")
            if VALIDATE_COMPILE_HOST_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"schema still owns {function_name}")
            if definition not in semantics_text:
                failures.append(f"semantics owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_semantic_constants_are_owned_by_semantics_module(self):
        schema_text = VALIDATE_COMPILE_HOST_SCHEMA.read_text(encoding="utf-8")
        semantics_text = (
            VALIDATE_COMPILE_HOST_SEMANTICS.read_text(encoding="utf-8")
            if VALIDATE_COMPILE_HOST_SEMANTICS.exists()
            else ""
        )

        failures: list[str] = []
        for constant_name in MOVED_CONSTANTS:
            assignment = f"{constant_name} ="
            if assignment in schema_text:
                failures.append(f"schema still owns {constant_name}")
            if assignment not in semantics_text:
                failures.append(f"semantics owner missing {constant_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_semantics_without_reverse_import(self):
        schema_text = VALIDATE_COMPILE_HOST_SCHEMA.read_text(encoding="utf-8")
        semantics_text = (
            VALIDATE_COMPILE_HOST_SEMANTICS.read_text(encoding="utf-8")
            if VALIDATE_COMPILE_HOST_SEMANTICS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_validate_compile_host_semantics import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_validate_compile_host_schema",
            semantics_text,
        )

    def test_validate_compile_host_schema_and_semantics_stay_small(self):
        self.assertLess(_line_count(VALIDATE_COMPILE_HOST_SCHEMA), 430)
        self.assertTrue(
            VALIDATE_COMPILE_HOST_SEMANTICS.exists(),
            "Validate CompileHost semantics owner file is missing",
        )
        self.assertLess(_line_count(VALIDATE_COMPILE_HOST_SEMANTICS), 170)


if __name__ == "__main__":
    unittest.main()
