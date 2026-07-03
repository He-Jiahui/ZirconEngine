"""Boundary tests for NativeDynamic package report schema helper ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACKAGE_REPORT_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py"
)
PACKAGE_REPORT_SCHEMA_HELPERS = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_package_report_schema_helpers.py"
)

MOVED_FUNCTIONS = (
    "table_required_non_empty_string_schema_diagnostics",
    "table_required_trimmed_non_empty_string_schema_diagnostics",
    "table_sha256_hex_string_schema_diagnostics",
    "table_safe_relative_path_string_schema_diagnostics",
    "table_non_negative_integer_schema_diagnostics",
    "object_array_required_non_empty_string_schema_diagnostics",
    "object_array_required_trimmed_non_empty_string_schema_diagnostics",
    "object_array_safe_relative_path_string_schema_diagnostics",
    "object_array_non_negative_integer_schema_diagnostics",
    "object_array_sha256_hex_string_schema_diagnostics",
    "string_array_unique_entries_schema_diagnostics",
    "object_array_unique_string_field_schema_diagnostics",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicPackageReportSchemaHelperOwnerBoundaryTests(unittest.TestCase):
    def test_package_report_schema_helper_owner_exists(self):
        self.assertTrue(
            PACKAGE_REPORT_SCHEMA_HELPERS.exists(),
            "NativeDynamic package report schema helper owner file is missing",
        )

    def test_package_report_schema_helpers_are_owned_by_helper_module(self):
        schema_text = PACKAGE_REPORT_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            PACKAGE_REPORT_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if PACKAGE_REPORT_SCHEMA_HELPERS.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in MOVED_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"schema still owns {function_name}")
            if definition not in helper_text:
                failures.append(f"helper owner missing {function_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_schema_imports_helpers_without_reverse_import(self):
        schema_text = PACKAGE_REPORT_SCHEMA.read_text(encoding="utf-8")
        helper_text = (
            PACKAGE_REPORT_SCHEMA_HELPERS.read_text(encoding="utf-8")
            if PACKAGE_REPORT_SCHEMA_HELPERS.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_package_report_schema_helpers import (",
            schema_text,
        )
        self.assertNotIn(
            ".pipeline_report_native_dynamic_package_report_schema",
            helper_text,
        )

    def test_package_report_schema_and_helper_owner_stay_small(self):
        self.assertLess(_line_count(PACKAGE_REPORT_SCHEMA), 470)
        self.assertTrue(
            PACKAGE_REPORT_SCHEMA_HELPERS.exists(),
            "NativeDynamic package report schema helper owner file is missing",
        )
        self.assertLess(_line_count(PACKAGE_REPORT_SCHEMA_HELPERS), 300)


if __name__ == "__main__":
    unittest.main()
