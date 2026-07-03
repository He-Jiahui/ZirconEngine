from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_source_template_report,
    _write_validate_report_with_strategies,
)


class PipelineReportSourceTemplateValidateBuildPlanTests(unittest.TestCase):
    def test_report_rejects_missing_source_template_validate_build_plan(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
            validate_report["plan_summary"].pop("source_template_build")
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate plan_summary.source_template_build must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_malformed_source_template_validate_build_plan_command(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"command": []},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_blank_source_template_validate_build_plan_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"command": ["cargo", ""]},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_non_string_command_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"command": ["cargo", 42]},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
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
                    "SourceTemplate Validate source_template_build command "
                    "must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_padded_source_template_validate_build_plan_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"command": [" cargo ", "build"]},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command[0] "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_missing_required_field(
        self,
    ) -> None:
        missing_fields = (
            (
                "cargo_profile",
                "SourceTemplate Validate source_template_build cargo_profile must be a non-empty string",
            ),
            (
                "command",
                "SourceTemplate Validate source_template_build command must be a non-empty string array",
            ),
            (
                "manifest_path",
                "SourceTemplate Validate source_template_build manifest_path must be a non-empty string",
            ),
            (
                "release",
                "SourceTemplate Validate source_template_build release must be a boolean",
            ),
            (
                "target_dir",
                "SourceTemplate Validate source_template_build target_dir must be a non-empty string",
            ),
        )
        for field, expected_diagnostic in missing_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _remove_source_template_validate_build_plan_field(out, field)
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("Validate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_validate_build_plan_blank_required_string_field(
        self,
    ) -> None:
        blank_fields = (
            (
                "cargo_profile",
                "SourceTemplate Validate source_template_build cargo_profile must be a non-empty string",
            ),
            (
                "manifest_path",
                "SourceTemplate Validate source_template_build manifest_path must be a non-empty string",
            ),
            (
                "target_dir",
                "SourceTemplate Validate source_template_build target_dir must be a non-empty string",
            ),
        )
        for field, expected_diagnostic in blank_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _update_source_template_validate_build_plan(out, {field: "   "})
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("Validate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_validate_build_plan_padded_required_string_field(
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
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _update_source_template_validate_build_plan(
                        out,
                        {field: f" {field}-value "},
                    )
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn("Validate", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            "SourceTemplate Validate source_template_build "
                            f"{field} must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_validate_build_plan_option_value(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"command": ["cargo", "build", "--manifest-path", "--release"]},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build command --manifest-path value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_target_triple_override(
        self,
    ) -> None:
        forbidden_commands = (
            [
                "cargo",
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--target-dir",
                "stages/source_template/target",
                "--target",
                "x86_64-unknown-linux-gnu",
            ],
            [
                "cargo",
                "build",
                "--manifest-path",
                "Cargo.toml",
                "--target-dir",
                "stages/source_template/target",
                "--target=x86_64-unknown-linux-gnu",
            ],
        )
        for command in forbidden_commands:
            with self.subTest(command=command):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    _update_source_template_validate_build_plan(
                        out,
                        {"command": command},
                    )
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn(
                        "SourceTemplate Validate source_template_build command "
                        "must not include --target because export target descriptor "
                        "owns platform target selection",
                        report["diagnostics"],
                    )

    def test_report_rejects_source_template_validate_build_plan_profile_release_mismatch(
        self,
    ) -> None:
        cases: tuple[tuple[str, dict[str, object], str], ...] = (
            (
                "unsupported profile",
                {"cargo_profile": "shipping"},
                "SourceTemplate Validate source_template_build cargo_profile "
                "must be debug or release",
            ),
            (
                "release field mismatch",
                {"cargo_profile": "release", "release": False},
                "SourceTemplate Validate source_template_build release "
                "must match cargo_profile",
            ),
            (
                "debug command with release flag",
                {"command_suffix": ["--release"]},
                "SourceTemplate Validate source_template_build command "
                "must not include --release for debug profile",
            ),
            (
                "release command without release flag",
                {"cargo_profile": "release", "release": True},
                "SourceTemplate Validate source_template_build command "
                "must include --release for release profile",
            ),
        )
        for name, updates, expected_diagnostic in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["source_template"])
                    validate_report_path = out / "stages" / "validate" / "report.json"
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    source_plan = validate_report["plan_summary"][
                        "source_template_build"
                    ]
                    plan_updates = dict(updates)
                    command_suffix = plan_updates.pop("command_suffix", [])
                    command = list(source_plan["command"])
                    command.extend(command_suffix)
                    source_plan["command"] = command
                    source_plan.update(plan_updates)
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )
                    _write_source_template_report(out)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertIn(expected_diagnostic, report["diagnostics"])

    def test_report_rejects_malformed_source_template_validate_build_plan_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"manifest_path": ""},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build manifest_path must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_absolute_source_template_validate_build_plan_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"manifest_path": str(root / "Cargo.toml")},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build manifest_path"
                    in diagnostic
                    and "must be relative" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_escaped_source_template_validate_build_plan_manifest_path(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"manifest_path": "../Cargo.toml"},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build manifest_path"
                    in diagnostic
                    and "escapes the project" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_malformed_source_template_validate_build_plan_target_dir(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"target_dir": ""},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build target_dir must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_target_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"target_dir": str(root / "other-target")},
            )
            _write_source_template_report(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build target_dir must match current SourceTemplate stage target"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_source_template_validate_build_plan_target_dir_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            target_dir = root / "other-target"
            _write_validate_report_with_strategies(out, ["source_template"])
            _update_source_template_validate_build_plan(
                out,
                {"target_dir": str(target_dir)},
            )
            _write_source_template_report(out)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if path == target_dir:
                    raise OSError("simulated source template target_dir resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "SourceTemplate Validate source_template_build target_dir"
                    in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated source template target_dir resolve failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _update_source_template_validate_build_plan(
    out: Path,
    updates: dict[str, object],
) -> None:
    validate_report_path = out / "stages" / "validate" / "report.json"
    validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
    validate_report["plan_summary"]["source_template_build"].update(updates)
    validate_report_path.write_text(
        json.dumps(validate_report, indent=2),
        encoding="utf-8",
    )


def _remove_source_template_validate_build_plan_field(out: Path, field: str) -> None:
    validate_report_path = out / "stages" / "validate" / "report.json"
    validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
    validate_report["plan_summary"]["source_template_build"].pop(field)
    validate_report_path.write_text(
        json.dumps(validate_report, indent=2),
        encoding="utf-8",
    )


if __name__ == "__main__":
    unittest.main()
