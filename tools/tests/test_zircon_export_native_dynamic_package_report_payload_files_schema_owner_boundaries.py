"""Boundary tests for NativeDynamic package report payload.files schema ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PACKAGE_REPORT_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_package_report_schema.py"
)
PAYLOAD_FILES_SCHEMA = (
    REPO_ROOT
    / "tools/zircon_export/pipeline_report_native_dynamic_package_report_payload_files_schema.py"
)

PAYLOAD_FILES_FUNCTIONS = (
    "platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics",
)
PAYLOAD_FILES_CONSTANTS = (
    "NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS",
    "NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS",
    "NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS",
    "NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicPackageReportPayloadFilesSchemaOwnerBoundaryTests(
    unittest.TestCase
):
    def test_payload_files_schema_owner_exists(self) -> None:
        self.assertTrue(
            PAYLOAD_FILES_SCHEMA.exists(),
            "NativeDynamic package report payload.files schema owner is missing",
        )

    def test_payload_files_schema_lives_in_leaf_owner(self) -> None:
        schema_text = PACKAGE_REPORT_SCHEMA.read_text(encoding="utf-8")
        payload_files_text = (
            PAYLOAD_FILES_SCHEMA.read_text(encoding="utf-8")
            if PAYLOAD_FILES_SCHEMA.exists()
            else ""
        )

        failures: list[str] = []
        for function_name in PAYLOAD_FILES_FUNCTIONS:
            definition = f"def {function_name}("
            if definition in schema_text:
                failures.append(f"package report schema still owns {function_name}")
            if definition not in payload_files_text:
                failures.append(f"payload.files owner missing {function_name}")
        for constant_name in PAYLOAD_FILES_CONSTANTS:
            if constant_name in schema_text:
                failures.append(f"package report schema still imports {constant_name}")
            if constant_name not in payload_files_text:
                failures.append(f"payload.files owner missing {constant_name}")

        if failures:
            self.fail("\n".join(failures))

    def test_package_report_schema_imports_payload_files_owner(self) -> None:
        schema_text = PACKAGE_REPORT_SCHEMA.read_text(encoding="utf-8")
        payload_files_text = (
            PAYLOAD_FILES_SCHEMA.read_text(encoding="utf-8")
            if PAYLOAD_FILES_SCHEMA.exists()
            else ""
        )

        self.assertIn(
            "from .pipeline_report_native_dynamic_package_report_payload_files_schema import (",
            schema_text,
        )
        self.assertNotIn(
            "from .pipeline_report_native_dynamic_package_report_schema import",
            payload_files_text,
        )

    def test_package_report_schema_owner_budget_stays_tight(self) -> None:
        self.assertLess(_line_count(PACKAGE_REPORT_SCHEMA), 320)
        self.assertTrue(
            PAYLOAD_FILES_SCHEMA.exists(),
            "NativeDynamic package report payload.files schema owner is missing",
        )
        self.assertLess(_line_count(PAYLOAD_FILES_SCHEMA), 130)


if __name__ == "__main__":
    unittest.main()
