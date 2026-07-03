"""Boundary tests for NativeDynamic build plan schema helper ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_PLAN_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema.py"
)
BUILD_PLAN_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_build_plan_schema_helpers.py"
)

MOVED_CONSTANTS = (
    "NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_FIELDS",
    "NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_FIELDS",
    "NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS",
    "NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS",
)

MOVED_FUNCTIONS = (
    "native_dynamic_build_plan_packages_schema_diagnostics",
    "native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics",
    "native_dynamic_build_plan_feature_array_schema_diagnostics",
    "native_dynamic_build_plan_command_array_schema_diagnostics",
    "native_dynamic_build_plan_diagnostics_array_schema_diagnostics",
    "native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean",
    "native_dynamic_build_plan_string_array_is_trimmed_non_empty",
    "native_dynamic_build_plan_string_array_is_schema_clean",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicBuildPlanSchemaHelperOwnerBoundaryTests(unittest.TestCase):
    def test_build_plan_schema_helper_owner_exists(self):
        self.assertTrue(
            BUILD_PLAN_SCHEMA_HELPERS.exists(),
            "NativeDynamic build plan schema helper owner file is missing",
        )

    def test_build_plan_schema_helpers_are_owned_by_helper_module(self):
        schema_text = BUILD_PLAN_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            BUILD_PLAN_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if BUILD_PLAN_SCHEMA_HELPERS.exists()
            else ""
        )

        failures: list[str] = []
        for constant_name in MOVED_CONSTANTS:
            definition = f"{constant_name} ="
            if definition in schema_text:
                failures.append(f"schema still owns {constant_name}")
            if definition not in helper_text:
                failures.append(f"helper owner missing {constant_name}")
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"schema still owns {function_name}")
            if definition not in helper_text:
                failures.append(f"helper owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_helpers_without_reverse_import(self):
        schema_text = BUILD_PLAN_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            BUILD_PLAN_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if BUILD_PLAN_SCHEMA_HELPERS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_build_plan_schema_helpers import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_native_dynamic_build_plan_schema",
            helper_text,
        )

    def test_build_plan_schema_and_helper_owner_stay_small(self):
        self.assertLess(_line_count(BUILD_PLAN_SCHEMA), 470)
        self.assertTrue(
            BUILD_PLAN_SCHEMA_HELPERS.exists(),
            "NativeDynamic build plan schema helper owner file is missing",
        )
        self.assertLess(_line_count(BUILD_PLAN_SCHEMA_HELPERS), 420)


if __name__ == "__main__":
    unittest.main()
