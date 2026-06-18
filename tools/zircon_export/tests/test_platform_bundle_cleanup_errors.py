from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    json_loads,
)


class PlatformBundleCleanupErrorsTests(unittest.TestCase):
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
                "tools.zircon_export.platform_bundle.shutil.rmtree",
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
                "tools.zircon_export.platform_bundle.shutil.copy2",
                side_effect=copy_or_fail,
            ), mock.patch(
                "tools.zircon_export.platform_bundle.shutil.rmtree",
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
