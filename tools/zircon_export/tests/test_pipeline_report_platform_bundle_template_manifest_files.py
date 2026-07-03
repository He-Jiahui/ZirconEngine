from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_template_report_helpers import (
    _template_manifest_text,
)
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateManifestFileTests(unittest.TestCase):
    def test_report_rejects_template_report_host_executable_not_declared_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            unlisted_host = template_dir / "unlisted-host.exe"
            unlisted_host.write_text("host", encoding="utf-8")
            template["host_executable"] = str(unlisted_host)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.host_executable "
                    "must be listed in template.files[].path" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_host_executable_missing_file(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            files = template["files"]
            self.assertIsInstance(files, list)
            host_file = files[0]
            self.assertIsInstance(host_file, dict)
            host_executable = template_dir / str(host_file["path"])
            template["host_executable"] = str(host_executable)
            self.assertTrue(host_executable.exists())
            host_executable.unlink()
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.host_executable "
                    in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_path_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            wrong_manifest = template_dir / "other.toml"
            wrong_manifest.write_text("format_version = 1\n", encoding="utf-8")
            template["manifest"] = str(wrong_manifest)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.manifest "
                    "does not match template_dir/template.toml" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_missing_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            manifest = template_dir / "template.toml"
            template["manifest"] = str(manifest)
            manifest.unlink()
            self.assertFalse(manifest.exists())
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.manifest "
                    in diagnostic
                    and "does not exist" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_invalid_toml(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            manifest = template_dir / "template.toml"
            manifest.write_text("format_version = [\n", encoding="utf-8")
            template["manifest"] = str(manifest)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.manifest "
                    in diagnostic
                    and "is not valid TOML" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_format_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            manifest = template_dir / "template.toml"
            manifest.write_text("format_version = 999\n", encoding="utf-8")
            template["manifest"] = str(manifest)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.manifest format_version 999 "
                    "is not supported; expected 1" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_template_id_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template_dir = Path(str(template["template_dir"]))
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(template_id="other-template"),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["template_id"] = "embedded-template"
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report template.manifest template_id "
                    "other-template does not match template.template_id "
                    "embedded-template" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
