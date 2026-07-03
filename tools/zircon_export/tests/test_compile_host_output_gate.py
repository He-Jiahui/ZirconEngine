from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _compile_host_args,
    _compile_host_plan,
    _run_compile_host_quiet,
    json_dumps,
    json_loads,
)


class CompileHostOutputGateTests(unittest.TestCase):
    def test_compile_host_rejects_plan_without_binary(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan.pop("binary")
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan binary must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_without_cargo_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan.pop("cargo_profile")
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=0,
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan cargo_profile must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_invalid_profile_release(
        self,
    ) -> None:
        cases: tuple[tuple[str, dict[str, object], str], ...] = (
            (
                "invalid_profile",
                {"cargo_profile": "shipping"},
                "CompileHost plan cargo_profile must be debug or release",
            ),
            (
                "release_non_bool",
                {"release": "debug"},
                "CompileHost plan release must be a boolean",
            ),
            (
                "release_true_debug_profile",
                {"release": True, "cargo_profile": "debug"},
                "CompileHost plan release must match cargo_profile",
            ),
            (
                "release_false_release_profile",
                {"release": False, "cargo_profile": "release"},
                "CompileHost plan release must match cargo_profile",
            ),
        )
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan.update(overrides)
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_padded_cargo_profile_before_profile_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["cargo_profile"] = " release "
            compile_plan["release"] = True
            compile_plan["command"] = [
                *_compile_host_plan()["command"],
                "--release",
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as cargo_call:
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            cargo_call.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost plan cargo_profile must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "CompileHost plan cargo_profile must be debug or release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_missing_required_evidence_field(
        self,
    ) -> None:
        cases = (
            "package",
            "manifest_path",
            "target_dir",
            "app_features",
            "runtime_features",
            "expected_runtime_plugins",
            "linked_runtime_crates",
        )
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    del compile_plan[field]
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            f"CompileHost plan {field} is required" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_malformed_string_evidence_field(
        self,
    ) -> None:
        cases: tuple[tuple[str, object, str], ...] = (
            (
                "package",
                "",
                "CompileHost plan package must be a non-empty trimmed string",
            ),
            (
                "binary",
                " zircon_runtime",
                "CompileHost plan binary must be a non-empty trimmed string",
            ),
            (
                "manifest_path",
                "../Cargo.toml",
                "CompileHost plan manifest_path must be a safe relative path",
            ),
            (
                "target_dir",
                "stages/../target",
                "CompileHost plan target_dir must be a safe relative path",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan[field] = value
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_invalid_target_selector(
        self,
    ) -> None:
        cases: tuple[tuple[str, str, str, str], ...] = (
            (
                "package",
                "zircon_tools",
                "-p",
                "CompileHost plan package must be zircon_app",
            ),
            (
                "binary",
                "zircon_tools",
                "--bin",
                "CompileHost plan binary must be zircon_runtime or zircon_editor",
            ),
        )
        for field, value, option, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan[field] = value
                    command = list(_compile_host_plan()["command"])
                    command[command.index(option) + 1] = value
                    compile_plan["command"] = command
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_non_string_evidence_field(
        self,
    ) -> None:
        cases: tuple[tuple[str, object, str], ...] = (
            ("package", 7, "CompileHost plan package must be a string"),
            (
                "manifest_path",
                ["Cargo.toml"],
                "CompileHost plan manifest_path must be a string",
            ),
            (
                "target_dir",
                {"path": "stages/compile_host/target"},
                "CompileHost plan target_dir must be a string",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan[field] = value
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_malformed_array_evidence_field(
        self,
    ) -> None:
        cases: tuple[tuple[str, object, str], ...] = (
            (
                "app_features",
                "target-client",
                "CompileHost plan app_features must be a string array",
            ),
            (
                "runtime_features",
                ["target-client", " "],
                "CompileHost plan runtime_features must not contain blank entries",
            ),
            (
                "expected_runtime_plugins",
                ["Animation"],
                "CompileHost plan expected_runtime_plugins[0] must start with a lowercase ASCII letter",
            ),
            (
                "linked_runtime_crates",
                "not-array",
                "CompileHost plan linked_runtime_crates must be an object array",
            ),
            (
                "linked_runtime_crates",
                [
                    {
                        "crate_name": "zircon_plugin_animation",
                        "provider_package_id": "animation",
                        "registration_kind": "runtime_plugin",
                    }
                ],
                "CompileHost plan linked_runtime_crates[0].path must be a string",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field, value=value):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan[field] = value
                    validate_report = root / "validate.json"
                    validate_report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "plan_summary": {
                                    "library_embed_compile_host": compile_plan,
                                },
                            }
                        ),
                        encoding="utf-8",
                    )
                    args = _compile_host_args(
                        out=root / "out",
                        validate_report=validate_report,
                    )
                    args.dry_run = False

                    with mock.patch(
                        "tools.zircon_export.compile_host.subprocess.run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as cargo_call:
                        exit_code = _run_compile_host_quiet(args)

                    report = json_loads(
                        (
                            root / "out" / "stages" / "compile_host" / "report.json"
                        ).read_text(encoding="utf-8")
                    )
                    self.assertEqual(exit_code, 2)
                    cargo_call.assert_not_called()
                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["command"], [])
                    self.assertIsNone(report["host_executable"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_empty_host_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            def cargo_success(
                command: list[str],
                cwd: Path,
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                target_dir = Path(command[command.index("--target-dir") + 1])
                host_output = target_dir / "debug" / (
                    "zircon_runtime.exe" if target_dir.drive else "zircon_runtime"
                )
                host_output.parent.mkdir(parents=True, exist_ok=True)
                host_output.write_bytes(b"")
                return subprocess.CompletedProcess(command, 0)

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                side_effect=cargo_success,
            ):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["host_executable"]).stat().st_size, 0)
            self.assertTrue(
                any(
                    "CompileHost output" in diagnostic
                    and "is empty" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_directory_host_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False

            def cargo_success(
                command: list[str],
                cwd: Path,
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                target_dir = Path(command[command.index("--target-dir") + 1])
                host_output = target_dir / "debug" / (
                    "zircon_runtime.exe" if target_dir.drive else "zircon_runtime"
                )
                host_output.mkdir(parents=True)
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout="compile stdout\n",
                    stderr="compile stderr\n",
                )

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                side_effect=cargo_success,
            ):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(Path(report["host_executable"]).is_dir())
            self.assertEqual(report["stdout_lines"], ["compile stdout"])
            self.assertEqual(report["stderr_lines"], ["compile stderr"])
            self.assertTrue(
                any(
                    "CompileHost output" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
