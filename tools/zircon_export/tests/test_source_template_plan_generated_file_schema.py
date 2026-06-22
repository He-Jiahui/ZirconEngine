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


class SourceTemplatePlanGeneratedFileSchemaTests(unittest.TestCase):
    def test_source_template_rejects_plan_with_padded_generated_file_string_field(
        self,
    ) -> None:
        padded_fields = (
            (
                "path",
                " Cargo.toml ",
                "SourceTemplate Validate generated file path must be a non-empty trimmed string",
            ),
            (
                "purpose",
                " generated runtime package manifest ",
                "SourceTemplate Validate generated_files[0].purpose must be a non-empty trimmed string",
            ),
        )
        for field, value, expected_diagnostic in padded_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    payload["plan_summary"]["generated_files"][0][field] = value
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
                    self.assertEqual(report["generated_files"], [])
                    self.assertFalse((stage_dir / "project").exists())
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
