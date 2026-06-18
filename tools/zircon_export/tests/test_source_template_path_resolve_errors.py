from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.source_template import run_source_template
from tools.zircon_export.tests.export_test_support import (
    _run_source_template_quiet,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class SourceTemplatePathResolveErrorsTests(unittest.TestCase):
    def test_source_template_rejects_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            repo_root = root / "repo"
            args = _source_template_args(
                out=root / "out",
                validate_report=validate_report,
                build=False,
                dry_run=True,
            )
            args.repo_root = str(repo_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated SourceTemplate repo_root resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_source_template(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("repo_root", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated SourceTemplate repo_root resolve failure", output)

    def test_source_template_rejects_validate_report_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            args = _source_template_args(
                out=root / "out",
                validate_report=validate_report,
                build=False,
                dry_run=True,
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(validate_report):
                    raise OSError(
                        "simulated SourceTemplate validate_report resolve failure"
                    )
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_source_template(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("validate_report", output)
            self.assertIn("could not be resolved", output)
            self.assertIn(
                "simulated SourceTemplate validate_report resolve failure",
                output,
            )

    def test_source_template_rejects_target_dir_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            target_dir = root / "target" / "source-template"
            args = _source_template_args(
                out=root / "out",
                validate_report=validate_report,
                build=False,
                dry_run=True,
            )
            args.target_dir = str(target_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(target_dir):
                    raise OSError("simulated SourceTemplate target_dir resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_source_template(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("target_dir", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated SourceTemplate target_dir resolve failure", output)
            self.assertNotIn("--target-dir", output)

    def test_source_template_stage_rejects_dependency_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            repo_root = root / "repo"
            repo_root.mkdir()
            failing_dependency = repo_root / "zircon_app"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_dependency:
                    raise OSError("simulated SourceTemplate dependency path failure")
                return original_resolve(path, *args, **kwargs)

            args = _source_template_args(
                out=root / "out",
                validate_report=validate_report,
                build=False,
                dry_run=False,
            )
            args.repo_root = str(repo_root)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_source_template_quiet(args)

            stage_dir = root / "out" / "stages" / "source_template"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((stage_dir / "project").exists())
            self.assertTrue(
                any(
                    "SourceTemplate dependency zircon_app path" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated SourceTemplate dependency path failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
