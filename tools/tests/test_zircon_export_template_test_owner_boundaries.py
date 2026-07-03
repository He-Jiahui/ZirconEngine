"""Boundary tests for export-template test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
EXPORT_TEMPLATE_VALIDATION_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_templates.py"
)
PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_platform_bundle_template_resolution.py"
)

PLATFORM_BUNDLE_TEMPLATE_TEST_METHODS = (
    "test_linux_template_materializes_directory_layout",
    "test_platform_bundle_rejects_host_copy_error",
    "test_platform_bundle_rejects_template_copy_source_resolve_error",
    "test_platform_bundle_rejects_bundle_output_path_resolve_error",
    "test_platform_bundle_rejects_bundle_manifest_write_error",
    "test_macos_template_materializes_app_bundle_layout",
    "test_template_root_resolves_compatible_platform_bundle_template",
    "test_template_root_rejects_workspace_manifest_directory",
    "test_template_root_skips_manifest_directory_candidate",
    "test_template_root_ignores_target_platform_from_wrong_profile_validate_report",
    "test_template_root_skips_invalid_matching_template_candidate",
    "test_template_root_skips_matching_candidate_with_blank_profile_entry",
    "test_template_root_skips_malformed_template_manifest",
    "test_template_root_reports_missing_profile_match",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class ExportTemplateTestOwnerBoundaryTests(unittest.TestCase):
    def test_platform_bundle_template_tests_have_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST.exists(),
            "PlatformBundle template resolution test owner is missing",
        )

        template_validation_text = EXPORT_TEMPLATE_VALIDATION_TEST.read_text(
            encoding="utf-8"
        )
        platform_bundle_text = (
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST.read_text(encoding="utf-8")
        )

        for method_name in PLATFORM_BUNDLE_TEMPLATE_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    template_validation_text,
                    "Export template validation test should not own PlatformBundle behavior",
                )
                self.assertIn(
                    f"def {method_name}(",
                    platform_bundle_text,
                    "PlatformBundle template resolution owner is missing coverage",
                )

    def test_export_template_validation_keeps_manifest_contract_tests(self):
        template_validation_text = EXPORT_TEMPLATE_VALIDATION_TEST.read_text(
            encoding="utf-8"
        )
        platform_bundle_text = (
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST.read_text(encoding="utf-8")
            if PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST.exists()
            else ""
        )

        for method_name in (
            "test_template_rejects_unknown_manifest_fields",
            "test_valid_template_resolves_declared_host",
            "test_template_rejects_aliasing_file_and_host_paths",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", template_validation_text)
                self.assertNotIn(f"def {method_name}(", platform_bundle_text)

    def test_export_template_test_owners_stay_small(self):
        self.assertLess(
            _line_count(EXPORT_TEMPLATE_VALIDATION_TEST),
            930,
            "Export template validation test should stay below split budget",
        )
        self.assertTrue(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST.exists(),
            "PlatformBundle template resolution test owner is missing",
        )
        self.assertLess(
            _line_count(PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_TEST),
            680,
            "PlatformBundle template resolution test owner should stay leaf-sized",
        )


if __name__ == "__main__":
    unittest.main()
