from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _report_args,
    _run_report_quiet,
    _write_compile_host_report,
    _write_pack_report,
    _write_platform_bundle_report_with_native_plugins_payload,
    _write_stage_report,
    _write_validate_report_with_strategies,
    _write_validate_report_with_strategies_value,
    json_loads,
)


def _native_dynamic_stage_operation_audit(**overrides: object) -> dict[str, object]:
    audit = {
        "enabled": True,
        "profile": "windows-store",
        "target_platform": "windows-x86_64",
        "allowed_platforms": ["windows"],
        "platform_allowed": True,
        "fatal": False,
        "package_count": 1,
        "diagnostics": [],
        "packages": [
            {
                "package_id": "animation",
                "artifact_count": 1,
                "artifacts": [
                    {
                        "artifact": "plugins/animation/native/zircon_plugin_animation.dll",
                        "package_relative_artifact": (
                            "native/zircon_plugin_animation.dll"
                        ),
                        "command": ["native-operation"],
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
    audit.update(overrides)
    return audit


def _native_dynamic_operation_audit_summary(
    audit: dict[str, object],
) -> dict[str, object]:
    return {
        "enabled": audit["enabled"],
        "profile": audit["profile"],
        "target_platform": audit["target_platform"],
        "allowed_platforms": audit["allowed_platforms"],
        "platform_allowed": audit["platform_allowed"],
        "fatal": audit["fatal"],
        "package_count": audit["package_count"],
    }


class PipelineReportStageTests(unittest.TestCase):
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
            self.assertEqual(
                pipeline_report["export_plan"],
                {
                    "strategies": ["library_embed", "source_template"],
                    "required_stages": [
                        "validate",
                        "source_template",
                        "compile_host",
                        "cook_assets",
                        "pack",
                        "platform_bundle",
                    ],
                    "completed_stages": [
                        "validate",
                        "source_template",
                        "compile_host",
                        "cook_assets",
                        "pack",
                        "platform_bundle",
                    ],
                    "unsupported_strategies": [],
                },
            )
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

    def test_report_stage_rejects_invalid_pack_delta_pack_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            for stage in ("validate", "compile_host", "cook_assets", "platform_bundle"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(out, pack, delta_pack_value=[])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("delta_pack must be a non-empty string" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_delta_without_pack_verification(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            bundled_delta = out / "bundle" / "windows-release" / "assets.delta.zrpd"
            for stage in ("validate", "compile_host", "cook_assets"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                delta_pack=bundled_delta,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("PlatformBundle report delta_pack is present" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_delta_source_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            pack_delta = root / "pack-output" / "assets.delta.zrpd"
            platform_delta = out / "bundle" / "windows-release" / "assets.delta.zrpd"
            for stage in ("validate", "compile_host", "cook_assets"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(
                out,
                pack,
                delta_pack=pack_delta,
                delta_apply_verified=True,
            )
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                delta_pack=platform_delta,
                delta_pack_source=root / "manual" / "other.delta.zrpd",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("does not match Pack report delta_pack" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_delta_without_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            pack_delta = root / "pack-output" / "assets.delta.zrpd"
            platform_delta = out / "bundle" / "windows-release" / "assets.delta.zrpd"
            for stage in ("validate", "compile_host", "cook_assets"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(
                out,
                pack,
                delta_pack=pack_delta,
                delta_apply_verified=True,
            )
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                delta_pack=platform_delta,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("delta_pack_source is required" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_pack_without_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            for stage in ("validate", "compile_host", "cook_assets"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                pack=platform_pack,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report pack_source must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_pack_source_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            for stage in ("validate", "compile_host", "cook_assets"):
                _write_stage_report(out, stage, fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                pack=platform_pack,
                pack_source=root / "manual" / "other.zrpack",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("pack_source does not match Pack report pack" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_host_without_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                pack=platform_pack,
                pack_source=pack,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_source must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_host_source_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=root / "manual" / "other-host.exe",
                pack=platform_pack,
                pack_source=pack,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "host_source does not match CompileHost report host_executable"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_allows_platform_argument_host_source(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            compile_host = root / "compile" / "zircon_runtime.exe"
            manual_host = root / "manual" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            manual_host.parent.mkdir(parents=True, exist_ok=True)
            manual_host.write_text("manual host placeholder", encoding="utf-8")
            pack.parent.mkdir(parents=True, exist_ok=True)
            pack.write_text("pack placeholder", encoding="utf-8")
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, compile_host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=manual_host,
                host_source_origin="argument",
                pack=platform_pack,
                pack_source=pack,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_stage_rejects_platform_bundle_without_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                write_bundle_manifest=False,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report bundle_manifest must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_bundle_manifest_missing_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            manifest = out / "bundle" / "windows-release" / "bundle.json"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                bundle_manifest=manifest,
                write_bundle_manifest=False,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("bundle_manifest" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_platform_bundle_manifest_host_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                bundle_manifest_overrides={
                    "host_source": str(root / "manual" / "other-host.exe")
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("bundle_manifest host_source does not match stage report" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_missing_platform_bundle_host_output(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                missing_output_fields={"host_executable"},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("host_executable" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_missing_platform_bundle_pack_output(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, pack)
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                missing_output_fields={"pack"},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("report pack" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_missing_platform_bundle_delta_output(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            host = root / "compile" / "zircon_runtime.exe"
            platform_host = out / "bundle" / "windows-release" / "zircon_runtime.exe"
            pack = root / "pack-output" / "assets.zrpack"
            pack_delta = root / "pack-output" / "assets.delta.zrpd"
            platform_pack = out / "bundle" / "windows-release" / "assets.zrpack"
            platform_delta = out / "bundle" / "windows-release" / "assets.delta.zrpd"
            _write_stage_report(out, "validate", fatal=False)
            _write_compile_host_report(out, host)
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(
                out,
                pack,
                delta_pack=pack_delta,
                delta_apply_verified=True,
            )
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                {},
                host_executable=platform_host,
                host_source=host,
                host_source_origin="compile_host_report",
                pack=platform_pack,
                pack_source=pack,
                delta_pack=platform_delta,
                delta_pack_source=pack_delta,
                missing_output_fields={"delta_pack"},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("delta_pack" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
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
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertTrue(
                any("profile other-profile" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_stage_rejects_unknown_validate_strategy_without_defaulting(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["future_export_path"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertEqual(
                report["export_plan"],
                {
                    "strategies": [],
                    "required_stages": ["validate"],
                    "completed_stages": ["validate"],
                    "unsupported_strategies": ["future_export_path"],
                },
            )
            self.assertTrue(
                any(
                    "unsupported export strategy future_export_path"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_empty_validate_strategies_without_defaulting(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, [])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertTrue(
                any(
                    "profile_summary.strategies must include at least one supported export strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_invalid_validate_strategies_without_defaulting(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies_value(out, "library_embed")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                [stage["stage_key"] for stage in report["stages"]],
                ["validate"],
            )
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "validate report profile_summary.strategies must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

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
            native_signing = _native_dynamic_stage_operation_audit()
            native_notarization = _native_dynamic_stage_operation_audit(
                profile="windows-attestation",
            )
            native_signing_summary = _native_dynamic_operation_audit_summary(
                native_signing
            )
            native_notarization_summary = _native_dynamic_operation_audit_summary(
                native_notarization
            )
            native_payload_source = str(out / "stages" / "native_dynamic" / "plugins")
            native_plugins_payload = {
                "stage_report": str(out / "stages" / "native_dynamic" / "report.json"),
                "source": native_payload_source,
                "content_hash": "native-payload-hash",
                "file_count": 3,
                "package_count": 1,
                "native_signing": native_signing_summary,
                "native_notarization": native_notarization_summary,
            }
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            for stage in ("native_dynamic", "compile_host", "cook_assets", "pack"):
                _write_stage_report(out, stage, fatal=False)
            native_dynamic_report = json.loads(
                (out / "stages" / "native_dynamic" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            native_dynamic_report["native_signing"] = native_signing
            native_dynamic_report["native_notarization"] = native_notarization
            (out / "stages" / "native_dynamic" / "report.json").write_text(
                json.dumps(native_dynamic_report, indent=2),
                encoding="utf-8",
            )
            _write_platform_bundle_report_with_native_plugins_payload(
                out,
                native_plugins_payload,
            )
            platform_bundle_path = out / "stages" / "platform_bundle" / "report.json"
            platform_bundle_report = json.loads(
                platform_bundle_path.read_text(encoding="utf-8")
            )
            platform_bundle_report["native_plugins_payload"]["source"] = (
                native_payload_source
            )
            platform_bundle_path.write_text(
                json.dumps(platform_bundle_report, indent=2),
                encoding="utf-8",
            )
            bundle_manifest_path = out / "bundle" / "windows-release" / "bundle.json"
            bundle_manifest = json.loads(bundle_manifest_path.read_text(encoding="utf-8"))
            bundle_manifest["native_plugins_payload"] = platform_bundle_report[
                "native_plugins_payload"
            ]
            bundle_manifest_path.write_text(
                json.dumps(bundle_manifest, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            native_plugins_payload["source"] = native_payload_source
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


if __name__ == "__main__":
    unittest.main()
