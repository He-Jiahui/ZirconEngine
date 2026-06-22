from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateCompileHostSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_compile_host_command_metadata_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "package",
                {"command": self._compile_host_command_with("-p", "zircon_editor")},
                "validate report plan_summary.library_embed_compile_host.command "
                "-p/--package must match "
                "validate report plan_summary.library_embed_compile_host.package",
            ),
            (
                "binary",
                {"command": self._compile_host_command_with("--bin", "zircon_editor")},
                "validate report plan_summary.library_embed_compile_host.command "
                "--bin must match "
                "validate report plan_summary.library_embed_compile_host.binary",
            ),
            (
                "target_dir",
                {
                    "command": self._compile_host_command_with(
                        "--target-dir",
                        "target/other-compile-host",
                    )
                },
                "validate report plan_summary.library_embed_compile_host.command "
                "--target-dir must match "
                "validate report plan_summary.library_embed_compile_host.target_dir",
            ),
            (
                "release_missing",
                {"release": True, "cargo_profile": "release"},
                "validate report plan_summary.library_embed_compile_host.command "
                "must include --release for release profile",
            ),
            (
                "release_unexpected",
                {
                    "command": [*_compile_host_plan()["command"], "--release"],
                    "release": False,
                    "cargo_profile": "debug",
                },
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --release for debug profile",
            ),
            (
                "missing_manifest_path",
                {"command": self._compile_host_command_without("--manifest-path", value=True)},
                "validate report plan_summary.library_embed_compile_host.command "
                "must include --manifest-path",
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
                    ],
                },
                "validate report plan_summary.library_embed_compile_host.command "
                "--manifest-path must match "
                "validate report plan_summary.library_embed_compile_host.manifest_path",
            ),
        )
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan.update(overrides)
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_command_feature_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "missing_no_default_features",
                {
                    "command": self._compile_host_command_without(
                        "--no-default-features"
                    )
                },
                "validate report plan_summary.library_embed_compile_host.command "
                "must include --no-default-features",
            ),
            (
                "missing_features",
                {"command": self._compile_host_command_without("--features", value=True)},
                "validate report plan_summary.library_embed_compile_host.command "
                "must include --features",
            ),
            (
                "duplicate_features",
                {"command": [*_compile_host_plan()["command"], "--features", "target-client"]},
                "validate report plan_summary.library_embed_compile_host.command "
                "--features must appear only once",
            ),
            (
                "feature_value_mismatch",
                {
                    "command": self._compile_host_command_with(
                        "--features",
                        "target-server",
                    )
                },
                "validate report plan_summary.library_embed_compile_host.command "
                "--features must match "
                "validate report plan_summary.library_embed_compile_host.app_features",
            ),
            (
                "all_features_broadening",
                {"command": [*_compile_host_plan()["command"], "--all-features"]},
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --all-features because CompileHost plan "
                "app_features owns feature selection",
            ),
        )
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan.update(overrides)
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_command_non_cargo(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            command = list(_compile_host_plan()["command"])
            command[0] = "python"
            compile_host_plan = _compile_host_plan()
            compile_host_plan["command"] = command
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host.command "
                "must run cargo build",
            )

    def test_report_stage_rejects_validate_compile_host_padded_command_entry(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            command = list(_compile_host_plan()["command"])
            command[0] = " cargo "
            compile_host_plan = _compile_host_plan()
            compile_host_plan["command"] = command
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "command[0] must be a non-empty trimmed string",
            )

    def test_report_stage_rejects_validate_compile_host_command_target_broadening(
        self,
    ) -> None:
        cases = (
            (
                "--all-targets",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --all-targets because CompileHost plan "
                "binary owns the single host target",
            ),
            (
                "--bins",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --bins because CompileHost plan binary owns "
                "the single host target",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
                    ]
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_command_target_triple_override(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            compile_host_plan = _compile_host_plan()
            compile_host_plan["command"] = [
                *list(_compile_host_plan()["command"]),
                "--target",
                "x86_64-unknown-linux-gnu",
            ]
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --target because export target descriptor "
                "owns platform target selection",
            )

    def test_report_stage_rejects_validate_compile_host_command_package_broadening(
        self,
    ) -> None:
        cases = (
            (
                "--workspace",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --workspace because CompileHost plan package "
                "owns package selection",
            ),
            (
                "--exclude",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --exclude because CompileHost plan package owns "
                "package selection",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
                    ]
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_command_profile_override(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            compile_host_plan = _compile_host_plan()
            compile_host_plan["command"] = [
                *list(_compile_host_plan()["command"]),
                "--profile",
                "shipping",
            ]
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --profile because CompileHost plan "
                "cargo_profile/release owns profile selection",
            )

    def test_report_stage_rejects_validate_compile_host_command_wrapper_policy_override(
        self,
    ) -> None:
        cases = (
            (
                "--locked",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --locked because CompileHost CLI owns Cargo "
                "lock/offline policy",
            ),
            (
                "--offline",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --offline because CompileHost CLI owns Cargo "
                "lock/offline policy",
            ),
            (
                "--frozen",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --frozen because CompileHost CLI owns Cargo "
                "lock/offline policy",
            ),
        )
        for flag, expected_diagnostic in cases:
            with self.subTest(flag=flag):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        flag,
                    ]
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_forbidden_command_equals_form(
        self,
    ) -> None:
        cases = (
            (
                "--target=x86_64-unknown-linux-gnu",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --target because export target descriptor "
                "owns platform target selection",
            ),
            (
                "--profile=shipping",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --profile because CompileHost plan "
                "cargo_profile/release owns profile selection",
            ),
            (
                "--offline=true",
                "validate report plan_summary.library_embed_compile_host.command "
                "must not include --offline because CompileHost CLI owns Cargo "
                "lock/offline policy",
            ),
        )
        for option, expected_diagnostic in cases:
            with self.subTest(option=option):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan["command"] = [
                        *list(_compile_host_plan()["command"]),
                        option,
                    ]
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_blank_string_array_entry(
        self,
    ) -> None:
        cases = (
            ("app_features", ["target-client", ""]),
            ("command", ["cargo", "build", "   "]),
            ("runtime_features", ["target-client", "   "]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_reports_with_compile_host_plan_field(out, field, value)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field} must not contain blank entries",
                    )

    def test_report_stage_rejects_validate_compile_host_string_array_entry_non_string(
        self,
    ) -> None:
        cases = (
            ("app_features", ["target-client", 42]),
            ("command", ["cargo", 42]),
            ("runtime_features", ["target-client", None]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_reports_with_compile_host_plan_field(out, field, value)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field}[1] must be a string",
                    )
                    self.assertFalse(
                        any(
                            "validate report plan_summary.library_embed_compile_host."
                            f"{field} must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_compile_host_padded_feature_entry(
        self,
    ) -> None:
        cases = (
            ("app_features", [" target-client "]),
            ("runtime_features", [" target-client "]),
        )
        for field, value in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_reports_with_compile_host_plan_field(out, field, value)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field}[0] must be a non-empty trimmed string",
                    )

    def test_report_stage_rejects_validate_compile_host_duplicate_feature_entry(
        self,
    ) -> None:
        cases = (
            (
                "app_features",
                {
                    "app_features": ["target-client", "target-client"],
                    "command": self._compile_host_command_with(
                        "--features",
                        "target-client target-client",
                    ),
                },
            ),
            (
                "runtime_features",
                {"runtime_features": ["target-client", "target-client"]},
            ),
        )
        for field, overrides in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan.update(overrides)
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field}[1] duplicates entry 0",
                    )

    def test_report_stage_rejects_validate_compile_host_duplicate_expected_plugin(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            compile_host_plan = _compile_host_plan()
            compile_host_plan["expected_runtime_plugins"] = [
                "rendering",
                "rendering",
            ]
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "expected_runtime_plugins[1] duplicates entry 0",
            )

    def test_report_stage_rejects_validate_compile_host_profile_release_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "invalid_profile",
                {"cargo_profile": "shipping"},
                "validate report plan_summary.library_embed_compile_host."
                "cargo_profile must be debug or release",
            ),
            (
                "release_true_debug_profile",
                {
                    "release": True,
                    "cargo_profile": "debug",
                    "command": [*_compile_host_plan()["command"], "--release"],
                },
                "validate report plan_summary.library_embed_compile_host.release "
                "must match cargo_profile",
            ),
            (
                "release_false_release_profile",
                {
                    "release": False,
                    "cargo_profile": "release",
                    "command": [*_compile_host_plan()["command"], "--release"],
                },
                "validate report plan_summary.library_embed_compile_host.release "
                "must match cargo_profile",
            ),
        )
        for case_name, overrides, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan.update(overrides)
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_compile_host_padded_cargo_profile_before_profile_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            compile_host_plan = _compile_host_plan()
            compile_host_plan["cargo_profile"] = " release "
            compile_host_plan["release"] = True
            compile_host_plan["command"] = [
                *_compile_host_plan()["command"],
                "--release",
            ]
            self._write_reports_with_compile_host_plan(out, compile_host_plan)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Validate", report["fatal_stages"])
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.library_embed_compile_host."
                "cargo_profile must be a non-empty trimmed string",
            )
            self.assertFalse(
                any(
                    "validate report plan_summary.library_embed_compile_host."
                    "cargo_profile must be debug or release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_compile_host_missing_required_field(
        self,
    ) -> None:
        cases = (
            "package",
            "command",
            "release",
            "linked_runtime_crates",
        )
        for field in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    del compile_host_plan[field]
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field} is required",
                    )

    def test_report_stage_rejects_validate_compile_host_blank_string_field(
        self,
    ) -> None:
        cases = (
            ("package", ""),
            ("binary", " "),
            ("manifest_path", " Cargo.toml"),
            ("target_dir", "stages/compile_host/target "),
        )
        for field, value in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan[field] = value
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field} must be a non-empty trimmed string",
                    )

    def test_report_stage_rejects_validate_compile_host_unsafe_path_field(
        self,
    ) -> None:
        cases = (
            ("manifest_path", "../Cargo.toml"),
            ("manifest_path", "/repo/Cargo.toml"),
            ("target_dir", "stages/../target"),
            ("target_dir", "/tmp/zircon-target"),
        )
        for field, value in cases:
            with self.subTest(field=field, value=value):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan[field] = value
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.library_embed_compile_host."
                        f"{field} must be a safe relative path",
                    )

    def test_report_stage_rejects_validate_compile_host_invalid_target_selector(
        self,
    ) -> None:
        cases = (
            (
                "package",
                "zircon_tools",
                "-p",
                "validate report plan_summary.library_embed_compile_host."
                "package must be zircon_app",
            ),
            (
                "binary",
                "zircon_tools",
                "--bin",
                "validate report plan_summary.library_embed_compile_host."
                "binary must be zircon_runtime or zircon_editor",
            ),
        )
        for field, value, option, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan[field] = value
                    compile_host_plan["command"] = self._compile_host_command_with(
                        option,
                        value,
                    )
                    self._write_reports_with_compile_host_plan(out, compile_host_plan)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("Validate", report["fatal_stages"])
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def _write_reports_with_compile_host_plan_field(
        self,
        out: Path,
        field: str,
        value: object,
    ) -> None:
        _write_validate_report_with_strategies(out, ["library_embed"])
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        self._update_validate_compile_host_plan(out, field, value)
        self._update_compile_host_link_plan(out, field, value)

    def _write_reports_with_compile_host_plan(
        self,
        out: Path,
        compile_host_plan: dict[str, object],
    ) -> None:
        _write_validate_report_with_strategies(out, ["library_embed"])
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        validate_report_path = out / "stages" / "validate" / "report.json"
        validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
        plan_summary = validate_report.get("plan_summary")
        if not isinstance(plan_summary, dict):
            plan_summary = {}
        plan_summary["library_embed_compile_host"] = compile_host_plan
        validate_report["plan_summary"] = plan_summary
        validate_report_path.write_text(
            json.dumps(validate_report, indent=2),
            encoding="utf-8",
        )

    def _update_validate_compile_host_plan(
        self,
        out: Path,
        field: str,
        value: object,
    ) -> None:
        validate_report_path = out / "stages" / "validate" / "report.json"
        validate_report = json.loads(validate_report_path.read_text(encoding="utf-8"))
        compile_host_plan = _compile_host_plan()
        compile_host_plan[field] = value
        plan_summary = validate_report.get("plan_summary")
        if not isinstance(plan_summary, dict):
            plan_summary = {}
        plan_summary["library_embed_compile_host"] = compile_host_plan
        validate_report["plan_summary"] = plan_summary
        validate_report_path.write_text(
            json.dumps(validate_report, indent=2),
            encoding="utf-8",
        )

    def _update_compile_host_link_plan(
        self,
        out: Path,
        field: str,
        value: object,
    ) -> None:
        compile_host_report_path = out / "stages" / "compile_host" / "report.json"
        compile_host_report = json.loads(
            compile_host_report_path.read_text(encoding="utf-8")
        )
        link_plan = compile_host_report["link_plan"]
        if isinstance(link_plan, dict) and field in link_plan:
            link_plan[field] = value
        compile_host_report_path.write_text(
            json.dumps(compile_host_report, indent=2),
            encoding="utf-8",
        )

    def _compile_host_command_with(self, option: str, value: str) -> list[str]:
        command = list(_compile_host_plan()["command"])
        command[command.index(option) + 1] = value
        return command

    def _compile_host_command_without(
        self,
        option: str,
        *,
        value: bool = False,
    ) -> list[str]:
        command = list(_compile_host_plan()["command"])
        index = command.index(option)
        end = index + 2 if value else index + 1
        del command[index:end]
        return command

    def _assert_diagnostic_contains(
        self,
        report: dict[str, object],
        expected_diagnostic: str,
    ) -> None:
        self.assertTrue(
            any(
                expected_diagnostic in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )


if __name__ == "__main__":
    unittest.main()
