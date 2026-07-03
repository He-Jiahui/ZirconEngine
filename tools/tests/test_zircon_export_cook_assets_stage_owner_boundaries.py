import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COOK_ASSETS_STAGE = REPO_ROOT / "tools/zircon_export/cook_assets.py"
COOK_ASSETS_MANIFEST = REPO_ROOT / "tools/zircon_export/cook_assets_manifest.py"


class ZirconExportCookAssetsStageOwnerBoundaryTests(unittest.TestCase):
    def test_asset_manifest_diagnostics_live_in_manifest_owner(self):
        self.assertTrue(
            COOK_ASSETS_MANIFEST.exists(),
            "CookAssets asset manifest diagnostics need a dedicated owner",
        )
        stage_text = COOK_ASSETS_STAGE.read_text(encoding="utf-8")
        manifest_text = COOK_ASSETS_MANIFEST.read_text(encoding="utf-8")

        for function_name in (
            "load_cooked_asset_manifest",
            "validate_asset_manifest_shape",
            "normalized_cooked_asset_manifest",
            "validate_asset_sources_exist",
            "manifest_with_default_asset_filter",
            "validate_asset_manifest_reference_closure",
            "safe_normalized_manifest_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the CookAssets asset manifest owner",
            )
            self.assertIn(f"def {function_name}(", manifest_text)

        self.assertIn(
            "from .cook_assets_manifest import",
            stage_text,
            "CookAssets stage runner should consume the asset manifest owner",
        )
        self.assertNotIn(
            "from .cook_assets import",
            manifest_text,
            "CookAssets asset manifest owner must not import the stage runner",
        )

    def test_cook_assets_stage_runner_stays_under_large_file_threshold(self):
        line_count = len(COOK_ASSETS_STAGE.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            560,
            "CookAssets stage runner should stay below 560 lines after manifest split",
        )

    def test_cook_assets_manifest_owner_stays_leaf_sized(self):
        self.assertTrue(
            COOK_ASSETS_MANIFEST.exists(),
            "CookAssets manifest owner should exist before its size can be checked",
        )
        line_count = len(COOK_ASSETS_MANIFEST.read_text(encoding="utf-8").splitlines())
        self.assertLess(
            line_count,
            360,
            "CookAssets manifest owner should stay below 360 lines",
        )


if __name__ == "__main__":
    unittest.main()
