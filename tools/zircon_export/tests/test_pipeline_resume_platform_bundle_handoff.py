from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_pipeline_quiet,
    _run_stage_quiet,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
    json_loads,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)


class PipelineResumePlatformBundleHandoffTests(unittest.TestCase):
    def test_pipeline_platform_bundle_uses_compile_host_report_host(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
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
            _write_compile_host_report(
                out,
                root / "compile" / "zircon_runtime.exe",
                host_value=[],
            )
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
            host = out / "compile" / "zircon_runtime.exe"
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
                    and "simulated compile host handoff resolve failure" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_uses_pack_report_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
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
            host = out / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            pack = root / "pack-output" / "custom-assets.zrpack"
            pack.parent.mkdir(parents=True)
            pack.write_text("custom pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack, pack_value=str(pack))

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
            self.assertEqual(
                bundled_pack.read_text(encoding="utf-8"),
                "custom pack placeholder",
            )
            self.assertFalse((out / "bundle" / "windows-release" / "assets.zrpack").exists())

    def test_stage_platform_bundle_uses_report_delta_pack_path(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
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
            _write_pack_report(
                out,
                pack,
                delta_pack=delta_pack,
                pack_value=str(pack),
                delta_pack_value=str(delta_pack),
            )

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
            host = out / "compile" / "zircon_runtime.exe"
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
                (
                    bundled_plugins
                    / "animation"
                    / "native"
                    / "zircon_plugin_animation.dll"
                ).exists()
            )
            self.assertEqual(Path(platform_report["native_plugins"]), bundled_plugins)

    def test_pipeline_platform_bundle_ignores_pack_report_without_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
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
            host = out / "compile" / "zircon_runtime.exe"
            host.parent.mkdir(parents=True)
            host.write_text("host placeholder", encoding="utf-8")
            default_pack = out / "stages" / "pack" / "assets.zrpack"
            default_pack.parent.mkdir(parents=True)
            default_pack.write_text("stale default pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(
                out,
                root / "pack-output" / "assets.zrpack",
                pack_value=[],
            )

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
                any(
                    "Pack report field pack must be a non-empty string" in diagnostic
                    for diagnostic in platform_report["diagnostics"]
                ),
                platform_report["diagnostics"],
            )

    def test_pipeline_platform_bundle_rejects_invalid_pack_report_delta_pack_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = out / "compile" / "zircon_runtime.exe"
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
