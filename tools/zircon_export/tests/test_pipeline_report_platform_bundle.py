from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_link_plan,
    _native_dynamic_build_execution_report,
    _native_dynamic_build_plan_report,
    _native_dynamic_operation_audit_report,
    _native_dynamic_operation_audit_summary_report,
    _native_dynamic_package_export,
    _write_validate_report_with_strategies,
)
from tools.zircon_export.tests.pack_test_support import empty_delta_manifest


class PlatformBundleReportValidationTests(unittest.TestCase):
    def test_report_rejects_stale_native_plugins_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            plugin_artifact = fixture["native_plugins"] / "animation" / "native" / "zircon_plugin_animation.dll"
            plugin_artifact.write_text("mutated plugin", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("native_plugins_payload content_hash" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_missing_native_plugins_payload_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            _remove_tree(fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("native_plugins" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_rejects_native_plugins_payload_package_count_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"package_count": 2},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("package_count does not match" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_bundle_path_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_plugins = root / "external" / "plugins"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"bundle_path": str(external_plugins)},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload bundle_path"
                    in diagnostic
                    and "does not match native_plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_stage_report_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_stage_report = root / "external" / "native_dynamic" / "report.json"
            _write_text(external_stage_report, "{}")
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"stage_report": str(external_stage_report)},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_missing_stage_report_for_pipeline_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = None
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report is required"
                    in diagnostic
                    and "NativeDynamic stage report is present"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_source_mismatch_for_stage_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_source = root / "external" / "plugins"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={
                    "stage_report": str(
                        out / "stages" / "native_dynamic" / "report.json"
                    ),
                    "source": str(external_source),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload source"
                    in diagnostic
                    and "does not match NativeDynamic plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_outside_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_report = root / "external" / "native_dynamic_package.toml"
            _write_text(external_report, 'package_id = "animation"\n')
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["package_report"] = str(external_report)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "is outside package destination"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_destination_outside_plugins(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_package = root / "external" / "animation"
            _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["destination"] = str(external_package)
            _write_stage_report(out, "platform_bundle", platform_report)
            bundle_manifest = Path(str(platform_report["bundle_manifest"]))
            _write_bundle_manifest_from_platform_report(
                bundle_manifest,
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] destination"
                    in diagnostic
                    and "is outside native_plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_missing_native_plugins_payload_package_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package_report = Path(str(package["package_report"]))
            package_report.unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "does not exist"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_accepts_current_native_plugins_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_accepts_current_native_plugins_payload_for_legacy_validate_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_rejects_legacy_native_plugins_payload_wrong_profile_stage_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["profile"] = "other-profile"
            _write_stage_report(out, "native_dynamic", native_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic report profile other-profile does not match requested profile windows-release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_legacy_native_plugins_payload_stage_report_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            native_report_path = out / "stages" / "native_dynamic" / "report.json"
            native_report_path.unlink()
            native_report_path.mkdir()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic report" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_external_native_plugins_payload_stage_report_for_legacy_validate_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            external_plugins = root / "external" / "native_dynamic" / "plugins"
            _write_native_plugins(external_plugins)
            external_stage_report = external_plugins.parent / "report.json"
            _write_text(external_stage_report, "{}")
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(external_stage_report)
            payload["source"] = str(external_plugins)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_for_library_embed_strategy(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"] = {"strategies": ["library_embed"]}
            _write_stage_report(out, "validate", validate_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload"
                    in diagnostic
                    and "native_dynamic strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_manual_native_plugins_payload_without_stage_handoff_noise(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"] = {"strategies": ["library_embed"]}
            _write_stage_report(out, "validate", validate_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = None
            payload["source"] = platform_report["native_plugins"]
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "native_plugins_payload" in diagnostic
                    and "native_dynamic strategy" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "stage_report is required" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_bundle_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out, include_bundle=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report bundle must be a string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_root_outside_current_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_bundle = root / "external" / "bundle" / "windows-release"
            _write_platform_bundle_fixture(
                out,
                bundle_dir=external_bundle,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report bundle must match current output bundle"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_host_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report.pop("host_executable")
            platform_report.pop("host_source")
            platform_report.pop("host_source_origin")
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_executable must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_pack_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report.pop("pack")
            platform_report.pop("pack_source")
            platform_report.pop("pack_source_origin")
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report pack must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_host_source_origin(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report.pop("host_source_origin")
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_source_origin must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_pack_source_origin(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report.pop("pack_source_origin")
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report pack_source_origin must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_without_delta_pack_source_origin(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report.pop("delta_pack_source_origin")
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "delta_pack_source_origin is required"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_bundle_manifest_outside_bundle_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside_manifest = root / "outside" / "bundle.json"
            _write_platform_bundle_fixture(
                out,
                bundle_manifest=outside_manifest,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "bundle_manifest" in diagnostic
                    and "outside PlatformBundle bundle" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_bundle_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            manifest_dir = fixture["bundle_manifest"].parent / "bundle-manifest-dir"
            manifest_dir.mkdir()
            platform_report["bundle_manifest"] = str(manifest_dir)
            _write_stage_report(out, "platform_bundle", platform_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "bundle_manifest" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_host_output_outside_bundle_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside_host = root / "outside" / "zircon_runtime.exe"
            _write_platform_bundle_fixture(out, host_output=outside_host)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "host_executable" in diagnostic
                    and "outside PlatformBundle bundle" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_pack_output_outside_bundle_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside_pack = root / "outside" / "assets.zrpack"
            _write_platform_bundle_fixture(out, pack_output=outside_pack)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack" in diagnostic
                    and "outside PlatformBundle bundle" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_output_outside_bundle_root(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            outside_template_file = root / "outside" / "Info.plist"
            _write_platform_bundle_fixture(
                out,
                with_template_file=True,
                template_output=outside_template_file,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "template_files destination" in diagnostic
                    and "outside PlatformBundle bundle" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_missing_template_file_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            fixture["template_file"].unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "template_files destination" in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_file_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            fixture["template_file"].write_text("mutated plist", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "template_files destination" in diagnostic
                    and "sha256" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_accepts_current_template_file_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out, with_template_file=True)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_rejects_platform_host_output_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            fixture["platform_host"].write_text("mutated host", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "host_executable" in diagnostic and "does not match host_source" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_pack_output_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            fixture["platform_pack"].write_text("mutated pack", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack" in diagnostic and "does not match pack_source" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_delta_output_hash_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            fixture["platform_delta"].write_text("mutated delta", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "delta_pack" in diagnostic
                    and "does not match delta_pack_source" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_missing_platform_host_source_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            fixture["host_source"].unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "host_source" in diagnostic and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_missing_platform_pack_source_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            fixture["pack_source"].unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack_source" in diagnostic and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_missing_platform_delta_source_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_delta=True)
            fixture["delta_source"].unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "delta_pack_source" in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def _write_platform_bundle_fixture(
    out: Path,
    *,
    payload_overrides: dict[str, object] | None = None,
    with_template_file: bool = False,
    with_delta: bool = False,
    include_bundle: bool = True,
    bundle_dir: Path | None = None,
    bundle_manifest: Path | None = None,
    host_output: Path | None = None,
    pack_output: Path | None = None,
    template_output: Path | None = None,
) -> dict[str, Path]:
    profile = "windows-release"
    host = out / "compile" / "zircon_runtime.exe"
    pack = out / "pack-output" / "assets.zrpack"
    bundle_dir = bundle_dir or out / "bundle" / profile
    platform_host = host_output or bundle_dir / "zircon_runtime.exe"
    platform_pack = pack_output or bundle_dir / "assets.zrpack"
    delta_pack = out / "pack-output" / "assets.delta.zrpd"
    platform_delta = bundle_dir / "assets.delta.zrpd"
    native_stage_plugins = out / "stages" / "native_dynamic" / "plugins"
    native_plugins = bundle_dir / "plugins"
    bundle_manifest = bundle_manifest or bundle_dir / "bundle.json"
    template_source = out / "template" / "Info.plist"
    template_file = template_output or bundle_dir / "Contents" / "Info.plist"
    for path, text in (
        (host, "host"),
        (pack, "pack"),
    ):
        _write_text(path, text)
    _copy_file(host, platform_host)
    _copy_file(pack, platform_pack)
    if with_delta:
        _write_text(delta_pack, "delta")
        _copy_file(delta_pack, platform_delta)
    _write_native_plugins(native_stage_plugins)
    _write_native_plugins(native_plugins)
    native_payload = _native_plugins_payload(native_plugins)
    native_payload["stage_report"] = str(
        out / "stages" / "native_dynamic" / "report.json"
    )
    native_payload["source"] = str(native_stage_plugins)
    if payload_overrides:
        native_payload.update(payload_overrides)
    cooked_manifest = out / "stages" / "cook_assets" / "assets.json"
    _write_text(cooked_manifest, json.dumps({"roots": [], "assets": []}, indent=2))
    template_files: list[dict[str, object]] = []
    template_report: dict[str, object] | None = None
    if with_template_file:
        _write_text(template_source, "<plist>zircon</plist>")
        _write_text(template_file, "<plist>zircon</plist>")
        template_payload = template_source.read_bytes()
        template_report = {
            "template_dir": str(template_source.parent),
            "files": [
                {
                    "path": template_source.name,
                    "bundle_path": "Contents/Info.plist",
                    "sha256": hashlib.sha256(template_payload).hexdigest(),
                    "purpose": "platform_metadata",
                }
            ],
        }
        template_files.append(
            {
                "source": str(template_source),
                "destination": str(template_file),
            }
        )

    _write_validate_report_with_strategies(out, ["native_dynamic"], profile=profile)
    _write_report(
        out,
        "native_dynamic",
        {
            "stage": "NativeDynamic",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "stage_output": str(out / "stages" / "native_dynamic"),
            "validate_report": str(out / "stages" / "validate" / "report.json"),
            "target_platform": "windows-x86_64",
            "artifact_extensions": [".dll"],
            "native_plugin_root": str(out / "zircon_plugins"),
            "plugins_dir": str(native_stage_plugins),
            "loader_manifest": str(native_stage_plugins / "native_plugins.toml"),
            "content_hash": _native_plugins_content_hash(
                _native_plugins_file_manifest(native_stage_plugins)
            ),
            "file_manifest": _native_plugins_file_manifest(native_stage_plugins),
            "native_dynamic_packages": ["animation"],
            "package_exports": [_native_dynamic_package_export()],
            "package_count": 1,
            "native_build_plan": _native_dynamic_build_plan_report(),
            "native_build_execution": _native_dynamic_build_execution_report(),
            "native_signing": _native_dynamic_operation_audit_report(),
            "native_notarization": _native_dynamic_operation_audit_report(),
            "materialized_packages": _native_plugins_payload(native_stage_plugins)[
                "materialized_packages"
            ],
        },
    )
    _write_report(
        out,
        "compile_host",
        {
            "stage": "CompileHost",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "command": ["cargo", "build"],
            "exit_code": 0,
            "host_executable": str(host),
            "link_plan": _compile_host_link_plan(),
            "stdout_lines": [],
            "stderr_lines": [],
        },
    )
    _write_report(
        out,
        "cook_assets",
        {
            "stage": "CookAssets",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "source_asset_manifest": None,
            "project_manifest": None,
            "generated_from_project": False,
            "project_default_scene": None,
            "cooked_asset_manifest": str(cooked_manifest),
            "cooked_asset_manifest_sha256": hashlib.sha256(
                cooked_manifest.read_bytes()
            ).hexdigest(),
            "asset_count": 0,
            "root_count": 0,
            "asset_filter": None,
        },
    )
    _write_report(
        out,
        "pack",
        {
            "stage": "Pack",
            "profile": profile,
            "fatal": False,
            "diagnostics": [],
            "asset_manifest": str(out / "stages" / "cook_assets" / "assets.json"),
            "pack": str(pack),
            "stage_output": str(out / "stages" / "pack"),
            "trim_report": {
                "included_assets": [],
                "trimmed_assets": [],
                "missing_dependencies": [],
                "duplicate_assets": [],
                "diagnostics": [],
            },
            "manifest": {
                "pack": {
                    "version": 1,
                    "chunks": [],
                    "total_size": 0,
                },
                "assets": [],
            },
            "asset_count": 0,
            "chunk_count": 0,
            "deduplicated_assets": [],
            "deterministic_double_run": False,
            "delta_pack": str(delta_pack) if with_delta else None,
            "delta_manifest": empty_delta_manifest() if with_delta else None,
            "delta_asset_count": 0,
            "delta_chunk_count": 0,
            "delta_removed_assets": [],
            "delta_reused_assets": [],
            "delta_apply_verified": True if with_delta else None,
        },
    )
    platform_report = {
        "stage": "PlatformBundle",
        "profile": profile,
        "fatal": False,
        "diagnostics": [],
        "template_resolution": None,
        "template": template_report,
        "host_executable": str(platform_host),
        "host_source": str(host),
        "host_source_origin": "compile_host_report",
        "pack": str(platform_pack),
        "pack_source": str(pack),
        "pack_source_origin": "pack_report",
        "delta_pack": str(platform_delta) if with_delta else None,
        "delta_pack_source": str(delta_pack) if with_delta else None,
        "delta_pack_source_origin": "pack_report" if with_delta else None,
        "native_plugins": str(native_plugins),
        "native_plugins_payload": native_payload,
        "template_files": template_files,
        "bundle_manifest": str(bundle_manifest),
    }
    if include_bundle:
        platform_report["bundle"] = str(bundle_dir)
    bundle_manifest.parent.mkdir(parents=True, exist_ok=True)
    bundle_manifest.write_text(
        json.dumps(
            {
                "profile": platform_report["profile"],
                "template_resolution": platform_report["template_resolution"],
                "template": platform_report["template"],
                "host_executable": platform_report.get("host_executable"),
                "host_source": platform_report.get("host_source"),
                "host_source_origin": platform_report.get("host_source_origin"),
                "pack": platform_report.get("pack"),
                "pack_source": platform_report.get("pack_source"),
                "pack_source_origin": platform_report.get("pack_source_origin"),
                "delta_pack": platform_report.get("delta_pack"),
                "delta_pack_source": platform_report.get("delta_pack_source"),
                "delta_pack_source_origin": platform_report.get(
                    "delta_pack_source_origin"
                ),
                "native_plugins": platform_report.get("native_plugins"),
                "native_plugins_payload": platform_report.get("native_plugins_payload"),
                "template_files": platform_report.get("template_files"),
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    _write_report(out, "platform_bundle", platform_report)
    return {
        "native_plugins": native_plugins,
        "template_file": template_file,
        "host_source": host,
        "pack_source": pack,
        "delta_source": delta_pack,
        "platform_host": platform_host,
        "platform_pack": platform_pack,
        "platform_delta": platform_delta,
        "bundle_manifest": bundle_manifest,
    }


def _write_native_plugins(native_plugins: Path) -> None:
    _write_text(
        native_plugins / "native_plugins.toml",
        '[[plugins]]\nid = "animation"\n',
    )
    _write_text(
        native_plugins / "animation" / "native" / "zircon_plugin_animation.dll",
        "plugin dll",
    )
    _write_native_dynamic_package_report(native_plugins / "animation")


def _native_plugins_payload(native_plugins: Path) -> dict[str, object]:
    file_manifest = _native_plugins_file_manifest(native_plugins)
    materialized_package = {
        "package_id": "animation",
        "destination": str(native_plugins / "animation"),
        "package_report": str(
            native_plugins / "animation" / "native_dynamic_package.toml"
        ),
        "loadable_artifact_count": 1,
        "loadable_artifacts": [
            "plugins/animation/native/zircon_plugin_animation.dll"
        ],
    }
    return {
        "stage_report": None,
        "source": str(native_plugins),
        "bundle_path": str(native_plugins),
        "loader_manifest": str(native_plugins / "native_plugins.toml"),
        "content_hash": _native_plugins_content_hash(file_manifest),
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": 1,
        "native_signing": _native_dynamic_operation_audit_summary_report(),
        "native_notarization": _native_dynamic_operation_audit_summary_report(),
        "materialized_packages": [materialized_package],
    }


def _native_plugins_file_manifest(native_plugins: Path) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for file_path in sorted(native_plugins.rglob("*")):
        if not file_path.is_file():
            continue
        payload = file_path.read_bytes()
        entries.append(
            {
                "path": f"plugins/{file_path.relative_to(native_plugins).as_posix()}",
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(entries, key=lambda entry: str(entry["path"]))


def _native_plugins_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def _native_plugin_package_payload_file_manifest(
    package_dir: Path,
) -> list[dict[str, object]]:
    entries: list[dict[str, object]] = []
    for file_path in sorted(package_dir.rglob("*")):
        if not file_path.is_file() or file_path.name == "native_dynamic_package.toml":
            continue
        payload = file_path.read_bytes()
        entries.append(
            {
                "path": file_path.relative_to(package_dir).as_posix(),
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(entries, key=lambda entry: str(entry["path"]))


def _write_native_dynamic_package_report(package_dir: Path) -> None:
    payload_files = _native_plugin_package_payload_file_manifest(package_dir)
    _write_text(
        package_dir / "native_dynamic_package.toml",
        _native_dynamic_package_report_toml(payload_files),
    )


def _native_dynamic_package_report_toml(
    payload_files: list[dict[str, object]],
) -> str:
    lines = [
        "# Generated by Zircon export. Native dynamic package report.",
        "format_version = 1",
        'package_id = "animation"',
        'directory = "animation"',
        'path = "plugins/animation"',
        'manifest = "plugins/animation/plugin.toml"',
        "",
        "[abi]",
        "abi_version = 3",
        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
        'descriptor_contract = "NativePluginAbiV3"',
        'runtime_entry_source = "NativePluginAbiV3.runtime_entry_name"',
        'editor_entry_source = "NativePluginAbiV3.editor_entry_name"',
        'host_function_table = "NativePluginHostFunctionTableV3"',
        'entry_report_contract = "NativePluginEntryReportV3"',
        'behavior_contract = "NativePluginBehaviorV3"',
        'state_snapshot_contract = "NativePluginBehaviorV3.save_state/restore_state"',
        'bridge_method_table = "NativePluginBridgeMethodTableV3"',
        "",
        "[payload]",
        f"file_count = {len(payload_files)}",
        f'content_hash = "{_native_plugins_content_hash(payload_files)}"',
    ]
    for entry in payload_files:
        lines.extend(
            [
                "",
                "[[payload.files]]",
                f'path = "{entry["path"]}"',
                f'bytes = {entry["bytes"]}',
                f'sha256 = "{entry["sha256"]}"',
            ]
        )
    return "\n".join(lines) + "\n"


def _write_report(out: Path, stage: str, report: dict[str, object]) -> None:
    report_dir = out / "stages" / stage
    report_dir.mkdir(parents=True, exist_ok=True)
    (report_dir / "report.json").write_text(
        json.dumps(report, indent=2),
        encoding="utf-8",
    )


def _read_stage_report(out: Path, stage: str) -> dict[str, object]:
    return json.loads(
        (out / "stages" / stage / "report.json").read_text(encoding="utf-8")
    )


def _write_stage_report(out: Path, stage: str, report: dict[str, object]) -> None:
    _write_report(out, stage, report)


def _write_bundle_manifest_from_platform_report(
    bundle_manifest: Path,
    platform_report: dict[str, object],
) -> None:
    bundle_manifest.write_text(
        json.dumps(
            {
                "profile": platform_report["profile"],
                "template_resolution": platform_report["template_resolution"],
                "template": platform_report["template"],
                "host_executable": platform_report.get("host_executable"),
                "host_source": platform_report.get("host_source"),
                "host_source_origin": platform_report.get("host_source_origin"),
                "pack": platform_report.get("pack"),
                "pack_source": platform_report.get("pack_source"),
                "pack_source_origin": platform_report.get("pack_source_origin"),
                "delta_pack": platform_report.get("delta_pack"),
                "delta_pack_source": platform_report.get("delta_pack_source"),
                "delta_pack_source_origin": platform_report.get(
                    "delta_pack_source_origin"
                ),
                "native_plugins": platform_report.get("native_plugins"),
                "native_plugins_payload": platform_report.get("native_plugins_payload"),
                "template_files": platform_report.get("template_files"),
            },
            indent=2,
        ),
        encoding="utf-8",
    )


def _write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _copy_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(source.read_bytes())


def _remove_tree(path: Path) -> None:
    for child in sorted(path.rglob("*"), reverse=True):
        if child.is_file():
            child.unlink()
        elif child.is_dir():
            child.rmdir()
    path.rmdir()
