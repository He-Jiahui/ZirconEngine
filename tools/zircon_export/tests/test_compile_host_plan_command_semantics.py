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


class CompileHostPlanCommandSemanticsTests(unittest.TestCase):
    def test_compile_host_rejects_plan_with_empty_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = []
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
                    "CompileHost plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_blank_command_entry(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", ""]
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
                    "CompileHost plan command must be a non-empty string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_non_cargo_command(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            command = list(compile_plan["command"])
            command[0] = "python"
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
                return_value=subprocess.CompletedProcess(command, 0),
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
                    "CompileHost plan command must run cargo build" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_command_metadata_mismatch(self) -> None:
        def command_with(option: str, value: str) -> list[str]:
            command = list(_compile_host_plan()["command"])
            command[command.index(option) + 1] = value
            return command

        def command_without(option: str, *, value: bool = False) -> list[str]:
            command = list(_compile_host_plan()["command"])
            index = command.index(option)
            end = index + 2 if value else index + 1
            del command[index:end]
            return command

        cases: tuple[tuple[str, dict[str, object], str], ...] = (
            (
                "package",
                {"command": command_with("-p", "zircon_editor")},
                "CompileHost plan command -p/--package must match CompileHost plan package",
            ),
            (
                "binary",
                {"command": command_with("--bin", "zircon_editor")},
                "CompileHost plan command --bin must match CompileHost plan binary",
            ),
            (
                "missing_no_default_features",
                {"command": command_without("--no-default-features")},
                "CompileHost plan command must include --no-default-features",
            ),
            (
                "features",
                {"command": command_with("--features", "target-server")},
                "CompileHost plan command --features must match CompileHost plan app_features",
            ),
            (
                "target_dir",
                {"command": command_with("--target-dir", "target/other-compile-host")},
                "CompileHost plan command --target-dir must match CompileHost plan target_dir",
            ),
            (
                "missing_manifest_path",
                {"command": command_without("--manifest-path", value=True)},
                "CompileHost plan command must include --manifest-path",
            ),
            (
                "manifest_path",
                {
                    "command": [
                        "cargo",
                        "build",
                        "--manifest-path",
                        "crates/other/Cargo.toml",
                        "-p",
                        "zircon_app",
                        "--bin",
                        "zircon_runtime",
                        "--no-default-features",
                        "--features",
                        "target-client",
                        "--target-dir",
                        "stages/compile_host/target",
                    ]
                },
                "CompileHost plan command --manifest-path must match CompileHost plan manifest_path",
            ),
            (
                "release_missing",
                {"cargo_profile": "release", "release": True},
                "CompileHost plan command must include --release for release profile",
            ),
            (
                "debug_extra_release",
                {"command": [*list(_compile_host_plan()["command"]), "--release"]},
                "CompileHost plan command must not include --release for debug profile",
            ),
            (
                "all_features_broadening",
                {"command": [*list(_compile_host_plan()["command"]), "--all-features"]},
                "CompileHost plan command must not include --all-features because "
                "CompileHost plan app_features owns feature selection",
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

    def test_compile_host_rejects_plan_command_target_broadening(self) -> None:
        cases: tuple[tuple[str, str], ...] = (
            (
                "--all-targets",
                "CompileHost plan command must not include --all-targets "
                "because CompileHost plan binary owns the single host target",
            ),
            (
                "--bins",
                "CompileHost plan command must not include --bins because "
                "CompileHost plan binary owns the single host target",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
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
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_command_target_triple_override(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = [
                *list(_compile_host_plan()["command"]),
                "--target",
                "x86_64-unknown-linux-gnu",
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
                    "CompileHost plan command must not include --target because "
                    "export target descriptor owns platform target selection"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_command_package_broadening(self) -> None:
        cases: tuple[tuple[str, str], ...] = (
            (
                "--workspace",
                "CompileHost plan command must not include --workspace because "
                "CompileHost plan package owns package selection",
            ),
            (
                "--exclude",
                "CompileHost plan command must not include --exclude because "
                "CompileHost plan package owns package selection",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
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
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_command_profile_override(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = [
                *list(_compile_host_plan()["command"]),
                "--profile",
                "shipping",
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
                    "CompileHost plan command must not include --profile because "
                    "CompileHost plan cargo_profile/release owns profile selection"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_command_wrapper_policy_override(self) -> None:
        cases: tuple[tuple[str, str], ...] = (
            (
                "--locked",
                "CompileHost plan command must not include --locked because "
                "CompileHost CLI owns Cargo lock/offline policy",
            ),
            (
                "--offline",
                "CompileHost plan command must not include --offline because "
                "CompileHost CLI owns Cargo lock/offline policy",
            ),
            (
                "--frozen",
                "CompileHost plan command must not include --frozen because "
                "CompileHost CLI owns Cargo lock/offline policy",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
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
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_forbidden_command_equals_form(self) -> None:
        cases: tuple[tuple[str, str], ...] = (
            (
                "--target=x86_64-unknown-linux-gnu",
                "CompileHost plan command must not include --target because "
                "export target descriptor owns platform target selection",
            ),
            (
                "--profile=shipping",
                "CompileHost plan command must not include --profile because "
                "CompileHost plan cargo_profile/release owns profile selection",
            ),
            (
                "--offline=true",
                "CompileHost plan command must not include --offline because "
                "CompileHost CLI owns Cargo lock/offline policy",
            ),
        )
        for option, expected_diagnostic in cases:
            with self.subTest(option=option):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    compile_plan = _compile_host_plan()
                    compile_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        option,
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
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_compile_host_rejects_plan_with_dangling_target_dir_option(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", "build", "--target-dir"]
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
                    "CompileHost plan command --target-dir must include a value"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_target_dir_option_with_option_value(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = ["cargo", "build", "--target-dir", "--release"]
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
                    "CompileHost plan command --target-dir value must not be another option"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_rejects_plan_with_duplicate_target_dir_option(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["command"] = [
                "cargo",
                "build",
                "--target-dir",
                str(root / "target-a"),
                "--target-dir",
                str(root / "target-b"),
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
                    "CompileHost plan command --target-dir must appear only once"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )



if __name__ == "__main__":
    unittest.main()
