from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.export_template_manifest import compute_template_content_hash
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


class PlatformBundleTemplateManifestIdentityTests(unittest.TestCase):
    def test_report_rejects_template_report_manifest_host_artifact_mismatch(
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
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(host_artifact="placeholder"),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["host_artifact"] = "precompiled"
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
                    "PlatformBundle report template.manifest host_artifact "
                    "placeholder does not match template.host_artifact "
                    "precompiled" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_engine_version_mismatch(self) -> None:
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
                _template_manifest_text(engine_version="9.9.9"),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["engine_version"] = "0.1.0"
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
                    "PlatformBundle report template.manifest engine_version "
                    "9.9.9 does not match template.engine_version 0.1.0" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_target_platform_mismatch(self) -> None:
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
                _template_manifest_text(target_platform="linux-x86_64"),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["target_platform"] = "windows-x86_64"
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
                    "PlatformBundle report template.manifest target_platform "
                    "linux-x86_64 does not match template.target_platform "
                    "windows-x86_64" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_strategy_field_mismatch(self) -> None:
        cases = (
            ("host_kind", "desktop", "headless"),
            ("resource_strategy", "filesystem_bundle", "browser_fetch"),
            (
                "plugin_strategy",
                "native_dynamic_allowed",
                "static_source_or_vm_only",
            ),
            ("bundle_format", "directory", "zip"),
        )
        for field, manifest_value, report_value in cases:
            with self.subTest(field=field):
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
                        _template_manifest_text(**{field: manifest_value}),
                        encoding="utf-8",
                    )
                    template["manifest"] = str(manifest)
                    template[field] = report_value
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
                            f"PlatformBundle report template.manifest {field} "
                            f"{manifest_value} does not match template.{field} "
                            f"{report_value}" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_content_hash_mismatch(self) -> None:
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
            computed_hash = compute_template_content_hash(
                [
                    {
                        "path": str(entry["path"]),
                        "bundle_path": str(entry.get("bundle_path", "")),
                        "sha256": str(entry["sha256"]),
                    }
                    for entry in files
                    if isinstance(entry, dict)
                ]
            )
            template["content_hash"] = computed_hash
            template["computed_content_hash"] = computed_hash
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(content_hash="0" * 64),
                encoding="utf-8",
            )
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
                    "PlatformBundle report template.manifest content_hash "
                    + ("0" * 64)
                    + " does not match template.content_hash "
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_compatible_profiles_mismatch(
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
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(
                    extra='compatible_profiles = ["windows-release"]\n',
                ),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["profile"] = "windows-release"
            template["compatible_profiles"] = ["windows-release", "qa-profile"]
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
                    "PlatformBundle report template.manifest compatible_profiles "
                    "does not match template.compatible_profiles" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_host_executable_mismatch(
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
            embedded_host = template_dir / str(host_file["path"])
            alternate_host = template_dir / "alt-host.exe"
            alternate_host.write_text("alternate host", encoding="utf-8")
            files.append(
                {
                    "path": alternate_host.name,
                    "bundle_path": alternate_host.name,
                    "sha256": hashlib.sha256(alternate_host.read_bytes()).hexdigest(),
                    "purpose": "host_executable",
                }
            )
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(
                    extra='[paths]\nhost_executable = "alt-host.exe"\n',
                ),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["host_executable"] = str(embedded_host)
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
                    "paths.host_executable alt-host.exe does not match "
                    "template.host_executable" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_bundle_field_mismatch(
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
            manifest = template_dir / "template.toml"
            manifest.write_text(
                _template_manifest_text(
                    extra='[bundle]\nhost_path = "bin/ZirconRuntime.exe"\n',
                ),
                encoding="utf-8",
            )
            template["manifest"] = str(manifest)
            template["bundle"] = {
                "delta_pack_path": "",
                "host_path": "ZirconRuntime.exe",
                "manifest_path": "bundle.json",
                "pack_path": "",
                "root": ".",
            }
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
                    "PlatformBundle report template.manifest bundle.host_path "
                    "bin/ZirconRuntime.exe does not match "
                    "template.bundle.host_path ZirconRuntime.exe" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
