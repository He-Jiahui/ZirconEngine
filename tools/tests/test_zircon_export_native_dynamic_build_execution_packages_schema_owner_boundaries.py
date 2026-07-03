"""Boundary tests for NativeDynamic build execution packages schema ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_EXECUTION_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_execution_schema.py"
)
BUILD_EXECUTION_PACKAGES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_execution_packages_schema.py"
)

MOVED_CONSTANTS = (
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_EMPTY_STRING_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_SAFE_RELATIVE_STRING_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_NEGATIVE_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_ARRAY_FIELDS",
)

MOVED_FUNCTIONS = (
    "native_dynamic_build_execution_packages_schema_diagnostics",
    "native_dynamic_build_execution_package_path_scope_array_diagnostics",
    "native_dynamic_build_execution_command_array_schema_diagnostics",
    "native_dynamic_build_execution_copied_sidecars_array_schema_diagnostics",
    "native_dynamic_build_execution_package_path_scope_diagnostics",
    "native_dynamic_build_execution_package_path_prefix",
    "native_dynamic_build_execution_safe_relative_path_array_diagnostics",
    "native_dynamic_build_execution_safe_relative_path_diagnostics",
    "native_dynamic_build_execution_normalized_safe_relative_path",
    "native_dynamic_build_execution_trimmed_non_empty_string_is_schema_clean",
    "native_dynamic_build_execution_has_drive_prefix",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicBuildExecutionPackagesSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_build_execution_packages_schema_owner_exists(self):
        self.assertTrue(
            BUILD_EXECUTION_PACKAGES_SCHEMA.exists(),
            "NativeDynamic build execution packages schema owner file is missing",
        )

    def test_build_execution_package_schema_members_are_owned_by_package_module(
        self,
    ):
        schema_text = BUILD_EXECUTION_SCHEMA.read_text(encoding="utf-8")
        owner_text = (
            BUILD_EXECUTION_PACKAGES_SCHEMA.read_text(encoding="utf-8")
            if BUILD_EXECUTION_PACKAGES_SCHEMA.exists()
            else ""
        )

        failures: list[str] = []
        for constant_name in MOVED_CONSTANTS:
            definition = f"{constant_name} ="
            if definition in schema_text:
                failures.append(f"schema still owns {constant_name}")
            if definition not in owner_text:
                failures.append(f"package owner missing {constant_name}")
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"schema still owns {function_name}")
            if definition not in owner_text:
                failures.append(f"package owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_packages_schema_without_reverse_import(self):
        schema_text = BUILD_EXECUTION_SCHEMA.read_text(encoding="utf-8")
        owner_text = (
            BUILD_EXECUTION_PACKAGES_SCHEMA.read_text(encoding="utf-8")
            if BUILD_EXECUTION_PACKAGES_SCHEMA.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_build_execution_packages_schema import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_native_dynamic_build_execution_schema",
            owner_text,
        )

    def test_build_execution_schema_and_package_owner_stay_small(self):
        self.assertLess(_line_count(BUILD_EXECUTION_SCHEMA), 430)
        self.assertTrue(
            BUILD_EXECUTION_PACKAGES_SCHEMA.exists(),
            "NativeDynamic build execution packages schema owner file is missing",
        )
        self.assertLess(_line_count(BUILD_EXECUTION_PACKAGES_SCHEMA), 430)


if __name__ == "__main__":
    unittest.main()
