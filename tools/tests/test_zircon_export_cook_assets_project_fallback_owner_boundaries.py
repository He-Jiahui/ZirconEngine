import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COOK_ASSETS = REPO_ROOT / "tools/zircon_export/cook_assets.py"
COOK_ASSETS_PROJECT_FALLBACK = (
    REPO_ROOT / "tools/zircon_export/cook_assets_project_fallback.py"
)


class ZirconExportCookAssetsProjectFallbackOwnerBoundaryTests(unittest.TestCase):
    def test_project_fallback_helpers_live_in_dedicated_owner(self):
        self.assertTrue(
            COOK_ASSETS_PROJECT_FALLBACK.exists(),
            "CookAssets project manifest fallback and res:// closure need a dedicated owner",
        )
        stage_text = COOK_ASSETS.read_text(encoding="utf-8")
        project_fallback_text = COOK_ASSETS_PROJECT_FALLBACK.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "project_default_scene_manifest",
            "project_asset_manifest_path",
            "project_direct_reference_assets",
            "project_direct_res_asset_references",
            "project_asset_reference_package_path",
            "load_project_manifest",
            "project_asset_package_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                stage_text,
                f"{function_name} belongs in the CookAssets project fallback owner",
            )
            self.assertIn(f"def {function_name}(", project_fallback_text)

        self.assertNotIn(
            "RES_ASSET_REFERENCE_RE =",
            stage_text,
            "res:// reference parsing belongs with CookAssets project fallback",
        )
        self.assertIn("RES_ASSET_REFERENCE_RE =", project_fallback_text)
        self.assertIn(
            "from .cook_assets_project_fallback import",
            stage_text,
            "CookAssets stage runner should consume the project fallback owner",
        )
        self.assertNotIn(
            "from .cook_assets import",
            project_fallback_text,
            "Project fallback owner must not import CookAssets stage orchestration",
        )

    def test_project_fallback_owner_stays_leaf_sized(self):
        stage_line_count = len(COOK_ASSETS.read_text(encoding="utf-8").splitlines())
        project_fallback_line_count = len(
            COOK_ASSETS_PROJECT_FALLBACK.read_text(encoding="utf-8").splitlines()
        )
        self.assertLess(
            stage_line_count,
            260,
            "CookAssets stage runner should stay below 260 lines after project fallback split",
        )
        self.assertLess(
            project_fallback_line_count,
            300,
            "CookAssets project fallback owner should stay below 300 lines",
        )


if __name__ == "__main__":
    unittest.main()
