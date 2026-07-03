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


class PlatformBundleStrategyValidationTests(unittest.TestCase):
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
