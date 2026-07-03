"""Boundary tests for PlatformBundle manifest schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_schema.py"
)
MANIFEST_TEMPLATE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_manifest_template_schema.py"
)
MANIFEST_SCHEMA_TEST_SUPPORT = (
    REPO_ROOT
    / "tools/zircon_export/tests/platform_bundle_manifest_schema_test_support.py"
)

TEMPLATE_SCHEMA_TEST_METHODS = (
    "test_report_rejects_template_report_string_fields_non_string",
    "test_report_rejects_template_report_files_non_object_array",
    "test_report_rejects_template_bundle_unknown_field",
    "test_report_rejects_template_resolution_unknown_field",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleManifestSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_manifest_template_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            MANIFEST_TEMPLATE_SCHEMA_TEST.exists(),
            "PlatformBundle manifest template schema test owner is missing",
        )

        root_text = MANIFEST_SCHEMA_TEST.read_text(encoding="utf-8")
        template_text = MANIFEST_TEMPLATE_SCHEMA_TEST.read_text(encoding="utf-8")

        for method_name in TEMPLATE_SCHEMA_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Manifest schema root test should not own template schema gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    template_text,
                    "Template schema owner is missing coverage",
                )

    def test_manifest_schema_root_keeps_bundle_manifest_tests(self):
        root_text = MANIFEST_SCHEMA_TEST.read_text(encoding="utf-8")
        template_text = (
            MANIFEST_TEMPLATE_SCHEMA_TEST.read_text(encoding="utf-8")
            if MANIFEST_TEMPLATE_SCHEMA_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_rejects_bundle_manifest_template_resolution_profile_mismatch",
            "test_report_rejects_platform_bundle_report_native_payload_schema_before_manifest_compare",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", template_text)

    def test_manifest_schema_template_resolution_support_has_dedicated_owner(self):
        self.assertTrue(
            MANIFEST_SCHEMA_TEST_SUPPORT.exists(),
            "PlatformBundle manifest schema test support owner is missing",
        )

        for path in (MANIFEST_SCHEMA_TEST, MANIFEST_TEMPLATE_SCHEMA_TEST):
            with self.subTest(path=path.name):
                text = path.read_text(encoding="utf-8") if path.exists() else ""
                self.assertNotIn("def _template_resolution(", text)
                self.assertIn(
                    "from tools.zircon_export.tests.platform_bundle_manifest_schema_test_support import",
                    text,
                )

    def test_manifest_schema_test_owners_stay_small(self):
        self.assertLess(
            _line_count(MANIFEST_SCHEMA_TEST),
            780,
            "Manifest schema root test should stay focused on bundle manifest gates",
        )
        for path, budget, description in (
            (MANIFEST_TEMPLATE_SCHEMA_TEST, 620, "template schema"),
            (MANIFEST_SCHEMA_TEST_SUPPORT, 60, "test support"),
        ):
            with self.subTest(owner=description):
                self.assertTrue(path.exists(), f"{description} owner is missing")
                self.assertLess(
                    _line_count(path),
                    budget,
                    f"{description} owner should stay focused",
                )


if __name__ == "__main__":
    unittest.main()
