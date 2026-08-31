import unittest
from pathlib import Path

from tools.zircon_build_plugin_manifest_contract import (
    PLUGIN_DISTRIBUTION_FORM_DIST,
    PLUGIN_DISTRIBUTION_FORM_EMBED,
)
from tools.zircon_build_plugin_packages import CargoPackage, PluginPackage


class _CountingCrates(tuple):
    def __new__(cls, values):
        instance = super().__new__(cls, values)
        instance.iterations = 0
        return instance

    def __iter__(self):
        self.iterations += 1
        return super().__iter__()


class Tooling08PluginPackageProjectionCachePerformanceContractTests(unittest.TestCase):
    def test_repeated_carrier_projections_scan_the_crates_once(self) -> None:
        native = CargoPackage(
            name="plugin_native",
            member="plugin/native",
            manifest_path=Path("plugin/native/Cargo.toml"),
            crate_types=("cdylib",),
        )
        runtime = CargoPackage(
            name="plugin_runtime",
            member="plugin/runtime",
            manifest_path=Path("plugin/runtime/Cargo.toml"),
            crate_types=("rlib",),
        )
        crates = _CountingCrates((native, runtime))
        package = PluginPackage(
            plugin_id="plugin",
            display_name="Plugin",
            manifest_path=Path("plugin/plugin.toml"),
            package_root=Path("plugin"),
            asset_roots=(),
            default_packaging=(),
            distribution_forms=(
                PLUGIN_DISTRIBUTION_FORM_DIST,
                PLUGIN_DISTRIBUTION_FORM_EMBED,
            ),
            dist_crate_name="plugin_native",
            module_crate_names=("plugin_native", "plugin_runtime"),
            shader_geometry_source_ids=(),
            shader_geometry_source_descriptors=(),
            shader_shading_model_ids=(),
            shader_shading_model_descriptors=(),
            crates=crates,
        )

        for _ in range(8):
            self.assertEqual((native,), package.native_dynamic_crates)
            self.assertEqual((runtime,), package.rlib_static_crates)
            self.assertEqual(("native_dynamic", "rlib_static"), package.carriers)

        self.assertEqual(2, crates.iterations)


if __name__ == "__main__":
    unittest.main()
