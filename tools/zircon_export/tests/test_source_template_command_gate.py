from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _run_source_template_quiet,
    _source_template_args,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class SourceTemplateCommandGateTests(unittest.TestCase):
    def test_source_template_rejects_non_cargo_build_plan_before_execution(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            payload["plan_summary"]["source_template_build"]["command"] = [
                "cargo",
                "check",
                "--manifest-path",
                "Cargo.toml",
                "--target-dir",
                "stages/source_template/target",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(json_dumps(payload), encoding="utf-8")
            calls: list[list[str]] = []

            def build_success(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                calls.append(command)
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout="cargo stdout line\n",
                    stderr="cargo stderr line\n",
                )

            with mock.patch(
                "tools.zircon_export.source_template.subprocess.run",
                side_effect=build_success,
            ):
                exit_code = _run_source_template_quiet(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        build=True,
                        dry_run=False,
                    )
                )

            stage_dir = root / "out" / "stages" / "source_template"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertEqual(calls, [])
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertFalse((stage_dir / "project").exists())
            self.assertTrue(
                any(
                    "SourceTemplate build plan command must run cargo build"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_rejects_target_triple_override_before_execution(
        self,
    ) -> None:
        for target_args in (
            ["--target", "x86_64-unknown-linux-gnu"],
            ["--target=x86_64-unknown-linux-gnu"],
        ):
            with self.subTest(target_args=target_args):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    command = list(
                        payload["plan_summary"]["source_template_build"]["command"]
                    )
                    command.extend(target_args)
                    payload["plan_summary"]["source_template_build"][
                        "command"
                    ] = command
                    validate_report = root / "validate.json"
                    validate_report.write_text(json_dumps(payload), encoding="utf-8")
                    calls: list[list[str]] = []

                    def build_success(
                        command: list[str],
                        **kwargs: object,
                    ) -> subprocess.CompletedProcess[str]:
                        calls.append(command)
                        return subprocess.CompletedProcess(
                            command,
                            0,
                            stdout="cargo stdout line\n",
                            stderr="cargo stderr line\n",
                        )

                    with mock.patch(
                        "tools.zircon_export.source_template.subprocess.run",
                        side_effect=build_success,
                    ):
                        exit_code = _run_source_template_quiet(
                            _source_template_args(
                                out=root / "out",
                                validate_report=validate_report,
                                build=True,
                                dry_run=False,
                            )
                        )

                    stage_dir = root / "out" / "stages" / "source_template"
                    report = json_loads(
                        (stage_dir / "report.json").read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertEqual(calls, [])
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertFalse((stage_dir / "project").exists())
                    self.assertTrue(
                        any(
                            "SourceTemplate build plan command must not include "
                            "--target because export target descriptor owns "
                            "platform target selection"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_source_template_rejects_profile_release_mismatch_before_execution(
        self,
    ) -> None:
        cases: tuple[tuple[str, dict[str, object], str], ...] = (
            (
                "unsupported profile",
                {"cargo_profile": "shipping"},
                "SourceTemplate build plan cargo_profile must be debug or release",
            ),
            (
                "release flag mismatch",
                {"cargo_profile": "release", "release": False},
                "SourceTemplate build plan release must match cargo_profile",
            ),
            (
                "debug command with release flag",
                {"command_suffix": ["--release"]},
                "SourceTemplate build plan command must not include --release "
                "for debug profile",
            ),
            (
                "release command without release flag",
                {"cargo_profile": "release", "release": True},
                "SourceTemplate build plan command must include --release "
                "for release profile",
            ),
        )
        for name, updates, expected_diagnostic in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    payload = _source_template_validate_report()
                    source_plan = payload["plan_summary"]["source_template_build"]
                    plan_updates = dict(updates)
                    command_suffix = plan_updates.pop("command_suffix", [])
                    command = list(source_plan["command"])
                    command.extend(command_suffix)
                    source_plan["command"] = command
                    source_plan.update(plan_updates)
                    validate_report = root / "validate.json"
                    validate_report.write_text(json_dumps(payload), encoding="utf-8")
                    calls: list[list[str]] = []

                    def build_success(
                        command: list[str],
                        **kwargs: object,
                    ) -> subprocess.CompletedProcess[str]:
                        calls.append(command)
                        return subprocess.CompletedProcess(
                            command,
                            0,
                            stdout="cargo stdout line\n",
                            stderr="cargo stderr line\n",
                        )

                    with mock.patch(
                        "tools.zircon_export.source_template.subprocess.run",
                        side_effect=build_success,
                    ):
                        exit_code = _run_source_template_quiet(
                            _source_template_args(
                                out=root / "out",
                                validate_report=validate_report,
                                build=True,
                                dry_run=False,
                            )
                        )

                    stage_dir = root / "out" / "stages" / "source_template"
                    report = json_loads(
                        (stage_dir / "report.json").read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    self.assertEqual(calls, [])
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertFalse((stage_dir / "project").exists())
                    self.assertIn(expected_diagnostic, report["diagnostics"])

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
                    "SourceTemplate Validate source_template_build command must be a non-empty string array"
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
