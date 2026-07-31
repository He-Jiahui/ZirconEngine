from __future__ import annotations

import contextlib
import hashlib
import io
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.source_template import run_source_template
from tools.zircon_export.pipeline_report_validate_stage_schema import (
    validate_report_schema_diagnostics,
)
from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _source_template_args,
    _source_template_validate_report,
)
from tools.zircon_export.validate_stage import validate_command


class SourceTemplateContentsArtifactTests(unittest.TestCase):
    def test_non_fatal_validate_report_requires_schema_v2_for_every_strategy(self) -> None:
        diagnostics = validate_report_schema_diagnostics(
            {
                "stage": "Validate",
                "profile": "windows-release",
                "project_manifest": "zircon-project.toml",
                "stage_output": "stages/validate",
                "profile_found": True,
                "fatal": False,
                "diagnostics": [],
                "fatal_diagnostics": [],
                "profile_summary": {},
                "plan_summary": {},
            }
        )

        self.assertIn(
            "non-fatal validate report schema_version must be 2",
            diagnostics,
        )

    def test_source_template_rejects_missing_validate_report_schema_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            del payload["schema_version"]
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            stage_dir = root / "out" / "stages" / "source_template"
            report = json.loads(
                (stage_dir / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((stage_dir / "project").exists())
            self.assertIn(
                "non-fatal validate report schema_version must be 2",
                report["diagnostics"],
            )

    def test_source_template_rejects_unknown_validate_root_before_project_reset(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out_root = root / "out"
            project_dir = out_root / "stages" / "source_template" / "project"
            project_dir.mkdir(parents=True)
            sentinel = project_dir / "preserve.txt"
            sentinel.write_text("preserve", encoding="utf-8")
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            payload["unexpected_root_field"] = True
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=out_root,
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            report = json.loads(
                (
                    out_root
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(sentinel.read_text(encoding="utf-8"), "preserve")
            self.assertIn(
                "validate report unknown field unexpected_root_field",
                report["diagnostics"],
            )

    def test_validate_command_requests_explicit_generated_contents_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            stage_dir = root / "out" / "stages" / "validate"
            report_path = stage_dir / "report.json"
            args = _export_args(out=root / "out", stage="validate", dry_run=True)

            command = validate_command(
                args,
                root,
                root / "zircon-project.toml",
                stage_dir,
                report_path,
                validator=root / "zircon_export_validate",
            )

            artifact_index = command.index("--contents-artifact")
            self.assertEqual(
                Path(command[artifact_index + 1]),
                stage_dir / "generated-contents.json",
            )

    def test_source_template_materializes_from_explicit_contents_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            stage_dir = root / "out" / "stages" / "source_template"
            report = json.loads(
                (stage_dir / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                (stage_dir / "project" / "src" / "main.rs").read_text(
                    encoding="utf-8"
                ),
                "fn main() {}\n",
            )

    def test_source_template_rejects_missing_contents_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "missing-generated-contents.json"
            payload, _ = compact_validate_handoff(artifact_path)
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            stage_dir = root / "out" / "stages" / "source_template"
            report = json.loads(
                (stage_dir / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((stage_dir / "project").exists())
            self.assertTrue(
                any(
                    "generated contents artifact" in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_wrong_contents_artifact_schema_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            artifact = json.loads(artifact_text)
            artifact["schema_version"] = 2
            artifact_text = json.dumps(artifact)
            payload["generated_contents_artifact_byte_length"] = len(
                artifact_text.encode("utf-8")
            )
            payload["generated_contents_artifact_digest"] = hashlib.sha256(
                artifact_text.encode("utf-8")
            ).hexdigest()
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            report = json.loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(
                any(
                    "generated contents artifact schema_version must be 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_boolean_contents_artifact_schema_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            artifact = json.loads(artifact_text)
            artifact["schema_version"] = True
            artifact_text = json.dumps(artifact)
            payload["generated_contents_artifact_byte_length"] = len(
                artifact_text.encode("utf-8")
            )
            payload["generated_contents_artifact_digest"] = hashlib.sha256(
                artifact_text.encode("utf-8")
            ).hexdigest()
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            report = json.loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertIn(
                "generated contents artifact schema_version must be 1",
                report["diagnostics"],
            )

    def test_source_template_rejects_compact_row_byte_length_drift(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            payload["plan_summary"]["generated_files"][0]["byte_length"] += 1
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            report = json.loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(
                any(
                    "generated contents artifact byte length for Cargo.toml"
                    " does not match compact Validate metadata"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_same_length_artifact_content_tampering(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            artifact_path = root / "generated-contents.json"
            payload, artifact_text = compact_validate_handoff(artifact_path)
            artifact = json.loads(artifact_text)
            artifact["generated_files"][1]["contents"] = "fn maim() {}\n"
            tampered_artifact_text = json.dumps(artifact)
            self.assertEqual(len(tampered_artifact_text), len(artifact_text))
            validate_report.write_text(json.dumps(payload, indent=2), encoding="utf-8")
            artifact_path.write_text(tampered_artifact_text, encoding="utf-8")

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = run_source_template(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=False,
                        dry_run=False,
                    )
                )

            report = json.loads(
                (
                    root
                    / "out"
                    / "stages"
                    / "source_template"
                    / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(
                any(
                    "generated contents artifact SHA-256 does not match Validate report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def compact_validate_handoff(artifact_path: Path) -> tuple[dict[str, object], str]:
    payload = _source_template_validate_report()
    fixture_path = Path(payload["generated_contents_artifact_path"])
    artifact = json.loads(fixture_path.read_text(encoding="utf-8"))
    artifact_text = json.dumps(artifact)
    for compact_file, artifact_file in zip(
        payload["plan_summary"]["generated_files"],
        artifact["generated_files"],
    ):
        compact_file["content_digest"] = hashlib.sha256(
            artifact_file["contents"].encode("utf-8")
        ).hexdigest()
    payload["generated_contents_artifact_path"] = str(artifact_path)
    payload["generated_contents_artifact_byte_length"] = len(
        artifact_text.encode("utf-8")
    )
    payload["generated_contents_artifact_digest"] = hashlib.sha256(
        artifact_text.encode("utf-8")
    ).hexdigest()
    return payload, artifact_text


if __name__ == "__main__":
    unittest.main()
