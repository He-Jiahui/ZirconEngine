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


class SourceTemplateGeneratedFilesGateTests(unittest.TestCase):
    def test_source_template_stage_rejects_invalid_generated_file_plan_rows(
        self,
    ) -> None:
        def blank_path(payload: dict[str, object]) -> None:
            payload["plan_summary"]["generated_files"][0]["path"] = "   "

        def blank_purpose(payload: dict[str, object]) -> None:
            payload["plan_summary"]["generated_files"][0]["purpose"] = "   "

        def missing_contents(payload: dict[str, object]) -> None:
            payload["plan_summary"]["generated_files"][0].pop("contents")

        def non_object_row(payload: dict[str, object]) -> None:
            payload["plan_summary"]["generated_files"].append("src/sidecar.rs")

        def unknown_field(payload: dict[str, object]) -> None:
            payload["plan_summary"]["generated_files"][0][
                "unsigned_sidecar"
            ] = "sidecar.bin"

        cases = (
            (
                "blank path",
                blank_path,
                "SourceTemplate Validate generated file path must be a non-empty string",
            ),
            (
                "blank purpose",
                blank_purpose,
                "SourceTemplate Validate generated_files[0].purpose must be a non-empty string",
            ),
            (
                "missing contents",
                missing_contents,
                "SourceTemplate Validate generated_files[0].contents must be a string",
            ),
            (
                "non-object row",
                non_object_row,
                "SourceTemplate Validate generated file entry must be an object",
            ),
            (
                "unknown field",
                unknown_field,
                "SourceTemplate Validate generated_files[0] unknown field unsigned_sidecar",
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

    def test_source_template_stage_rejects_duplicate_generated_file_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            generated_files = payload["plan_summary"]["generated_files"]
            generated_files.append(
                {
                    "path": "src/main.rs",
                    "purpose": "duplicate generated runtime entrypoint",
                    "contents": "fn main() {}\n",
                }
            )
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
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["generated_files"], [])
            self.assertFalse(project.exists())
            self.assertTrue(
                any(
                    "SourceTemplate generated file path src/main.rs is duplicated"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
