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


class SourceTemplatePlanCommandSchemaTests(unittest.TestCase):
    def test_source_template_rejects_plan_with_padded_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            command = list(payload["plan_summary"]["source_template_build"]["command"])
            command[0] = " cargo "
            payload["plan_summary"]["source_template_build"]["command"] = command
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")
            out = root / "out"

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=out,
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            stage_dir = out / "stages" / "source_template"
            report = json_loads(
                (stage_dir / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertFalse((stage_dir / "project").exists())
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command[0] "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_plan_with_non_string_command_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            command = list(payload["plan_summary"]["source_template_build"]["command"])
            command.insert(1, 42)
            payload["plan_summary"]["source_template_build"]["command"] = command
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")
            out = root / "out"

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=out,
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            stage_dir = out / "stages" / "source_template"
            report = json_loads(
                (stage_dir / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertFalse((stage_dir / "project").exists())
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command[1] "
                    "must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "SourceTemplate build plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
