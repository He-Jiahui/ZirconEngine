from __future__ import annotations

import shutil
import subprocess
import os
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.export.tests.export_test_support import (
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    json_loads,
)
from tools.export.platform_bundle_materialize import remove_platform_bundle_dir
from tools.export.cli_arguments import parse_args


class PlatformBundleCleanupErrorsTests(unittest.TestCase):
    def test_export_cli_rejects_unsafe_profile_names(self) -> None:
        for profile in ("../outside", "C:\\outside", "C:relative", "/tmp/out", "nul", "COM1"):
            with self.subTest(profile=profile), self.assertRaises(SystemExit):
                parse_args(["--profile", profile])

        for profile in ("windows-release", "editor_2"):
            with self.subTest(profile=profile):
                self.assertEqual(parse_args(["--profile", profile]).profile, profile)

    def test_bundle_cleanup_rejects_profile_traversal_and_preserves_outside(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside = root / "sentinel"
            outside.mkdir()
            (outside / "keep.txt").write_text("keep", encoding="utf-8")
            diagnostics: list[str] = []

            removed = remove_platform_bundle_dir(
                "test bundle cleanup", outside, out, "../sentinel", diagnostics
            )

            self.assertFalse(removed)
            self.assertTrue((outside / "keep.txt").exists())
            self.assertTrue(any("invalid export profile" in item for item in diagnostics))

    def test_bundle_cleanup_rejects_symlinked_output_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            real_out = root / "real-out"
            target = real_out / "bundle" / "release"
            target.mkdir(parents=True)
            sentinel = target / "keep.txt"
            sentinel.write_text("keep", encoding="utf-8")
            linked_out = root / "linked-out"
            try:
                linked_out.symlink_to(real_out, target_is_directory=True)
            except OSError as error:
                self.skipTest(f"could not create required directory symlink: {error}")
            diagnostics: list[str] = []

            removed = remove_platform_bundle_dir(
                "test bundle cleanup", linked_out / "bundle" / "release", linked_out, "release", diagnostics
            )

            self.assertFalse(removed)
            self.assertTrue(sentinel.exists())
            self.assertTrue(any("symlink or reparse point" in item for item in diagnostics))

    def test_bundle_cleanup_rejects_junctioned_output_root(self) -> None:
        if os.name != "nt":
            self.skipTest("directory junction regression requires Windows")
        cmd = shutil.which("cmd.exe")
        if cmd is None:
            self.fail("cmd.exe is unavailable on Windows; cannot run the directory junction regression")

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            real_out = root / "real-out"
            target = real_out / "bundle" / "release"
            target.mkdir(parents=True)
            sentinel = target / "keep.txt"
            sentinel.write_text("keep", encoding="utf-8")
            linked_out = root / "linked-out"
            command = (
                f'{subprocess.list2cmdline([cmd])} /d /c '
                f'mklink /J "{linked_out}" "{real_out}"'
            )
            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                encoding="utf-8",
                errors="replace",
                check=False,
            )
            if result.returncode != 0 or not linked_out.is_dir():
                detail = result.stderr.strip() or result.stdout.strip() or "no command output"
                if result.returncode in (5, 1314):
                    self.skipTest(
                        f"Windows denied directory junction creation "
                        f"(exit {result.returncode}): {detail}"
                    )
                self.fail(
                    f"Windows mklink /J failed to create a directory junction "
                    f"(exit {result.returncode}): {detail}"
                )

            try:
                diagnostics: list[str] = []
                removed = remove_platform_bundle_dir(
                    "test bundle cleanup",
                    linked_out / "bundle" / "release",
                    linked_out,
                    "release",
                    diagnostics,
                )

                self.assertFalse(removed)
                self.assertTrue(sentinel.exists(), "cleanup must preserve the real target sentinel")
                self.assertTrue(
                    any("symlink or reparse point" in item for item in diagnostics),
                    diagnostics,
                )
            finally:
                linked_out.rmdir()

    def test_platform_bundle_rejects_stale_bundle_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            bundle_dir = out / "bundle" / "windows-release"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            bundle_dir.mkdir(parents=True)
            (bundle_dir / "stale.txt").write_text("stale", encoding="utf-8")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            original_rmtree = shutil.rmtree

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == bundle_dir.resolve():
                    raise OSError("simulated stale bundle cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.export.platform_bundle_materialize.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertTrue((bundle_dir / "stale.txt").exists())
            self.assertTrue(
                any(
                    "stale PlatformBundle profile bundle" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated stale bundle cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_reports_failed_bundle_cleanup_after_copy_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            bundle_dir = out / "bundle" / "windows-release"
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            original_copy2 = shutil.copy2
            original_rmtree = shutil.rmtree

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == host.resolve():
                    raise OSError("simulated host copy failure")
                original_copy2(source, destination)

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == bundle_dir.resolve():
                    raise OSError("simulated partial bundle cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.export.platform_bundle_materialize.shutil.copy2",
                side_effect=copy_or_fail,
            ), mock.patch(
                "tools.export.platform_bundle_materialize.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertTrue(bundle_dir.exists())
            self.assertTrue(
                any(
                    "host executable" in diagnostic
                    and "could not be copied" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "partial PlatformBundle profile bundle" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated partial bundle cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_bundle_root_create_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "zircon_runtime.exe"
            pack = root / "assets.zrpack"
            bundle_dir = (out / "bundle" / "windows-release").resolve()
            host.write_text("host placeholder", encoding="utf-8")
            pack.write_text("pack placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            original_mkdir = Path.mkdir

            def mkdir_or_fail(path: Path, *args: object, **kwargs: object) -> None:
                if path.resolve() == bundle_dir and path.exists():
                    raise OSError("simulated bundle root create failure")
                original_mkdir(path, *args, **kwargs)

            with mock.patch.object(Path, "mkdir", mkdir_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse(bundle_dir.exists())
            self.assertTrue(
                any(
                    "PlatformBundle bundle root" in diagnostic
                    and "could not be created" in diagnostic
                    and "simulated bundle root create failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
