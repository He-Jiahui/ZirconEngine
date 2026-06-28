import tempfile
import unittest
from pathlib import Path

from tools import zircon_build


class ZirconBuildPluginCarrierTests(unittest.TestCase):
    def test_zircon_build_classifies_forms_from_manifest(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(
                plugins_root,
                [
                    "dist_only/native",
                    "embed_only/native",
                ],
            )
            self._write_crate(
                plugins_root / "dist_only/native",
                "zircon_plugin_dist_only_native",
                ['"cdylib"'],
            )
            self._write_crate(
                plugins_root / "embed_only/native",
                "zircon_plugin_embed_only_native",
                ['"cdylib"'],
            )
            self._write_plugin(
                plugins_root / "dist_only/plugin.toml",
                "dist_only",
                "zircon_plugin_dist_only_native",
                """
[distribution]
forms = ["dist"]
dist_crate = "zircon_plugin_dist_only_native"
""",
            )
            self._write_plugin(
                plugins_root / "embed_only/plugin.toml",
                "embed_only",
                "zircon_plugin_embed_only_native",
                """
[distribution]
forms = ["embed"]
""",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(("native_dynamic",), packages["dist_only"].carriers)
        self.assertEqual(("rlib_static",), packages["embed_only"].carriers)
        self.assertEqual(
            {"dist_only"},
            {
                package.plugin_id
                for package in zircon_build.filter_plugins_by_carrier(
                    packages.values(),
                    "native_dynamic",
                )
            },
        )
        self.assertEqual(
            {"embed_only"},
            {
                package.plugin_id
                for package in zircon_build.filter_plugins_by_carrier(
                    packages.values(),
                    "rlib_static",
                )
            },
        )

    def test_zircon_build_rejects_plugin_manifest_missing_distribution_forms(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["legacy_cdylib/native"])
            self._write_crate(
                plugins_root / "legacy_cdylib/native",
                "zircon_plugin_legacy_cdylib_native",
                ['"cdylib"'],
            )
            (plugins_root / "legacy_cdylib/plugin.toml").write_text(
                """
id = "legacy_cdylib"
display_name = "legacy_cdylib"

[[modules]]
id = "legacy_cdylib.native"
crate_name = "zircon_plugin_legacy_cdylib_native"
""",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                SystemExit,
                "distribution.forms must be a non-empty array",
            ):
                zircon_build.discover_plugins(repo_root)

    def test_zircon_build_rejects_plugin_manifest_unknown_distribution_form(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["bad_form/native"])
            self._write_crate(
                plugins_root / "bad_form/native",
                "zircon_plugin_bad_form_native",
                ['"cdylib"'],
            )
            self._write_plugin(
                plugins_root / "bad_form/plugin.toml",
                "bad_form",
                "zircon_plugin_bad_form_native",
                """
[distribution]
forms = ["dist", "sidecar"]
dist_crate = "zircon_plugin_bad_form_native"
""",
            )

            with self.assertRaisesRegex(
                SystemExit,
                "distribution.forms\\[2\\] must be one of",
            ):
                zircon_build.discover_plugins(repo_root)

    def test_zircon_build_discovers_plugin_shader_permutation_records(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["virtual_geometry/native"])
            self._write_crate(
                plugins_root / "virtual_geometry/native",
                "zircon_plugin_virtual_geometry_native",
                ['"cdylib"'],
            )
            self._write_plugin(
                plugins_root / "virtual_geometry/plugin.toml",
                "virtual_geometry",
                "zircon_plugin_virtual_geometry_native",
                """
[[shader_permutation.geometry_source_ids]]
token = "custom:virtual_geometry"
id = 4

[[shader_permutation.shading_model_ids]]
token = "custom:toon"
id = 16
""",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            ("custom:virtual_geometry=4",),
            packages["virtual_geometry"].shader_geometry_source_ids,
        )
        self.assertEqual(
            ("custom:toon=16",),
            packages["virtual_geometry"].shader_shading_model_ids,
        )

    def test_zircon_build_discovers_plugin_shading_model_descriptors_as_shader_ids(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["toon/native"])
            self._write_crate(
                plugins_root / "toon/native",
                "zircon_plugin_toon_native",
                ['"cdylib"'],
            )
            self._write_plugin(
                plugins_root / "toon/plugin.toml",
                "toon",
                "zircon_plugin_toon_native",
                """
[[shading_models]]
id = 16
token = "custom:toon"
forward_include = "zr_shading_toon"
gbuffer_encode_include = "zr_gbuffer_encode_toon"
deferred_include = "zr_shade_deferred_toon"
required_channels = 7
""",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            ("custom:toon=16",),
            packages["toon"].shader_shading_model_ids,
        )

    def test_zircon_build_discovers_plugin_geometry_source_descriptors_as_shader_ids(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["virtual_geometry/native"])
            self._write_crate(
                plugins_root / "virtual_geometry/native",
                "zircon_plugin_virtual_geometry_native",
                ['"cdylib"'],
            )
            self._write_plugin(
                plugins_root / "virtual_geometry/plugin.toml",
                "virtual_geometry",
                "zircon_plugin_virtual_geometry_native",
                """
[[geometry_sources]]
id = 4
token = "custom:virtual_geometry"
wgsl_include = "zr_geometry_virtual_geometry.wgsl"
vertex_attributes = ["position", "normal", "uv0"]
required_bindings = []
shader_defines = []
""",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            ("custom:virtual_geometry=4",),
            packages["virtual_geometry"].shader_geometry_source_ids,
        )

    def test_zircon_build_discovers_plugin_asset_roots_for_shader_prewarm(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["toon/native"])
            self._write_crate(
                plugins_root / "toon/native",
                "zircon_plugin_toon_native",
                ['"cdylib"'],
            )
            (plugins_root / "toon/assets").mkdir()
            (plugins_root / "toon/shaders").mkdir()
            (plugins_root / "toon/plugin.toml").write_text(
                """
id = "toon"
display_name = "toon"
asset_roots = ["assets", "shaders", "missing"]

[distribution]
forms = ["embed"]

[[modules]]
id = "toon.native"
crate_name = "zircon_plugin_toon_native"
""",
                encoding="utf-8",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            (
                plugins_root / "toon/assets",
                plugins_root / "toon/shaders",
            ),
            packages["toon"].asset_roots,
        )

    def test_zircon_build_discovers_distribution_assets_as_plugin_asset_roots(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["native_fixture/native"])
            self._write_crate(
                plugins_root / "native_fixture/native",
                "zircon_plugin_native_fixture_native",
                ['"cdylib"'],
            )
            (plugins_root / "native_fixture/assets").mkdir()
            (plugins_root / "native_fixture/plugin.toml").write_text(
                """
id = "native_fixture"
display_name = "native_fixture"

[distribution]
forms = ["dist"]
assets = ["assets/**"]

[[modules]]
id = "native_fixture.native"
crate_name = "zircon_plugin_native_fixture_native"
""",
                encoding="utf-8",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            (plugins_root / "native_fixture/assets",),
            packages["native_fixture"].asset_roots,
        )

    def test_zircon_build_uses_existing_default_plugin_assets_root(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["default_assets/native"])
            self._write_crate(
                plugins_root / "default_assets/native",
                "zircon_plugin_default_assets_native",
                ['"cdylib"'],
            )
            (plugins_root / "default_assets/assets").mkdir()
            self._write_plugin(
                plugins_root / "default_assets/plugin.toml",
                "default_assets",
                "zircon_plugin_default_assets_native",
                "",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(
            (plugins_root / "default_assets/assets",),
            packages["default_assets"].asset_roots,
        )

    def test_zircon_build_rejects_invalid_plugin_asset_roots_shape(self):
        with tempfile.TemporaryDirectory() as tmp:
            repo_root = Path(tmp)
            plugins_root = repo_root / "zircon_plugins"
            plugins_root.mkdir()
            self._write_workspace(plugins_root, ["bad_assets/native"])
            self._write_crate(
                plugins_root / "bad_assets/native",
                "zircon_plugin_bad_assets_native",
                ['"cdylib"'],
            )
            (plugins_root / "bad_assets/plugin.toml").write_text(
                """
id = "bad_assets"
display_name = "bad_assets"
asset_roots = ""

[distribution]
forms = ["embed"]

[[modules]]
id = "bad_assets.native"
crate_name = "zircon_plugin_bad_assets_native"
""",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(SystemExit, "asset_roots must be a list"):
                zircon_build.discover_plugins(repo_root)

    def test_zircon_build_rejects_staged_legacy_ui_document_suffixes(self):
        with tempfile.TemporaryDirectory() as tmp:
            source = Path(tmp) / "assets/ui/editor/panel.ui.toml"

            zircon_build.validate_staged_engine_asset_suffix(
                Path("ui/editor/panel.zui"),
                source.with_suffix(".zui"),
            )
            for relative in (
                Path("ui/editor/panel.ui.toml"),
                Path("ui/editor/panel.v2.ui.toml"),
            ):
                with self.subTest(relative=relative):
                    with self.assertRaisesRegex(
                        SystemExit,
                        "Legacy UI document suffix is not stageable",
                    ):
                        zircon_build.validate_staged_engine_asset_suffix(relative, source)

    def _write_workspace(self, plugins_root: Path, members: list[str]):
        members_toml = "\n".join(f'    "{member}",' for member in members)
        (plugins_root / "Cargo.toml").write_text(
            f"""
[workspace]
members = [
{members_toml}
]
""",
            encoding="utf-8",
        )

    def _write_crate(self, crate_root: Path, package_name: str, crate_types: list[str]):
        crate_root.mkdir(parents=True)
        (crate_root / "Cargo.toml").write_text(
            f"""
[package]
name = "{package_name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = [{", ".join(crate_types)}]
""",
            encoding="utf-8",
        )

    def _write_plugin(
        self,
        manifest_path: Path,
        plugin_id: str,
        crate_name: str,
        distribution: str,
    ):
        distribution = distribution or """
[distribution]
forms = ["embed"]
"""
        if "[distribution]" not in distribution:
            distribution = f"""
[distribution]
forms = ["embed"]
{distribution}
"""
        manifest_path.write_text(
            f"""
id = "{plugin_id}"
display_name = "{plugin_id}"

[[modules]]
id = "{plugin_id}.native"
crate_name = "{crate_name}"
{distribution}
""",
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
