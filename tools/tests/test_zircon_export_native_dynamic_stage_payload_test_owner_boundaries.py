"""Boundary tests for NativeDynamic stage payload test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_PAYLOAD_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_payload.py"
)
LOADER_MANIFEST_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_native_dynamic_stage_loader_manifest.py"
)

LOADER_MANIFEST_TEST_METHODS = (
    "test_report_stage_rejects_native_dynamic_loader_manifest_path_mismatch",
    "test_report_stage_rejects_native_dynamic_loader_manifest_abi_mismatch",
    "test_report_stage_rejects_native_dynamic_loader_manifest_abi_missing_required_field",
    "test_report_stage_rejects_native_dynamic_loader_manifest_missing_plugins_table",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class NativeDynamicStagePayloadTestOwnerBoundaryTests(unittest.TestCase):
    def test_loader_manifest_tests_have_dedicated_owner(self):
        self.assertTrue(
            LOADER_MANIFEST_TEST.exists(),
            "NativeDynamic stage loader-manifest test owner is missing",
        )

        root_text = STAGE_PAYLOAD_TEST.read_text(encoding="utf-8")
        loader_text = LOADER_MANIFEST_TEST.read_text(encoding="utf-8")

        for method_name in LOADER_MANIFEST_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Stage payload root test should not own loader-manifest gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    loader_text,
                    "Loader-manifest owner is missing coverage",
                )

    def test_native_dynamic_stage_payload_root_keeps_build_execution_tests(self):
        root_text = STAGE_PAYLOAD_TEST.read_text(encoding="utf-8")
        loader_text = (
            LOADER_MANIFEST_TEST.read_text(encoding="utf-8")
            if LOADER_MANIFEST_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_stage_rejects_native_dynamic_schema_before_payload_semantics",
            "test_report_stage_rejects_native_dynamic_build_execution_copied_artifact_mismatch",
            "test_report_stage_rejects_native_dynamic_build_execution_command_mismatch",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", loader_text)

    def test_native_dynamic_stage_payload_test_owners_stay_small(self):
        self.assertLess(
            _line_count(STAGE_PAYLOAD_TEST),
            620,
            "NativeDynamic stage payload root test should stay focused",
        )
        self.assertTrue(
            LOADER_MANIFEST_TEST.exists(),
            "NativeDynamic stage loader-manifest test owner is missing",
        )
        self.assertLess(
            _line_count(LOADER_MANIFEST_TEST),
            620,
            "NativeDynamic stage loader-manifest owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
