from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _remove_tree,
    _write_bundle_manifest_from_platform_report,
    _write_native_plugins,
    _write_platform_bundle_fixture,
    _write_stage_report,
    _write_text,
    _write_validate_report_with_strategies,
)


class PlatformBundleReportValidationTests(unittest.TestCase):
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
