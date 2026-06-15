from __future__ import annotations

import argparse
import contextlib
import hashlib
import io
import os
import shutil
import tempfile
import tomllib
import unittest
from unittest import mock
from pathlib import Path

from tools.zircon_export.cli import (
    compile_host_command,
    main,
    parse_args,
    apply_pipeline_stage_defaults,
    run_cook_assets,
    run_compile_host,
    run_pack,
    run_pipeline,
    run_platform_bundle,
    run_report,
    run_stage,
    run_source_template,
    validate_export_template,
)
from tools.zircon_export.source_template import source_template_command
from tools.zircon_export.pipeline_report import build_pipeline_report


REPO_ROOT = Path(__file__).resolve().parents[3]
VALID_TEMPLATE = REPO_ROOT / "export-templates" / "windows-x86_64-library_embed-debug"
LINUX_TEMPLATE = REPO_ROOT / "export-templates" / "linux-x86_64-library_embed-debug"
MACOS_TEMPLATE = REPO_ROOT / "export-templates" / "macos-aarch64-library_embed-debug"


class ExportTemplateValidationTests(unittest.TestCase):
    def test_template_version_mismatch_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    "format_version = 1",
                    "format_version = 999",
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("format_version 999" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

    def test_valid_template_resolves_declared_host(self) -> None:
        report = validate_export_template(
            template_dir=VALID_TEMPLATE,
            expected_engine_version="0.1.0",
            profile="windows-release",
            expected_target_platform="windows-x86_64",
        )

        self.assertFalse(report["fatal"], report["diagnostics"])
        self.assertEqual(report["format_version"], 1)
        self.assertEqual(report["target_platform"], "windows-x86_64")
        self.assertEqual(
            Path(report["host_executable"]),
            VALID_TEMPLATE / "bin" / "zircon_runtime.host-placeholder",
        )
        self.assertEqual(
            report["computed_content_hash"],
            _template_content_hash(
                "bin/zircon_runtime.host-placeholder",
                report["files"][0]["sha256"],
            ),
        )

    def test_template_rejects_aliasing_file_and_host_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            aliased_path = "bin/./zircon_runtime.host-placeholder"
            bundle_path = "bin/zircon_runtime.host-placeholder"
            aliased_hash = _template_content_hash(
                aliased_path,
                _file_sha256(template_dir / "bin" / "zircon_runtime.host-placeholder"),
                bundle_path=bundle_path,
            )
            manifest.write_text(
                manifest.read_text(encoding="utf-8")
                .replace(
                    'host_executable = "bin/zircon_runtime.host-placeholder"',
                    f'host_executable = "{aliased_path}"',
                )
                .replace(
                    'path = "bin/zircon_runtime.host-placeholder"',
                    f'path = "{aliased_path}"\nbundle_path = "{bundle_path}"',
                )
                .replace(
                    'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
                    f'content_hash = "{aliased_hash}"',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("paths.host_executable must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )
        self.assertTrue(
            any("[[files]] entry 0 path must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

    def test_linux_template_materializes_directory_layout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=LINUX_TEMPLATE,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            self.assertEqual(exit_code, 0)
            self.assertTrue((root / "out" / "bundle" / "linux-release" / "ZirconRuntime").exists())
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "data" / "assets.zrpack").exists()
            )
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "zircon-export.json").exists()
            )

    def test_macos_template_materializes_app_bundle_layout(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="macos-release",
                    template_dir=MACOS_TEMPLATE,
                    pack_file=pack,
                    target_platform="macos-aarch64",
                )
            )

            app_root = root / "out" / "bundle" / "macos-release" / "ZirconRuntime.app"
            self.assertEqual(exit_code, 0)
            self.assertTrue((app_root / "Contents" / "MacOS" / "ZirconRuntime").exists())
            self.assertTrue((app_root / "Contents" / "Resources" / "assets.zrpack").exists())
            self.assertTrue((app_root / "Contents" / "Info.plist").exists())
            self.assertTrue((app_root / "Contents" / "Resources" / "zircon-export.json").exists())

    def test_template_root_resolves_compatible_platform_bundle_template(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=REPO_ROOT / "export-templates",
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                Path(report["template_resolution"]["template_dir"]),
                LINUX_TEMPLATE,
            )
            self.assertEqual(report["template"]["template_id"], "linux-x86_64-library_embed-debug")
            self.assertTrue(
                (root / "out" / "bundle" / "linux-release" / "data" / "assets.zrpack").exists()
            )

    def test_template_root_skips_invalid_matching_template_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_root = root / "templates"
            valid_template = template_root / "linux-valid"
            invalid_template = template_root / "linux-invalid"
            shutil.copytree(LINUX_TEMPLATE, valid_template)
            shutil.copytree(LINUX_TEMPLATE, invalid_template)
            manifest = invalid_template / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'content_hash = "ba15973051598ad7709f6314f11ab35863f322306cf565ff875747e999896398"',
                    'content_hash = "0000000000000000000000000000000000000000000000000000000000000000"',
                ),
                encoding="utf-8",
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=template_root,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["template_resolution"]["template_dir"]), valid_template)
            self.assertEqual(report["template"]["template_id"], "linux-x86_64-library_embed-debug")
            skipped_candidates = report["template_resolution"]["skipped_candidates"]
            self.assertEqual(len(skipped_candidates), 1)
            self.assertEqual(Path(skipped_candidates[0]["template_dir"]), invalid_template)
            self.assertTrue(
                any(
                    "content_hash" in diagnostic
                    for diagnostic in skipped_candidates[0]["diagnostics"]
                ),
                skipped_candidates[0]["diagnostics"],
            )

    def test_template_root_skips_malformed_template_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_root = root / "templates"
            valid_template = template_root / "linux-valid"
            malformed_template = template_root / "malformed"
            shutil.copytree(LINUX_TEMPLATE, valid_template)
            malformed_template.mkdir(parents=True)
            (malformed_template / "template.toml").write_text(
                'format_version = "not closed',
                encoding="utf-8",
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="linux-release",
                    template_dir=None,
                    template_root=template_root,
                    pack_file=pack,
                    target_platform="linux-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(Path(report["template_resolution"]["template_dir"]), valid_template)
            skipped_candidates = report["template_resolution"]["skipped_candidates"]
            self.assertEqual(len(skipped_candidates), 1)
            self.assertEqual(Path(skipped_candidates[0]["template_dir"]), malformed_template)
            self.assertTrue(
                any(
                    "not valid TOML" in diagnostic
                    for diagnostic in skipped_candidates[0]["diagnostics"]
                ),
                skipped_candidates[0]["diagnostics"],
            )

    def test_template_root_reports_missing_profile_match(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")

            exit_code = _run_platform_bundle_quiet(
                _platform_bundle_args(
                    out=root / "out",
                    profile="missing-profile",
                    template_dir=None,
                    template_root=REPO_ROOT / "export-templates",
                    pack_file=pack,
                    target_platform="windows-x86_64",
                )
            )

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["template_resolution"]["fatal"])
            self.assertTrue(
                any(
                    "no export template" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                (root / "out" / "bundle" / "missing-profile" / "assets.zrpack").exists()
            )
            self.assertFalse((root / "out" / "bundle" / "missing-profile").exists())
            self.assertIsNone(report["bundle_manifest"])

    def test_platform_bundle_failure_cleans_previous_profile_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)

            first_exit = _run_platform_bundle_quiet(args)
            bundle_dir = root / "out" / "bundle" / "windows-release"
            self.assertEqual(first_exit, 0)
            self.assertTrue((bundle_dir / "zircon_runtime.exe").exists())
            self.assertTrue((bundle_dir / "assets.zrpack").exists())
            self.assertTrue((bundle_dir / "bundle.json").exists())

            pack.unlink()
            second_exit = _run_platform_bundle_quiet(args)
            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )

            self.assertEqual(second_exit, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((bundle_dir / "zircon_runtime.exe").exists())
            self.assertFalse((bundle_dir / "assets.zrpack").exists())
            self.assertFalse((bundle_dir / "bundle.json").exists())

    def test_platform_bundle_copies_native_dynamic_plugins_dir(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native")
            native_file_manifest = _native_dynamic_plugins_file_manifest(native_plugins)
            native_content_hash = _native_dynamic_content_hash(native_file_manifest)

            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            bundled_plugins = root / "out" / "bundle" / "windows-release" / "plugins"
            bundle_manifest = json_loads(
                (root / "out" / "bundle" / "windows-release" / "bundle.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertTrue((bundled_plugins / "native_plugins.toml").exists())
            self.assertTrue(
                (bundled_plugins / "animation" / "native_dynamic_package.toml").exists()
            )
            self.assertEqual(Path(report["native_plugins"]), bundled_plugins)
            self.assertIsNone(report["native_plugins_payload"]["stage_report"])
            self.assertEqual(report["native_plugins_payload"]["source"], str(native_plugins))
            self.assertEqual(
                report["native_plugins_payload"]["bundle_path"],
                str(bundled_plugins),
            )
            self.assertEqual(
                report["native_plugins_payload"]["content_hash"],
                native_content_hash,
            )
            self.assertEqual(
                report["native_plugins_payload"]["file_count"],
                len(native_file_manifest),
            )
            self.assertEqual(
                report["native_plugins_payload"]["file_manifest"],
                native_file_manifest,
            )
            self.assertEqual(report["native_plugins_payload"]["package_count"], 1)
            self.assertEqual(
                report["native_plugins_payload"]["materialized_packages"],
                [
                    {
                        "package_id": "animation",
                        "destination": str(bundled_plugins / "animation"),
                        "package_report": str(
                            bundled_plugins / "animation" / "native_dynamic_package.toml"
                        ),
                        "loadable_artifact_count": 1,
                        "loadable_artifacts": [
                            "plugins/animation/native/zircon_plugin_animation.dll"
                        ],
                    }
                ],
            )
            self.assertEqual(
                report["native_plugins_payload"],
                bundle_manifest["native_plugins_payload"],
            )

    def test_platform_bundle_rejects_malformed_native_dynamic_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native")
            out = root / "out"
            native_report = out / "stages" / "native_dynamic" / "report.json"
            native_report.parent.mkdir(parents=True)
            native_report.write_text("{not valid json", encoding="utf-8")

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            bundle_dir = out / "bundle" / "windows-release"
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any("NativeDynamic report" in diagnostic for diagnostic in report["diagnostics"])
            )
            self.assertFalse(bundle_dir.exists())
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins_payload"])

    def test_platform_bundle_explicit_native_dir_uses_bundle_plugin_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            generated_plugins = _write_native_dynamic_stage_plugins(root / "generated")
            native_plugins = root / "manual-native-payload"
            shutil.copytree(generated_plugins, native_plugins)

            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            payload = report["native_plugins_payload"]
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertEqual(
                [entry["path"] for entry in payload["file_manifest"]],
                [
                    "plugins/animation/native/zircon_plugin_animation.dll",
                    "plugins/animation/native_dynamic_package.toml",
                    "plugins/native_plugins.toml",
                ],
            )
            self.assertEqual(
                payload["materialized_packages"][0]["loadable_artifacts"],
                ["plugins/animation/native/zircon_plugin_animation.dll"],
            )

    def test_platform_bundle_native_plugins_replaces_template_plugins_dir(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            template_dir = root / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            template_stale_plugin = template_dir / "plugins" / "template-stale.dll"
            template_stale_plugin.parent.mkdir(parents=True)
            template_stale_plugin.write_text("template stale plugin", encoding="utf-8")
            _append_template_file_entry(
                template_dir,
                path="plugins/template-stale.dll",
                sha256=_file_sha256(template_stale_plugin),
            )
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native")

            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=template_dir,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            bundled_plugins = root / "out" / "bundle" / "windows-release" / "plugins"
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertTrue((bundled_plugins / "native_plugins.toml").exists())
            self.assertTrue(
                (bundled_plugins / "animation" / "native" / "zircon_plugin_animation.dll").exists()
            )
            self.assertFalse((bundled_plugins / "template-stale.dll").exists())
            self.assertNotIn(
                "plugins/template-stale.dll",
                [entry["path"] for entry in report["native_plugins_payload"]["file_manifest"]],
            )
            self.assertFalse(
                any(
                    Path(entry["destination"]) == bundled_plugins / "template-stale.dll"
                    for entry in report["template_files"]
                ),
                report["template_files"],
            )

    def test_pipeline_platform_bundle_uses_native_dynamic_report_plugins(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(out / "stages" / "native_dynamic")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            bundled_plugins = out / "bundle" / "windows-release" / "plugins"
            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertTrue((bundled_plugins / "native_plugins.toml").exists())
            self.assertTrue(
                (bundled_plugins / "animation" / "native" / "zircon_plugin_animation.dll").exists()
            )
            self.assertEqual(Path(report["native_plugins"]), bundled_plugins)

    def test_pipeline_platform_bundle_preserves_native_dynamic_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(out / "stages" / "native_dynamic")
            native_file_manifest = _native_dynamic_plugins_file_manifest(native_plugins)
            native_content_hash = _native_dynamic_content_hash(native_file_manifest)
            native_materialized_packages = [
                {
                    "package_id": "animation",
                    "destination": str(native_plugins / "animation"),
                    "loadable_artifact_count": 1,
                    "loadable_artifacts": [
                        "plugins/animation/native/zircon_plugin_animation.dll"
                    ],
                }
            ]
            native_signing = {
                "enabled": True,
                "profile": "windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
            }
            native_notarization = {
                "enabled": True,
                "profile": "windows-attestation",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
            }
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=native_signing,
                native_notarization=native_notarization,
            )

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            bundle_manifest = json_loads(
                (out / "bundle" / "windows-release" / "bundle.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertEqual(report["native_plugins_payload"]["content_hash"], native_content_hash)
            self.assertEqual(report["native_plugins_payload"]["file_count"], len(native_file_manifest))
            self.assertEqual(
                report["native_plugins_payload"]["file_manifest"],
                native_file_manifest,
            )
            self.assertEqual(
                report["native_plugins_payload"]["materialized_packages"],
                [
                    {
                        "package_id": "animation",
                        "destination": str(
                            out / "bundle" / "windows-release" / "plugins" / "animation"
                        ),
                        "loadable_artifact_count": 1,
                        "loadable_artifacts": [
                            "plugins/animation/native/zircon_plugin_animation.dll"
                        ],
                    }
                ],
            )
            self.assertEqual(
                report["native_plugins_payload"]["native_signing"],
                native_signing,
            )
            self.assertEqual(
                report["native_plugins_payload"]["native_notarization"],
                native_notarization,
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["content_hash"],
                native_content_hash,
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["materialized_packages"],
                report["native_plugins_payload"]["materialized_packages"],
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["native_signing"],
                native_signing,
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["native_notarization"],
                native_notarization,
            )

    def test_pipeline_platform_bundle_rejects_stale_native_dynamic_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(out / "stages" / "native_dynamic")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)
            (native_plugins / "animation" / "native" / "zircon_plugin_animation.dll").write_text(
                "stale report payload",
                encoding="utf-8",
            )

            exit_code = _run_pipeline_quiet(
                _export_args(out=out, stage="platform_bundle", dry_run=False),
                "platform_bundle",
            )

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(
                any("NativeDynamic report content_hash" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

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

            def compile_success(command: list[str], cwd: Path) -> int:
                expected_host.parent.mkdir(parents=True)
                expected_host.write_text("host", encoding="utf-8")
                return 0

            with mock.patch(
                "tools.zircon_export.cli.subprocess.call",
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
            self.assertTrue(
                any("build validation skipped" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_source_template_stage_marks_invalid_generated_file_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            payload = _source_template_validate_report()
            generated_files = payload["plan_summary"]["generated_files"]
            generated_files.append(
                {
                    "path": "../escape.txt",
                    "purpose": "invalid generated file outside project",
                    "contents": "escape",
                }
            )
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

            stage_dir = root / "out" / "stages" / "source_template"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertFalse((stage_dir / "escape.txt").exists())
            self.assertTrue(
                any(
                    "escapes the SourceTemplate project" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_writes_package_export_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["stage"], "NativeDynamic")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(report["package_count"], 1)
            self.assertEqual(report["package_exports"][0]["package_id"], "animation")
            self.assertEqual(report["package_exports"][0]["directory"], "animation")
            self.assertEqual(report["package_exports"][0]["path"], "plugins/animation")
            self.assertEqual(report["package_exports"][0]["manifest"], "plugins/animation/plugin.toml")

    def test_native_dynamic_stage_materializes_package_and_loader_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            materialized_package = stage_dir / "plugins" / "animation"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            package_report = (materialized_package / "native_dynamic_package.toml").read_text(
                encoding="utf-8"
            )
            loader_manifest = (stage_dir / "plugins" / "native_plugins.toml").read_text(
                encoding="utf-8"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["loader_manifest"], str(stage_dir / "plugins" / "native_plugins.toml"))
            self.assertEqual(len(report["materialized_packages"]), 1)
            self.assertFalse(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], None)
            self.assertTrue((materialized_package / "plugin.toml").exists())
            self.assertTrue((materialized_package / "native" / "zircon_plugin_animation.dll").exists())
            self.assertTrue((materialized_package / "resources" / "animation.asset").exists())
            self.assertFalse((materialized_package / "src" / "lib.rs").exists())
            self.assertIn('package_id = "animation"', package_report)
            self.assertIn('[abi]', package_report)
            self.assertIn('package_report = "plugins/animation/native_dynamic_package.toml"', loader_manifest)
            self.assertIn("[plugins.abi]", loader_manifest)

    def test_native_dynamic_stage_reports_materialized_file_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            file_manifest = report["file_manifest"]
            manifest_paths = [entry["path"] for entry in file_manifest]
            self.assertEqual(exit_code, 0)
            self.assertEqual(
                manifest_paths,
                [
                    "plugins/animation/native/zircon_plugin_animation.dll",
                    "plugins/animation/native_dynamic_package.toml",
                    "plugins/animation/plugin.toml",
                    "plugins/animation/resources/animation.asset",
                    "plugins/native_plugins.toml",
                ],
            )
            self.assertEqual(manifest_paths, sorted(manifest_paths))
            for entry in file_manifest:
                self.assertGreater(entry["bytes"], 0)
                self.assertEqual(len(entry["sha256"]), 64)
                self.assertEqual(entry["sha256"], _file_sha256(stage_dir / entry["path"]))
            self.assertEqual(len(report["content_hash"]), 64)
            self.assertEqual(report["content_hash"], _native_dynamic_content_hash(file_manifest))

    def test_native_dynamic_package_report_records_package_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            package = out / "stages" / "native_dynamic" / "plugins" / "animation"
            package_report_path = package / "native_dynamic_package.toml"
            expected_files = _native_dynamic_package_payload_file_manifest(package)
            with package_report_path.open("rb") as package_report_file:
                package_report = tomllib.load(package_report_file)
            self.assertEqual(exit_code, 0)
            self.assertEqual(package_report["payload"]["file_count"], len(expected_files))
            self.assertEqual(
                package_report["payload"]["content_hash"],
                _native_dynamic_content_hash(expected_files),
            )
            self.assertEqual(package_report["payload"]["files"], expected_files)

    def test_native_dynamic_stage_removes_stale_unselected_packages(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            stale_package = out / "stages" / "native_dynamic" / "plugins" / "stale"
            (stale_package / "native").mkdir(parents=True)
            (stale_package / "plugin.toml").write_text('id = "stale"\n', encoding="utf-8")
            (stale_package / "native" / "zircon_plugin_stale.dll").write_text(
                "stale native payload",
                encoding="utf-8",
            )
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertFalse(stale_package.exists())
            self.assertFalse(any(path.startswith("plugins/stale/") for path in manifest_paths))
            self.assertTrue((stage_dir / "plugins" / "animation").exists())

    def test_native_dynamic_stage_filters_artifacts_by_target_platform(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            package_native_dir = repo_root / "zircon_plugins" / "animation" / "native"
            (package_native_dir / "zircon_plugin_animation.pdb").write_text(
                "windows debug symbols",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.so").write_text(
                "linux dynamic payload",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.dbg").write_text(
                "linux debug symbols",
                encoding="utf-8",
            )
            (package_native_dir / "libzircon_plugin_animation.dylib").write_text(
                "macos dynamic payload",
                encoding="utf-8",
            )
            (package_native_dir / "zircon_plugin_animation.dsym").write_text(
                "macos debug symbols",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            manifest_paths = [entry["path"] for entry in report["file_manifest"]]
            self.assertEqual(exit_code, 0)
            self.assertIn("plugins/animation/native/zircon_plugin_animation.dll", manifest_paths)
            self.assertIn("plugins/animation/native/zircon_plugin_animation.pdb", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.so", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.dbg", manifest_paths)
            self.assertNotIn("plugins/animation/native/libzircon_plugin_animation.dylib", manifest_paths)
            self.assertNotIn("plugins/animation/native/zircon_plugin_animation.dsym", manifest_paths)

    def test_native_dynamic_stage_requires_platform_loadable_artifact(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            package = repo_root / "zircon_plugins" / "animation"
            (package / "native").mkdir(parents=True)
            (package / "resources").mkdir()
            (package / "plugin.toml").write_text('id = "animation"\n', encoding="utf-8")
            (package / "native" / "zircon_plugin_animation.pdb").write_text(
                "windows debug symbols without runtime library",
                encoding="utf-8",
            )
            (package / "resources" / "animation.asset").write_text(
                "asset",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertTrue(
                any(
                    "has no loadable native library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_removes_partial_package_on_artifact_filter_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            package = repo_root / "zircon_plugins" / "animation"
            (package / "native").mkdir(parents=True)
            (package / "resources").mkdir()
            (package / "plugin.toml").write_text('id = "animation"\n', encoding="utf-8")
            (package / "native" / "libzircon_plugin_animation.so").write_text(
                "linux payload",
                encoding="utf-8",
            )
            (package / "resources" / "animation.asset").write_text(
                "asset",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertTrue(
                any(
                    "has no dynamic library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_removes_all_packages_when_any_package_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            physics = repo_root / "zircon_plugins" / "physics"
            (physics / "native").mkdir(parents=True)
            (physics / "resources").mkdir()
            (physics / "plugin.toml").write_text('id = "physics"\n', encoding="utf-8")
            (physics / "native" / "libzircon_plugin_physics.so").write_text(
                "linux payload",
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "physics"],
                extra_package_exports=[
                    _native_dynamic_package_export(
                        {
                            "package_id": "physics",
                            "directory": "physics",
                            "path": "plugins/physics",
                            "manifest": "plugins/physics/plugin.toml",
                            "package_report": "plugins/physics/native_dynamic_package.toml",
                        }
                    )
                ],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["cleanup_reason"], "fatal_diagnostics")
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertFalse((stage_dir / "plugins" / "animation").exists())
            self.assertFalse((stage_dir / "plugins" / "physics").exists())
            self.assertTrue(
                any(
                    "physics" in diagnostic and "has no dynamic library artifacts" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_inconsistent_package_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "path": "plugins/wrong-animation",
                    "manifest": "plugins/wrong-animation/plugin.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "path must be plugins/animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_inconsistent_package_report_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "package_report": "plugins/wrong-animation/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_report must be plugins/animation/native_dynamic_package.toml" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_derives_missing_package_report_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            validate_report = out / "stages" / "validate" / "report.json"
            validate_payload = json_loads(validate_report.read_text(encoding="utf-8"))
            validate_payload["plan_summary"]["native_dynamic_package_exports"][0].pop(
                "package_report"
            )
            validate_report.write_text(json_dumps(validate_payload), encoding="utf-8")
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            loader_manifest = (stage_dir / "plugins" / "native_plugins.toml").read_text(
                encoding="utf-8"
            )
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertEqual(
                report["package_exports"][0]["package_report"],
                "plugins/animation/native_dynamic_package.toml",
            )
            self.assertIn(
                'package_report = "plugins/animation/native_dynamic_package.toml"',
                loader_manifest,
            )

    def test_native_dynamic_stage_accepts_sanitized_package_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation.fx")
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
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

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0)
            self.assertFalse(report["fatal"])
            self.assertTrue((stage_dir / "plugins" / "animation_fx" / "plugin.toml").exists())
            self.assertTrue((stage_dir / "plugins" / "native_plugins.toml").exists())

    def test_native_dynamic_stage_rejects_package_directory_id_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "directory": "animation_copy",
                    "path": "plugins/animation_copy",
                    "manifest": "plugins/animation_copy/plugin.toml",
                    "package_report": "plugins/animation_copy/native_dynamic_package.toml",
                },
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "directory must be animation for package_id animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_duplicate_package_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                extra_package_exports=[
                    _native_dynamic_package_export(
                        {
                            "package_id": "animation",
                            "directory": "animation_copy",
                            "path": "plugins/animation_copy",
                            "manifest": "plugins/animation_copy/plugin.toml",
                        }
                    )
                ],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_id animation duplicates entry 0" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_source_manifest_id_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        'id = "wrong-animation"',
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "manifest id wrong-animation does not match selected package animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_source_manifest_parse_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                'id = "animation"\n[broken\n',
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "direct manifest could not be parsed" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_source_manifest_missing_id(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root, package_id="animation")
            animation_manifest = repo_root / "zircon_plugins" / "animation" / "plugin.toml"
            animation_manifest.write_text(
                "\n".join(
                    [
                        'version = "0.1.0"',
                        'default_packaging = ["native_dynamic"]',
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertTrue(report["payload_cleaned"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "direct manifest id must be a non-empty string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "no plugin.toml was found" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_unselected_package_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=[],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "package_export animation is not present in native_dynamic_packages" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_duplicate_selected_package_ids(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "animation"],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "native_dynamic_packages entry animation duplicates entry 0" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_rejects_missing_selected_package_export(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            _write_native_dynamic_package_fixture(repo_root)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(
                out,
                native_dynamic_packages=["animation", "physics"],
            )
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            stage_dir = out / "stages" / "native_dynamic"
            report = json_loads((stage_dir / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["loader_manifest"], None)
            self.assertEqual(report["materialized_packages"], [])
            self.assertFalse((stage_dir / "plugins" / "native_plugins.toml").exists())
            self.assertTrue(
                any(
                    "native_dynamic_packages entry physics has no package_export" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_native_dynamic_stage_reports_missing_package_source_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            repo_root = root / "repo"
            (repo_root / "zircon_plugins").mkdir(parents=True)
            out = root / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            args = _export_args(out=out, stage="native_dynamic", dry_run=False)
            args.repo_root = str(repo_root)

            exit_code = _run_stage_quiet(args)

            report = json_loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertEqual(report["materialized_packages"], [])
            self.assertEqual(report["loader_manifest"], None)
            self.assertFalse(
                (out / "stages" / "native_dynamic" / "plugins" / "native_plugins.toml").exists()
            )
            self.assertTrue(
                any("no plugin.toml was found" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_cook_assets_stage_writes_default_manifest_and_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "source"
            source_dir.mkdir()
            (source_dir / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest = source_dir / "assets.json"
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "asset_filter": "shipping",
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                                "dependencies": [],
                                "labels": ["shipping"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=root / "out", asset_manifest=source_manifest)
            )

            staged_manifest = root / "out" / "stages" / "cook_assets" / "assets.json"
            report = root / "out" / "stages" / "cook_assets" / "report.json"
            self.assertEqual(exit_code, 0)
            self.assertTrue(staged_manifest.exists())
            self.assertTrue(report.exists())
            manifest = json_loads(staged_manifest.read_text(encoding="utf-8"))
            self.assertEqual(
                manifest["assets"][0]["source"],
                str((source_dir / "main.scene").resolve()),
            )
            stage_report = json_loads(report.read_text(encoding="utf-8"))
            self.assertFalse(stage_report["fatal"], stage_report["diagnostics"])
            self.assertEqual(stage_report["asset_count"], 1)

    def test_pipeline_cook_assets_uses_validate_report_asset_filter(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = root / "source" / "assets.json"
            source_manifest.parent.mkdir(parents=True)
            (source_manifest.parent / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                                "labels": ["shipping"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["asset_filter"], "shipping")

    def test_cook_assets_preserves_manifest_asset_filter_over_pipeline_default(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source_manifest = root / "source" / "assets.json"
            source_manifest.parent.mkdir(parents=True)
            (source_manifest.parent / "main.scene").write_text("scene", encoding="utf-8")
            source_manifest.write_text(
                json_dumps(
                    {
                        "roots": ["scenes/main.zscene"],
                        "asset_filter": "editor",
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": "main.scene",
                                "labels": ["editor"],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            _write_validate_report_with_asset_filter(out, "shipping")
            args = _cook_assets_args(out=out, asset_manifest=source_manifest)

            apply_pipeline_stage_defaults(args, "cook_assets")
            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["asset_filter"], "editor")

    def test_cook_assets_derives_project_default_scene_without_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            scene = root / "project" / "assets" / "scenes" / "main.scene.toml"
            scene.parent.mkdir(parents=True)
            scene.write_text("scene", encoding="utf-8")
            project.write_text(
                "\n".join(
                    [
                        'name = "Export Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/main.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"
            args = _cook_assets_args(out=out, project=project)
            args.asset_filter = "shipping"

            exit_code = _run_cook_assets_quiet(args)

            staged_manifest = json_loads(
                (out / "stages" / "cook_assets" / "assets.json").read_text(
                    encoding="utf-8"
                )
            )
            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 0)
            self.assertEqual(staged_manifest["roots"], ["scenes/main.scene.toml"])
            self.assertEqual(staged_manifest["asset_filter"], "shipping")
            self.assertEqual(staged_manifest["assets"][0]["path"], "scenes/main.scene.toml")
            self.assertEqual(staged_manifest["assets"][0]["labels"], ["shipping"])
            self.assertEqual(staged_manifest["assets"][0]["source"], str(scene.resolve()))
            self.assertTrue(report["generated_from_project"])
            self.assertEqual(report["project_manifest"], str(project.resolve()))
            self.assertEqual(report["project_default_scene"], "res://scenes/main.scene.toml")

    def test_cook_assets_reports_missing_project_default_scene_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            project = root / "project" / "zircon-project.toml"
            project.parent.mkdir(parents=True)
            project.write_text(
                "\n".join(
                    [
                        'name = "Missing Scene Fixture"',
                        "format_version = 1",
                        'default_scene = "res://scenes/missing.scene.toml"',
                        "library_version = 3",
                    ]
                ),
                encoding="utf-8",
            )
            out = root / "out"

            exit_code = _run_cook_assets_quiet(
                _cook_assets_args(out=out, project=project)
            )

            report = json_loads(
                (out / "stages" / "cook_assets" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"])
            self.assertFalse((out / "stages" / "cook_assets" / "assets.json").exists())
            self.assertTrue(
                any("does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_pack_defaults_to_cook_assets_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _pack_args(out=root / "out")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            self.assertIn(
                f"asset_manifest={_default_cooked_manifest(root / 'out')}",
                stdout.getvalue(),
            )

    def test_pack_command_forwards_profile_to_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            args = _pack_args(out=root / "out")

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            output = stdout.getvalue()
            self.assertIn("--profile", output)
            self.assertIn("windows-release", output)

    def test_pack_reports_missing_asset_manifest_before_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            args = _pack_args(out=out, dry_run=False)

            with mock.patch("tools.zircon_export.cli.subprocess.call", return_value=0) as packer:
                exit_code = _run_pack_quiet(args)

            report_path = out / "stages" / "pack" / "report.json"
            self.assertEqual(exit_code, 2)
            packer.assert_not_called()
            self.assertTrue(report_path.exists())
            report = json_loads(report_path.read_text(encoding="utf-8"))
            self.assertTrue(report["fatal"])
            self.assertEqual(report["stage"], "Pack")
            self.assertEqual(report["profile"], "windows-release")
            self.assertEqual(Path(report["asset_manifest"]), _default_cooked_manifest(out))
            self.assertEqual(Path(report["pack"]), out / "stages" / "pack" / "assets.zrpack")
            self.assertTrue(
                any(
                    "asset manifest" in diagnostic and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_pack_delta_args_are_forwarded_to_packer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            previous_pack = root / "previous.zrpack"
            delta_pack = root / "out" / "stages" / "pack" / "assets.delta.zrpd"
            args = _pack_args(out=root / "out")
            args.previous_pack = str(previous_pack)
            args.delta_pack = str(delta_pack)

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                exit_code = run_pack(args)

            self.assertEqual(exit_code, 0)
            output = stdout.getvalue()
            self.assertIn(f"previous_pack={previous_pack}", output)
            self.assertIn(f"delta_pack={delta_pack}", output)
            self.assertIn("--previous-pack", output)
            self.assertIn(str(previous_pack), output)
            self.assertIn("--delta-pack", output)
            self.assertIn(str(delta_pack), output)

    def test_report_stage_aggregates_stage_reports(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            _write_validate_report_with_strategies(out, ["source_template", "library_embed"])
            for stage in (
                "source_template",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)

            exit_code = _run_report_quiet(_report_args(out=out))

            pipeline_report = json_loads((out / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(exit_code, 0)
            self.assertFalse(pipeline_report["fatal"], pipeline_report["diagnostics"])
            self.assertEqual(pipeline_report["missing_stages"], [])
            self.assertEqual(len(pipeline_report["stages"]), 6)

    def test_report_stage_allows_missing_optional_source_template_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertNotIn("source_template", report["missing_stages"])
            self.assertEqual(len(report["stages"]), 5)

    def test_report_stage_ignores_stale_strategy_reports(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            for stage in (
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            _write_stage_report(out, "source_template", fatal=True)
            _write_stage_report(out, "native_dynamic", fatal=True)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["fatal_stages"], [])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                [
                    "validate",
                    "compile_host",
                    "cook_assets",
                    "pack",
                    "platform_bundle",
                ],
            )

    def test_report_stage_rejects_unverified_delta_pack(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            delta_pack = root / "pack-output" / "assets.delta.zrpd"
            for stage in ("validate", "compile_host", "cook_assets", "platform_bundle"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(out, pack, delta_pack=delta_pack, delta_apply_verified=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("delta_apply_verified" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

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

    def test_report_stage_ignores_profile_mismatch_validate_strategies(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(
                out,
                ["source_template"],
                profile="other-profile",
            )
            _write_stage_report(out, "source_template", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("compile_host", report["missing_stages"])
            self.assertIn("cook_assets", report["missing_stages"])
            self.assertIn("pack", report["missing_stages"])
            self.assertIn("platform_bundle", report["missing_stages"])
            self.assertNotIn("source_template", report["missing_stages"])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertTrue(
                any("profile other-profile" in diagnostic for diagnostic in report["diagnostics"]),
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

    def test_report_stage_requires_native_dynamic_for_native_dynamic_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["native_dynamic"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("native_dynamic", report["missing_stages"])
            self.assertIn("compile_host", report["missing_stages"])
            self.assertIn("cook_assets", report["missing_stages"])
            self.assertIn("pack", report["missing_stages"])
            self.assertIn("platform_bundle", report["missing_stages"])
            self.assertNotIn("source_template", report["missing_stages"])

    def test_report_stage_projects_native_dynamic_release_audit(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_plugins_payload = {
                "content_hash": "native-payload-hash",
                "file_count": 3,
                "package_count": 1,
                "native_signing": {
                    "enabled": True,
                    "profile": "windows-store",
                    "target_platform": "windows-x86_64",
                    "allowed_platforms": ["windows"],
                    "platform_allowed": True,
                    "fatal": False,
                    "package_count": 1,
                },
                "native_notarization": {
                    "enabled": True,
                    "profile": "windows-attestation",
                    "target_platform": "windows-x86_64",
                    "allowed_platforms": ["windows"],
                    "platform_allowed": True,
                    "fatal": False,
                    "package_count": 1,
                },
            }
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            for stage in ("native_dynamic", "compile_host", "cook_assets", "pack"):
                _write_stage_report(out, stage, fatal=False)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                native_plugins_payload,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["native_plugins_payload"], native_plugins_payload)

    def test_report_stage_does_not_project_fatal_platform_bundle_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_plugins_payload = {
                "content_hash": "native-payload-hash",
                "file_count": 3,
                "package_count": 1,
            }
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            for stage in ("native_dynamic", "compile_host", "cook_assets", "pack"):
                _write_stage_report(out, stage, fatal=False)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                native_plugins_payload,
                fatal=True,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertNotIn("native_plugins_payload", report)

    def test_report_stage_does_not_project_profile_mismatch_platform_bundle_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_plugins_payload = {
                "content_hash": "native-payload-hash",
                "file_count": 3,
                "package_count": 1,
            }
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            for stage in ("native_dynamic", "compile_host", "cook_assets", "pack"):
                _write_stage_report(out, stage, fatal=False)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                native_plugins_payload,
                profile="other-profile",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("profile other-profile" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_stage_marks_missing_stage_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_stage_report(out, "validate", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("compile_host", report["missing_stages"])
            self.assertTrue(
                any("compile_host report" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

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
            _write_stage_report(out, "pack", fatal=False)

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
            self.assertFalse(pipeline_report["fatal"], pipeline_report["diagnostics"])

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


def _template_content_hash(path: str, sha256: str, *, bundle_path: str | None = None) -> str:
    hasher = hashlib.sha256()
    hasher.update(path.encode("utf-8"))
    hasher.update(b"\0")
    hasher.update((bundle_path or path).encode("utf-8"))
    hasher.update(b"\0")
    hasher.update(sha256.lower().encode("ascii"))
    hasher.update(b"\n")
    return hasher.hexdigest()


def _append_template_file_entry(template_dir: Path, *, path: str, sha256: str) -> None:
    manifest = template_dir / "template.toml"
    manifest_text = manifest.read_text(encoding="utf-8")
    entries = [{"path": "bin/zircon_runtime.host-placeholder", "sha256": _file_sha256(template_dir / "bin" / "zircon_runtime.host-placeholder")}]
    entries.append({"path": path, "sha256": sha256})
    hasher = hashlib.sha256()
    for entry in sorted(entries, key=lambda value: value["path"]):
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["sha256"].lower().encode("ascii"))
        hasher.update(b"\n")
    manifest_text = manifest_text.replace(
        'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
        f'content_hash = "{hasher.hexdigest()}"',
    )
    manifest_text += (
        "\n[[files]]\n"
        f'path = "{path}"\n'
        'purpose = "test stale template plugin cleanup"\n'
        f'sha256 = "{sha256}"\n'
    )
    manifest.write_text(manifest_text, encoding="utf-8")


def _file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _native_dynamic_plugins_file_manifest(plugins_dir: Path) -> list[dict[str, object]]:
    stage_dir = plugins_dir.parent
    file_manifest: list[dict[str, object]] = []
    for file_path in sorted(plugins_dir.rglob("*")):
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(stage_dir).as_posix()
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": file_path.stat().st_size,
                "sha256": _file_sha256(file_path),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def _native_dynamic_package_payload_file_manifest(package_dir: Path) -> list[dict[str, object]]:
    file_manifest: list[dict[str, object]] = []
    for file_path in sorted(package_dir.rglob("*")):
        if not file_path.is_file() or file_path.name == "native_dynamic_package.toml":
            continue
        relative_path = file_path.relative_to(package_dir).as_posix()
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": file_path.stat().st_size,
                "sha256": _file_sha256(file_path),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def _native_dynamic_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def _platform_bundle_args(
    *,
    out: Path,
    profile: str,
    template_dir: Path | None,
    template_root: Path | None = None,
    pack_file: Path,
    target_platform: str,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile=profile,
        project="zircon-project.toml",
        out=str(out),
        stage="platform_bundle",
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=None,
        native_plugin_root=None,
        asset_manifest=None,
        pack_file=str(pack_file),
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=str(template_dir) if template_dir else None,
        template_root=str(template_root) if template_root else None,
        engine_version="0.1.0",
        target_platform=target_platform,
        determinism_check=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=False,
    )


def _run_platform_bundle_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_platform_bundle(args)


def _run_cook_assets_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_cook_assets(args)


def _run_pack_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_pack(args)


def _run_report_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_report(args)


def _run_source_template_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_source_template(args)


def _run_stage_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_stage(args)


def _run_pipeline_quiet(args: argparse.Namespace, resume_from: str) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_pipeline(args, resume_from)


def _cook_assets_args(
    *,
    out: Path,
    asset_manifest: Path | None = None,
    project: Path | None = None,
) -> argparse.Namespace:
    args = _export_args(
        out=out,
        stage="cook_assets",
        asset_manifest=asset_manifest,
        dry_run=False,
    )
    if project is not None:
        args.project = str(project)
    return args


def _pack_args(*, out: Path, dry_run: bool = True) -> argparse.Namespace:
    return _export_args(out=out, stage="pack", dry_run=dry_run)


def _report_args(*, out: Path) -> argparse.Namespace:
    return _export_args(out=out, stage="report", dry_run=False)


def _source_template_args(
    *,
    out: Path,
    validate_report: Path | None = None,
    build: bool = False,
    dry_run: bool = True,
) -> argparse.Namespace:
    args = _export_args(out=out, stage="source_template", dry_run=dry_run)
    args.validate_report = str(validate_report) if validate_report else None
    args.source_template_build = build
    return args


def _export_args(
    *,
    out: Path,
    stage: str,
    asset_manifest: Path | None = None,
    dry_run: bool,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile="windows-release",
        project="zircon-project.toml",
        out=str(out),
        stage=stage,
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=None,
        native_plugin_root=None,
        asset_manifest=str(asset_manifest) if asset_manifest else None,
        asset_filter=None,
        pack_file=None,
        previous_pack=None,
        delta_pack=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        source_template_build=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=dry_run,
    )


def _compile_host_args(
    *,
    out: Path,
    validate_report: Path | None = None,
) -> argparse.Namespace:
    return argparse.Namespace(
        profile="windows-release",
        project="zircon-project.toml",
        out=str(out),
        stage="compile_host",
        resume_from="validate",
        repo_root=str(REPO_ROOT),
        cargo="cargo",
        validator=None,
        packer=None,
        validate_report=str(validate_report) if validate_report else None,
        native_plugin_root=None,
        asset_manifest=None,
        asset_filter=None,
        pack_file=None,
        host_executable=None,
        native_plugins_dir=None,
        template_dir=None,
        template_root=None,
        engine_version="0.1.0",
        target_platform=None,
        determinism_check=False,
        source_template_build=False,
        target_dir=None,
        offline=False,
        no_locked=False,
        pretty=False,
        dry_run=True,
    )


def _compile_host_plan() -> dict[str, object]:
    return {
        "package": "zircon_app",
        "binary": "zircon_runtime",
        "manifest_path": "Cargo.toml",
        "target_dir": "stages/compile_host/target",
        "cargo_profile": "debug",
        "release": False,
        "app_features": ["target-client"],
        "runtime_features": ["target-client"],
        "expected_runtime_plugins": [],
        "linked_runtime_crates": [],
        "command": [
            "cargo",
            "build",
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
    }


def _source_template_plan() -> dict[str, object]:
    return {
        "manifest_path": "Cargo.toml",
        "target_dir": "stages/source_template/target",
        "cargo_profile": "debug",
        "release": False,
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "Cargo.toml",
            "--target-dir",
            "stages/source_template/target",
        ],
    }


def _source_template_validate_report() -> dict[str, object]:
    return {
        "stage": "Validate",
        "profile": "windows-release",
        "fatal": False,
        "plan_summary": {
            "source_template_build": _source_template_plan(),
            "generated_files": [
                {
                    "path": "Cargo.toml",
                    "purpose": "generated runtime package manifest",
                    "contents": (
                        "[package]\n"
                        "name = \"source-template-smoke\"\n"
                        "version = \"0.1.0\"\n"
                        "edition = \"2021\"\n\n"
                        "[dependencies]\n"
                        "zircon_app = { path = \"../../zircon_app\", default-features = false }\n"
                    ),
                },
                {
                    "path": "src/main.rs",
                    "purpose": "generated runtime entrypoint",
                    "contents": "fn main() {}\n",
                },
            ],
        },
    }


def _run_compile_host_quiet(args: argparse.Namespace) -> int:
    with contextlib.redirect_stdout(io.StringIO()):
        return run_compile_host(args)


def json_dumps(value: object) -> str:
    import json

    return json.dumps(value, indent=2)


def json_loads(value: str) -> object:
    import json

    return json.loads(value)


def _write_stage_report(out: Path, stage: str, *, fatal: bool) -> None:
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": _stage_label(stage),
                "profile": "windows-release",
                "fatal": fatal,
                "diagnostics": ["fatal smoke"] if fatal else [],
            }
        ),
        encoding="utf-8",
    )


def _write_validate_report_with_asset_filter(out: Path, asset_filter: str) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "Validate",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "profile_summary": {
                    "asset_filter": asset_filter,
                },
            }
        ),
        encoding="utf-8",
    )


def _write_validate_report_with_strategies(
    out: Path,
    strategies: list[str],
    *,
    profile: str = "windows-release",
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "Validate",
                "profile": profile,
                "fatal": False,
                "diagnostics": [],
                "profile_summary": {
                    "strategies": strategies,
                },
            }
        ),
        encoding="utf-8",
    )


def _write_validate_report_with_native_dynamic_exports(
    out: Path,
    package_export_overrides: dict[str, object] | None = None,
    extra_package_exports: list[dict[str, object]] | None = None,
    native_dynamic_packages: list[str] | None = None,
) -> None:
    report_dir = out / "stages" / "validate"
    report_dir.mkdir(parents=True, exist_ok=True)
    package_exports = [
        _native_dynamic_package_export(package_export_overrides)
    ]
    if extra_package_exports:
        package_exports.extend(extra_package_exports)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "Validate",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "profile_summary": {
                    "strategies": ["native_dynamic"],
                    "target_platform": "windows-x86_64",
                },
                "plan_summary": {
                    "native_dynamic_packages": (
                        native_dynamic_packages
                        if native_dynamic_packages is not None
                        else ["animation"]
                    ),
                    "native_dynamic_package_exports": package_exports,
                },
            }
        ),
        encoding="utf-8",
    )


def _native_dynamic_package_export(
    overrides: dict[str, object] | None = None,
) -> dict[str, object]:
    package_export: dict[str, object] = {
        "package_id": "animation",
        "directory": "animation",
        "path": "plugins/animation",
        "manifest": "plugins/animation/plugin.toml",
        "package_report": "plugins/animation/native_dynamic_package.toml",
        "abi": {
            "abi_version": 3,
            "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
            "descriptor_contract": "NativePluginAbiV3",
            "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
            "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
            "host_function_table": "NativePluginHostFunctionTableV3",
            "entry_report_contract": "NativePluginEntryReportV3",
            "behavior_contract": "NativePluginBehaviorV3",
            "state_snapshot_contract": (
                "NativePluginBehaviorV3.save_state/restore_state"
            ),
            "bridge_method_table": "NativePluginBridgeMethodTableV3",
        },
    }
    if overrides:
        package_export.update(overrides)
    return package_export


def _write_native_dynamic_package_fixture(
    repo_root: Path,
    package_id: str = "animation",
) -> None:
    package = repo_root / "zircon_plugins" / package_id
    (package / "native").mkdir(parents=True)
    (package / "resources").mkdir()
    (package / "src").mkdir()
    (package / "plugin.toml").write_text(
        "\n".join(
            [
                f'id = "{package_id}"',
                'version = "0.1.0"',
                'default_packaging = ["native_dynamic"]',
            ]
        ),
        encoding="utf-8",
    )
    (package / "native" / f"zircon_plugin_{package_id}.dll").write_text(
        "native dynamic placeholder",
        encoding="utf-8",
    )
    (package / "resources" / f"{package_id}.asset").write_text(
        "resource placeholder",
        encoding="utf-8",
    )
    (package / "src" / "lib.rs").write_text(
        "pub fn should_not_ship() {}",
        encoding="utf-8",
    )


def _write_compile_host_report(out: Path, host_executable: Path) -> None:
    report_dir = out / "stages" / "compile_host"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "CompileHost",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "host_executable": str(host_executable),
            }
        ),
        encoding="utf-8",
    )


def _write_cook_assets_report(out: Path, cooked_manifest: Path) -> None:
    report_dir = out / "stages" / "cook_assets"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "CookAssets",
                "profile": "windows-release",
                "fatal": False,
                "diagnostics": [],
                "cooked_asset_manifest": str(cooked_manifest),
            }
        ),
        encoding="utf-8",
    )


def _write_pack_report(
    out: Path,
    pack: Path,
    *,
    delta_pack: Path | None = None,
    delta_apply_verified: bool | None = None,
) -> None:
    report_dir = out / "stages" / "pack"
    report_dir.mkdir(parents=True, exist_ok=True)
    report: dict[str, object] = {
        "stage": "Pack",
        "fatal": False,
        "diagnostics": [],
        "pack": str(pack),
    }
    if delta_pack is not None:
        report["delta_pack"] = str(delta_pack)
    if delta_apply_verified is not None:
        report["delta_apply_verified"] = delta_apply_verified
    report_dir.joinpath("report.json").write_text(json_dumps(report), encoding="utf-8")


def _write_platform_bundle_report_with_native_plugins_payload(
    out: Path,
    native_plugins_payload: dict[str, object],
    *,
    fatal: bool = False,
    profile: str = "windows-release",
) -> None:
    report_dir = out / "stages" / "platform_bundle"
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(
            {
                "stage": "PlatformBundle",
                "profile": profile,
                "fatal": fatal,
                "diagnostics": ["platform bundle failed"] if fatal else [],
                "native_plugins_payload": native_plugins_payload,
            }
        ),
        encoding="utf-8",
    )


def _write_native_dynamic_report(
    out: Path,
    plugins_dir: Path,
    *,
    native_signing: dict[str, object] | None = None,
    native_notarization: dict[str, object] | None = None,
) -> None:
    report_dir = out / "stages" / "native_dynamic"
    file_manifest = _native_dynamic_plugins_file_manifest(plugins_dir)
    report = {
        "stage": "NativeDynamic",
        "profile": "windows-release",
        "fatal": False,
        "diagnostics": [],
        "plugins_dir": str(plugins_dir),
        "loader_manifest": str(plugins_dir / "native_plugins.toml"),
        "file_manifest": file_manifest,
        "content_hash": _native_dynamic_content_hash(file_manifest),
        "materialized_packages": [
            {
                "package_id": "animation",
                "destination": str(plugins_dir / "animation"),
                "loadable_artifact_count": 1,
                "loadable_artifacts": [
                    "plugins/animation/native/zircon_plugin_animation.dll"
                ],
            }
        ],
    }
    if native_signing is not None:
        report["native_signing"] = native_signing
    if native_notarization is not None:
        report["native_notarization"] = native_notarization
    report_dir.mkdir(parents=True, exist_ok=True)
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def _write_native_dynamic_stage_plugins(stage_root: Path) -> Path:
    plugins_dir = stage_root / "plugins"
    package = plugins_dir / "animation"
    (package / "native").mkdir(parents=True)
    (plugins_dir / "native_plugins.toml").write_text(
        '[[plugins]]\nid = "animation"\n',
        encoding="utf-8",
    )
    (package / "native_dynamic_package.toml").write_text(
        'package_id = "animation"\n',
        encoding="utf-8",
    )
    (package / "native" / "zircon_plugin_animation.dll").write_text(
        "native dynamic placeholder",
        encoding="utf-8",
    )
    return plugins_dir


def _stage_label(stage: str) -> str:
    return "".join(part.capitalize() for part in stage.split("_"))


def _default_cooked_manifest(out: Path) -> Path:
    return out / "stages" / "cook_assets" / "assets.json"


if __name__ == "__main__":
    unittest.main()
