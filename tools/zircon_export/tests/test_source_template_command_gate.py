from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _run_source_template_quiet,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class SourceTemplateCommandGateTests(unittest.TestCase):
    def test_source_template_rejects_plan_with_blank_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_plan_with_dangling_manifest_path_option(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--manifest-path",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --manifest-path must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_manifest_path_option_with_option_value(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--manifest-path",
                "--release",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --manifest-path value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_plan_with_duplicate_manifest_path_option(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--manifest-path",
                "other/Cargo.toml",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --manifest-path must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_plan_with_dangling_target_dir_option(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--target-dir",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --target-dir must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_target_dir_option_with_option_value(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--target-dir",
                "--release",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --target-dir value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_plan_with_duplicate_target_dir_option(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "build",
                "--target-dir",
                "target-a",
                "--target-dir",
                "target-b",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertTrue(
                any(
                    "SourceTemplate build plan command --target-dir must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
