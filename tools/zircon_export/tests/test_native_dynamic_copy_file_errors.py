from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_stage_quiet,
    _write_native_dynamic_package_fixture,
    _write_validate_report_with_native_dynamic_exports,
    json_loads,
)


class NativeDynamicCopyFileErrorsTests(unittest.TestCase):
    def test_native_dynamic_stage_rejects_package_source_listing_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            source_package = (repo_root / "zircon_plugins" / "animation").resolve()
            original_iterdir = Path.iterdir

            def iterdir_or_fail(path: Path):
                if path.resolve() == source_package:
                    raise OSError("simulated source package listing failure")
                return original_iterdir(path)

            with mock.patch.object(Path, "iterdir", iterdir_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "plugins"
                    / "animation"
                ).exists()
            )
            self.assertTrue(
                any(
                    "NativeDynamic package animation source directory" in diagnostic
                    and "could not be listed" in diagnostic
                    and "simulated source package listing failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_package_artifact_copy_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            native_artifact = (
                repo_root
                / "zircon_plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()
            original_copy2 = shutil.copy2

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == native_artifact:
                    raise OSError("simulated native artifact copy failure")
                original_copy2(source, destination)

            with mock.patch(
                "tools.zircon_export.native_dynamic.shutil.copy2",
                side_effect=copy_or_fail,
            ):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "plugins"
                    / "animation"
                ).exists()
            )
            self.assertTrue(
                any(
                    "NativeDynamic package animation artifact" in diagnostic
                    and "could not be copied" in diagnostic
                    and "simulated native artifact copy failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_package_report_write_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            package_report = (
                out
                / "stages"
                / "native_dynamic"
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            ).resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == package_report:
                    raise OSError("simulated package report write failure")
                return original_write_text(path, *args, **kwargs)

            with mock.patch.object(Path, "write_text", write_text_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "plugins"
                    / "animation"
                ).exists()
            )
            self.assertTrue(
                any(
                    "NativeDynamic package animation report" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated package report write failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_loader_manifest_write_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            loader_manifest = (
                out
                / "stages"
                / "native_dynamic"
                / "plugins"
                / "native_plugins.toml"
            ).resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == loader_manifest:
                    raise OSError("simulated loader manifest write failure")
                return original_write_text(path, *args, **kwargs)

            with mock.patch.object(Path, "write_text", write_text_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse(loader_manifest.exists())
            self.assertTrue(
                any(
                    "NativeDynamic loader manifest" in diagnostic
                    and "could not be written" in diagnostic
                    and "simulated loader manifest write failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_stale_plugins_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            plugins_dir = out / "stages" / "native_dynamic" / "plugins"
            plugins_dir.mkdir(parents=True)
            (plugins_dir / "stale.txt").write_text("stale", encoding="utf-8")
            original_rmtree = shutil.rmtree

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == plugins_dir.resolve():
                    raise OSError("simulated stale plugins cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.zircon_export.native_dynamic.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "stale_payload_cleanup_failed")
            self.assertTrue((plugins_dir / "stale.txt").exists())
            self.assertTrue(
                any(
                    "NativeDynamic plugins directory" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated stale plugins cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_reports_partial_package_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            native_artifact = (
                repo_root
                / "zircon_plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()
            partial_package = (
                out / "stages" / "native_dynamic" / "plugins" / "animation"
            ).resolve()
            original_copy2 = shutil.copy2
            original_rmtree = shutil.rmtree

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == native_artifact:
                    raise OSError("simulated native artifact copy failure")
                original_copy2(source, destination)

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == partial_package:
                    raise OSError("simulated partial package cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.zircon_export.native_dynamic.shutil.copy2",
                side_effect=copy_or_fail,
            ), mock.patch(
                "tools.zircon_export.native_dynamic.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse(partial_package.exists())
            self.assertTrue(
                any(
                    "NativeDynamic package animation partial package" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated partial package cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_reports_final_payload_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            native_artifact = (
                repo_root
                / "zircon_plugins"
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            ).resolve()
            plugins_dir = (out / "stages" / "native_dynamic" / "plugins").resolve()
            original_copy2 = shutil.copy2
            original_rmtree = shutil.rmtree

            def copy_or_fail(source: Path, destination: Path) -> None:
                if Path(source).resolve() == native_artifact:
                    raise OSError("simulated native artifact copy failure")
                original_copy2(source, destination)

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == plugins_dir:
                    raise OSError("simulated final payload cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.zircon_export.native_dynamic.shutil.copy2",
                side_effect=copy_or_fail,
            ), mock.patch(
                "tools.zircon_export.native_dynamic.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out
                    / "stages"
                    / "native_dynamic"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_cleanup_failed")
            self.assertTrue(plugins_dir.exists())
            self.assertTrue(
                any(
                    "NativeDynamic plugins directory" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated final payload cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
