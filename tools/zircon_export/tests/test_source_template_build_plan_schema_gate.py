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


class SourceTemplateBuildPlanSchemaGateTests(unittest.TestCase):
    def test_source_template_stage_rejects_invalid_build_plan_schema(
        self,
    ) -> None:
        def missing_cargo_profile(payload: dict[str, object]) -> None:
            payload["plan_summary"]["source_template_build"].pop("cargo_profile")

        def blank_manifest_path(payload: dict[str, object]) -> None:
            payload["plan_summary"]["source_template_build"]["manifest_path"] = "   "

        def malformed_release(payload: dict[str, object]) -> None:
            payload["plan_summary"]["source_template_build"]["release"] = "false"

        def unknown_field(payload: dict[str, object]) -> None:
            payload["plan_summary"]["source_template_build"][
                "unsigned_sidecar"
            ] = "sidecar.bin"

        def blank_command_entry(payload: dict[str, object]) -> None:
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "",
            ]

        cases = (
            (
                "missing cargo_profile",
                missing_cargo_profile,
                "SourceTemplate Validate source_template_build cargo_profile must be a non-empty string",
            ),
            (
                "blank manifest_path",
                blank_manifest_path,
                "SourceTemplate Validate source_template_build manifest_path must be a non-empty string",
            ),
            (
                "malformed release",
                malformed_release,
                "SourceTemplate Validate source_template_build release must be a boolean",
            ),
            (
                "unknown field",
                unknown_field,
                "SourceTemplate Validate source_template_build unknown field unsigned_sidecar",
            ),
            (
                "blank command entry",
                blank_command_entry,
                "SourceTemplate Validate source_template_build command must be a non-empty string array",
            ),
        )

        for name, mutate_payload, expected_diagnostic in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    mutate_payload(payload)
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

                    stage_dir = root / "out" / "stages" / "source_template"
                    project = stage_dir / "project"
                    report = json_loads(
                        (stage_dir / "report.json").read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["generated_files"], [])
                    self.assertEqual(report["command"], [])
                    self.assertFalse(project.exists())
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
