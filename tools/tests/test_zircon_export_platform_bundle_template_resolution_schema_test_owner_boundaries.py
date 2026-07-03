"""Boundary tests for PlatformBundle template resolution schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_schema.py"
)
CANDIDATE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_resolution_candidate_schema.py"
)
SUPPORT_FILE = (
    REPO_ROOT
    / "tools/zircon_export/tests/platform_bundle_template_resolution_schema_test_support.py"
)

CANDIDATE_SCHEMA_TESTS = (
    "test_report_rejects_template_resolution_candidate_dir_outside_root",
    "test_report_rejects_template_resolution_skipped_candidate_dir_outside_root",
    "test_report_rejects_template_resolution_candidate_missing_required_field",
    "test_report_rejects_template_resolution_candidate_identity_mismatch",
    "test_report_rejects_template_resolution_duplicate_candidate_template_dir",
    "test_report_rejects_template_resolution_candidate_also_skipped",
    "test_report_rejects_template_resolution_candidate_string_field_blank",
    "test_report_rejects_template_resolution_skipped_candidate_padded_template_dir",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleTemplateResolutionSchemaTestOwnerBoundaryTests(
    unittest.TestCase
):
    def test_candidate_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            CANDIDATE_SCHEMA_TEST.exists(),
            "PlatformBundle template resolution candidate schema owner is missing",
        )

        root_text = ROOT_SCHEMA_TEST.read_text(encoding="utf-8")
        candidate_text = CANDIDATE_SCHEMA_TEST.read_text(encoding="utf-8")
        for test_name in CANDIDATE_SCHEMA_TESTS:
            with self.subTest(test=test_name):
                self.assertNotIn(f"def {test_name}", root_text)
                self.assertIn(f"def {test_name}", candidate_text)

    def test_template_resolution_fixture_lives_in_shared_support(self):
        root_text = ROOT_SCHEMA_TEST.read_text(encoding="utf-8")
        support_text = SUPPORT_FILE.read_text(encoding="utf-8")

        self.assertIn("class PlatformBundleTemplateResolutionReportAssertions", support_text)
        self.assertIn("def _template_resolution", support_text)
        self.assertNotIn("def _assert_template_resolution_diagnostic", root_text)
        self.assertNotIn("def _template_resolution", root_text)

    def test_template_resolution_schema_test_owners_stay_small(self):
        budgets = (
            (ROOT_SCHEMA_TEST, 620, "root template resolution schema owner"),
            (CANDIDATE_SCHEMA_TEST, 620, "candidate schema owner"),
            (SUPPORT_FILE, 120, "template resolution schema support"),
        )
        for path, budget, description in budgets:
            with self.subTest(owner=description):
                self.assertTrue(path.exists(), f"{description} is missing")
                self.assertLess(
                    _line_count(path),
                    budget,
                    f"{description} should stay focused",
                )


if __name__ == "__main__":
    unittest.main()
