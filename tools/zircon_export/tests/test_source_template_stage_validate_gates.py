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


class SourceTemplateStageValidateGateTests(unittest.TestCase):
    def test_source_template_stage_rejects_empty_build_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = []
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
            self.assertIn(
                "SourceTemplate Validate source_template_build command "
                "must be a non-empty string array",
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_invalid_validate_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["fatal"] = []
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
            self.assertFalse((root / "out" / "stages" / "source_template" / "project").exists())
            self.assertTrue(
                any(
                    "Validate report fatal must be a boolean" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_validate_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.mkdir()

            exit_code = _run_source_template_quiet(
                _source_template_args(
                    out=root / "out",
                    validate_report=validate_report,
                    build=False,
                    dry_run=False,
                )
            )

            stage_report = json_loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(stage_report["fatal"])
            self.assertTrue(
                any(
                    f"validate report {validate_report} is not a file"
                    in diagnostic
                    for diagnostic in stage_report["diagnostics"]
                ),
                stage_report["diagnostics"],
            )

    def test_source_template_stage_requires_source_template_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["profile_summary"] = {"strategies": ["library_embed"]}
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

            project = root / "out" / "stages" / "source_template" / "project"
            report = json_loads(
                (root / "out" / "stages" / "source_template" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertFalse(project.exists())
            self.assertTrue(
                any(
                    "SourceTemplate stage requires the source_template strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_invalid_strategy_metadata(self) -> None:
        cases = (
            ("source_template", "profile_summary.strategies must be a list"),
            (
                [],
                "profile_summary.strategies must include at least one supported export strategy",
            ),
            (["source_template", "ghost_path"], "unsupported export strategy ghost_path"),
        )
        for strategies, expected_diagnostic in cases:
            with self.subTest(strategies=strategies):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    payload["profile_summary"] = {"strategies": strategies}
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

                    project = root / "out" / "stages" / "source_template" / "project"
                    report = json_loads(
                        (
                            root
                            / "out"
                            / "stages"
                            / "source_template"
                            / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertFalse(project.exists())
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_source_template_stage_rejects_escaped_manifest_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["manifest_path"] = "../Cargo.toml"
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
            self.assertFalse((root / "out" / "stages" / "source_template" / "project").exists())
            self.assertTrue(
                any(
                    "SourceTemplate build plan manifest_path ../Cargo.toml escapes the generated project"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_marks_invalid_generated_file_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            generated_files = payload["plan_summary"]["generated_files"]
            generated_files.append(
                {
                    "path": "../escape.txt",
                    "purpose": "invalid generated file outside project",
                    "byte_length": len("escape".encode("utf-8")),
                    "content_digest": "0" * 64,
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
            self.assertFalse(project.exists())
            self.assertFalse((stage_dir / "escape.txt").exists())
            self.assertTrue(
                any(
                    "escapes the SourceTemplate project" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )




if __name__ == "__main__":
    unittest.main()
