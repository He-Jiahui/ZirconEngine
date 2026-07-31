from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateTests(unittest.TestCase):
    def test_report_stage_uses_source_template_profile_requirements(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_stage_report(out, "source_template", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate", "source_template"],
            )

    def test_report_rejects_missing_source_template_generated_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            (project_dir / "src" / "main.rs").unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate generated file" in diagnostic
                    and "src/main.rs" in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_without_project(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out, report_overrides={"project": ""})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report project must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_project_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out).resolve()
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == project_dir:
                    raise OSError("simulated source template project resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate report project" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template project resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_stage_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            report_path = (out / "stages" / "source_template" / "report.json").resolve()
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == report_path:
                    raise OSError("simulated source template stage path resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate stage report path" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template stage path resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_project_outside_stage_dir(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_project = root / "external_project"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            (external_project / "src").mkdir(parents=True)
            cargo_toml = external_project / "Cargo.toml"
            main_rs = external_project / "src" / "main.rs"
            cargo_toml.write_text(
                (
                    "[package]\n"
                    "name = \"source-template-smoke\"\n"
                    "version = \"0.1.0\"\n"
                    "edition = \"2021\"\n"
                ),
                encoding="utf-8",
            )
            main_rs.write_text("fn main() {}\n", encoding="utf-8")
            generated_files = []
            for relative_path in ("Cargo.toml", "src/main.rs"):
                output = external_project / relative_path
                contents = output.read_bytes()
                generated_files.append(
                    {
                        "path": relative_path,
                        "purpose": "external generated file",
                        "size": len(contents),
                        "sha256": hashlib.sha256(contents).hexdigest(),
                    }
                )
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(external_project / "Cargo.toml"),
            ]
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report.update(
                {
                    "project": str(external_project),
                    "generated_files": generated_files,
                    "command": command,
                    "build_validation": {
                        "requested": False,
                        "executed": False,
                        "status": "skipped",
                        "exit_code": None,
                        "working_dir": str(external_project),
                        "command": command,
                    },
                }
            )
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report project must match current SourceTemplate stage project"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_command_manifest_path_outside_project(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside_manifest = root / "Cargo.toml"
            outside_manifest.write_text("[package]\nname = \"outside\"\n", encoding="utf-8")
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(outside_manifest),
            ]
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["command"] = command
            stage_report["build_validation"]["command"] = command
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report command manifest-path must target current project Cargo.toml"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_command_manifest_path_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            manifest_path = root / "Cargo.toml"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            command = [
                "cargo",
                "build",
                "--manifest-path",
                str(manifest_path),
            ]
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["command"] = command
            stage_report["build_validation"]["command"] = command
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == manifest_path:
                    raise OSError(
                        "simulated source template command manifest resolve failure"
                    )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate report command manifest-path" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template command manifest resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_outside_project(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "generated_files": [
                        {
                            "path": "../escape.txt",
                            "purpose": "escaped output",
                            "size": 0,
                            "sha256": "0" * 64,
                        }
                    ],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file path ../escape.txt escapes the project"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_path_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            generated_path = (project_dir / "src" / "main.rs").resolve()
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == generated_path:
                    raise OSError(
                        "simulated source template generated file path resolve failure"
                    )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate generated file path src/main.rs"
                    in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template generated file path resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            (project_dir / "src" / "main.rs").unlink()
            (project_dir / "src" / "main.rs").mkdir()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            unreadable_file = (project_dir / "src" / "main.rs").resolve()
            original_read_bytes = Path.read_bytes

            def read_bytes_or_fail(path: Path) -> bytes:
                if path.resolve() == unreadable_file:
                    raise OSError("simulated read failure")
                return original_read_bytes(path)

            with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs could not be read"
                    in diagnostic
                    and "simulated read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            (project_dir / "src" / "main.rs").write_text(
                "fn main() { panic!(\"mutated\"); }\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs sha256" in diagnostic
                    and "does not match report sha256" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_missing_size(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "generated_files": [
                        {
                            "path": "Cargo.toml",
                            "purpose": "generated runtime package manifest",
                            "sha256": "0" * 64,
                        }
                    ],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file Cargo.toml size must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_missing_sha256(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "generated_files": [
                        {
                            "path": "Cargo.toml",
                            "purpose": "generated runtime package manifest",
                            "size": 1,
                        }
                    ],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file Cargo.toml sha256 must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_malformed_content_evidence(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "generated_files": [
                        {
                            "path": "Cargo.toml",
                            "purpose": "generated runtime package manifest",
                            "size": "1",
                            "sha256": "not-a-hash",
                        }
                    ],
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate generated file Cargo.toml size must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "SourceTemplate generated file Cargo.toml sha256 must be a 64-character hex string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_generated_file_missing_from_plan(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["generated_files"] = [
                file
                for file in stage_report["generated_files"]
                if file["path"] != "src/main.rs"
            ]
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report missing generated file from Validate plan: src/main.rs"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_unplanned_generated_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            project_dir = _write_source_template_report(out)
            extra = project_dir / "src" / "extra.rs"
            extra.write_text("pub fn extra() {}\n", encoding="utf-8")
            contents = extra.read_bytes()
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["generated_files"].append(
                {
                    "path": "src/extra.rs",
                    "purpose": "unexpected generated source",
                    "size": len(contents),
                    "sha256": hashlib.sha256(contents).hexdigest(),
                }
            )
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report generated file src/extra.rs is not declared by Validate plan"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_invalid_source_template_validate_generated_files(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"] = "Cargo.toml"
            validate_path.write_text(json.dumps(validate_report, indent=2), encoding="utf-8")
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate plan_summary.generated_files must be a list"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_non_object_source_template_validate_generated_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"] = ["Cargo.toml"]
            validate_path.write_text(json.dumps(validate_report, indent=2), encoding="utf-8")
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate generated file entry must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_duplicate_source_template_validate_generated_file_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"].append(
                {
                    "path": "src/main.rs",
                    "purpose": "duplicate generated runtime entrypoint",
                    "byte_length": len("fn main() {}\n".encode("utf-8")),
                    "content_digest": "0" * 64,
                }
            )
            validate_path.write_text(json.dumps(validate_report, indent=2), encoding="utf-8")
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate generated file path src/main.rs is duplicated"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_duplicate_source_template_report_generated_file_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            report_path = out / "stages" / "source_template" / "report.json"
            stage_report = json.loads(report_path.read_text(encoding="utf-8"))
            stage_report["generated_files"].append(stage_report["generated_files"][1])
            report_path.write_text(json.dumps(stage_report, indent=2), encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report generated file path src/main.rs is duplicated"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_absolute_source_template_validate_generated_file_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"][0]["path"] = str(
                out / "escape.toml"
            )
            validate_path.write_text(json.dumps(validate_report, indent=2), encoding="utf-8")
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate generated file path "
                    in diagnostic
                    and "must be relative"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_escaped_source_template_validate_generated_file_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"]["generated_files"][0]["path"] = "../escape.toml"
            validate_path.write_text(json.dumps(validate_report, indent=2), encoding="utf-8")
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate generated file path ../escape.toml escapes the project"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_report_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(
                out,
                report_overrides={
                    "validate_report": str(root / "other" / "stages" / "validate" / "report.json")
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate report validate_report must match current Validate report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_report_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _write_source_template_report(out)
            validate_report_path = (out / "stages" / "validate" / "report.json").resolve()
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == validate_report_path:
                    raise OSError("simulated source template validate report resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate report validate_report" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template validate report resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_requires_source_template_for_source_template_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("source_template", report["missing_stages"])
            self.assertNotIn("compile_host", report["missing_stages"])
if __name__ == "__main__":
    unittest.main()
