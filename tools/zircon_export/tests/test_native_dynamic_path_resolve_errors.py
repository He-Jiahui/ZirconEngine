from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_stage_quiet,
    _write_validate_report_with_native_dynamic_exports,
    json_loads,
)


class NativeDynamicPathResolveErrorTests(unittest.TestCase):
    def test_native_dynamic_reports_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            repo_root = root / "repo"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated NativeDynamic repo_root failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["native_plugin_root"])
            self.assertEqual(report["materialized_packages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic repo_root" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated NativeDynamic repo_root failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_reports_validate_report_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            repo_root = root / "repo"
            validate_report = root / "validate.json"
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.validate_report = str(validate_report)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(validate_report):
                    raise OSError("simulated NativeDynamic validate_report failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["validate_report"])
            self.assertEqual(report["package_exports"], [])
            self.assertTrue(
                any(
                    "NativeDynamic validate_report" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated NativeDynamic validate_report failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_reports_native_plugin_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            repo_root = root / "repo"
            native_plugin_root = root / "plugins"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            args.native_plugin_root = str(native_plugin_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(native_plugin_root):
                    raise OSError("simulated NativeDynamic native_plugin_root failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (
                    out / "stages" / "native_dynamic" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["native_plugin_root"])
            self.assertEqual(report["materialized_packages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic native_plugin_root" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated NativeDynamic native_plugin_root failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
