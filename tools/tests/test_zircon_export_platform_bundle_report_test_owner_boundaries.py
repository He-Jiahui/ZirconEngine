"""Boundary tests for PlatformBundle report test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE_REPORT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_platform_bundle.py"
)
NATIVE_PAYLOAD_REPORT_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_native_plugins_payload.py"
)

NATIVE_PAYLOAD_TEST_METHODS = (
    "test_report_rejects_stale_native_plugins_payload_hash",
    "test_report_rejects_native_plugins_payload_stage_report_mismatch",
    "test_report_accepts_current_native_plugins_payload",
    "test_report_rejects_manual_native_plugins_payload_without_stage_handoff_noise",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleReportTestOwnerBoundaryTests(unittest.TestCase):
    def test_native_plugins_payload_tests_have_dedicated_owner(self):
        self.assertTrue(
            NATIVE_PAYLOAD_REPORT_TEST.exists(),
            "PlatformBundle native payload test owner is missing",
        )

        root_text = PLATFORM_BUNDLE_REPORT_TEST.read_text(encoding="utf-8")
        native_payload_text = NATIVE_PAYLOAD_REPORT_TEST.read_text(encoding="utf-8")

        for method_name in NATIVE_PAYLOAD_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "PlatformBundle root test should not own native payload gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    native_payload_text,
                    "Native payload owner is missing coverage",
                )

    def test_platform_bundle_root_keeps_output_integrity_tests(self):
        root_text = PLATFORM_BUNDLE_REPORT_TEST.read_text(encoding="utf-8")
        native_payload_text = (
            NATIVE_PAYLOAD_REPORT_TEST.read_text(encoding="utf-8")
            if NATIVE_PAYLOAD_REPORT_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_rejects_platform_bundle_without_bundle_root",
            "test_report_rejects_platform_host_output_hash_mismatch",
            "test_report_rejects_missing_platform_delta_source_file",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", native_payload_text)

    def test_platform_bundle_report_test_owners_stay_small(self):
        self.assertLess(
            _line_count(PLATFORM_BUNDLE_REPORT_TEST),
            520,
            "PlatformBundle root report test should stay focused",
        )
        self.assertTrue(
            NATIVE_PAYLOAD_REPORT_TEST.exists(),
            "PlatformBundle native payload test owner is missing",
        )
        self.assertLess(
            _line_count(NATIVE_PAYLOAD_REPORT_TEST),
            620,
            "PlatformBundle native payload owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
