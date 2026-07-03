import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_PLUGIN_ASSETS = REPO_ROOT / "tools/zircon_build_plugin_assets.py"


class ZirconBuildPluginAssetOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_asset_roots_live_in_plugin_assets_owner(self):
        self.assertTrue(
            ZIRCON_BUILD_PLUGIN_ASSETS.exists(),
            "plugin asset root and distribution.assets build preflight belongs in zircon_build_plugin_assets.py",
        )
        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        asset_text = ZIRCON_BUILD_PLUGIN_ASSETS.read_text(encoding="utf-8")

        self.assertIn(
            "from .zircon_build_plugin_assets import collect_plugin_asset_roots",
            build_text,
        )
        self.assertIn(
            "from zircon_build_plugin_assets import collect_plugin_asset_roots",
            build_text,
        )
        for function_name in (
            "collect_plugin_asset_roots",
            "append_plugin_asset_roots_from_field",
            "append_plugin_asset_roots_from_distribution_assets",
            "distribution_asset_root_text",
            "normalized_plugin_asset_root",
            "validate_plugin_distribution_assets_for_build",
            "unique_asset_roots",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_plugin_assets.py",
            )
            self.assertIn(f"def {function_name}(", asset_text)

        self.assertIn("plugin_validate_distribution_assets", asset_text)
        self.assertIn(
            "plugin_validate_distribution_assets(",
            asset_text,
            "zircon_build plugin asset preflight should reuse PluginValidate distribution.assets semantics",
        )
        self.assertNotIn(
            "validate_plugin_distribution_zui_asset(",
            build_text,
            "zircon_build.py must not own .zui document kind checks",
        )
        self.assertLessEqual(
            len(asset_text.splitlines()),
            150,
            "zircon_build_plugin_assets.py should stay focused on plugin asset root semantics",
        )


if __name__ == "__main__":
    unittest.main()
