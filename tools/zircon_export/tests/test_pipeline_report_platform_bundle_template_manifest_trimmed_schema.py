from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleTemplateManifestTrimmedSchemaTests(unittest.TestCase):
    def test_report_rejects_template_report_manifest_padded_scalar_field(
        self,
    ) -> None:
        fields = (
            "template_id",
            "engine_version",
            "target_platform",
            "host_kind",
            "host_artifact",
            "resource_strategy",
            "plugin_strategy",
            "bundle_format",
            "content_hash",
        )
        for field in fields:
            with self.subTest(field=field):
                report = self._build_report_for_manifest_replacement(
                    field,
                    lambda template, _template_file, field=field: (
                        f'" {template[field]} "\n'
                    ),
                )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertEqual(report["missing_stages"], [])
                self.assertTrue(
                    any(
                        "PlatformBundle report template.manifest field "
                        f"{field} must be a non-empty trimmed string"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_report_rejects_template_report_manifest_padded_compatible_profile_entry(
        self,
    ) -> None:
        report = self._build_report_for_manifest_replacement(
            "compatible_profiles",
            lambda _template, _template_file: (
                'compatible_profiles = [" windows-release "]\n'
            ),
        )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertEqual(report["missing_stages"], [])
        self.assertTrue(
            any(
                "PlatformBundle report template.manifest field "
                "compatible_profiles[0] must be a non-empty trimmed string"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_report_rejects_template_report_manifest_padded_path_field(self) -> None:
        cases = (
            (
                "paths",
                lambda _template, _template_file: (
                    '\n[paths]\nhost_executable = " Info.plist "\n'
                ),
                "template.manifest field paths.host_executable "
                "must be a non-empty trimmed string",
            ),
            (
                "bundle",
                lambda _template, _template_file: '\n[bundle]\nroot = " . "\n',
                "template.manifest field bundle.root "
                "must be a non-empty trimmed string",
            ),
            (
                "bundle",
                lambda _template, _template_file: (
                    '\n[bundle]\nmanifest_path = " bundle.json "\n'
                ),
                "template.manifest field bundle.manifest_path "
                "must be a non-empty trimmed string",
            ),
            (
                "bundle",
                lambda _template, _template_file: (
                    '\n[bundle]\nhost_path = " zircon_runtime.exe "\n'
                ),
                "template.manifest field bundle.host_path "
                "must be a non-empty trimmed string",
            ),
            (
                "bundle",
                lambda _template, _template_file: (
                    '\n[bundle]\npack_path = " assets.zrpack "\n'
                ),
                "template.manifest field bundle.pack_path "
                "must be a non-empty trimmed string",
            ),
            (
                "bundle",
                lambda _template, _template_file: (
                    '\n[bundle]\ndelta_pack_path = " assets.delta.zrpack "\n'
                ),
                "template.manifest field bundle.delta_pack_path "
                "must be a non-empty trimmed string",
            ),
            (
                "files",
                lambda _template, template_file: _template_manifest_file_rows(
                    template_file,
                    path=f' {template_file["path"]} ',
                ),
                "template.manifest [[files]] entry 0 path "
                "must be a non-empty trimmed string",
            ),
            (
                "files",
                lambda _template, template_file: _template_manifest_file_rows(
                    template_file,
                    bundle_path=f' {template_file["bundle_path"]} ',
                ),
                "template.manifest file Info.plist bundle_path "
                "must be a non-empty trimmed string",
            ),
            (
                "files",
                lambda _template, template_file: _template_manifest_file_rows(
                    template_file,
                    purpose=f' {template_file["purpose"]} ',
                ),
                "template.manifest file Info.plist purpose "
                "must be a non-empty trimmed string",
            ),
        )
        for replace_field, replacement_factory, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                report = self._build_report_for_manifest_replacement(
                    replace_field,
                    replacement_factory,
                )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertEqual(report["missing_stages"], [])
                self.assertTrue(
                    any(
                        f"PlatformBundle report {expected_diagnostic}"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_report_rejects_template_report_manifest_padded_file_sha256(self) -> None:
        report = self._build_report_for_manifest_replacement(
            "files",
            lambda _template, template_file: _template_manifest_file_rows(
                template_file,
                sha256=f' {template_file["sha256"]} ',
            ),
        )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertEqual(report["missing_stages"], [])
        self.assertTrue(
            any(
                "PlatformBundle report template.manifest file Info.plist sha256 "
                "must be a non-empty trimmed string"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def _build_report_for_manifest_replacement(
        self,
        replace_field: str,
        replacement_factory,
    ) -> dict[str, object]:
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
            template_files = template["files"]
            self.assertIsInstance(template_files, list)
            template_file = template_files[0]
            self.assertIsInstance(template_file, dict)
            manifest = template_dir / "template.toml"
            replacement = replacement_factory(template, template_file)
            manifest_text = _template_manifest_text(
                template_file,
                replace_field=replace_field,
                replacement=replacement,
            )
            if replace_field != "content_hash":
                manifest_text = manifest_text.replace(
                    f'content_hash = "{"a" * 64}"',
                    f'content_hash = "{template["content_hash"]}"',
                )
            if replace_field != "paths":
                manifest_text = manifest_text.replace(
                    'host_executable = "zircon_runtime.exe"',
                    'host_executable = "Info.plist"',
                )
            manifest.write_text(manifest_text, encoding="utf-8")
            template["manifest"] = str(manifest)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            return build_pipeline_report(out, "windows-release")


def _template_manifest_text(
    template_file: dict[str, object],
    *,
    replace_field: str,
    replacement: str | None,
) -> str:
    fields = {
        "template_id": '"fixture-template"\n',
        "engine_version": '"0.1.0"\n',
        "target_platform": '"windows-x86_64"\n',
        "host_kind": '"desktop"\n',
        "host_artifact": '"precompiled"\n',
        "resource_strategy": '"filesystem_bundle"\n',
        "plugin_strategy": '"native_dynamic_allowed"\n',
        "bundle_format": '"directory"\n',
        "content_hash": f'"{"a" * 64}"\n',
    }
    compatible_profiles = 'compatible_profiles = ["windows-release"]\n'
    paths = '\n[paths]\nhost_executable = "zircon_runtime.exe"\n'
    bundle = ""
    files = _template_manifest_file_rows(template_file)
    if replace_field == "compatible_profiles":
        compatible_profiles = replacement or ""
    elif replace_field == "paths":
        paths = replacement or ""
    elif replace_field == "bundle":
        bundle = replacement or ""
    elif replace_field == "files":
        files = replacement or ""
    elif replacement is None:
        fields.pop(replace_field)
    else:
        fields[replace_field] = replacement
    lines = ["format_version = 1\n"]
    lines.extend(f"{field} = {value}" for field, value in fields.items())
    lines.extend(
        [
            compatible_profiles,
            paths,
            bundle,
            files,
        ]
    )
    return "".join(lines)


def _template_manifest_file_rows(
    template_file: dict[str, object],
    *,
    path: str | None = None,
    bundle_path: str | None = None,
    sha256: str | None = None,
    purpose: str | None = None,
) -> str:
    return (
        "\n[[files]]\n"
        f'path = "{path if path is not None else template_file["path"]}"\n'
        "bundle_path = "
        f'"{bundle_path if bundle_path is not None else template_file["bundle_path"]}"\n'
        f'sha256 = "{sha256 if sha256 is not None else template_file["sha256"]}"\n'
        f'purpose = "{purpose if purpose is not None else template_file["purpose"]}"\n'
    )
