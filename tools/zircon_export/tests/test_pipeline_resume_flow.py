from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.cli import (
    RESUMABLE_STAGES,
    STAGES,
    main,
    parse_args,
    run_pipeline,
)
from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_pipeline_quiet,
    _run_stage_quiet,
    _write_compile_host_report,
    _write_cook_assets_report,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
    _write_validate_report_with_strategies_value,
    json_dumps,
    json_loads,
)


class PipelineResumeFlowTests(unittest.TestCase):
    def test_cli_stage_choices_match_shared_pipeline_order(self) -> None:
        expected = (
            "validate",
            "source_template",
            "native_dynamic",
            "compile_host",
            "cook_assets",
            "pack",
            "platform_bundle",
            "report",
        )

        self.assertEqual(STAGES, expected)
        self.assertEqual(RESUMABLE_STAGES, expected)

    def test_resume_from_rejects_explicit_stage(self) -> None:
        with self.assertRaises(SystemExit) as raised:
            with contextlib.redirect_stderr(io.StringIO()):
                parse_args(
                    [
                        "--profile",
                        "windows-release",
                        "--out",
                        "zircon-export",
                        "--stage",
                        "report",
                        "--resume-from",
                        "pack",
                    ]
                )

        self.assertEqual(raised.exception.code, 2)

    def test_cli_help_describes_report_handoff_defaults(self) -> None:
        stdout = io.StringIO()
        with self.assertRaises(SystemExit) as raised:
            with contextlib.redirect_stdout(stdout):
                main(["--help"])

        output = stdout.getvalue()
        self.assertEqual(raised.exception.code, 0)
        self.assertIn("Pack defaults from a matching", output)
        self.assertIn("CookAssets report", output)
        self.assertIn("Defaults from a matching Validate report", output)
        self.assertIn("PlatformBundle defaults from a", output)
        self.assertIn("matching Pack report", output)
        self.assertIn("Defaults from a matching NativeDynamic", output)
        self.assertIn("report when reported", output)
        self.assertNotIn("Main pipeline defaults", output)

    def test_omitting_stage_runs_main_pipeline_from_validate(self) -> None:
        with mock.patch("tools.zircon_export.cli.run_pipeline", return_value=17) as pipeline:
            exit_code = main(
                [
                    "--profile",
                    "windows-release",
                    "--out",
                    "zircon-export",
                ]
            )

        self.assertEqual(exit_code, 17)
        pipeline.assert_called_once()
        args, resume_from = pipeline.call_args.args
        self.assertEqual(resume_from, "validate")
        self.assertFalse(args.stage_explicit)

    def test_pipeline_from_validate_uses_source_template_profile_stages(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies(out, ["source_template"])
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["validate", "source_template", "report"])

    def test_pipeline_from_validate_rejects_unknown_strategy_without_defaulting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies(out, ["future_export_path"])
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["validate", "report"])

    def test_pipeline_from_validate_rejects_empty_strategies_without_defaulting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies(out, [])
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["validate", "report"])

    def test_pipeline_from_validate_rejects_invalid_strategies_without_defaulting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies_value(out, "library_embed")
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["validate", "report"])

    def test_pipeline_from_validate_rejects_invalid_validate_metadata_without_defaulting(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    validate_path = out / "stages" / "validate" / "report.json"
                    validate_report = json_loads(
                        validate_path.read_text(encoding="utf-8")
                    )
                    validate_report["diagnostics"] = "not-a-list"
                    validate_path.write_text(
                        json_dumps(validate_report),
                        encoding="utf-8",
                    )
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["validate", "report"])

    def test_pipeline_from_validate_uses_native_dynamic_profile_stages(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                if args.stage == "validate":
                    _write_validate_report_with_strategies(out, ["native_dynamic"])
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="validate", dry_run=False),
                    "validate",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(
                visited,
                [
                    "validate",
                    "native_dynamic",
                    "compile_host",
                    "cook_assets",
                    "pack",
                    "platform_bundle",
                    "report",
                ],
            )

    def test_resume_from_pack_dry_run_runs_remaining_main_pipeline(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = main(
                    [
                        "--profile",
                        "windows-release",
                        "--out",
                        str(out),
                        "--resume-from",
                        "pack",
                        "--dry-run",
                    ]
                )

            output = stdout.getvalue()
            self.assertEqual(exit_code, 0)
            self.assertIn("resume_from=pack", output)
            self.assertIn("pipeline_stages=pack,platform_bundle,report", output)
            self.assertIn("zircon_export stage=Pack", output)
            self.assertIn("zircon_export stage=PlatformBundle", output)
            self.assertIn("zircon_export stage=Report", output)
            self.assertNotIn("zircon_export stage=Validate", output)
            self.assertNotIn("zircon_export stage=CookAssets", output)

    def test_resume_from_ignores_stage_outside_validated_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="source_template", dry_run=False),
                    "source_template",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["report"])

    def test_resume_from_invalid_validate_metadata_does_not_use_fallback_stages(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            validate_path = out / "stages" / "validate" / "report.json"
            validate_report = json_loads(validate_path.read_text(encoding="utf-8"))
            validate_report["fatal"] = []
            validate_path.write_text(json_dumps(validate_report), encoding="utf-8")
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="pack", dry_run=False),
                    "pack",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["report"])

    def test_resume_from_validate_report_directory_does_not_use_fallback_stages(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            validate_path = out / "stages" / "validate" / "report.json"
            validate_path.mkdir(parents=True)
            visited: list[str] = []

            def run_stage_side_effect(args: argparse.Namespace) -> int:
                visited.append(args.stage)
                return 0

            with mock.patch(
                "tools.zircon_export.cli.run_stage",
                side_effect=run_stage_side_effect,
            ):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="pack", dry_run=False),
                    "pack",
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(visited, ["report"])

    def test_resume_from_platform_bundle_stops_before_report_on_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            args = _export_args(out=out, stage="platform_bundle", dry_run=False)

            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            self.assertEqual(exit_code, 2)
            self.assertTrue((out / "stages" / "platform_bundle" / "report.json").exists())
            self.assertFalse((out / "stages" / "report" / "report.json").exists())
            self.assertFalse((out / "report.json").exists())

    def test_pipeline_pack_uses_cook_assets_report_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            cooked_manifest = root / "cook-output" / "shipping-assets.json"
            cooked_manifest.parent.mkdir(parents=True)
            cooked_manifest.write_text("{}", encoding="utf-8")
            _write_cook_assets_report(out, cooked_manifest)

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pipeline(
                    _export_args(out=out, stage="pack", dry_run=True),
                    "pack",
                )

            self.assertEqual(exit_code, 0)
            self.assertIn(f"asset_manifest={cooked_manifest}", stdout.getvalue())

    def test_stage_pack_uses_cook_assets_report_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            cooked_manifest = root / "cook-output" / "shipping-assets.json"
            cooked_manifest.parent.mkdir(parents=True)
            cooked_manifest.write_text("{}", encoding="utf-8")
            default_manifest = out / "stages" / "cook_assets" / "assets.json"
            default_manifest.parent.mkdir(parents=True)
            default_manifest.write_text("{}", encoding="utf-8")
            _write_cook_assets_report(out, cooked_manifest)
            args = _export_args(out=out, stage="pack", dry_run=False)

            captured_command: list[str] = []

            def packer_success(command: list[str], cwd: Path) -> int:
                captured_command.extend(command)
                pack_path = Path(command[command.index("--pack") + 1])
                _write_pack_report(out, pack_path)
                return 0

            with mock.patch(
                "tools.zircon_export.cli.subprocess.call",
                side_effect=packer_success,
            ):
                exit_code = _run_stage_quiet(args)

            self.assertEqual(exit_code, 0)
            self.assertIn(str(cooked_manifest), captured_command)
            self.assertNotIn(str(default_manifest), captured_command)

    def test_stage_pack_rejects_empty_explicit_asset_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            cooked_manifest = root / "cook-output" / "shipping-assets.json"
            cooked_manifest.parent.mkdir(parents=True)
            cooked_manifest.write_text("{}", encoding="utf-8")
            _write_cook_assets_report(out, cooked_manifest)
            args = _export_args(out=out, stage="pack", dry_run=False)
            args.asset_manifest = ""

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "pack" / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertEqual(args.asset_manifest, "")
            self.assertIsNone(report["asset_manifest"])
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "asset_manifest argument must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pipeline_pack_preserves_empty_explicit_asset_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            cooked_manifest = root / "cook-output" / "shipping-assets.json"
            cooked_manifest.parent.mkdir(parents=True)
            cooked_manifest.write_text("{}", encoding="utf-8")
            _write_cook_assets_report(out, cooked_manifest)
            args = _export_args(out=out, stage="pack", dry_run=False)
            args.asset_manifest = ""

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
                exit_code = _run_pipeline_quiet(args, "pack")

            report = json_loads(
                (out / "stages" / "pack" / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertEqual(args.asset_manifest, "")
            self.assertIsNone(report["asset_manifest"])
            self.assertFalse((out / "stages" / "platform_bundle" / "report.json").exists())
            self.assertFalse((out / "report.json").exists())
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "asset_manifest argument must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pack_rejects_empty_explicit_pack_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            cooked_manifest = root / "cook-output" / "shipping-assets.json"
            cooked_manifest.parent.mkdir(parents=True)
            cooked_manifest.write_text("{}", encoding="utf-8")
            _write_cook_assets_report(out, cooked_manifest)
            args = _export_args(out=out, stage="pack", dry_run=False)
            args.pack_file = ""

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
                exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "pack" / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertEqual(args.pack_file, "")
            self.assertEqual(Path(report["asset_manifest"]), cooked_manifest)
            self.assertIsNone(report["pack"])
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "pack_file argument must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pipeline_pack_rejects_invalid_cook_assets_report_manifest_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            default_manifest = out / "stages" / "cook_assets" / "assets.json"
            default_manifest.parent.mkdir(parents=True)
            default_manifest.write_text("{}", encoding="utf-8")
            args = _export_args(out=out, stage="pack", dry_run=False)
            _write_cook_assets_report(
                out,
                root / "cook-output" / "shipping-assets.json",
                manifest_value=[],
            )

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
                exit_code = _run_pipeline_quiet(args, "pack")

            report = json_loads(
                (out / "stages" / "pack" / "report.json").read_text(encoding="utf-8")
            )
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "CookAssets report field cooked_asset_manifest must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pipeline_platform_bundle_uses_compile_host_report_host(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = out / "stages" / "pack" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundled_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            pipeline_report = json_loads((out / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0, pipeline_report["diagnostics"])
            self.assertTrue(bundled_host.exists())
            self.assertEqual(Path(platform_report["host_executable"]), bundled_host)
            self.assertEqual(Path(platform_report["host_source"]), host)
            self.assertEqual(platform_report["host_source_origin"], "compile_host_report")
            bundle_manifest = json_loads(
                (out / "bundle" / "windows-release" / "bundle.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(Path(bundle_manifest["host_source"]), host)
            self.assertEqual(bundle_manifest["host_source_origin"], "compile_host_report")
            self.assertFalse(pipeline_report["fatal"], pipeline_report["diagnostics"])

    def test_pipeline_platform_bundle_rejects_invalid_compile_host_report_host_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, root / "compile" / "zircon_runtime.exe", host_value=[])
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertIsNone(platform_report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost report field host_executable must be a non-empty string"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_rejects_compile_host_report_host_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(host):
                    raise OSError("simulated compile host handoff resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_pipeline_quiet(
                    _export_args(out=out, stage="platform_bundle", dry_run=False),
                    "platform_bundle",
                )

            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertIsNone(platform_report["host_executable"])
            self.assertTrue(
                any(
                    "CompileHost report field host_executable" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated compile host handoff resolve failure"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_uses_pack_report_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundled_pack = out / "bundle" / "windows-release" / "custom-assets.zrpack"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            pipeline_report = json_loads((out / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0, pipeline_report["diagnostics"])
            self.assertTrue(bundled_pack.exists())
            self.assertEqual(Path(platform_report["pack"]), bundled_pack)
            self.assertFalse(pipeline_report["fatal"], pipeline_report["diagnostics"])

    def test_stage_platform_bundle_uses_report_handoff_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)

            exit_code = _run_stage_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False)
            )

            bundled_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            bundled_pack = out / "bundle" / "windows-release" / "custom-assets.zrpack"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, platform_report["diagnostics"])
            self.assertEqual(Path(platform_report["host_executable"]), bundled_host)
            self.assertEqual(Path(platform_report["host_source"]), host)
            self.assertEqual(platform_report["host_source_origin"], "compile_host_report")
            self.assertEqual(Path(platform_report["pack"]), bundled_pack)
            self.assertEqual(bundled_pack.read_text(encoding="utf-8"), "custom pack placeholder")
            self.assertFalse((out / "bundle" / "windows-release" / "assets.zrpack").exists())

    def test_stage_platform_bundle_uses_report_delta_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            delta_pack = root / "pack-output" / "custom-assets.delta.zrpd"
            delta_pack.write_text("custom delta placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack=delta_pack)

            exit_code = _run_stage_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False)
            )

            bundled_delta = out / "bundle" / "windows-release" / "custom-assets.delta.zrpd"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, platform_report["diagnostics"])
            self.assertEqual(Path(platform_report["delta_pack"]), bundled_delta)
            self.assertEqual(
                bundled_delta.read_text(encoding="utf-8"),
                "custom delta placeholder",
            )

    def test_stage_platform_bundle_uses_native_dynamic_report_plugins(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)

            exit_code = _run_stage_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False)
            )

            bundled_plugins = out / "bundle" / "windows-release" / "plugins"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, platform_report["diagnostics"])
            self.assertTrue((bundled_plugins / "native_plugins.toml").exists())
            self.assertTrue(
                (bundled_plugins / "animation" / "native" / "zircon_plugin_animation.dll").exists()
            )
            self.assertEqual(Path(platform_report["native_plugins"]), bundled_plugins)

    def test_pipeline_platform_bundle_ignores_pack_report_without_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            stale_pack = root / "pack-output" / "stale-assets.zrpack"
            stale_pack.parent.mkdir(parents=True)
            stale_pack.write_text("stale pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, stale_pack, profile=None)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse((bundle / "stale-assets.zrpack").exists())
            self.assertIsNone(platform_report["pack"])
            self.assertTrue(
                any(
                    "Pack report profile is missing or invalid" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_rejects_invalid_pack_report_pack_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            default_pack = out / "stages" / "pack" / "assets.zrpack"
            default_pack.parent.mkdir(parents=True)
            default_pack.write_text("stale default pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, root / "pack-output" / "assets.zrpack", pack_value=[])

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse((bundle / "assets.zrpack").exists())
            self.assertIsNone(platform_report["pack"])
            self.assertTrue(
                any("Pack report field pack must be a non-empty string" in diagnostic for diagnostic in platform_report["diagnostics"]),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_rejects_invalid_pack_report_delta_pack_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, delta_pack_value=[])

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertIsNone(platform_report["pack"])
            self.assertIsNone(platform_report["delta_pack"])
            self.assertTrue(
                any(
                    "Pack report field delta_pack must be a non-empty string"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
