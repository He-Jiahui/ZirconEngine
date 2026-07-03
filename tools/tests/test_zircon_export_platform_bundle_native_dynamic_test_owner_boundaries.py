"""Boundary tests for PlatformBundle NativeDynamic test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLATFORM_BUNDLE_NATIVE_DYNAMIC_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_platform_bundle_native_dynamic.py"
)
PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_platform_bundle_native_dynamic_pipeline_payload.py"
)
PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_platform_bundle_strategy_validation.py"
)

PIPELINE_PAYLOAD_TEST_METHODS = (
    "test_pipeline_platform_bundle_uses_native_dynamic_report_plugins",
    "test_pipeline_platform_bundle_rejects_inherited_native_dynamic_report_directory",
    "test_pipeline_platform_bundle_rejects_profile_mismatch_native_dynamic_report",
    "test_pipeline_platform_bundle_rejects_invalid_native_dynamic_metadata",
    "test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash",
    "test_pipeline_platform_bundle_rejects_native_payload_destination_summary_resolve_error",
    "test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash",
    "test_pipeline_platform_bundle_requires_native_dynamic_payload_for_native_dynamic_profile",
)

STRATEGY_VALIDATION_TEST_METHODS = (
    "test_platform_bundle_rejects_invalid_validate_metadata_for_strategy",
    "test_platform_bundle_explicit_native_dir_rejects_invalid_validate_metadata",
    "test_platform_bundle_explicit_native_dir_requires_native_dynamic_strategy",
    "test_platform_bundle_staged_native_plugins_require_native_dynamic_strategy",
    "test_platform_bundle_rejects_unknown_validate_strategy",
    "test_platform_bundle_rejects_empty_validate_strategies",
    "test_platform_bundle_rejects_invalid_validate_strategies",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PlatformBundleNativeDynamicTestOwnerBoundaryTests(unittest.TestCase):
    def test_pipeline_payload_tests_have_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST.exists(),
            "PlatformBundle NativeDynamic pipeline payload test owner is missing",
        )

        root_text = PLATFORM_BUNDLE_NATIVE_DYNAMIC_TEST.read_text(encoding="utf-8")
        payload_text = PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in PIPELINE_PAYLOAD_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic root test should not own pipeline payload behavior",
                )
                self.assertIn(
                    f"def {method_name}(",
                    payload_text,
                    "Pipeline payload owner is missing coverage",
                )

    def test_strategy_validation_tests_have_dedicated_owner(self):
        self.assertTrue(
            PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST.exists(),
            "PlatformBundle strategy validation test owner is missing",
        )

        root_text = PLATFORM_BUNDLE_NATIVE_DYNAMIC_TEST.read_text(encoding="utf-8")
        strategy_text = PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in STRATEGY_VALIDATION_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "NativeDynamic root test should not own strategy validation gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    strategy_text,
                    "Strategy validation owner is missing coverage",
                )

    def test_native_dynamic_root_keeps_direct_bundle_tests(self):
        root_text = PLATFORM_BUNDLE_NATIVE_DYNAMIC_TEST.read_text(encoding="utf-8")
        payload_text = (
            PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST.read_text(
                encoding="utf-8"
            )
            if PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST.exists()
            else ""
        )
        strategy_text = (
            PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST.read_text(encoding="utf-8")
            if PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST.exists()
            else ""
        )

        for method_name in (
            "test_platform_bundle_copies_native_dynamic_plugins_dir",
            "test_platform_bundle_explicit_native_dir_uses_bundle_plugin_paths",
            "test_platform_bundle_native_plugins_replaces_template_plugins_dir",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", payload_text)
                self.assertNotIn(f"def {method_name}(", strategy_text)

    def test_platform_bundle_native_dynamic_test_owners_stay_small(self):
        self.assertLess(
            _line_count(PLATFORM_BUNDLE_NATIVE_DYNAMIC_TEST),
            650,
            "NativeDynamic root bundle test should stay focused on direct bundling",
        )
        self.assertTrue(
            PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST.exists(),
            "PlatformBundle NativeDynamic pipeline payload test owner is missing",
        )
        self.assertLess(
            _line_count(PLATFORM_BUNDLE_NATIVE_DYNAMIC_PIPELINE_PAYLOAD_TEST),
            560,
            "Pipeline payload test owner should stay focused",
        )
        self.assertTrue(
            PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST.exists(),
            "PlatformBundle strategy validation test owner is missing",
        )
        self.assertLess(
            _line_count(PLATFORM_BUNDLE_STRATEGY_VALIDATION_TEST),
            380,
            "Strategy validation test owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
