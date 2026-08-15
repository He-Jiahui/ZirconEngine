from __future__ import annotations

import os
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock
from uuid import uuid4


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_RUNNER = REPO_ROOT / "tools" / "zircon_build.py"
ENVIRONMENT_HELPER = REPO_ROOT / "tools" / "zircon_build_cargo_environment.py"
FONT_SDF_BUILDER = REPO_ROOT / "tools" / "zircon_build_font_sdf.py"
RUNTIME_FEATURE_CHECKERS = (
    REPO_ROOT / "tools" / "check-runtime-domain-features.ps1",
    REPO_ROOT / "tools" / "check-runtime-profile-features.ps1",
)


class ZirconBuildCargoEnvironmentTests(unittest.TestCase):
    def test_managed_environment_creates_only_staged_cargo_locations(self) -> None:
        from tools.zircon_build_cargo_environment import managed_cargo_environment

        managed_root = r"D:\ZirconBuilds" if os.name == "nt" else None
        with tempfile.TemporaryDirectory(dir=managed_root) as temporary_root:
            cache_root = Path(temporary_root)
            target_dir = cache_root / "targets" / "runtime"

            environment = managed_cargo_environment(target_dir, cache_root)

            expected = {
                "CARGO_TARGET_DIR": target_dir,
                "CARGO_HOME": cache_root / "cargo-home",
                "SCCACHE_DIR": cache_root / "sccache",
                "TEMP": target_dir / "temporary",
                "TMP": target_dir / "temporary",
                "TMPDIR": target_dir / "temporary",
            }
            for name, directory in expected.items():
                self.assertEqual(str(directory.resolve()), environment[name])
                self.assertTrue(directory.is_dir(), f"{name} must stay in staging")

    def test_managed_environment_rejects_target_outside_cache_before_creation(self) -> None:
        from tools.zircon_build_cargo_environment import managed_cargo_environment

        managed_root = r"D:\ZirconBuilds" if os.name == "nt" else None
        with tempfile.TemporaryDirectory(dir=managed_root) as temporary_root:
            cache_root = Path(temporary_root) / "cache"
            outside_target = Path(temporary_root) / "outside" / "runtime"

            with self.assertRaisesRegex(ValueError, "outside"):
                managed_cargo_environment(outside_target, cache_root)

            self.assertFalse(outside_target.exists())

    @unittest.skipUnless(os.name == "nt", "Windows staging roots are Windows-only")
    def test_windows_unmanaged_root_is_rejected_before_creation(self) -> None:
        from tools.zircon_build_cargo_environment import assert_managed_windows_build_root

        unmanaged_root = Path(r"C:\ZirconBuilds") / f"must-not-be-created-{uuid4().hex}"
        with self.assertRaisesRegex(ValueError, "approved build root"):
            assert_managed_windows_build_root(unmanaged_root)

        self.assertFalse(unmanaged_root.exists())

    @unittest.skipUnless(os.name == "nt", "Windows staging roots are Windows-only")
    def test_build_rejects_unmanaged_output_before_staging_directories(self) -> None:
        from tools.zircon_build import build

        out_root = Path(r"C:\ZirconBuilds") / f"must-not-be-created-{uuid4().hex}"
        config = types.SimpleNamespace(
            dry_run=False,
            out_root=out_root,
            engine_root=out_root / "ZirconEngine",
            targets_root=out_root / "targets",
            targets=(),
            prewarm_shaders=False,
        )

        self.assertFalse(out_root.exists())
        with self.assertRaisesRegex(ValueError, "approved build root"):
            build(config)
        self.assertFalse(out_root.exists())

    def test_run_cargo_resolves_relative_target_dir_from_the_cargo_working_directory(
        self,
    ) -> None:
        from tools.zircon_build import run_cargo

        managed_root = r"D:\ZirconBuilds" if os.name == "nt" else None
        with tempfile.TemporaryDirectory(dir=managed_root) as temporary_root:
            repo_root = Path(temporary_root)
            targets_root = repo_root / "targets"
            config = types.SimpleNamespace(
                cargo="cargo",
                locked=False,
                mode="debug",
                jobs=None,
                dry_run=False,
                repo_root=repo_root,
                targets_root=targets_root,
            )

            with mock.patch("tools.zircon_build.subprocess.run") as cargo_run:
                run_cargo(config, ["build", "--target-dir", "targets/runtime"])

            _, keyword_arguments = cargo_run.call_args
            self.assertEqual(repo_root, keyword_arguments["cwd"])
            self.assertEqual(
                str((targets_root / "runtime").resolve()),
                keyword_arguments["env"]["CARGO_TARGET_DIR"],
            )


    def test_build_runner_uses_a_managed_environment_for_cargo_children(self) -> None:
        source = BUILD_RUNNER.read_text(encoding="utf-8")

        self.assertIn("zircon_build_cargo_environment", source)
        self.assertIn("managed_cargo_environment", source)
        self.assertIn("assert_managed_windows_build_root", source)
        self.assertIn("assert_managed_windows_build_root(config.engine_root)", source)
        self.assertIn("assert_managed_windows_build_root(config.targets_root)", source)
        self.assertIn("env=environment", source)

    def test_managed_environment_is_scoped_to_the_resolved_build_targets_root(self) -> None:
        source = ENVIRONMENT_HELPER.read_text(encoding="utf-8")

        self.assertIn("target_dir.resolve()", source)
        self.assertIn("cache_root.resolve()", source)
        self.assertIn("assert_managed_windows_build_root(cache_root)", source)
        self.assertIn("relative_to(cache_root)", source)
        self.assertIn('"CARGO_TARGET_DIR"', source)
        self.assertIn('"CARGO_HOME"', source)
        self.assertIn('"SCCACHE_DIR"', source)
        for name in ("TEMP", "TMP", "TMPDIR"):
            self.assertIn(f'"{name}"', source)

    def test_specialized_cargo_runs_reuse_the_managed_environment(self) -> None:
        build_source = BUILD_RUNNER.read_text(encoding="utf-8")
        font_sdf_source = FONT_SDF_BUILDER.read_text(encoding="utf-8")
        prewarm_source = build_source.split("def prewarm_shaders", 1)[1].split(
            "def build_editor", 1
        )[0]
        font_sdf_run_source = font_sdf_source.split(
            "def bake_font_sdf_manifest", 1
        )[1].split("def _spec_from_record", 1)[0]

        self.assertIn('config.targets_root / "shader_prewarm"', prewarm_source)
        self.assertIn("env=environment", prewarm_source)
        self.assertIn("zircon_build_cargo_environment", font_sdf_source)
        self.assertIn('config.targets_root / "font-sdf"', font_sdf_run_source)
        self.assertIn("env=environment", font_sdf_run_source)

    def test_windows_build_output_requires_an_approved_root_after_physical_resolution(self) -> None:
        source = ENVIRONMENT_HELPER.read_text(encoding="utf-8")

        self.assertIn('os.name != "nt"', source)
        self.assertIn("out_root.resolve()", source)
        self.assertIn("APPROVED_WINDOWS_BUILD_ROOTS", source)
        self.assertIn("relative_to(approved_root)", source)
        for root in (
            r"D:\cargo-targets",
            r"E:\cargo-targets",
            r"F:\cargo-targets",
            r"D:\targets",
            r"E:\targets",
            r"F:\targets",
            r"D:\ZirconBuilds",
            r"E:\ZirconBuilds",
            r"F:\ZirconBuilds",
        ):
            self.assertIn(root, source)

    def test_runtime_feature_checkers_use_managed_cargo_environments(self) -> None:
        for checker in RUNTIME_FEATURE_CHECKERS:
            source = checker.read_text(encoding="utf-8")

            self.assertIn("WindowsPathResolver.psm1", source, checker.name)
            self.assertIn("Resolve-ZirconWindowsPath", source, checker.name)
            self.assertIn("Resolve-ManagedCargoTargetPath", source, checker.name)
            self.assertIn("Push-ManagedCargoEnvironment", source, checker.name)
            self.assertIn("Pop-ManagedCargoEnvironment", source, checker.name)
            self.assertIn("cargo-targets\\zircon-runtime-", source, checker.name)
            for name in (
                "CARGO_TARGET_DIR",
                "CARGO_HOME",
                "SCCACHE_DIR",
                "TEMP",
                "TMP",
                "TMPDIR",
            ):
                self.assertIn(name, source, checker.name)


if __name__ == "__main__":
    unittest.main()
