"""The developer staging tree publishes a plugin load list, not arbitrary cdylibs."""

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from tools.build import zircon_build as build
from tools.build import zircon_build_support as naming


class NativePluginTargetTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.manifest = self.root / "plugin.toml"
        self.manifest.write_text('id = "fixture"\n', encoding="utf-8")
        self.crate = SimpleNamespace(name="fixture")
        self.package = SimpleNamespace(
            plugin_id="fixture", package_root=self.root, manifest_path=self.manifest,
            native_dynamic_crates=(self.crate,), rlib_static_crates=(),
        )

    def config(self, target=None, *, dry_run=False):
        args = ["--targets", "plugins", "--plugins", "fixture", "--mode", "debug",
                "--out", str(self.root / "out")]
        if target:
            args += ["--target-triple", target]
        if dry_run:
            args.append("--dry-run")
        return build.resolve_config(build.parse_args(args), Path(__file__).resolve().parents[2], (self.package,))

    def test_non_windows_explicit_targets_stop_before_plugin_build_or_publication(self):
        for triple in ("x86_64-unknown-linux-gnu", "aarch64-apple-darwin",
                       "aarch64-linux-android", "aarch64-apple-ios", "wasm32-unknown-unknown"):
            for dry_run in (False, True):
                with self.subTest(target=triple, dry_run=dry_run):
                    config = self.config(triple, dry_run=dry_run)
                    with patch.object(build, "run_plugin_cargo") as cargo:
                        with self.assertRaisesRegex(SystemExit, f"{triple}.*Windows"):
                            build.build_plugins(config)
                        cargo.assert_not_called()
                    self.assertFalse(config.engine_root.exists())

    def test_windows_target_stages_a_dll_even_with_a_posix_host_namer(self):
        triple = "x86_64-pc-windows-msvc"
        config = self.config(triple)
        gen = build.plugin_target_dir(config, self.package) / triple / "debug"
        gen.mkdir(parents=True)
        (gen / "fixture.dll").write_bytes(b"selected Windows fixture")
        (gen / "libfixture.so").write_bytes(b"not a Windows plugin")
        # Change only the naming module's host view, never pathlib's host semantics.
        with (patch.object(naming, "os", SimpleNamespace(name="posix")),
              patch.object(naming.platform, "system", return_value="Linux"),
              patch.object(build, "run_plugin_cargo") as cargo):
            build.build_plugins(config)
        cargo.assert_called_once()
        native = config.engine_root / "plugins" / "fixture" / "native"
        self.assertEqual(["fixture.dll"], sorted(p.name for p in native.iterdir()))
        self.assertEqual(b"selected Windows fixture", (native / "fixture.dll").read_bytes())
        self.assertTrue((config.engine_root / build.PLUGIN_LOAD_MANIFEST).is_file())

    def test_explicit_linux_static_only_package_needs_no_native_load_manifest(self):
        self.package.native_dynamic_crates = ()
        self.package.rlib_static_crates = (self.crate,)
        config = self.config("x86_64-unknown-linux-gnu")
        with patch.object(build, "run_plugin_cargo") as cargo:
            build.build_plugins(config)
        cargo.assert_called_once_with(config, build.plugin_target_dir(config, self.package), ["fixture"])
        self.assertFalse((config.engine_root / build.PLUGIN_LOAD_MANIFEST).exists())

    def test_no_explicit_target_preserves_existing_host_naming(self):
        config = self.config()
        expected = naming.platform_dynamic_library_name("fixture")
        gen = build.plugin_target_dir(config, self.package) / "debug"
        gen.mkdir(parents=True)
        (gen / expected).write_bytes(b"host fixture")
        with patch.object(build, "run_plugin_cargo"):
            build.build_native_dynamic_plugin(config, self.package)
        self.assertTrue((config.engine_root / "plugins" / "fixture" / "native" / expected).is_file())
