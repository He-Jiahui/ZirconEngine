"""Boundary tests for PlatformBundle template report schema test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_REPORT_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_schema.py"
)
TEMPLATE_REPORT_MANIFEST_FILE_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_files.py"
)
TEMPLATE_REPORT_MANIFEST_IDENTITY_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_manifest_identity.py"
)
TEMPLATE_REPORT_SEMANTICS_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_platform_bundle_template_report_semantics.py"
)

MANIFEST_FILE_TEST_METHODS = (
    "test_report_rejects_template_report_host_executable_not_declared_file",
    "test_report_rejects_template_report_host_executable_missing_file",
    "test_report_rejects_template_report_manifest_path_mismatch",
    "test_report_rejects_template_report_manifest_missing_file",
    "test_report_rejects_template_report_manifest_invalid_toml",
    "test_report_rejects_template_report_manifest_format_version_mismatch",
    "test_report_rejects_template_report_manifest_template_id_mismatch",
)

MANIFEST_IDENTITY_TEST_METHODS = (
    "test_report_rejects_template_report_manifest_host_artifact_mismatch",
    "test_report_rejects_template_report_manifest_engine_version_mismatch",
    "test_report_rejects_template_report_manifest_target_platform_mismatch",
    "test_report_rejects_template_report_manifest_strategy_field_mismatch",
    "test_report_rejects_template_report_manifest_content_hash_mismatch",
    "test_report_rejects_template_report_manifest_compatible_profiles_mismatch",
    "test_report_rejects_template_report_manifest_host_executable_mismatch",
    "test_report_rejects_template_report_manifest_bundle_field_mismatch",
)

REPORT_SEMANTICS_TEST_METHODS = (
    "test_report_rejects_template_report_missing_profile_membership",
    "test_report_rejects_template_report_duplicate_compatible_profile_entry",
    "test_report_rejects_template_report_padded_duplicate_compatible_profile_before_uniqueness",
    "test_report_rejects_template_report_enum_field_unknown_value",
    "test_report_rejects_template_report_engine_version_mismatch",
    "test_report_rejects_template_report_target_platform_mismatch",
    "test_report_rejects_template_report_content_hash_mismatch",
    "test_report_rejects_template_report_hash_field_malformed",
    "test_report_rejects_template_report_format_version_mismatch",
    "test_report_rejects_template_report_string_field_blank",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleTemplateReportSchemaTestOwnerBoundaryTests(unittest.TestCase):
    def _assert_methods_moved(
        self,
        owner_path: Path,
        method_names: tuple[str, ...],
        description: str,
    ) -> None:
        self.assertTrue(owner_path.exists(), f"{description} owner is missing")
        root_text = TEMPLATE_REPORT_SCHEMA_TEST.read_text(encoding="utf-8")
        owner_text = owner_path.read_text(encoding="utf-8")

        for method_name in method_names:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    f"Template report schema root should not own {description}",
                )
                self.assertIn(
                    f"def {method_name}(",
                    owner_text,
                    f"{description} owner is missing coverage",
                )

    def test_manifest_file_tests_have_dedicated_owner(self):
        self._assert_methods_moved(
            TEMPLATE_REPORT_MANIFEST_FILE_TEST,
            MANIFEST_FILE_TEST_METHODS,
            "manifest file semantics",
        )

    def test_manifest_identity_tests_have_dedicated_owner(self):
        self._assert_methods_moved(
            TEMPLATE_REPORT_MANIFEST_IDENTITY_TEST,
            MANIFEST_IDENTITY_TEST_METHODS,
            "manifest identity semantics",
        )

    def test_report_semantics_tests_have_dedicated_owner(self):
        self._assert_methods_moved(
            TEMPLATE_REPORT_SEMANTICS_TEST,
            REPORT_SEMANTICS_TEST_METHODS,
            "report semantics",
        )

    def test_template_report_schema_root_keeps_shape_tests(self):
        root_text = TEMPLATE_REPORT_SCHEMA_TEST.read_text(encoding="utf-8")
        moved_texts = [
            path.read_text(encoding="utf-8") if path.exists() else ""
            for path in (
                TEMPLATE_REPORT_MANIFEST_FILE_TEST,
                TEMPLATE_REPORT_MANIFEST_IDENTITY_TEST,
                TEMPLATE_REPORT_SEMANTICS_TEST,
            )
        ]

        for method_name in (
            "test_report_rejects_template_report_missing_success_evidence_field",
            "test_report_rejects_template_report_padded_required_string_field",
            "test_report_rejects_template_report_fatal_without_diagnostics",
            "test_report_rejects_template_report_non_fatal_with_diagnostics",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                for moved_text in moved_texts:
                    self.assertNotIn(f"def {method_name}(", moved_text)

    def test_template_report_schema_test_owners_stay_small(self):
        self.assertLess(
            _line_count(TEMPLATE_REPORT_SCHEMA_TEST),
            340,
            "Template report schema root test should stay focused on shape",
        )
        for path, budget, description in (
            (
                TEMPLATE_REPORT_MANIFEST_FILE_TEST,
                300,
                "manifest file semantics",
            ),
            (
                TEMPLATE_REPORT_MANIFEST_IDENTITY_TEST,
                430,
                "manifest identity semantics",
            ),
            (
                TEMPLATE_REPORT_SEMANTICS_TEST,
                520,
                "report semantics",
            ),
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
