from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import run_validate
from tools.zircon_export.tests.export_test_support import _export_args


class ValidatePathResolveErrorTests(unittest.TestCase):
    def test_validate_rejects_repo_root_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            args = _export_args(out=root / "out", stage="validate", dry_run=True)
            args.repo_root = str(repo_root)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(repo_root):
                    raise OSError("simulated validate repo_root resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=Validate repo_root", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated validate repo_root resolve failure", output)
            self.assertIn("command=<skipped>", output)

    def test_validate_rejects_project_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            args = _export_args(out=root / "out", stage="validate", dry_run=True)
            args.project = str(project)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(project):
                    raise OSError("simulated validate project resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=Validate project", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated validate project resolve failure", output)
            self.assertIn("command=<skipped>", output)

    def test_validate_rejects_validator_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validator = root / "tools" / "zircon_export_validate.exe"
            args = _export_args(out=root / "out", stage="validate", dry_run=True)
            args.validator = str(validator)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(validator):
                    raise OSError("simulated validator resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=validator", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated validator resolve failure", output)
            self.assertIn("command=<skipped>", output)

    def test_validate_rejects_target_dir_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            target_dir = root / "target" / "validate"
            args = _export_args(out=root / "out", stage="validate", dry_run=True)
            args.target_dir = str(target_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(target_dir):
                    raise OSError("simulated target dir resolve failure")
                return original_resolve(path, *args, **kwargs)

            stdout = io.StringIO()
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(args)

            output = stdout.getvalue()
            self.assertEqual(exit_code, 2)
            self.assertIn("diagnostic=target_dir", output)
            self.assertIn("could not be resolved", output)
            self.assertIn("simulated target dir resolve failure", output)
            self.assertIn("command=<skipped>", output)
            self.assertNotIn("--target-dir", output)


if __name__ == "__main__":
    unittest.main()
