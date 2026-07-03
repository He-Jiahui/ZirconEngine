"""Boundary tests for NativeDynamic build/signing test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
NATIVE_DYNAMIC_BUILD_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_native_dynamic_build_signing.py"
)
NATIVE_DYNAMIC_SIGNING_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_native_dynamic_signing_notarization.py"
)

SIGNING_NOTARIZATION_TEST_METHODS = (
    "test_native_dynamic_signs_loadable_artifact_before_manifest_hash",
    "test_native_dynamic_signing_profile_records_platform_gate",
    "test_native_dynamic_signing_rejects_schema_invalid_arguments_before_external_command",
    "test_native_dynamic_signing_profile_rejects_platform_mismatch",
    "test_native_dynamic_notarization_runs_after_signing_before_manifest_hash",
    "test_native_dynamic_notarization_rejects_schema_invalid_arguments_before_external_command",
    "test_native_dynamic_notarization_profile_rejects_platform_mismatch",
    "test_native_dynamic_signing_failure_cleans_staged_payload",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicBuildSigningTestOwnerBoundaryTests(unittest.TestCase):
    def test_signing_and_notarization_tests_have_dedicated_owner(self):
        self.assertTrue(
            NATIVE_DYNAMIC_SIGNING_TEST.exists(),
            "NativeDynamic signing/notarization test owner is missing",
        )

        build_text = NATIVE_DYNAMIC_BUILD_TEST.read_text(encoding="utf-8")
        signing_text = NATIVE_DYNAMIC_SIGNING_TEST.read_text(encoding="utf-8")

        for method_name in SIGNING_NOTARIZATION_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    build_text,
                    "NativeDynamic build test should not own signing/notarization behavior",
                )
                self.assertIn(
                    f"def {method_name}(",
                    signing_text,
                    "NativeDynamic signing/notarization owner is missing coverage",
                )

    def test_native_dynamic_build_test_keeps_plan_and_execution_tests(self):
        build_text = NATIVE_DYNAMIC_BUILD_TEST.read_text(encoding="utf-8")
        signing_text = (
            NATIVE_DYNAMIC_SIGNING_TEST.read_text(encoding="utf-8")
            if NATIVE_DYNAMIC_SIGNING_TEST.exists()
            else ""
        )

        for method_name in (
            "test_native_dynamic_stage_reports_native_cdylib_build_plan",
            "test_native_dynamic_build_plan_records_cargo_features",
            "test_native_dynamic_build_executes_plan_and_stages_cdylib",
            "test_native_dynamic_build_rejects_staged_cdylib_copy_error",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", build_text)
                self.assertNotIn(f"def {method_name}(", signing_text)

    def test_native_dynamic_build_signing_test_owners_stay_small(self):
        self.assertLess(
            _line_count(NATIVE_DYNAMIC_BUILD_TEST),
            960,
            "NativeDynamic build test should stay focused on build plan/execution",
        )
        self.assertTrue(
            NATIVE_DYNAMIC_SIGNING_TEST.exists(),
            "NativeDynamic signing/notarization test owner is missing",
        )
        self.assertLess(
            _line_count(NATIVE_DYNAMIC_SIGNING_TEST),
            540,
            "NativeDynamic signing/notarization test owner should stay leaf-sized",
        )


if __name__ == "__main__":
    unittest.main()
