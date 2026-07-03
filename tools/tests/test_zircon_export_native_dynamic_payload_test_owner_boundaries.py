"""Boundary tests for NativeDynamic payload report test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PAYLOAD_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload.py"
)
PAYLOAD_PACKAGE_REPORT_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_payload_package_report.py"
)

PACKAGE_REPORT_TEST_METHODS = (
    "test_report_rejects_native_plugins_payload_package_report_package_id_mismatch",
    "test_report_rejects_stage_backed_native_plugins_payload_missing_package_report",
    "test_report_rejects_stage_backed_native_plugins_payload_file_manifest_drift",
    "test_report_rejects_native_plugins_payload_package_report_payload_count_mismatch",
    "test_report_rejects_native_plugins_payload_package_report_abi_version_mismatch",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicPayloadTestOwnerBoundaryTests(unittest.TestCase):
    def test_payload_package_report_tests_have_dedicated_owner(self):
        self.assertTrue(
            PAYLOAD_PACKAGE_REPORT_TEST.exists(),
            "NativeDynamic payload package-report test owner is missing",
        )

        root_text = PAYLOAD_TEST.read_text(encoding="utf-8")
        package_report_text = PAYLOAD_PACKAGE_REPORT_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in PACKAGE_REPORT_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic payload root test should not own package-report gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    package_report_text,
                    "Package-report owner is missing coverage",
                )

    def test_native_dynamic_payload_root_keeps_operation_audit_tests(self):
        root_text = PAYLOAD_TEST.read_text(encoding="utf-8")
        package_report_text = (
            PAYLOAD_PACKAGE_REPORT_TEST.read_text(encoding="utf-8")
            if PAYLOAD_PACKAGE_REPORT_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_rejects_missing_native_plugins_payload_signing_audit",
            "test_report_rejects_spoofed_native_plugins_payload_signing_audit",
            "test_report_rejects_spoofed_native_plugins_payload_notarization_audit",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", package_report_text)

    def test_native_dynamic_payload_test_owners_stay_small(self):
        self.assertLess(
            _line_count(PAYLOAD_TEST),
            620,
            "NativeDynamic payload root test should stay focused on audit payload gates",
        )
        self.assertTrue(
            PAYLOAD_PACKAGE_REPORT_TEST.exists(),
            "NativeDynamic payload package-report test owner is missing",
        )
        self.assertLess(
            _line_count(PAYLOAD_PACKAGE_REPORT_TEST),
            700,
            "NativeDynamic payload package-report owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
