from __future__ import annotations

import contextlib
import io
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import compile_host_command, run_compile_host
from tools.zircon_export.source_template_plan_command import source_template_command
from tools.zircon_export.tests.export_test_support import (
    REPO_ROOT,
    _compile_host_args,
    _compile_host_plan,
    _run_compile_host_quiet,
    _run_source_template_quiet,
    _source_template_args,
    _source_template_plan,
    _source_template_validate_report,
    json_dumps,
    json_loads,
)


class CompileHostSourceTemplateTests(unittest.TestCase):
    def test_compile_host_command_uses_validated_plan_and_output_target_dir(self) -> None:
        plan = _compile_host_plan()
        args = _compile_host_args(out=Path("E:/export-out"))

        command = compile_host_command(args, Path("E:/export-out"), plan)

        self.assertIn("--locked", command)
        self.assertEqual(command[0], "cargo")
        self.assertEqual(
            command[command.index("--target-dir") + 1],
            str((Path("E:/export-out") / "stages" / "compile_host" / "target").resolve()),
        )

    def test_compile_host_dry_run_rejects_profile_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report = root / "validate.json"
            report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "other-profile",
                        "fatal": False,
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_compile_host_quiet(
                _compile_host_args(out=root / "out", validate_report=report)
            )

            self.assertEqual(exit_code, 2)

    def test_compile_host_dry_run_rejects_invalid_validate_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report = root / "validate.json"
            report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": [],
                        "diagnostics": [],
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )

            args = _compile_host_args(out=root / "out", validate_report=report)
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_compile_host(args)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "Validate report fatal must be a boolean",
                stdout.getvalue(),
            )

    def test_compile_host_dry_run_rejects_validate_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report = root / "validate.json"
            report.mkdir()

            args = _compile_host_args(out=root / "out", validate_report=report)
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_compile_host(args)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                f"validate report {report} is not a file",
                stdout.getvalue(),
            )

    def test_compile_host_dry_run_requires_host_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            report = root / "validate.json"
            report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "profile_summary": {
                            "strategies": ["source_template"],
                        },
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )

            args = _compile_host_args(out=root / "out", validate_report=report)
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_compile_host(args)

            self.assertEqual(exit_code, 2)
            self.assertIn(
                "CompileHost stage requires library_embed or native_dynamic strategy",
                stdout.getvalue(),
            )

    def test_compile_host_reports_target_dir_resolve_error(self) -> None:
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
                        "profile_summary": {
                            "strategies": ["library_embed"],
                        },
                        "plan_summary": {
                            "library_embed_compile_host": _compile_host_plan(),
                        },
                    }
                ),
                encoding="utf-8",
            )
            target_dir = root / "target-dir"
            args = _compile_host_args(
                out=root / "out",
                validate_report=validate_report,
            )
            args.dry_run = False
            args.target_dir = str(target_dir)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(target_dir):
                    raise OSError("simulated CompileHost target dir failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (
                    root / "out" / "stages" / "compile_host" / "report.json"
                ).read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["command"], [])
            self.assertIsNone(report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost target_dir" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated CompileHost target dir failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_compile_host_dry_run_rejects_invalid_strategy_metadata(self) -> None:
        cases = (
            ("library_embed", "profile_summary.strategies must be a list"),
            (
                [],
                "profile_summary.strategies must include at least one supported export strategy",
            ),
            (["library_embed", "ghost_path"], "unsupported export strategy ghost_path"),
        )
        for strategies, expected_diagnostic in cases:
            with self.subTest(strategies=strategies):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    report = root / "validate.json"
                    report.write_text(
                        json_dumps(
                            {
                                "stage": "Validate",
                                "profile": "windows-release",
                                "fatal": False,
                                "diagnostics": [],
                                "profile_summary": {
                                    "strategies": strategies,
                                },
                                "plan_summary": {
                                    "library_embed_compile_host": _compile_host_plan(),
                                },
                            }
                        ),
                        encoding="utf-8",
                    )

                    args = _compile_host_args(out=root / "out", validate_report=report)
                    stdout = io.StringIO()
                    with contextlib.redirect_stdout(stdout):
                        exit_code = run_compile_host(args)

                    output = stdout.getvalue()
                    self.assertEqual(exit_code, 2)
                    self.assertIn(expected_diagnostic, output)
                    self.assertNotIn("--target-dir", output)

    def test_compile_host_report_respects_target_dir_override(self) -> None:
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
            target_dir = root / "custom-target"
            args = _compile_host_args(out=root / "out", validate_report=validate_report)
            args.target_dir = str(target_dir)
            args.dry_run = False
            expected_host = target_dir / "debug" / (
                "zircon_runtime.exe" if os.name == "nt" else "zircon_runtime"
            )

            def compile_success(
                command: list[str],
                cwd: Path,
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                expected_host.parent.mkdir(parents=True)
                expected_host.write_text("host", encoding="utf-8")
                return subprocess.CompletedProcess(
                    command,
                    0,
                    stdout="compile host stdout\nsecond line\n",
                    stderr="compile host stderr\n",
                )

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                side_effect=compile_success,
            ):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "compile_host" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["host_executable"]), expected_host)
            self.assertEqual(
                report["stdout_lines"],
                ["compile host stdout", "second line"],
            )
            self.assertEqual(report["stderr_lines"], ["compile host stderr"])

    def test_compile_host_report_preserves_library_embed_link_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            compile_plan = _compile_host_plan()
            compile_plan["app_features"] = ["target-client", "plugin-rendering"]
            command = list(compile_plan["command"])
            command[command.index("--features") + 1] = "target-client,plugin-rendering"
            compile_plan["command"] = command
            compile_plan["runtime_features"] = ["target-client", "rendering"]
            compile_plan["expected_runtime_plugins"] = ["rendering"]
            compile_plan["linked_runtime_crates"] = [
                {
                    "crate_name": "zircon_plugin_rendering_runtime",
                    "path": "zircon_plugins/rendering/runtime",
                    "registration_kind": "runtime_plugin",
                    "provider_package_id": "rendering",
                }
            ]
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(
                    {
                        "stage": "Validate",
                        "profile": "windows-release",
                        "fatal": False,
                        "diagnostics": [],
                        "profile_summary": {
                            "strategies": ["library_embed"],
                        },
                        "plan_summary": {
                            "library_embed_compile_host": compile_plan,
                        },
                    }
                ),
                encoding="utf-8",
            )
            args = _compile_host_args(out=root / "out", validate_report=validate_report)
            args.dry_run = False
            target_dir = root / "out" / "stages" / "compile_host" / "target"
            expected_host = target_dir / "debug" / (
                "zircon_runtime.exe" if os.name == "nt" else "zircon_runtime"
            )

            def compile_success(
                command: list[str],
                cwd: Path,
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                expected_host.parent.mkdir(parents=True)
                expected_host.write_text("host", encoding="utf-8")
                return subprocess.CompletedProcess(command, 0, stdout="", stderr="")

            with mock.patch(
                "tools.zircon_export.compile_host.subprocess.run",
                side_effect=compile_success,
            ):
                exit_code = _run_compile_host_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "compile_host" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertEqual(
                report["link_plan"],
                {
                    "app_features": compile_plan["app_features"],
                    "runtime_features": compile_plan["runtime_features"],
                    "expected_runtime_plugins": compile_plan[
                        "expected_runtime_plugins"
                    ],
                    "linked_runtime_crates": compile_plan["linked_runtime_crates"],
                },
            )

    def test_source_template_command_rewrites_manifest_and_target_dir(self) -> None:
        source_plan = _source_template_plan()
        args = _source_template_args(out=Path("E:/export-out"))
        project_dir = Path("E:/export-out") / "stages" / "source_template" / "project"

        command = source_template_command(args, project_dir, source_plan)

        self.assertIn("--locked", command)
        self.assertEqual(command[0], "cargo")
        self.assertEqual(
            command[command.index("--manifest-path") + 1],
            str((project_dir / "Cargo.toml").resolve()),
        )
        self.assertEqual(
            command[command.index("--target-dir") + 1],
            str((Path("E:/export-out") / "stages" / "source_template" / "target").resolve()),
        )

    def test_source_template_stage_materializes_generated_project_without_build(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )

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
            self.assertEqual(exit_code, 0)
            self.assertTrue((project / "Cargo.toml").exists())
            self.assertTrue((project / "src" / "main.rs").exists())
            self.assertIn(
                (REPO_ROOT / "zircon_app").as_posix(),
                (project / "Cargo.toml").read_text(encoding="utf-8"),
            )
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertFalse(report["build_executed"])
            self.assertEqual(
                report["build_validation"],
                {
                    "requested": False,
                    "executed": False,
                    "status": "skipped",
                    "exit_code": None,
                    "working_dir": str(project),
                    "command": report["command"],
                    "stdout_lines": [],
                    "stderr_lines": [],
                },
            )
            generated_by_path = {
                file["path"]: file for file in report["generated_files"]
            }
            self.assertIn("sha256", generated_by_path["Cargo.toml"])
            self.assertIn("size", generated_by_path["Cargo.toml"])
            self.assertIn("sha256", generated_by_path["src/main.rs"])
            self.assertIn("size", generated_by_path["src/main.rs"])
            self.assertTrue(
                any("build validation skipped" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_generated_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_payload = _source_template_validate_report()
            generated_files = validate_payload["plan_summary"]["generated_files"]
            for file in generated_files:
                if file["path"] == "Cargo.toml":
                    file["path"] = "Cargo.toml/nested.txt"
                    file["contents"] = "nested"
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(validate_payload),
                encoding="utf-8",
            )

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
            self.assertTrue(
                any(
                    "SourceTemplate manifest" in diagnostic
                    and "Cargo.toml" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_generated_file_path_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            generated_main = (
                root
                / "out"
                / "stages"
                / "source_template"
                / "project"
                / "src"
                / "main.rs"
            )
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == generated_main:
                    raise OSError("simulated SourceTemplate generated path failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
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
            self.assertTrue(
                any(
                    "generated file path src/main.rs could not be resolved" in diagnostic
                    and "simulated SourceTemplate generated path failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_generated_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            unreadable_file = (
                root
                / "out"
                / "stages"
                / "source_template"
                / "project"
                / "src"
                / "main.rs"
            ).resolve()
            original_read_bytes = Path.read_bytes

            def read_bytes_or_fail(path: Path) -> bytes:
                if path.resolve() == unreadable_file:
                    raise OSError("simulated generated file read failure")
                return original_read_bytes(path)

            with mock.patch.object(Path, "read_bytes", read_bytes_or_fail):
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
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs could not be read"
                    in diagnostic
                    and "simulated generated file read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_rejects_generated_file_write_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            unwritable_file = (
                root
                / "out"
                / "stages"
                / "source_template"
                / "project"
                / "src"
                / "main.rs"
            ).resolve()
            original_write_text = Path.write_text

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == unwritable_file:
                    raise OSError("simulated generated file write failure")
                return original_write_text(path, *args, **kwargs)

            with mock.patch.object(Path, "write_text", write_text_or_fail):
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
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs could not be written"
                    in diagnostic
                    and "simulated generated file write failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_cleans_stale_generated_project_files(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            project = root / "out" / "stages" / "source_template" / "project"
            (project / "src").mkdir(parents=True)
            (project / "src" / "stale.rs").write_text("stale", encoding="utf-8")

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
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertTrue((project / "src" / "main.rs").exists())
            self.assertFalse((project / "src" / "stale.rs").exists())

    def test_source_template_stage_rejects_stale_project_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            project = root / "out" / "stages" / "source_template" / "project"
            project.mkdir(parents=True)
            (project / "stale.rs").write_text("stale", encoding="utf-8")
            original_rmtree = shutil.rmtree

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == project.resolve():
                    raise OSError("simulated stale project cleanup failure")
                original_rmtree(path)

            with mock.patch(
                "tools.zircon_export.source_template_generated_project.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_source_template_quiet(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        dry_run=False,
                    )
                )

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
            self.assertFalse(report["project_cleaned"])
            self.assertEqual(report["cleanup_reason"], "stale_project_cleanup_failed")
            self.assertTrue((project / "stale.rs").exists())
            self.assertTrue(
                any(
                    "SourceTemplate generated project" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated stale project cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_reports_final_project_cleanup_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            project = root / "out" / "stages" / "source_template" / "project"
            generated_main = (project / "src" / "main.rs").resolve()
            original_write_text = Path.write_text
            original_rmtree = shutil.rmtree

            def write_text_or_fail(path: Path, *args: object, **kwargs: object) -> int:
                if path.resolve() == generated_main:
                    raise OSError("simulated generated file write failure")
                return original_write_text(path, *args, **kwargs)

            def rmtree_or_fail(path: Path) -> None:
                if Path(path).resolve() == project.resolve():
                    raise OSError("simulated final project cleanup failure")
                original_rmtree(path)

            with mock.patch.object(Path, "write_text", write_text_or_fail), mock.patch(
                "tools.zircon_export.source_template_generated_project.shutil.rmtree",
                side_effect=rmtree_or_fail,
            ):
                exit_code = _run_source_template_quiet(
                    _source_template_args(
                        out=root / "out",
                        validate_report=validate_report,
                        dry_run=False,
                    )
                )

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
            self.assertFalse(report["project_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_cleanup_failed")
            self.assertTrue(project.exists())
            self.assertTrue(
                any(
                    "SourceTemplate generated file src/main.rs could not be written"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "SourceTemplate generated project" in diagnostic
                    and "could not be removed" in diagnostic
                    and "simulated final project cleanup failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_source_template_stage_reports_successful_build_validation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )
            calls: list[tuple[list[str], Path]] = []

            def build_success(command: list[str], cwd: Path, **kwargs: object) -> subprocess.CompletedProcess[str]:
                calls.append((command, cwd))
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
            project = stage_dir / "project"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(len(calls), 1)
            self.assertEqual(calls[0][1], project)
            self.assertTrue(report["build_executed"])
            self.assertEqual(
                report["build_validation"],
                {
                    "requested": True,
                    "executed": True,
                    "status": "passed",
                    "exit_code": 0,
                    "working_dir": str(project),
                    "command": report["command"],
                    "stdout_lines": ["cargo stdout line"],
                    "stderr_lines": ["cargo stderr line"],
                },
            )

    def test_source_template_stage_reports_failed_build_validation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            validate_report = root / "validate.json"
            validate_report.write_text(
                json_dumps(_source_template_validate_report()),
                encoding="utf-8",
            )

            with mock.patch(
                "tools.zircon_export.source_template.subprocess.run",
                return_value=subprocess.CompletedProcess(
                    ["cargo", "build"],
                    42,
                    stdout="build stdout\nnext line\n",
                    stderr="build stderr\n",
                ),
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
            project = stage_dir / "project"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse(project.exists())
            self.assertTrue(report["build_executed"])
            self.assertEqual(
                report["build_validation"],
                {
                    "requested": True,
                    "executed": True,
                    "status": "failed",
                    "exit_code": 42,
                    "working_dir": str(project),
                    "command": report["command"],
                    "stdout_lines": ["build stdout", "next line"],
                    "stderr_lines": ["build stderr"],
                },
            )
            self.assertTrue(
                any("SourceTemplate cargo build exited with code 42" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

if __name__ == "__main__":
    unittest.main()
