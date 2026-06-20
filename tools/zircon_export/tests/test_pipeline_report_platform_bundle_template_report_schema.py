from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.export_template import compute_template_content_hash
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateReportSchemaTests(unittest.TestCase):
    def test_report_rejects_template_report_missing_success_evidence_field(
        self,
    ) -> None:
        cases = (
            ("bundle_format", "must be a string"),
            ("compatible_profiles", "must be a string array"),
            ("computed_content_hash", "must be a string"),
            ("content_hash", "must be a string"),
            ("diagnostics", "must be a string array"),
            ("engine_version", "must be a string"),
            ("expected_engine_version", "must be a string"),
            ("expected_format_version", "must be an integer"),
            ("expected_target_platform", "must be a string"),
            ("fatal", "must be a boolean"),
            ("files", "must be an object array"),
            ("format_version", "must be an integer"),
            ("host_executable", "must be a string"),
            ("host_kind", "must be a string"),
            ("manifest", "must be a string"),
            ("plugin_strategy", "must be a string"),
            ("profile", "must be a string"),
            ("resource_strategy", "must be a string"),
            ("target_platform", "must be a string"),
            ("template_dir", "must be a string"),
            ("template_id", "must be a string"),
        )
        for field, expected_type in cases:
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
                    template.pop(field, None)
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
                            f"PlatformBundle report template.{field} {expected_type}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_fatal_without_diagnostics(self) -> None:
        for diagnostics in ([], None):
            with self.subTest(diagnostics=diagnostics):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    template = platform_report["template"]
                    self.assertIsInstance(template, dict)
                    template["fatal"] = True
                    if diagnostics is None:
                        template.pop("diagnostics", None)
                    else:
                        template["diagnostics"] = diagnostics
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
                            "PlatformBundle report template fatal report "
                            "must include diagnostics" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_non_fatal_with_diagnostics(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["fatal"] = False
            template["diagnostics"] = ["forced warning"]
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
                    "PlatformBundle report template non-fatal report "
                    "must not include diagnostics" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

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
                self._template_manifest_text(template_id="other-template"),
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
                self._template_manifest_text(engine_version="9.9.9"),
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
                self._template_manifest_text(target_platform="linux-x86_64"),
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
                        self._template_manifest_text(**{field: manifest_value}),
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
                self._template_manifest_text(content_hash="0" * 64),
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
                self._template_manifest_text(
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
                self._template_manifest_text(
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
                self._template_manifest_text(
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

    @staticmethod
    def _template_manifest_text(extra: str = "", **overrides: str) -> str:
        template_file_hash = hashlib.sha256(b"<plist>zircon</plist>").hexdigest()
        content_hash = compute_template_content_hash(
            [
                {
                    "path": "Info.plist",
                    "bundle_path": "Contents/Info.plist",
                    "sha256": template_file_hash,
                }
            ]
        )
        fields = {
            "template_id": "fixture-template",
            "engine_version": "0.1.0",
            "target_platform": "windows-x86_64",
            "host_kind": "desktop",
            "resource_strategy": "filesystem_bundle",
            "plugin_strategy": "native_dynamic_allowed",
            "bundle_format": "directory",
            "content_hash": content_hash,
        }
        fields.update(overrides)
        lines = ["format_version = 1\n"]
        lines.extend(f'{field} = "{value}"\n' for field, value in fields.items())
        if "compatible_profiles" not in extra:
            lines.append('compatible_profiles = ["windows-release"]\n')
        if extra:
            lines.append(extra)
        if "[paths]" not in extra:
            lines.extend(
                [
                    "\n[paths]\n",
                    'host_executable = "Info.plist"\n',
                ]
            )
        if "[[files]]" not in extra:
            lines.extend(
                [
                    "\n[[files]]\n",
                    'path = "Info.plist"\n',
                    'bundle_path = "Contents/Info.plist"\n',
                    'purpose = "platform_metadata"\n',
                    f'sha256 = "{template_file_hash}"\n',
                ]
            )
        return "".join(lines)

    def test_report_rejects_template_report_missing_profile_membership(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["profile"] = "windows-release"
            template["compatible_profiles"] = ["linux-release"]
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
                    "PlatformBundle report template.compatible_profiles "
                    "does not include profile windows-release" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_duplicate_compatible_profile_entry(
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
            template["compatible_profiles"] = ["windows-release", "windows-release"]
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
                    "PlatformBundle report template.compatible_profiles "
                    "duplicate entry windows-release" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_enum_field_unknown_value(self) -> None:
        cases = (
            (
                "bundle_format",
                "unknown_format",
                "app_bundle, directory, web_static, zip",
            ),
            (
                "host_kind",
                "unknown_host",
                "browser, desktop, headless, mobile_app",
            ),
            (
                "plugin_strategy",
                "unknown_plugin_strategy",
                "native_dynamic_allowed, static_source_or_vm_only",
            ),
            (
                "resource_strategy",
                "unknown_resource_strategy",
                "browser_fetch, filesystem_bundle, mobile_asset_bundle",
            ),
        )
        for field, value, expected_values in cases:
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
                    template[field] = value
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
                            f"PlatformBundle report template.{field}={value!r} "
                            f"is not one of {expected_values}" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_engine_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["engine_version"] = "0.1.0"
            template["expected_engine_version"] = "9.9.9"
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
                    "PlatformBundle report template.engine_version 0.1.0 "
                    "does not match expected_engine_version 9.9.9" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_target_platform_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=True,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["target_platform"] = "windows-x86_64"
            template["expected_target_platform"] = "linux-x86_64"
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
                    "PlatformBundle report template.target_platform windows-x86_64 "
                    "does not match expected_target_platform linux-x86_64" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_content_hash_mismatch(self) -> None:
        for field in ("computed_content_hash", "content_hash"):
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
                    template[field] = "0" * 64
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
                            f"PlatformBundle report template.{field} "
                            "does not match computed content hash"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_hash_field_malformed(self) -> None:
        for field in ("computed_content_hash", "content_hash"):
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
                    template[field] = "not-a-hash"
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
                            f"PlatformBundle report template.{field} must be a SHA-256 hex digest"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_format_version_mismatch(self) -> None:
        for field in ("expected_format_version", "format_version"):
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
                    template[field] = 999
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
                            f"PlatformBundle report template.{field} must be 1"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_string_field_blank(self) -> None:
        for field in (
            "bundle_format",
            "computed_content_hash",
            "content_hash",
            "engine_version",
            "expected_engine_version",
            "expected_target_platform",
            "host_executable",
            "host_kind",
            "manifest",
            "plugin_strategy",
            "profile",
            "resource_strategy",
            "target_platform",
            "template_dir",
            "template_id",
        ):
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
                    template[field] = " "
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
                            f"PlatformBundle report template.{field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
