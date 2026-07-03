from __future__ import annotations

import contextlib
import hashlib
import io
import json
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

from tools.zircon_export.cli import main
from tools.zircon_export.tests.plugin_validate_support import _replace_manifest_line


class PluginBuildTests(unittest.TestCase):
    def test_plugin_build_emits_isolated_package_dir(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            target_dir = root / "target"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_fake_cargo_build_script(repo_root)

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(out),
                        "--target-dir",
                        str(target_dir),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                    ]
                )

            package_dir = out / "native_dynamic_fixture"
            package_report_path = package_dir / "native_dynamic_package.toml"
            cargo_args = json.loads((repo_root / "cargo_args.json").read_text())
            with package_report_path.open("rb") as report_file:
                package_report = tomllib.load(report_file)

            self.assertEqual(exit_code, 0)
            self.assertTrue((package_dir / "plugin.toml").is_file())
            self.assertTrue((package_dir / "native_dynamic_fixture.dll").is_file())
            self.assertTrue(
                (
                    package_dir
                    / "native"
                    / "zircon_plugin_native_dynamic_fixture_native.dll"
                ).is_file()
            )
            self.assertTrue(package_report_path.is_file())
            self.assertFalse((out / "stages").exists())
            self.assertEqual(package_report["package_id"], "native_dynamic_fixture")
            self.assertEqual(package_report["directory"], "native_dynamic_fixture")
            self.assertEqual(package_report["path"], "native_dynamic_fixture")
            self.assertEqual(package_report["manifest"], "native_dynamic_fixture/plugin.toml")
            self.assertEqual(package_report["abi"]["abi_version"], 3)
            self.assertEqual(package_report["payload"]["file_count"], 4)
            self.assertEqual(
                [entry["path"] for entry in package_report["payload"]["files"]],
                [
                    "native/zircon_plugin_native_dynamic_fixture_native.dll",
                    "native_dynamic_fixture.dll",
                    "native_dynamic_fixture.sig",
                    "plugin.toml",
                ],
            )
            self.assertIn("--manifest-path", cargo_args)
            self.assertIn(str(repo_root / "zircon_plugins" / "Cargo.toml"), cargo_args)
            self.assertIn("-p", cargo_args)
            self.assertIn(crate_name, cargo_args)
            self.assertIn("--target-dir", cargo_args)
            self.assertIn(str(target_dir.resolve()), cargo_args)
            self.assertIn("--no-default-features", cargo_args)
            self.assertIn("--features", cargo_args)
            self.assertIn("dist", cargo_args)
            self.assertIn("--locked", cargo_args)
            self.assertIn("--release", cargo_args)

    def test_plugin_build_materializes_feature_provider_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            target_dir = root / "target"
            crate_name = "zircon_plugin_sound_timeline_animation_dist"
            _write_feature_provider_workspace(repo_root, crate_name)
            _write_fake_cargo_build_script(repo_root)

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "sound_timeline_animation_track",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(out),
                        "--target-dir",
                        str(target_dir),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                    ]
                )

            package_dir = out / "sound_timeline_animation_track"
            package_manifest_path = package_dir / "plugin.toml"
            package_report_path = package_dir / "native_dynamic_package.toml"
            loader_manifest_path = out / "native_plugins.toml"
            cargo_args = json.loads((repo_root / "cargo_args.json").read_text())
            with package_manifest_path.open("rb") as manifest_file:
                package_manifest = tomllib.load(manifest_file)
            with package_report_path.open("rb") as report_file:
                package_report = tomllib.load(report_file)
            with loader_manifest_path.open("rb") as manifest_file:
                loader_manifest = tomllib.load(manifest_file)

            self.assertEqual(exit_code, 0)
            self.assertTrue((package_dir / "sound_timeline_animation_track.dll").is_file())
            self.assertTrue((package_dir / "native" / f"{crate_name}.dll").is_file())
            self.assertEqual(package_manifest["id"], "sound_timeline_animation_track")
            self.assertEqual(package_manifest["package_kind"], "feature_extension")
            self.assertEqual(package_manifest["distribution"]["dist_crate"], crate_name)
            self.assertEqual(package_manifest["distribution"]["abi_version"], 3)
            self.assertEqual(
                package_manifest["distribution"]["runtime_entry"],
                "zircon_plugin_sound_timeline_animation_runtime_entry_v3",
            )
            feature_extension = package_manifest["feature_extensions"][0]
            self.assertEqual(feature_extension["id"], "sound.timeline_animation_track")
            self.assertEqual(feature_extension["owner_plugin_id"], "sound")
            self.assertEqual(feature_extension["default_packaging"], ["native_dynamic"])
            self.assertEqual(
                feature_extension["modules"][0]["crate_name"],
                crate_name,
            )
            self.assertEqual(feature_extension["modules"][0]["kind"], "runtime")
            self.assertEqual(package_report["package_id"], "sound_timeline_animation_track")
            self.assertEqual(loader_manifest["plugins"][0]["id"], "sound_timeline_animation_track")
            self.assertIn("-p", cargo_args)
            self.assertIn(crate_name, cargo_args)

    def test_plugin_dist_build_is_byte_reproducible(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_fake_cargo_build_script(repo_root)

            first_package = self._run_plugin_build_for_package(
                repo_root,
                root / "out-a",
                root / "target-a",
            )
            second_package = self._run_plugin_build_for_package(
                repo_root,
                root / "out-b",
                root / "target-b",
            )

            self.assertEqual(
                _package_file_bytes(first_package),
                _package_file_bytes(second_package),
            )

    def test_plugin_build_includes_plugin_zrpack_asset_subpackage(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name, assets=True)
            _write_fake_cargo_build_script(repo_root)
            _write_fake_cargo_pack_script(repo_root)

            package_dir = self._run_plugin_build_for_package(
                repo_root,
                root / "out",
                root / "target",
            )

            package_report_path = package_dir / "native_dynamic_package.toml"
            with package_report_path.open("rb") as report_file:
                package_report = tomllib.load(report_file)
            pack_args = json.loads((repo_root / "cargo_run_args.json").read_text())
            pack_manifest = json.loads(
                (repo_root / "cargo_pack_manifest.json").read_text()
            )

            self.assertTrue((package_dir / "native_dynamic_fixture.zrpack").is_file())
            self.assertIn("--manifest", pack_args)
            self.assertIn("--pack", pack_args)
            self.assertIn("--determinism-check", pack_args)
            self.assertEqual(pack_manifest["roots"], ["assets/shader.wgsl"])
            self.assertEqual(pack_manifest["assets"][0]["path"], "assets/shader.wgsl")
            self.assertTrue(
                Path(pack_manifest["assets"][0]["source"]).is_absolute()
            )
            self.assertEqual(package_report["payload"]["file_count"], 5)
            self.assertEqual(
                [entry["path"] for entry in package_report["payload"]["files"]],
                [
                    "native/zircon_plugin_native_dynamic_fixture_native.dll",
                    "native_dynamic_fixture.dll",
                    "native_dynamic_fixture.sig",
                    "native_dynamic_fixture.zrpack",
                    "plugin.toml",
                ],
            )

    def test_plugin_build_rejects_asset_pack_with_retired_ui_suffixes(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name, assets=True)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            ui_root = plugin_root / "assets" / "ui"
            ui_root.mkdir(parents=True, exist_ok=True)
            (ui_root / "retired_component.ui.toml").write_text(
                '[asset]\nkind = "component"\n',
                encoding="utf-8",
            )
            (ui_root / "retired_panel.v2.ui.toml").write_text(
                '[asset]\nkind = "view"\n',
                encoding="utf-8",
            )
            _write_fake_cargo_build_script(repo_root)
            _write_fake_cargo_pack_script(repo_root)

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(root / "out"),
                        "--target-dir",
                        str(root / "target"),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                    ]
                )

            self.assertEqual(exit_code, 2)
            self.assertFalse(
                (root / "repo" / "cargo_pack_manifest.json").exists(),
                "asset pack command must not run after retired UI asset suffix diagnostics",
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[0] matched retired UI asset suffix assets/ui/retired_component.ui.toml; use .zui",
                output.getvalue(),
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[0] matched retired UI asset suffix assets/ui/retired_panel.v2.ui.toml; use .zui",
                output.getvalue(),
            )

    def test_plugin_build_rejects_asset_pack_with_zui_kind_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name, assets=True)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            ui_root = plugin_root / "assets" / "ui"
            ui_root.mkdir(parents=True, exist_ok=True)
            (ui_root / "bad_kind.zui").write_text(
                '[asset]\nkind = "blueprint"\n',
                encoding="utf-8",
            )
            _write_fake_cargo_build_script(repo_root)
            _write_fake_cargo_pack_script(repo_root)

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(root / "out"),
                        "--target-dir",
                        str(root / "target"),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                    ]
                )

            self.assertEqual(exit_code, 2)
            self.assertFalse(
                (root / "repo" / "cargo_pack_manifest.json").exists(),
                "asset pack command must not run after .zui asset.kind diagnostics",
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[0] "
                "matched .zui asset assets/ui/bad_kind.zui has unsupported "
                "asset.kind blueprint; expected one of component, style, "
                "theme_tokens, view",
                output.getvalue(),
            )

    def test_plugin_build_rejects_asset_pack_with_retired_ui_suffix_patterns(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name, assets=True)
            plugin_root = repo_root / "zircon_plugins" / "native_dynamic_fixture"
            _replace_manifest_line(
                plugin_root / "plugin.toml",
                'assets = ["assets/**"]',
                'assets = ["assets/ui/missing_component.ui.toml", '
                '"assets/ui/missing_panel.v2.ui.toml"]',
            )
            _write_fake_cargo_build_script(repo_root)
            _write_fake_cargo_pack_script(repo_root)

            output = io.StringIO()
            with contextlib.redirect_stdout(output):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(root / "out"),
                        "--target-dir",
                        str(root / "target"),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                    ]
                )

            self.assertEqual(exit_code, 2)
            self.assertFalse(
                (root / "repo" / "cargo_pack_manifest.json").exists(),
                "asset pack command must not run after retired UI asset suffix pattern diagnostics",
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[0] targets retired UI asset suffix assets/ui/missing_component.ui.toml; use .zui",
                output.getvalue(),
            )
            self.assertIn(
                "plugin native_dynamic_fixture distribution.assets[1] targets retired UI asset suffix assets/ui/missing_panel.v2.ui.toml; use .zui",
                output.getvalue(),
            )

    def test_native_plugin_load_manifest_assembles_signed_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            target_dir = root / "target"
            crate_name = "zircon_plugin_native_dynamic_fixture_native"
            _write_dist_plugin_workspace(repo_root, crate_name)
            _write_fake_cargo_build_script(repo_root)
            signer = _write_fake_sign_script(repo_root)

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = main(
                    [
                        "plugin",
                        "build",
                        "native_dynamic_fixture",
                        "--repo-root",
                        str(repo_root),
                        "--out",
                        str(out),
                        "--target-dir",
                        str(target_dir),
                        "--platform",
                        "windows-x86_64",
                        "--mode",
                        "release",
                        "--cargo",
                        sys.executable,
                        "--sign-command",
                        sys.executable,
                        "--sign-arg",
                        str(signer),
                        "--sign-arg",
                        "{artifact}",
                        "--sign-arg",
                        "{package_id}",
                        "--sign-arg",
                        "{target_platform}",
                    ]
                )

            package_dir = out / "native_dynamic_fixture"
            artifact = package_dir / "native_dynamic_fixture.dll"
            native_artifact = (
                package_dir / "native" / "zircon_plugin_native_dynamic_fixture_native.dll"
            )
            package_report_path = package_dir / "native_dynamic_package.toml"
            loader_manifest_path = out / "native_plugins.toml"
            export_loader_manifest_path = out / "plugins" / "native_plugins.toml"
            signature_path = package_dir / "native_dynamic_fixture.sig"
            with package_report_path.open("rb") as report_file:
                package_report = tomllib.load(report_file)
            with loader_manifest_path.open("rb") as manifest_file:
                loader_manifest = tomllib.load(manifest_file)
            with export_loader_manifest_path.open("rb") as manifest_file:
                export_loader_manifest = tomllib.load(manifest_file)
            with signature_path.open("rb") as signature_file:
                signature = tomllib.load(signature_file)

            artifact_hash = hashlib.sha256(artifact.read_bytes()).hexdigest()
            native_artifact_hash = hashlib.sha256(native_artifact.read_bytes()).hexdigest()
            payload_entries = {
                entry["path"]: entry
                for entry in package_report["payload"]["files"]
            }

            self.assertEqual(exit_code, 0)
            self.assertTrue(loader_manifest_path.is_file())
            self.assertTrue(export_loader_manifest_path.is_file())
            self.assertTrue(signature_path.is_file())
            self.assertIn("signed:native_dynamic_fixture:windows-x86_64", artifact.read_text(encoding="utf-8"))
            self.assertEqual(loader_manifest, export_loader_manifest)
            self.assertEqual(loader_manifest["plugins"][0]["id"], "native_dynamic_fixture")
            self.assertEqual(loader_manifest["plugins"][0]["path"], "native_dynamic_fixture")
            self.assertEqual(
                loader_manifest["plugins"][0]["manifest"],
                "native_dynamic_fixture/plugin.toml",
            )
            self.assertEqual(
                loader_manifest["plugins"][0]["package_report"],
                "native_dynamic_fixture/native_dynamic_package.toml",
            )
            self.assertEqual(loader_manifest["plugins"][0]["abi"]["abi_version"], 3)
            self.assertEqual(payload_entries["native_dynamic_fixture.dll"]["sha256"], artifact_hash)
            self.assertEqual(
                payload_entries["native/zircon_plugin_native_dynamic_fixture_native.dll"][
                    "sha256"
                ],
                native_artifact_hash,
            )
            self.assertIn("native_dynamic_fixture.sig", payload_entries)
            self.assertEqual(signature["loadable_artifact_count"], 2)
            self.assertEqual(
                {entry["path"] for entry in signature["loadable_artifacts"]},
                {
                    "native/zircon_plugin_native_dynamic_fixture_native.dll",
                    "native_dynamic_fixture.dll",
                },
            )
            self.assertTrue(signature["signing"]["enabled"])
            self.assertEqual(signature["signing"]["artifact_count"], 2)
            self.assertEqual(
                {entry["after_sha256"] for entry in signature["signing"]["artifacts"]},
                {artifact_hash, native_artifact_hash},
            )

    def _run_plugin_build_for_package(
        self,
        repo_root: Path,
        out: Path,
        target_dir: Path,
    ) -> Path:
        with contextlib.redirect_stdout(io.StringIO()):
            exit_code = main(
                [
                    "plugin",
                    "build",
                    "native_dynamic_fixture",
                    "--repo-root",
                    str(repo_root),
                    "--out",
                    str(out),
                    "--target-dir",
                    str(target_dir),
                    "--platform",
                    "windows-x86_64",
                    "--mode",
                    "release",
                    "--cargo",
                    sys.executable,
                ]
            )
        self.assertEqual(exit_code, 0)
        return out / "native_dynamic_fixture"


def _write_dist_plugin_workspace(
    repo_root: Path,
    crate_name: str,
    *,
    assets: bool = False,
) -> None:
    plugins_root = repo_root / "zircon_plugins"
    plugin_root = plugins_root / "native_dynamic_fixture"
    crate_root = plugin_root / "native"
    crate_root.mkdir(parents=True)
    asset_lines = ['assets = ["assets/**"]'] if assets else []
    (plugins_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                'members = ["native_dynamic_fixture/native"]',
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )
    (plugin_root / "plugin.toml").write_text(
        "\n".join(
            [
                'id = "native_dynamic_fixture"',
                'version = "0.1.0"',
                'display_name = "Native Dynamic Fixture"',
                "",
                "[distribution]",
                'forms = ["dist"]',
                'default_packaging = ["native_dynamic"]',
                "abi_version = 3",
                'engine_compat = ">=0.1, <0.2"',
                f'dist_crate = "{crate_name}"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'runtime_entry = "zircon_native_dynamic_fixture_runtime_entry_v3"',
                'editor_entry = "zircon_native_dynamic_fixture_editor_entry_v3"',
                *asset_lines,
                "",
                "[[modules]]",
                'name = "native_dynamic_fixture.runtime"',
                'kind = "runtime"',
                f'crate_name = "{crate_name}"',
            ]
        ),
        encoding="utf-8",
    )
    if assets:
        asset_path = plugin_root / "assets" / "shader.wgsl"
        asset_path.parent.mkdir(parents=True, exist_ok=True)
        asset_path.write_text("@fragment fn main() {}", encoding="utf-8")
    (crate_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[lib]",
                'crate-type = ["cdylib"]',
                "",
                "[features]",
                'default = ["dist"]',
                "dist = []",
                "",
                "[dependencies]",
                'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
            ]
        ),
        encoding="utf-8",
    )


def _write_feature_provider_workspace(repo_root: Path, crate_name: str) -> None:
    plugins_root = repo_root / "zircon_plugins"
    plugin_root = plugins_root / "sound"
    crate_root = plugin_root / "features" / "timeline_animation_track" / "dist"
    crate_root.mkdir(parents=True)
    (plugins_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[workspace]",
                'members = ["sound/features/timeline_animation_track/dist"]',
                'resolver = "2"',
            ]
        ),
        encoding="utf-8",
    )
    (plugin_root / "plugin.toml").write_text(
        "\n".join(
            [
                'id = "sound"',
                'version = "0.1.0"',
                'display_name = "Sound"',
                'sdk_api_version = "0.1.0"',
                'category = "runtime"',
                'maturity = "beta"',
                'supported_targets = ["client_runtime", "editor_host"]',
                'supported_platforms = ["windows", "linux", "macos"]',
                'capabilities = ["runtime.plugin.sound"]',
                "",
                "[[optional_features]]",
                'id = "sound.timeline_animation_track"',
                'display_name = "Sound Timeline Animation Track"',
                'owner_plugin_id = "sound"',
                'provider_package_id = "sound_timeline_animation_track"',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                'default_packaging = ["source_template", "library_embed", "native_dynamic"]',
                "enabled_by_default = false",
                "",
                "[[optional_features.dependencies]]",
                'plugin_id = "sound"',
                'capability = "runtime.plugin.sound"',
                "primary = true",
                "",
                "[[optional_features.dependencies]]",
                'plugin_id = "animation"',
                'capability = "runtime.feature.animation.timeline_event_track"',
                "primary = false",
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.runtime"',
                'kind = "runtime"',
                'crate_name = "zircon_plugin_sound_timeline_animation_runtime"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[[optional_features.modules]]",
                'name = "sound.timeline_animation_track.dist"',
                'kind = "native"',
                f'crate_name = "{crate_name}"',
                'target_modes = ["client_runtime", "editor_host"]',
                'capabilities = ["runtime.feature.sound.timeline_animation_track"]',
                "",
                "[optional_features.distribution]",
                'forms = ["dist"]',
                'default_packaging = ["native_dynamic"]',
                "abi_version = 3",
                'engine_compat = ">=0.1, <0.2"',
                f'dist_crate = "{crate_name}"',
                'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                'runtime_entry = "zircon_plugin_sound_timeline_animation_runtime_entry_v3"',
            ]
        ),
        encoding="utf-8",
    )
    (crate_root / "Cargo.toml").write_text(
        "\n".join(
            [
                "[package]",
                f'name = "{crate_name}"',
                'version = "0.1.0"',
                'edition = "2021"',
                "",
                "[lib]",
                'crate-type = ["cdylib"]',
                "",
                "[features]",
                'default = ["dist"]',
                "dist = []",
                "",
                "[dependencies]",
                'zircon_plugin_sdk = { workspace = true, default-features = false, features = ["native"] }',
            ]
        ),
        encoding="utf-8",
    )


def _write_fake_cargo_build_script(repo_root: Path) -> None:
    (repo_root / "build").write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import json",
                "import sys",
                "",
                "args = sys.argv[1:]",
                "Path('cargo_args.json').write_text(json.dumps(args), encoding='utf-8')",
                "target_dir = Path(args[args.index('--target-dir') + 1])",
                "crate_name = args[args.index('-p') + 1]",
                "profile = 'release' if '--release' in args else 'debug'",
                "artifact = target_dir / profile / f'{crate_name}.dll'",
                "artifact.parent.mkdir(parents=True, exist_ok=True)",
                "artifact.write_text('built native dynamic artifact', encoding='utf-8')",
            ]
        ),
        encoding="utf-8",
    )


def _write_fake_cargo_pack_script(repo_root: Path) -> None:
    (repo_root / "run").write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import json",
                "import sys",
                "",
                "args = sys.argv[1:]",
                "Path('cargo_run_args.json').write_text(json.dumps(args), encoding='utf-8')",
                "packer_args = args[args.index('--') + 1:]",
                "manifest = Path(packer_args[packer_args.index('--manifest') + 1])",
                "pack = Path(packer_args[packer_args.index('--pack') + 1])",
                "report = Path(packer_args[packer_args.index('--report') + 1])",
                "Path('cargo_pack_manifest.json').write_text(manifest.read_text(encoding='utf-8'), encoding='utf-8')",
                "pack.parent.mkdir(parents=True, exist_ok=True)",
                "pack.write_bytes(b'fake zrpack bytes')",
                "report.parent.mkdir(parents=True, exist_ok=True)",
                "report.write_text(json.dumps({'fatal': False, 'asset_count': 1, 'chunk_count': 1}), encoding='utf-8')",
            ]
        ),
        encoding="utf-8",
    )


def _write_fake_sign_script(repo_root: Path) -> Path:
    signer = repo_root / "sign.py"
    signer.write_text(
        "\n".join(
            [
                "from pathlib import Path",
                "import sys",
                "",
                "artifact = Path(sys.argv[1])",
                "package_id = sys.argv[2]",
                "target_platform = sys.argv[3]",
                "artifact.write_text(",
                "    artifact.read_text(encoding='utf-8')",
                "    + f'\\nsigned:{package_id}:{target_platform}',",
                "    encoding='utf-8',",
                ")",
            ]
        ),
        encoding="utf-8",
    )
    return signer


def _package_file_bytes(package_dir: Path) -> dict[str, bytes]:
    return {
        path.relative_to(package_dir).as_posix(): path.read_bytes()
        for path in sorted(package_dir.rglob("*"))
        if path.is_file()
    }


if __name__ == "__main__":
    unittest.main()
