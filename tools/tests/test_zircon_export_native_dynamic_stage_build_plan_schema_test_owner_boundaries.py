"""Boundary tests for NativeDynamic stage build-plan schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ROOT_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_plan_schema.py"
)
PACKAGE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_build_plan_package_schema.py"
)
SUPPORT_FILE = (
    REPO_ROOT
    / "tools/zircon_export/tests/native_dynamic_stage_schema_test_support.py"
)

PACKAGE_SCHEMA_TESTS = (
    "test_report_stage_rejects_native_dynamic_build_plan_package_unknown_field",
    "test_report_stage_rejects_native_dynamic_build_plan_package_missing_required_field",
    "test_report_stage_rejects_native_dynamic_build_plan_package_header_mismatch",
    "test_report_stage_rejects_native_dynamic_build_plan_expected_artifact_mismatch",
    "test_report_stage_rejects_native_dynamic_build_plan_package_field_types",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicStageBuildPlanSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def test_package_schema_tests_have_dedicated_owner(self):
        self.assertTrue(
            PACKAGE_SCHEMA_TEST.exists(),
            "NativeDynamic build-plan package schema test owner is missing",
        )

        root_text = ROOT_SCHEMA_TEST.read_text(encoding="utf-8")
        package_text = PACKAGE_SCHEMA_TEST.read_text(encoding="utf-8")
        for test_name in PACKAGE_SCHEMA_TESTS:
            with self.subTest(test=test_name):
                self.assertNotIn(f"def {test_name}", root_text)
                self.assertIn(f"def {test_name}", package_text)

    def test_build_plan_package_feature_fixture_lives_in_shared_support(self):
        root_text = ROOT_SCHEMA_TEST.read_text(encoding="utf-8")
        package_text = PACKAGE_SCHEMA_TEST.read_text(encoding="utf-8")
        support_text = SUPPORT_FILE.read_text(encoding="utf-8")

        self.assertIn("def _native_build_plan_package_with_features", support_text)
        self.assertNotIn("def _native_build_plan_package_with_features", root_text)
        self.assertNotIn("def _native_build_plan_package_with_features", package_text)

    def test_build_plan_schema_test_owners_stay_small(self):
        budgets = (
            (ROOT_SCHEMA_TEST, 760, "root build-plan schema owner"),
            (PACKAGE_SCHEMA_TEST, 520, "package schema owner"),
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
