from __future__ import annotations

import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    VALID_TEMPLATE,
    _append_template_file_entry,
    _export_args,
    _file_sha256,
    _native_dynamic_content_hash,
    _native_dynamic_plugins_file_manifest,
    _platform_bundle_args,
    _run_pipeline_quiet,
    _run_platform_bundle_quiet,
    _write_compile_host_report,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
    _write_validate_report_with_strategies_value,
    json_dumps,
    json_loads,
)


class PlatformBundleNativeDynamicTests(unittest.TestCase):
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
            first_report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            first_manifest = json_loads(
                (bundle_dir / "bundle.json").read_text(encoding="utf-8")
            )
            self.assertEqual(first_exit, 0)
            self.assertTrue((bundle_dir / "zircon_runtime.exe").exists())
            self.assertTrue((bundle_dir / "assets.zrpack").exists())
            self.assertTrue((bundle_dir / "bundle.json").exists())
            self.assertEqual(Path(first_report["host_source"]), host)
            self.assertEqual(first_report["host_source_origin"], "argument")
            self.assertEqual(Path(first_manifest["host_source"]), host)
            self.assertEqual(first_manifest["host_source_origin"], "argument")

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

    def test_platform_bundle_rejects_host_directory_input(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            host = root / "zircon_runtime.exe"
            host.mkdir()
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

            exit_code = _run_platform_bundle_quiet(args)
            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "host executable" in diagnostic and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])

    def test_platform_bundle_rejects_pack_directory_input(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "assets.zrpack"
            pack.mkdir()
            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)

            exit_code = _run_platform_bundle_quiet(args)
            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "pack file" in diagnostic and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])

    def test_platform_bundle_requires_bundle_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            out = root / "out"
            _write_validate_report_with_strategies(out, ["source_template"])
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)

            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "PlatformBundle stage requires library_embed or native_dynamic strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])

    def test_platform_bundle_rejects_delta_pack_directory_input(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            delta_pack = root / "assets.zrpdelta"
            delta_pack.mkdir()
            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.delta_pack = str(delta_pack)

            exit_code = _run_platform_bundle_quiet(args)
            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "delta pack file" in diagnostic and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])

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

    def test_platform_bundle_rejects_native_dynamic_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native")
            out = root / "out"
            native_report = out / "stages" / "native_dynamic" / "report.json"
            native_report.mkdir(parents=True)

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
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())
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

    def test_platform_bundle_explicit_native_dir_rejects_payload_rewrite_resolve_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            generated_plugins = _write_native_dynamic_stage_plugins(root / "generated")
            native_plugins = root / "manual-native-payload"
            shutil.copytree(generated_plugins, native_plugins)
            package_dir = native_plugins / "animation"

            args = _platform_bundle_args(
                out=root / "out",
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)
            original_resolve = Path.resolve
            package_dir_resolve_count = 0

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                nonlocal package_dir_resolve_count
                if str(path) == str(package_dir):
                    package_dir_resolve_count += 1
                    if package_dir_resolve_count > 1:
                        raise OSError("simulated explicit payload rewrite failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (root / "out" / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] destination"
                    in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated explicit payload rewrite failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_platform_bundle_rejects_native_dynamic_package_report_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(root / "native")
            package_report = native_plugins / "animation" / "native_dynamic_package.toml"
            package_report.unlink()
            package_report.mkdir()

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
            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "native_dynamic_package.toml" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((root / "out" / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins_payload"])

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
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)

            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_pipeline_quiet(args, "platform_bundle")

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

    def test_pipeline_platform_bundle_rejects_inherited_native_dynamic_report_directory(
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
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            native_report = out / "stages" / "native_dynamic" / "report.json"
            native_report.mkdir(parents=True)

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
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertIsNone(report["bundle_manifest"])
            self.assertIsNone(report["native_plugins_payload"])

    def test_pipeline_platform_bundle_rejects_profile_mismatch_native_dynamic_report(
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
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(
                out,
                native_plugins,
                profile="other-profile",
            )

            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            bundled_plugins = out / "bundle" / "windows-release" / "plugins"
            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse(bundled_plugins.exists())
            self.assertTrue(
                any("NativeDynamic report profile other-profile" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_pipeline_platform_bundle_rejects_invalid_native_dynamic_metadata(
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
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)
            native_report = out / "stages" / "native_dynamic" / "report.json"
            native_payload = json_loads(native_report.read_text(encoding="utf-8"))
            native_payload["fatal"] = []
            native_report.write_text(json_dumps(native_payload), encoding="utf-8")

            args = _export_args(out=out, stage="platform_bundle", dry_run=False)
            args.native_plugins_dir = str(native_plugins)
            exit_code = _run_pipeline_quiet(args, "platform_bundle")

            bundled_plugins = out / "bundle" / "windows-release" / "plugins"
            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertFalse(bundled_plugins.exists())
            self.assertTrue(
                any(
                    "NativeDynamic report fatal must be a boolean" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

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
            native_signing_summary = {
                "enabled": True,
                "profile": "windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
            }
            native_signing = {
                **native_signing_summary,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "animation",
                        "artifact_count": 1,
                        "artifacts": [
                            {
                                "artifact": str(
                                    native_plugins
                                    / "animation"
                                    / "native"
                                    / "zircon_plugin_animation.dll"
                                ),
                                "package_relative_artifact": (
                                    "native/zircon_plugin_animation.dll"
                                ),
                                "command": ["signtool", "sign"],
                                "exit_code": 0,
                                "stdout": "",
                                "stderr": "",
                                "before_sha256": "before-hash",
                                "after_sha256": "after-hash",
                            }
                        ],
                    }
                ],
            }
            native_notarization_summary = {
                "enabled": True,
                "profile": "windows-attestation",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
            }
            native_notarization = {
                **native_notarization_summary,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "animation",
                        "artifact_count": 1,
                        "artifacts": [
                            {
                                "artifact": str(
                                    native_plugins
                                    / "animation"
                                    / "native"
                                    / "zircon_plugin_animation.dll"
                                ),
                                "package_relative_artifact": (
                                    "native/zircon_plugin_animation.dll"
                                ),
                                "command": ["notarytool", "submit"],
                                "exit_code": 0,
                                "stdout": "",
                                "stderr": "",
                                "before_sha256": "before-hash",
                                "after_sha256": "after-hash",
                            }
                        ],
                    }
                ],
            }
            _write_validate_report_with_strategies(out, ["native_dynamic"])
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
            self.assertEqual(exit_code, 0, report["diagnostics"])
            bundle_manifest = json_loads(
                (out / "bundle" / "windows-release" / "bundle.json").read_text(
                    encoding="utf-8"
                )
            )
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
                native_signing_summary,
            )
            self.assertEqual(
                report["native_plugins_payload"]["native_notarization"],
                native_notarization_summary,
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
                native_signing_summary,
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["native_notarization"],
                native_notarization_summary,
            )

    def test_pipeline_platform_bundle_rejects_native_payload_destination_summary_resolve_error(
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
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins)
            package_dir = native_plugins / "animation"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if str(path) == str(package_dir):
                    raise OSError(
                        "simulated native payload destination rewrite failure"
                    )
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
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
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertIsNone(report["native_plugins_payload"])
            self.assertIsNone(report["bundle_manifest"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "NativeDynamic payload materialized_packages[0] destination"
                    in diagnostic
                    and "could not be resolved" in diagnostic
                    and "simulated native payload destination rewrite failure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
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
            _write_validate_report_with_strategies(out, ["native_dynamic"])
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

    def test_pipeline_platform_bundle_requires_native_dynamic_payload_for_native_dynamic_profile(
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
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_native_dynamic_report(out, native_plugins, profile="other-profile")

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
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertFalse((out / "report.json").exists())
            self.assertTrue(
                any(
                    "NativeDynamic profile requires native plugins"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_rejects_invalid_validate_metadata_for_strategy(
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
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            validate_report = out / "stages" / "validate" / "report.json"
            validate_payload = json_loads(validate_report.read_text(encoding="utf-8"))
            validate_payload["fatal"] = []
            validate_report.write_text(json_dumps(validate_payload), encoding="utf-8")

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            exit_code = _run_platform_bundle_quiet(args)

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertFalse((out / "report.json").exists())
            self.assertTrue(
                any(
                    "Validate report fatal must be a boolean" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_explicit_native_dir_rejects_invalid_validate_metadata(
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
            native_plugins = _write_native_dynamic_stage_plugins(root / "manual-native")
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            validate_report = out / "stages" / "validate" / "report.json"
            validate_payload = json_loads(validate_report.read_text(encoding="utf-8"))
            validate_payload["fatal"] = []
            validate_report.write_text(json_dumps(validate_payload), encoding="utf-8")

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

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertTrue(
                any(
                    "Validate report fatal must be a boolean" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_explicit_native_dir_requires_native_dynamic_strategy(
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
            native_plugins = _write_native_dynamic_stage_plugins(root / "manual-native")
            _write_validate_report_with_strategies(out, ["library_embed"])

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

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertTrue(
                any(
                    "native_plugins"
                    in diagnostic
                    and "native_dynamic strategy"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_staged_native_plugins_require_native_dynamic_strategy(
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
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_native_dynamic_report(out, native_plugins)

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            exit_code = _run_platform_bundle_quiet(args)

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertTrue(
                any(
                    "native_plugins"
                    in diagnostic
                    and "native_dynamic strategy"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_rejects_unknown_validate_strategy(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_validate_report_with_strategies(out, ["future_export_path"])

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            exit_code = _run_platform_bundle_quiet(args)

            bundle = out / "bundle" / "windows-release"
            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse(bundle.exists())
            self.assertTrue(
                any(
                    "unsupported export strategy future_export_path" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_rejects_empty_validate_strategies(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_validate_report_with_strategies(out, [])

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            exit_code = _run_platform_bundle_quiet(args)

            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "profile_summary.strategies must include at least one supported export strategy"
                    in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_platform_bundle_rejects_invalid_validate_strategies(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_validate_report_with_strategies_value(out, "library_embed")

            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            exit_code = _run_platform_bundle_quiet(args)

            platform_report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(exit_code, 2)
            self.assertTrue(platform_report["fatal"])
            self.assertFalse((out / "bundle" / "windows-release").exists())
            self.assertTrue(
                any(
                    "profile_summary.strategies must be a list" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )
            self.assertEqual(
                platform_report["diagnostics"].count(
                    "profile_summary.strategies must be a list"
                ),
                1,
                platform_report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
