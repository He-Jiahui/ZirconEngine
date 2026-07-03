"""Boundary tests for CookAssets/Pack stage test ownership."""

from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COOK_ASSETS_PACK_STAGE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_cook_assets_pack_stage.py"
)
COOK_ASSETS_PROJECT_FALLBACK_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_cook_assets_project_fallback.py"
)
PACK_STAGE_CLI_TEST = REPO_ROOT / "tools/zircon_export/tests/test_pack_stage_cli.py"

PROJECT_FALLBACK_TEST_METHODS = (
    "test_cook_assets_derives_project_default_scene_without_manifest",
    "test_cook_assets_project_fallback_records_direct_res_asset_references",
    "test_cook_assets_project_fallback_rejects_missing_direct_reference",
    "test_cook_assets_project_fallback_rejects_unsafe_direct_reference",
    "test_cook_assets_rejects_project_default_scene_source_resolve_error",
    "test_cook_assets_rejects_asset_manifest_directory",
    "test_cook_assets_rejects_project_manifest_directory",
    "test_cook_assets_reports_missing_project_default_scene_source",
)

PACK_STAGE_CLI_TEST_METHODS = (
    "test_pack_defaults_to_cook_assets_manifest",
    "test_pack_command_forwards_profile_to_packer",
    "test_pack_requires_bundle_strategy",
    "test_pack_reports_missing_asset_manifest_before_packer",
    "test_pack_delta_args_are_forwarded_to_packer",
    "test_pack_rejects_unpaired_previous_pack",
    "test_pack_rejects_empty_delta_pack_argument",
    "test_pack_rejects_previous_pack_resolve_error",
    "test_pack_rejects_delta_pack_resolve_error",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return text.count("\n") + int(bool(text))


class CookAssetsPackStageTestOwnerBoundaryTests(unittest.TestCase):
    def test_project_fallback_tests_have_dedicated_owner(self):
        self.assertTrue(
            COOK_ASSETS_PROJECT_FALLBACK_TEST.exists(),
            "CookAssets project fallback test owner is missing",
        )

        root_text = COOK_ASSETS_PACK_STAGE_TEST.read_text(encoding="utf-8")
        fallback_text = COOK_ASSETS_PROJECT_FALLBACK_TEST.read_text(encoding="utf-8")

        for method_name in PROJECT_FALLBACK_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "CookAssets root test should not own project fallback behavior",
                )
                self.assertIn(
                    f"def {method_name}(",
                    fallback_text,
                    "CookAssets project fallback owner is missing coverage",
                )

    def test_pack_stage_cli_tests_have_dedicated_owner(self):
        self.assertTrue(
            PACK_STAGE_CLI_TEST.exists(),
            "Pack stage CLI test owner is missing",
        )

        root_text = COOK_ASSETS_PACK_STAGE_TEST.read_text(encoding="utf-8")
        pack_text = PACK_STAGE_CLI_TEST.read_text(encoding="utf-8")

        for method_name in PACK_STAGE_CLI_TEST_METHODS:
            with self.subTest(method=method_name):
                self.assertNotIn(
                    f"def {method_name}(",
                    root_text,
                    "CookAssets root test should not own Pack CLI behavior",
                )
                self.assertIn(
                    f"def {method_name}(",
                    pack_text,
                    "Pack stage CLI owner is missing coverage",
                )

    def test_cook_assets_root_keeps_manifest_and_strategy_tests(self):
        root_text = COOK_ASSETS_PACK_STAGE_TEST.read_text(encoding="utf-8")
        fallback_text = (
            COOK_ASSETS_PROJECT_FALLBACK_TEST.read_text(encoding="utf-8")
            if COOK_ASSETS_PROJECT_FALLBACK_TEST.exists()
            else ""
        )
        pack_text = (
            PACK_STAGE_CLI_TEST.read_text(encoding="utf-8")
            if PACK_STAGE_CLI_TEST.exists()
            else ""
        )

        for method_name in (
            "test_cook_assets_stage_writes_default_manifest_and_report",
            "test_pipeline_cook_assets_uses_validate_report_asset_filter",
            "test_stage_cook_assets_rejects_invalid_strategy_metadata",
            "test_cook_assets_dry_run_rejects_empty_explicit_asset_filter",
        ):
            with self.subTest(method=method_name):
                self.assertIn(f"def {method_name}(", root_text)
                self.assertNotIn(f"def {method_name}(", fallback_text)
                self.assertNotIn(f"def {method_name}(", pack_text)

    def test_cook_assets_pack_stage_test_owners_stay_small(self):
        self.assertLess(
            _line_count(COOK_ASSETS_PACK_STAGE_TEST),
            850,
            "CookAssets root test should stay focused on manifest and strategy gates",
        )
        self.assertTrue(
            COOK_ASSETS_PROJECT_FALLBACK_TEST.exists(),
            "CookAssets project fallback test owner is missing",
        )
        self.assertLess(
            _line_count(COOK_ASSETS_PROJECT_FALLBACK_TEST),
            860,
            "CookAssets project fallback test owner should stay focused",
        )
        self.assertTrue(
            PACK_STAGE_CLI_TEST.exists(),
            "Pack stage CLI test owner is missing",
        )
        self.assertLess(
            _line_count(PACK_STAGE_CLI_TEST),
            300,
            "Pack stage CLI test owner should stay leaf-sized",
        )


if __name__ == "__main__":
    unittest.main()
