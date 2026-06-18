from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _compile_host_args,
    _run_compile_host_quiet,
    json_loads,
)


class CompileHostPathResolveErrorTests(unittest.TestCase):
    def test_compile_host_reports_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            args = _compile_host_args(out=root / "out")
            args.repo_root = str(repo_root)
            args.dry_run = False
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated CompileHost repo_root failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost repo_root" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated CompileHost repo_root failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_reports_validate_report_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(validate_report):
                    raise OSError("simulated CompileHost validate_report failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["validate_report"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost validate_report" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated CompileHost validate_report failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
