"""Boundary tests for pipeline report stage metadata test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
STAGE_METADATA_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_pipeline_report_stage_metadata.py"
)
STAGE_METADATA_ASSETS_PACK_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_stage_metadata_assets_pack_schema.py"
)
STAGE_METADATA_TEST_SUPPORT = (
    REPO_ROOT / "tools/zircon_export/tests/pipeline_report_stage_metadata_test_support.py"
)
COMPILE_HOST_STAGE_SCHEMA_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_pipeline_report_compile_host_stage_schema.py"
)

ASSETS_PACK_TEST_METHODS = (
    "test_report_stage_rejects_cook_assets_unknown_top_level_field",
    "test_report_stage_rejects_cook_assets_string_fields_non_string",
    "test_report_stage_rejects_cook_assets_manifest_count_mismatch",
    "test_report_stage_rejects_pack_unknown_top_level_field",
    "test_report_stage_rejects_pack_missing_release_evidence_field",
    "test_report_stage_rejects_invalid_validate_metadata_without_defaulting",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class PipelineReportStageMetadataTestOwnerBoundaryTests(unittest.TestCase):
    def test_assets_pack_metadata_tests_have_dedicated_owner(self):
        self.assertTrue(
            STAGE_METADATA_ASSETS_PACK_TEST.exists(),
            "Stage metadata assets/pack test owner is missing",
        )

        root_text = STAGE_METADATA_TEST.read_text(encoding="utf-8")
        assets_pack_text = STAGE_METADATA_ASSETS_PACK_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in ASSETS_PACK_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "Stage metadata root test should not own assets/pack schema gates",
                )
                self.assertIn(
                    f"def {method_name}(",
                    assets_pack_text,
                    "Assets/pack owner is missing coverage",
                )

    def test_stage_metadata_root_keeps_compile_host_schema_tests(self):
        root_text = STAGE_METADATA_TEST.read_text(encoding="utf-8")
        assets_pack_text = (
            STAGE_METADATA_ASSETS_PACK_TEST.read_text(encoding="utf-8")
            if STAGE_METADATA_ASSETS_PACK_TEST.exists()
            else ""
        )

        for method_name in (
            "test_report_stage_rejects_compile_host_unknown_top_level_field",
            "test_report_stage_rejects_compile_host_exit_code_non_integer",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", assets_pack_text)

    def test_compile_host_stage_owner_rejects_legacy_link_plan(self):
        self.assertTrue(
            COMPILE_HOST_STAGE_SCHEMA_TEST.exists(),
            "CompileHost stage schema tests need a dedicated owner",
        )
        root_text = STAGE_METADATA_TEST.read_text(encoding="utf-8")
        compile_host_text = COMPILE_HOST_STAGE_SCHEMA_TEST.read_text(encoding="utf-8")
        method_name = "test_legacy_link_plan_field_is_rejected"

        self.assertNotIn(f"def {method_name}(", root_text)
        self.assertIn(f"def {method_name}(", compile_host_text)
        self.assertIn('report["link_plan"] = {}', compile_host_text)
        self.assertIn("unknown field link_plan", compile_host_text)
        self.assertLess(
            _line_count(COMPILE_HOST_STAGE_SCHEMA_TEST),
            120,
            "CompileHost stage schema test owner should remain focused",
        )

    def test_stage_metadata_test_support_has_dedicated_owner(self):
        self.assertTrue(
            STAGE_METADATA_TEST_SUPPORT.exists(),
            "Stage metadata test support owner is missing",
        )
        for path in (STAGE_METADATA_TEST, STAGE_METADATA_ASSETS_PACK_TEST):
            with self.subTest(path=path.name):
                text = path.read_text(encoding="utf-8") if path.exists() else ""
                self.assertNotIn("def _write_library_embed_reports(", text)

        assets_pack_text = (
            STAGE_METADATA_ASSETS_PACK_TEST.read_text(encoding="utf-8")
            if STAGE_METADATA_ASSETS_PACK_TEST.exists()
            else ""
        )
        self.assertIn(
            "from tools.zircon_export.tests.pipeline_report_stage_metadata_test_support import",
            assets_pack_text,
        )
        self.assertIn("write_library_embed_reports", assets_pack_text)

    def test_stage_metadata_test_owners_stay_small(self):
        self.assertLess(
            _line_count(STAGE_METADATA_TEST),
            700,
            "Stage metadata root test should stay focused on compile_host schema gates",
        )
        for path, budget, description in (
            (STAGE_METADATA_ASSETS_PACK_TEST, 720, "assets/pack schema"),
            (STAGE_METADATA_TEST_SUPPORT, 80, "test support"),
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
