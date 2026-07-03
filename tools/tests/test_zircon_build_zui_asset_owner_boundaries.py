import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_ASSET_STAGING = REPO_ROOT / "tools/zircon_build_asset_staging.py"
ZIRCON_BUILD_ZUI_ASSETS = REPO_ROOT / "tools/zircon_build_zui_assets.py"


class ZirconBuildZuiAssetOwnerBoundaryTests(unittest.TestCase):
    def test_staged_zui_asset_checks_live_in_zui_asset_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_ZUI_ASSETS.exists(),
            "staged .zui asset validation belongs in zircon_build_zui_assets.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        asset_staging_text = ZIRCON_BUILD_ASSET_STAGING.read_text(encoding="utf-8")
        zui_assets_text = ZIRCON_BUILD_ZUI_ASSETS.read_text(encoding="utf-8")

        self.assertIn(
            "from .zircon_build_asset_staging import (",
            build_text,
            "zircon_build.py should consume staged assets through the asset staging owner",
        )
        self.assertIn(
            "from zircon_build_asset_staging import (",
            build_text,
            "direct zircon_build.py execution should use the same asset staging owner",
        )
        self.assertIn(
            "from .zircon_build_zui_assets import validate_staged_engine_asset_suffix",
            asset_staging_text,
            "asset staging should consume staged .zui validation through its owner",
        )
        self.assertIn(
            "from zircon_build_zui_assets import validate_staged_engine_asset_suffix",
            asset_staging_text,
            "direct asset staging execution should use the same staged .zui owner",
        )
        self.assertIn("def validate_staged_engine_asset_suffix(", zui_assets_text)
        self.assertIn("validate_plugin_distribution_zui_asset", zui_assets_text)
        for symbol in (
            "tomllib.loads",
            "PLUGIN_VALIDATE_ZUI_ASSET_KINDS",
            "matched .zui asset",
            "asset.kind",
        ):
            self.assertNotIn(symbol, build_text)
            self.assertNotIn(symbol, zui_assets_text)
        self.assertLessEqual(
            len(zui_assets_text.splitlines()),
            80,
            "staged .zui asset owner should stay narrow",
        )


if __name__ == "__main__":
    unittest.main()
