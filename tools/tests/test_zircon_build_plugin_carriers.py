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
                    "legacy_cdylib/native",
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
            self._write_crate(
                plugins_root / "legacy_cdylib/native",
                "zircon_plugin_legacy_cdylib_native",
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
            self._write_plugin(
                plugins_root / "legacy_cdylib/plugin.toml",
                "legacy_cdylib",
                "zircon_plugin_legacy_cdylib_native",
                "",
            )

            packages = {
                package.plugin_id: package
                for package in zircon_build.discover_plugins(repo_root)
            }

        self.assertEqual(("native_dynamic",), packages["dist_only"].carriers)
        self.assertEqual(("rlib_static",), packages["embed_only"].carriers)
        self.assertEqual(("native_dynamic",), packages["legacy_cdylib"].carriers)
        self.assertEqual(
            {"dist_only", "legacy_cdylib"},
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
