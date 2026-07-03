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
    _platform_bundle_args,
    _run_pipeline_quiet,
    _run_platform_bundle_quiet,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
    _write_validate_report_with_strategies_value,
    json_dumps,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _native_dynamic_content_hash,
    _native_dynamic_plugins_file_manifest,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)


class PlatformBundleNativeDynamicPipelinePayloadTests(unittest.TestCase):
    def test_pipeline_platform_bundle_uses_native_dynamic_report_plugins(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "stages" / "compile_host" / "zircon_runtime.exe"
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
            host = out / "stages" / "compile_host" / "zircon_runtime.exe"
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
                                "artifact": (
                                    "plugins/animation/native/"
                                    "zircon_plugin_animation.dll"
                                ),
                                "package_relative_artifact": (
                                    "native/zircon_plugin_animation.dll"
                                ),
                                "command": ["signtool", "sign"],
                                "exit_code": 0,
                                "stdout": "",
                                "stderr": "",
                                "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                                "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
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
                                "artifact": (
                                    "plugins/animation/native/"
                                    "zircon_plugin_animation.dll"
                                ),
                                "package_relative_artifact": (
                                    "native/zircon_plugin_animation.dll"
                                ),
                                "command": ["notarytool", "submit"],
                                "exit_code": 0,
                                "stdout": "",
                                "stderr": "",
                                "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                                "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
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
                        "source": str(out / "zircon_plugins" / "animation"),
                        "destination": str(
                            out / "bundle" / "windows-release" / "plugins" / "animation"
                        ),
                        "package_report": str(
                            out
                            / "bundle"
                            / "windows-release"
                            / "plugins"
                            / "animation"
                            / "native_dynamic_package.toml"
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


if __name__ == "__main__":
    unittest.main()
