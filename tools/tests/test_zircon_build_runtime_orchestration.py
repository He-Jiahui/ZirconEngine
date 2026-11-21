import os
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

from tools.build.zircon_build import (
    build_editor,
    build_native_dynamic_plugin,
    build_runtime,
    copy_artifact,
    parse_args,
    resolve_config,
)
from tools.build.zircon_build_abi import AbiValidationError
from tools.build.zircon_build_product_output import _validate_product_abis
from tools.build.zircon_build_product_profile import load_product_build_profile
from tools.build.zircon_build_support import platform_dynamic_library_name


class ArtifactTargetIdentityTests(unittest.TestCase):
    TARGET = "x86_64-pc-windows-msvc"
    OTHER_TARGET = "aarch64-pc-windows-msvc"
    RUNTIME_DLL = "zircon_runtime.dll"
    RUNTIME_PDB = "zircon_runtime.pdb"

    def setUp(self):
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory(dir=os.environ.get("TMPDIR"))))

    def config(self, *options, plugins=()):
        args = parse_args(["--out", str(self.root / "out"), *options])
        return resolve_config(args, Path(__file__).resolve().parents[2], plugins)

    @staticmethod
    def write_artifact_pair(artifact, dll, pdb):
        artifact.parent.mkdir(parents=True, exist_ok=True)
        artifact.write_bytes(dll)
        artifact.with_suffix(".pdb").write_bytes(pdb)

    def test_explicit_target_artifact_ignores_flat_host_output(self):
        config = self.config("--product-profile", "runtime-windows")
        self.assertEqual(self.TARGET, config.effective_target_triple)
        target_dir = config.targets_root / "runtime" / "lib"
        self.write_artifact_pair(
            target_dir / "shipping" / self.RUNTIME_DLL, b"flat-dll", b"flat-pdb"
        )
        self.write_artifact_pair(
            target_dir / self.TARGET / "shipping" / self.RUNTIME_DLL,
            b"selected-dll",
            b"selected-pdb",
        )

        copy_artifact(config, target_dir, self.RUNTIME_DLL)

        self.assertEqual(
            (b"selected-dll", b"selected-pdb"),
            (
                (config.engine_root / self.RUNTIME_DLL).read_bytes(),
                (config.symbols_root / self.RUNTIME_PDB).read_bytes(),
            ),
        )
        self.assertFalse((config.engine_root / self.RUNTIME_PDB).exists())

    def test_native_plugin_artifact_ignores_sorting_first_other_target(self):
        package_root = self.root / "fixture"
        package_root.mkdir()
        manifest_path = package_root / "plugin.toml"
        manifest_path.write_text("id = 'fixture'\n", encoding="utf-8")
        package = types.SimpleNamespace(
            plugin_id="fixture",
            package_root=package_root,
            manifest_path=manifest_path,
            native_dynamic_crates=(types.SimpleNamespace(name="fixture"),),
        )
        config = self.config(
            "--targets", "plugins", "--plugins", "fixture", "--mode", "debug",
            f"--target={self.TARGET}", plugins=(package,),
        )
        target_dir = config.targets_root / "plugins" / "fixture"
        artifact_name = platform_dynamic_library_name("fixture")
        self.write_artifact_pair(
            target_dir / self.OTHER_TARGET / "debug" / artifact_name,
            b"other-dll",
            b"other-pdb",
        )
        self.write_artifact_pair(
            target_dir / self.TARGET / "debug" / artifact_name,
            b"selected-dll",
            b"selected-pdb",
        )

        with mock.patch("tools.build.zircon_build.run_cargo") as run_cargo:
            build_native_dynamic_plugin(config, package)
        run_cargo.assert_called_once()

        artifact = config.engine_root / "plugins" / "fixture" / "native" / artifact_name
        self.assertEqual(
            (b"selected-dll", b"selected-pdb"),
            (artifact.read_bytes(), artifact.with_suffix(".pdb").read_bytes()),
        )

    def test_missing_explicit_target_artifact_fails_closed(self):
        config = self.config("--targets", "runtime", "--mode", "debug", f"--target={self.TARGET}")
        target_dir = config.targets_root / "runtime" / "lib"
        self.write_artifact_pair(
            target_dir / "debug" / self.RUNTIME_DLL, b"flat-dll", b"flat-pdb"
        )
        self.write_artifact_pair(
            target_dir / self.OTHER_TARGET / "debug" / self.RUNTIME_DLL,
            b"other-dll",
            b"other-pdb",
        )
        (target_dir / self.TARGET / "debug").mkdir(parents=True)

        with self.assertRaisesRegex(SystemExit, "Built artifact not found") as failure:
            copy_artifact(config, target_dir, self.RUNTIME_DLL)

        self.assertIn(self.TARGET, str(failure.exception))
        self.assertIn(self.RUNTIME_DLL, str(failure.exception))
        self.assertFalse((config.engine_root / self.RUNTIME_DLL).exists())
        self.assertFalse((config.engine_root / self.RUNTIME_PDB).exists())

    def test_no_explicit_target_preserves_legacy_flat_output(self):
        config = self.config("--targets", "runtime", "--mode", "debug")
        self.assertIsNone(config.effective_target_triple)
        target_dir = config.targets_root / "runtime" / "lib"
        self.write_artifact_pair(
            target_dir / "debug" / self.RUNTIME_DLL, b"flat-dll", b"flat-pdb"
        )

        copy_artifact(config, target_dir, self.RUNTIME_DLL)

        self.assertEqual(
            (b"flat-dll", b"flat-pdb"),
            (
                (config.engine_root / self.RUNTIME_DLL).read_bytes(),
                (config.engine_root / self.RUNTIME_PDB).read_bytes(),
            ),
        )

    def test_explicit_target_rejects_path_shaped_lookup(self):
        target_dir = self.root / "out" / "targets" / "runtime" / "lib"
        target_dir.mkdir(parents=True)
        self.write_artifact_pair(
            target_dir.parent / "foreign" / "debug" / self.RUNTIME_DLL,
            b"foreign-dll",
            b"foreign-pdb",
        )
        for target in ("../foreign", r"..\foreign"):
            with self.subTest(target=target):
                config = self.config("--targets", "runtime", "--mode", "debug", f"--target={target}")
                with self.assertRaisesRegex(SystemExit, "requires a Rust target triple, not a path"):
                    copy_artifact(config, target_dir, self.RUNTIME_DLL)
                self.assertFalse((config.engine_root / self.RUNTIME_DLL).exists())

    def test_explicit_target_rejects_redirected_profile(self):
        from tools.build.zircon_build_staging_manifest import write_staging_manifest

        config = self.config("--targets", "runtime", "--mode", "debug", f"--target={self.TARGET}")
        target_dir = config.targets_root / "runtime" / "lib"
        expected = target_dir.resolve() / self.TARGET / "debug"
        redirected = target_dir.resolve() / self.OTHER_TARGET / "debug"
        for profile in (expected, redirected):
            self.write_artifact_pair(profile / self.RUNTIME_DLL, b"foreign-dll", b"foreign-pdb")
        actual_resolve = Path.resolve

        def resolve(path, *args, **kwargs):
            # Model a junction to U without requiring Windows symlink privileges.
            resolved = actual_resolve(path, *args, **kwargs)
            if resolved.is_relative_to(expected):
                return redirected / resolved.relative_to(expected)
            return resolved

        with mock.patch.object(Path, "resolve", autospec=True, side_effect=resolve):
            with self.subTest(consumer="copy"):
                with self.assertRaisesRegex(SystemExit, "Built artifact not found"):
                    copy_artifact(config, target_dir, self.RUNTIME_DLL)
                self.assertFalse((config.engine_root / self.RUNTIME_DLL).exists())
            with self.subTest(consumer="provenance"):
                config.engine_root.mkdir(parents=True, exist_ok=True)
                (config.engine_root / self.RUNTIME_DLL).write_bytes(b"foreign-dll")
                with self.assertRaisesRegex(SystemExit, "no source provenance"):
                    write_staging_manifest(config)


class RuntimeBuildOrchestrationTests(unittest.TestCase):
    def config(self, *, shipping=False):
        return types.SimpleNamespace(
            targets_root=Path("output/targets"),
            runtime_preview_feature_arg="target-client",
            dry_run=False,
            mode="shipping" if shipping else "debug",
            is_shipping=shipping,
        )

    def test_runtime_library_and_preview_have_independent_build_commands(self):
        config = self.config()
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            build_runtime(config, "target-client", include_preview=True)
        library = run.call_args_list[0].args[1]
        preview = run.call_args_list[1].args[1]
        self.assertEqual(
            library[:7],
            ["rustc", "-p", "zircon_runtime", "--lib", "--crate-type", "cdylib", "--no-default-features"],
        )
        self.assertNotIn("--", library)
        self.assertEqual(library[library.index("--features") + 1], "target-client")
        self.assertEqual(
            preview[:5], ["build", "-p", "zircon_app", "--bin", "zircon_runtime"]
        )
        self.assertEqual(copy.call_count, 2)

    def test_development_runtime_keeps_non_api_feature_compositions_available(self):
        config = self.config()
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            build_runtime(config, "target-server", include_preview=False)
        self.assertEqual(run.call_count, 1)
        self.assertEqual(copy.call_count, 1)

    def test_editor_only_library_and_host_reuse_dependency_outputs(self):
        config = self.config()
        config.targets = ("editor",)
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ):
            build_runtime(config, "target-editor-host", include_preview=False)
            build_editor(config, "target-editor-host")
        self.assertEqual(run.call_count, 2)
        for call in run.call_args_list:
            command = call.args[1]
            self.assertEqual(
                command[command.index("--target-dir") + 1], str(config.targets_root / "editor")
            )
            self.assertEqual(command[command.index("--features") + 1], "target-editor-host")

    def test_development_features_fail_before_any_build_or_copy(self):
        for features in (
            "dev-dynamic-linking",
            "target-client,zircon_runtime/dev-dynamic-linking",
        ):
            with self.subTest(features=features), mock.patch(
                "tools.build.zircon_build.run_cargo"
            ) as run, mock.patch("tools.build.zircon_build.copy_artifact") as copy:
                with self.assertRaisesRegex(ValueError, "C ABI products"):
                    build_runtime(self.config(shipping=True), features, include_preview=False)
                run.assert_not_called()
                copy.assert_not_called()

    def test_core_only_features_fail_before_any_build_or_copy(self):
        with mock.patch("tools.build.zircon_build.run_cargo") as run, mock.patch(
            "tools.build.zircon_build.copy_artifact"
        ) as copy:
            with self.assertRaisesRegex(ValueError, "dynamic-api"):
                build_runtime(self.config(shipping=True), "core-min", include_preview=False)
            run.assert_not_called()
            copy.assert_not_called()

    def test_product_finalization_requires_runtime_and_host_artifacts(self):
        profile = load_product_build_profile("runtime-windows")
        with tempfile.TemporaryDirectory() as temporary:
            config = types.SimpleNamespace(
                engine_root=Path(temporary) / "ZirconEngine",
                repo_root=Path.cwd(),
            )
            config.engine_root.mkdir()
            with self.assertRaisesRegex(AbiValidationError, "required product artifact"):
                _validate_product_abis(config, profile)


if __name__ == "__main__":
    unittest.main()
