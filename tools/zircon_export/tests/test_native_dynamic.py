from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import _run_stage_quiet
from tools.zircon_export.tests.native_dynamic_test_support import (
    _export_args,
    _write_macos_native_dynamic_package_fixture,
    _write_validate_report_with_native_dynamic_exports,
    _write_windows_native_dynamic_package_fixture_at,
)
from tools.zircon_export.native_dynamic_payload import native_dynamic_stage_payload_summary


class NativeDynamicArtifactTests(unittest.TestCase):
    def test_native_dynamic_stage_rejects_wrong_v3_descriptor_symbol(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                abi_overrides={
                    "descriptor_symbol": "zircon_native_plugin_descriptor_legacy"
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn(
                "abi.descriptor_symbol must be zircon_native_plugin_descriptor_v3",
                diagnostics,
            )
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_non_v3_abi_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                abi_overrides={"abi_version": 2},
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn("abi.abi_version must be 3", diagnostics)
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_duplicate_recursive_package_sources(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("group_a") / "animation_runtime",
                package_id="animation",
            )
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("group_b") / "animation_runtime",
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            diagnostics = "\n".join(report["diagnostics"])
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertIn("multiple source package manifests", diagnostics)
            self.assertIsNone(report["loader_manifest"])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_copies_macos_dsym_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_macos_native_dynamic_package_fixture(repo_root)
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="macos-release",
                target_platform="macos-aarch64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.profile = "macos-release"
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertIn(
                "plugins/animation/native/libzircon_plugin_animation.dylib",
                manifest_paths,
            )
            self.assertIn(
                "plugins/animation/native/zircon_plugin_animation.dSYM/Contents/Resources/DWARF/zircon_plugin_animation",
                manifest_paths,
            )

    def test_native_dynamic_stage_reports_package_loadable_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json.loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            materialized_package = report["materialized_packages"][0]
            self.assertEqual(exit_code, 0)
            self.assertEqual(materialized_package["package_id"], "animation")
            self.assertEqual(materialized_package["loadable_artifact_count"], 1)
            self.assertEqual(
                materialized_package["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation.dll"],
            )

    def test_native_dynamic_payload_summary_keeps_loadable_artifact_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)
            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            package_summary = summary["materialized_packages"][0]
            self.assertEqual(package_summary["package_id"], "animation")
            self.assertEqual(package_summary["loadable_artifact_count"], 1)
            self.assertEqual(
                package_summary["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation.dll"],
            )

    def test_native_dynamic_payload_summary_rejects_reported_plugins_dir_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            plugins_dir = out / "stages" / "native_dynamic" / "plugins"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(plugins_dir):
                    raise OSError("simulated reported plugins_dir resolve failure")
                return original_resolve(path, *args, **kwargs)

            diagnostics: list[str] = []
            with mock.patch.object(Path, "resolve", resolve_or_fail):
                summary = native_dynamic_stage_payload_summary(
                    out,
                    "windows-release",
                    plugins_dir,
                    diagnostics,
                )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertTrue(
                any(
                    "NativeDynamic report plugins_dir" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated reported plugins_dir resolve failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_current_plugins_dir_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            plugins_dir = out / "stages" / "native_dynamic" / "plugins"
            original_resolve = Path.resolve
            plugins_dir_resolve_count = 0

            def resolve_or_fail_second_plugins_dir(
                path: Path,
                *args: object,
                **kwargs: object,
            ) -> Path:
                nonlocal plugins_dir_resolve_count
                if str(path) == str(plugins_dir):
                    plugins_dir_resolve_count += 1
                    if plugins_dir_resolve_count > 1:
                        raise OSError(
                            "simulated current plugins_dir resolve failure"
                        )
                return original_resolve(path, *args, **kwargs)

            diagnostics: list[str] = []
            with mock.patch.object(Path, "resolve", resolve_or_fail_second_plugins_dir):
                summary = native_dynamic_stage_payload_summary(
                    out,
                    "windows-release",
                    plugins_dir,
                    diagnostics,
                )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertTrue(
                any(
                    "NativeDynamic current plugins_dir" in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated current plugins_dir resolve failure" in diagnostic
                    for diagnostic in diagnostics
                ),
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_malformed_package_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            del report["materialized_packages"][0]["loadable_artifact_count"]
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report materialized_packages[0].loadable_artifact_count must be an integer",
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_file_manifest_field_type(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            report["file_manifest"][0]["bytes"] = "1"
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report file_manifest[0].bytes must be an integer",
                diagnostics,
            )
            self.assertNotIn(
                "NativeDynamic report file_manifest is malformed",
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_materialized_package_field_type(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            report["materialized_packages"][0]["loadable_artifact_count"] = "1"
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report "
                "materialized_packages[0].loadable_artifact_count "
                "must be an integer",
                diagnostics,
            )
            self.assertNotIn(
                "NativeDynamic report materialized_packages are malformed",
                diagnostics,
            )

    def test_native_dynamic_payload_summary_rejects_loadable_artifact_not_in_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation"),
                package_id="animation",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)
            exit_code = _run_stage_quiet(args)
            report_path = out / "stages" / "native_dynamic" / "report.json"
            report = json.loads(report_path.read_text(encoding="utf-8"))
            report["materialized_packages"][0]["loadable_artifacts"] = [
                "plugins/animation/native/missing_plugin.dll"
            ]
            report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")

            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertIsNone(summary)
            self.assertIn(
                "NativeDynamic report loadable_artifacts are not present in file_manifest",
                diagnostics,
            )

    def test_native_dynamic_payload_summary_accepts_sanitized_package_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            out = root / "out"
            _write_windows_native_dynamic_package_fixture_at(
                repo_root,
                Path("animation.fx"),
                package_id="animation.fx",
            )
            _write_validate_report_with_native_dynamic_exports(
                out,
                profile="windows-release",
                target_platform="windows-x86_64",
                native_dynamic_packages=["animation.fx"],
                package_export_overrides={
                    "package_id": "animation.fx",
                    "directory": "animation_fx",
                    "path": "plugins/animation_fx",
                    "manifest": "plugins/animation_fx/plugin.toml",
                    "package_report": "plugins/animation_fx/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)
            diagnostics: list[str] = []
            summary = native_dynamic_stage_payload_summary(
                out,
                "windows-release",
                out / "stages" / "native_dynamic" / "plugins",
                diagnostics,
            )

            self.assertEqual(exit_code, 0)
            self.assertEqual(diagnostics, [])
            self.assertIsNotNone(summary)
            self.assertEqual(
                summary["materialized_packages"][0]["loadable_artifacts"],
                ["plugins/animation_fx/native/zircon_plugin_animation.dll"],
            )
