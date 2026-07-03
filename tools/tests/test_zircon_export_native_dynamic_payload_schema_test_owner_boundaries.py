"""Boundary tests for NativeDynamic payload schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_schema.py"
)
PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_package_report_schema.py"
)

PACKAGE_REPORT_SCHEMA_TEST_METHODS = (
    "test_report_rejects_native_plugins_payload_package_report_missing_required_field",
    "test_report_rejects_native_plugins_payload_package_report_payload_missing_required_field",
    "test_report_rejects_native_plugins_payload_package_report_abi_missing_required_field",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicPayloadSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_payload_package_report_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.exists(),
            "NativeDynamic payload package-report schema test owner is missing",
        )

        root_text = PAYLOAD_SCHEMA_TEST.read_text(encoding="utf-8")
        package_report_text = PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in PACKAGE_REPORT_SCHEMA_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Payload schema root test should not own package-report schema gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    package_report_text,
                    "Package-report schema owner is missing coverage",
                )

    def test_native_dynamic_payload_schema_root_keeps_payload_shape_tests(self):
        root_text = PAYLOAD_SCHEMA_TEST.read_text(encoding="utf-8")
        package_report_text = (
            PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.read_text(encoding="utf-8")
            if PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_rejects_native_plugins_payload_unknown_top_level_field",
            "test_report_rejects_native_plugins_payload_file_manifest_duplicate_path",
            "test_report_rejects_native_plugins_payload_operation_audit_platform_allowed_mismatch",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", package_report_text)

    def test_payload_package_report_schema_owns_toml_helpers(self):
        self.assertTrue(
            PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.exists(),
            "NativeDynamic payload package-report schema test owner is missing",
        )
        root_text = PAYLOAD_SCHEMA_TEST.read_text(encoding="utf-8")
        package_report_text = PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.read_text(
            encoding="utf-8"
        )

        self.assertNotIn("def _assert_package_report_schema_diagnostic(", root_text)
        self.assertNotIn("def _read_toml(", root_text)
        self.assertNotIn("def _write_toml(", root_text)
        self.assertIn("def _assert_package_report_schema_diagnostic(", package_report_text)
        self.assertIn("def _read_toml(", package_report_text)
        self.assertIn("def _write_toml(", package_report_text)

    def test_native_dynamic_payload_schema_test_owners_stay_small(self):
        self.assertLess(
            _line_count(PAYLOAD_SCHEMA_TEST),
            940,
            "NativeDynamic payload schema root test should stay below large-file budget",
        )
        self.assertTrue(
            PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST.exists(),
            "NativeDynamic payload package-report schema test owner is missing",
        )
        self.assertLess(
            _line_count(PAYLOAD_PACKAGE_REPORT_SCHEMA_TEST),
            260,
            "NativeDynamic payload package-report schema owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
