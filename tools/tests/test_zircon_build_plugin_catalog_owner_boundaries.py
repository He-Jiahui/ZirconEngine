import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
ZIRCON_BUILD = REPO_ROOT / "tools/zircon_build.py"
ZIRCON_BUILD_PLUGIN_PACKAGES = REPO_ROOT / "tools/zircon_build_plugin_packages.py"
ZIRCON_BUILD_PLUGIN_SELECTION = REPO_ROOT / "tools/zircon_build_plugin_selection.py"
ZIRCON_BUILD_PLUGIN_WORKSPACE_CRATES = (
    REPO_ROOT / "tools/zircon_build_plugin_workspace_crates.py"
)


class ZirconBuildPluginCatalogOwnerBoundaryTests(unittest.TestCase):
    def test_plugin_catalog_models_selection_and_workspace_crates_have_owners(self):
        for owner_path in (
            ZIRCON_BUILD_PLUGIN_PACKAGES,
            ZIRCON_BUILD_PLUGIN_SELECTION,
            ZIRCON_BUILD_PLUGIN_WORKSPACE_CRATES,
        ):
            self.assertTrue(
                owner_path.exists(),
                f"{owner_path.name} should own a focused plugin catalog responsibility",
            )

        build_text = ZIRCON_BUILD.read_text(encoding="utf-8")
        package_text = ZIRCON_BUILD_PLUGIN_PACKAGES.read_text(encoding="utf-8")
        selection_text = ZIRCON_BUILD_PLUGIN_SELECTION.read_text(encoding="utf-8")
        workspace_text = ZIRCON_BUILD_PLUGIN_WORKSPACE_CRATES.read_text(
            encoding="utf-8"
        )

        for import_name in (
            "from .zircon_build_plugin_packages import PluginPackage",
            "from .zircon_build_plugin_selection import (",
            "from .zircon_build_plugin_workspace_crates import discover_plugin_workspace_crates",
            "from zircon_build_plugin_packages import PluginPackage",
            "from zircon_build_plugin_selection import (",
            "from zircon_build_plugin_workspace_crates import discover_plugin_workspace_crates",
        ):
            self.assertIn(import_name, build_text)

        for class_name in ("CargoPackage", "PluginPackage"):
            self.assertNotIn(
                f"class {class_name}",
                build_text,
                f"{class_name} belongs in zircon_build_plugin_packages.py",
            )
            self.assertIn(f"class {class_name}", package_text)

        self.assertNotIn("def discover_plugin_workspace_crates(", build_text)
        self.assertIn("def discover_plugin_workspace_crates(", workspace_text)

        for function_name in (
            "filter_plugins_by_carrier",
            "select_plugins",
            "select_index",
            "select_range",
            "unique_plugins",
            "print_plugin_catalog",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                build_text,
                f"{function_name} belongs in zircon_build_plugin_selection.py",
            )
            self.assertIn(f"def {function_name}(", selection_text)

        self.assertLessEqual(
            len(package_text.splitlines()),
            120,
            "zircon_build_plugin_packages.py should stay focused on plugin package models",
        )
        self.assertLessEqual(
            len(selection_text.splitlines()),
            150,
            "zircon_build_plugin_selection.py should stay focused on plugin selection",
        )
        self.assertLessEqual(
            len(workspace_text.splitlines()),
            120,
            "zircon_build_plugin_workspace_crates.py should stay focused on plugin workspace crates",
        )

    def test_plugin_catalog_owners_preserve_package_workspace_and_selection_semantics(
        self,
    ):
        from tools.zircon_build_plugin_manifest_contract import (
            PLUGIN_DISTRIBUTION_FORM_DIST,
            PLUGIN_DISTRIBUTION_FORM_EMBED,
        )
        from tools.zircon_build_plugin_packages import CargoPackage, PluginPackage
        from tools.zircon_build_plugin_selection import (
            filter_plugins_by_carrier,
            select_plugins,
        )
        from tools.zircon_build_plugin_workspace_crates import (
            discover_plugin_workspace_crates,
        )

        native_crate = CargoPackage(
            name="zircon_plugin_catalog_native",
            member="catalog/native",
            manifest_path=Path("catalog/native/Cargo.toml"),
            crate_types=("cdylib",),
        )
        rlib_crate = CargoPackage(
            name="zircon_plugin_catalog_runtime",
            member="catalog/runtime",
            manifest_path=Path("catalog/runtime/Cargo.toml"),
            crate_types=("rlib",),
        )
        package = PluginPackage(
            plugin_id="catalog",
            display_name="Catalog",
            manifest_path=Path("catalog/plugin.toml"),
            package_root=Path("catalog"),
            asset_roots=(),
            default_packaging=(),
            distribution_forms=(
                PLUGIN_DISTRIBUTION_FORM_DIST,
                PLUGIN_DISTRIBUTION_FORM_EMBED,
            ),
            dist_crate_name="zircon_plugin_catalog_native",
            module_crate_names=(
                "zircon_plugin_catalog_native",
                "zircon_plugin_catalog_runtime",
            ),
            shader_geometry_source_ids=(),
            shader_geometry_source_descriptors=(),
            shader_shading_model_ids=(),
            shader_shading_model_descriptors=(),
            crates=(native_crate, rlib_crate),
        )

        self.assertEqual((native_crate,), package.native_dynamic_crates)
        self.assertEqual((rlib_crate,), package.rlib_static_crates)
        self.assertEqual(("native_dynamic", "rlib_static"), package.carriers)
        self.assertEqual([package], filter_plugins_by_carrier([package], "all"))
        self.assertEqual(
            [package],
            filter_plugins_by_carrier([package], "native_dynamic"),
        )
        self.assertEqual([package], select_plugins([package], "1,catalog,native"))

        with tempfile.TemporaryDirectory() as tmp:
            plugins_root = Path(tmp) / "zircon_plugins"
            crate_root = plugins_root / "catalog" / "native"
            crate_root.mkdir(parents=True)
            (plugins_root / "Cargo.toml").write_text(
                """
[workspace]
members = ["catalog/native", "missing/member"]
""",
                encoding="utf-8",
            )
            (crate_root / "Cargo.toml").write_text(
                """
[package]
name = "zircon_plugin_catalog_native"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
""",
                encoding="utf-8",
            )

            discovered = discover_plugin_workspace_crates(plugins_root)

        self.assertEqual(1, len(discovered))
        self.assertEqual("zircon_plugin_catalog_native", discovered[0].name)
        self.assertEqual("catalog/native", discovered[0].member)
        self.assertTrue(discovered[0].is_native_dynamic)


if __name__ == "__main__":
    unittest.main()
