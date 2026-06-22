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


class SourceTemplatePlanStringSchemaTests(unittest.TestCase):
    def test_source_template_rejects_plan_with_padded_required_string_field(
        self,
    ) -> None:
        padded_fields = (
            "cargo_profile",
            "manifest_path",
            "target_dir",
        )
        for field in padded_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    source_plan = payload["plan_summary"]["source_template_build"]
                    source_plan[field] = f" {field}-value "
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
                            "SourceTemplate Validate source_template_build "
                            f"{field} must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
