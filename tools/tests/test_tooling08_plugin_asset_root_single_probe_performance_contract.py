import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_build_plugin_assets import collect_plugin_asset_roots


class Tooling08PluginAssetRootSingleProbePerformanceContractTests(unittest.TestCase):
    def test_each_asset_root_uses_one_directory_probe(self) -> None:
        manifest = Path("plugin/plugin.toml")
        roots = [Path(f"plugin/assets_{index}") for index in range(64)]
        exists_calls = 0
        directory_calls = 0

        def exists(_path: Path) -> bool:
            nonlocal exists_calls
            exists_calls += 1
            return True

        def is_dir(_path: Path) -> bool:
            nonlocal directory_calls
            directory_calls += 1
            return True

        with (
            mock.patch(
                "tools.zircon_build_plugin_assets.validate_plugin_distribution_assets_for_build"
            ),
            mock.patch(
                "tools.zircon_build_plugin_assets.normalized_plugin_asset_root",
                side_effect=roots,
            ),
            mock.patch.object(Path, "exists", exists),
            mock.patch.object(Path, "is_dir", is_dir),
        ):
            actual = collect_plugin_asset_roots(
                manifest,
                {"asset_roots": [str(index) for index in range(64)]},
                {},
                "plugin",
            )

        self.assertEqual(tuple(roots), actual)
        self.assertEqual(0, exists_calls)
        self.assertEqual(64, directory_calls)


if __name__ == "__main__":
    unittest.main()
