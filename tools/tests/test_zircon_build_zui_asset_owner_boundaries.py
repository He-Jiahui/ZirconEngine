# 核对暂存的 ZUI 资源由其资源入口负责校验。
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/build/zircon_build.py"
ZIRCON_BUILD_ZUI_ASSETS = REPO_ROOT / "tools/build/zircon_build_zui_assets.py"


class ZirconBuildZuiAssetOwnerBoundaryTests(unittest.TestCase):
    def test_staged_zui_asset_checks_live_in_zui_asset_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_ZUI_ASSETS.exists(),
            "staged .zui asset validation belongs in zircon_build_zui_assets.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        zui_assets_text = ZIRCON_BUILD_ZUI_ASSETS.read_text(encoding="utf-8")

        from tools.build import zircon_build_asset_staging, zircon_build_zui_assets

        self.assertIs(
            zircon_build_asset_staging.validate_staged_engine_asset_suffix,
            zircon_build_zui_assets.validate_staged_engine_asset_suffix,
            "asset staging should consume staged .zui validation through its owner",
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
