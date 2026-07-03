"""Boundary tests for NativeDynamic stage test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_STAGE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_native_dynamic_stage.py"
)
NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_native_dynamic_stage_source_manifest.py"
)
NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_native_dynamic_stage_selection_strategy.py"
)

SOURCE_MANIFEST_TEST_METHODS = (
    "test_native_dynamic_stage_rejects_source_manifest_id_mismatch",
    "test_native_dynamic_stage_rejects_padded_source_manifest_id_before_package_match",
    "test_native_dynamic_stage_rejects_non_string_source_manifest_id_before_missing_id",
    "test_native_dynamic_stage_rejects_padded_recursive_source_manifest_id_before_missing_manifest",
    "test_native_dynamic_stage_rejects_recursive_source_manifest_parse_error_before_missing_manifest",
    "test_native_dynamic_stage_rejects_source_manifest_parse_error",
    "test_native_dynamic_stage_rejects_source_manifest_directory",
    "test_native_dynamic_stage_rejects_source_manifest_missing_id",
)

SELECTION_STRATEGY_TEST_METHODS = (
    "test_native_dynamic_stage_rejects_unselected_package_export",
    "test_native_dynamic_stage_rejects_duplicate_selected_package_ids",
    "test_native_dynamic_stage_rejects_padded_selected_package_id_before_uniqueness",
    "test_native_dynamic_stage_rejects_non_string_selected_package_id_before_array_shape",
    "test_native_dynamic_stage_rejects_missing_selected_package_export",
    "test_native_dynamic_stage_rejects_padded_target_platform_before_artifact_selection",
    "test_native_dynamic_stage_rejects_invalid_validate_metadata",
    "test_native_dynamic_stage_rejects_validate_report_directory",
    "test_native_dynamic_stage_requires_native_dynamic_strategy",
    "test_native_dynamic_stage_rejects_invalid_strategy_metadata",
    "test_native_dynamic_stage_reports_missing_package_source_fatal",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicStageTestOwnerBoundaryTests(unittest.TestCase):
    def test_source_manifest_tests_have_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST.exists(),
            "NativeDynamic stage source manifest test owner is missing",
        )

        root_text = NATIVE_DYNAMIC_STAGE_TEST.read_text(encoding="utf-8")
        source_text = NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in SOURCE_MANIFEST_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic stage root should not own source manifest gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    source_text,
                    "Source manifest owner is missing coverage",
                )

    def test_selection_strategy_tests_have_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST.exists(),
            "NativeDynamic stage selection/strategy test owner is missing",
        )

        root_text = NATIVE_DYNAMIC_STAGE_TEST.read_text(encoding="utf-8")
        selection_text = NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in SELECTION_STRATEGY_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic stage root should not own selection/strategy gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    selection_text,
                    "Selection/strategy owner is missing coverage",
                )

    def test_native_dynamic_stage_root_keeps_materialization_tests(self):
        root_text = NATIVE_DYNAMIC_STAGE_TEST.read_text(encoding="utf-8")
        source_text = (
            NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST.read_text(encoding="utf-8")
            if NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST.exists()
            else ""
        )
        selection_text = (
            NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST.read_text(
                encoding="utf-8"
            )
            if NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST.exists()
            else ""
        )

        for method_name in (
            "test_native_dynamic_stage_materializes_package_and_loader_manifest",
            "test_native_dynamic_stage_filters_artifacts_by_target_platform",
            "test_native_dynamic_stage_rejects_inconsistent_package_paths",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", source_text)
                self.assertNotIn(f"def {method_name}(", selection_text)

    def test_native_dynamic_stage_test_owners_stay_small(self):
        self.assertLess(
            _line_count(NATIVE_DYNAMIC_STAGE_TEST),
            700,
            "NativeDynamic stage root test should stay focused on materialization",
        )
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST.exists(),
            "NativeDynamic stage source manifest test owner is missing",
        )
        self.assertLess(
            _line_count(NATIVE_DYNAMIC_STAGE_SOURCE_MANIFEST_TEST),
            380,
            "Source manifest test owner should stay focused",
        )
        self.assertTrue(
            NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST.exists(),
            "NativeDynamic stage selection/strategy test owner is missing",
        )
        self.assertLess(
            _line_count(NATIVE_DYNAMIC_STAGE_SELECTION_STRATEGY_TEST),
            430,
            "Selection/strategy test owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
