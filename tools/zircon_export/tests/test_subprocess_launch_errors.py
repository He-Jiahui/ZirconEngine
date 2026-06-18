from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import run_compile_host, run_pack, run_validate
from tools.zircon_export.source_template import run_source_template
from tools.zircon_export.tests.export_test_support import (
    _compile_host_args,
    _compile_host_plan,
    _export_args,
    _pack_args,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class SubprocessLaunchErrorTests(unittest.TestCase):
    def test_validate_reports_successful_validator_without_stage_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            args = _export_args(out=out, stage="validate", dry_run=False)

            stdout = io.StringIO()
            with mock.patch(
                "tools.zircon_export.cli.subprocess.call",
                return_value=0,
            ):
                with contextlib.redirect_stdout(stdout):
                    exit_code = run_validate(args)

            report_path = out / "stages" / "validate" / "report.json"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["stage"], "Validate")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(report["project"], str(Path(args.project).resolve()))
            self.assertEqual(report["exit_code"], 0)
            self.assertTrue(
                any(
                    "Validate command exited with code 0 but did not write report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_validate_reports_validator_launch_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            args = _export_args(out=out, stage="validate", dry_run=False)
            args.validator = str(root / "missing-validator.exe")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_validate(args)

            report_path = out / "stages" / "validate" / "report.json"
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("missing-validator.exe", stdout.getvalue())
            self.assertTrue(
                any(
                    "Validate command" in diagnostic
                    and "could not start" in diagnostic
                    and "missing-validator.exe" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pack_reports_packer_launch_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_manifest = root / "assets.json"
            (root / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": str(root / "main.scene"),
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _pack_args(out=out, dry_run=False)
            args.asset_manifest = str(source_manifest)
            args.packer = str(root / "missing-packer.exe")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            report_path = out / "stages" / "pack" / "report.json"
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIn("missing-packer.exe", stdout.getvalue())
            self.assertTrue(
                any(
                    "Pack command" in diagnostic
                    and "could not start" in diagnostic
                    and "missing-packer.exe" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_reports_cargo_launch_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(out=root / "out", validate_report=validate_report)
            args.dry_run = False
            args.cargo = str(root / "missing-cargo.exe")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_compile_host(args)

            report_path = root / "out" / "stages" / "compile_host" / "report.json"
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["exit_code"], 2)
            self.assertEqual(report["stdout_lines"], [])
            self.assertEqual(report["stderr_lines"], [])
            self.assertIn("missing-cargo.exe", stdout.getvalue())
            self.assertTrue(
                any(
                    "CompileHost cargo command" in diagnostic
                    and "could not start" in diagnostic
                    and "missing-cargo.exe" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_reports_cargo_launch_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            args = _source_template_args(
                out=root / "out",
                validate_report=validate_report,
                build=True,
                dry_run=False,
            )
            args.cargo = str(root / "missing-cargo.exe")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_source_template(args)

            report_path = root / "out" / "stages" / "source_template" / "report.json"
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(report["build_executed"])
            self.assertEqual(report["build_validation"]["status"], "failed")
            self.assertIsNone(report["build_validation"]["exit_code"])
            self.assertEqual(report["build_validation"]["stdout_lines"], [])
            self.assertEqual(report["build_validation"]["stderr_lines"], [])
            self.assertIn("missing-cargo.exe", stdout.getvalue())
            self.assertTrue(
                any(
                    "SourceTemplate cargo build command" in diagnostic
                    and "could not start" in diagnostic
                    and "missing-cargo.exe" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
