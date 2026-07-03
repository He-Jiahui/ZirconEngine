import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_DISTRIBUTION_ASSETS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_assets.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ZUI_ASSETS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_zui_assets.py"
)


class PluginValidateDistributionZuiAssetOwnerBoundaryTests(unittest.TestCase):
    def test_distribution_zui_assets_live_in_zui_assets_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ZUI_ASSETS.exists(),
            ".zui distribution asset validation belongs in plugin_validate_distribution_zui_assets.py",
        )
        assets_text = PLUGIN_VALIDATE_DISTRIBUTION_ASSETS.read_text(encoding="utf-8")
        zui_assets_text = PLUGIN_VALIDATE_DISTRIBUTION_ZUI_ASSETS.read_text(
            encoding="utf-8"
        )

        self.assertIn("from .plugin_validate_distribution_zui_assets import", assets_text)
        self.assertIn("def validate_plugin_distribution_zui_asset(", zui_assets_text)
        for symbol in (
            "PLUGIN_VALIDATE_ZUI_ASSET_KINDS",
            "tomllib.loads",
            "matched .zui asset",
            "asset.kind",
        ):
            self.assertNotIn(symbol, assets_text)
            self.assertIn(symbol, zui_assets_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(forbidden_import, zui_assets_text)
        self.assertLessEqual(len(assets_text.splitlines()), 125)
        self.assertLessEqual(len(zui_assets_text.splitlines()), 95)


if __name__ == "__main__":
    unittest.main()
