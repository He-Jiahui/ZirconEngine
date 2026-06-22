from __future__ import annotations

import hashlib
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


class PlatformBundleTemplateManifestSchemaTests(unittest.TestCase):
    def test_report_rejects_template_report_manifest_unknown_fields(self) -> None:
        cases = (
            ("extra_root = true\n", "template.manifest unknown field extra_root"),
            (
                "[paths]\nextra_path = \"host.exe\"\n",
                "template.manifest paths unknown field extra_path",
            ),
            (
                "[bundle]\nextra_bundle = \"assets\"\n",
                "template.manifest bundle unknown field extra_bundle",
            ),
            (
                "[[files]]\nextra_file = \"ignored\"\n",
                "template.manifest files[0] unknown field extra_file",
            ),
        )
        for extra_manifest_text, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                        "format_version = 1\n" + extra_manifest_text,
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_shape_mismatch(self) -> None:
        cases = (
            (
                'paths = "host.exe"\n',
                "template.manifest table [paths] is required",
            ),
            (
                'bundle = "app"\n',
                "template.manifest table [bundle] must be a table when present",
            ),
            (
                'files = "Info.plist"\n',
                "template.manifest [[files]] entries must form an array",
            ),
            (
                'files = ["Info.plist"]\n',
                "template.manifest [[files]] entry 0 must be a table",
            ),
        )
        for manifest_text, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                        "format_version = 1\n" + manifest_text,
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_scalar_field_schema(self) -> None:
        cases = (
            (
                "template_id",
                None,
                "template.manifest field template_id must be a non-empty string",
            ),
            (
                "engine_version",
                '" "\n',
                "template.manifest field engine_version must be a non-empty string",
            ),
            (
                "target_platform",
                '["windows-x86_64"]\n',
                "template.manifest field target_platform must be a non-empty string",
            ),
            (
                "host_kind",
                '"service"\n',
                "template.manifest field host_kind='service' is not one of "
                "browser, desktop, headless, mobile_app",
            ),
            (
                "host_artifact",
                '"generated"\n',
                "template.manifest field host_artifact='generated' is not one of "
                "placeholder, precompiled",
            ),
            (
                "resource_strategy",
                '"unknown_resource"\n',
                "template.manifest field resource_strategy='unknown_resource' "
                "is not one of browser_fetch, filesystem_bundle, mobile_asset_bundle",
            ),
            (
                "plugin_strategy",
                '"unknown_plugin"\n',
                "template.manifest field plugin_strategy='unknown_plugin' "
                "is not one of native_dynamic_allowed, static_source_or_vm_only",
            ),
            (
                "bundle_format",
                '"msi"\n',
                "template.manifest field bundle_format='msi' is not one of "
                "app_bundle, directory, web_static, zip",
            ),
            (
                "content_hash",
                '"not-a-hash"\n',
                "template.manifest field content_hash must be a SHA-256 hex digest",
            ),
        )
        for field, value, expected_diagnostic in cases:
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
                    template_files = template["files"]
                    self.assertIsInstance(template_files, list)
                    template_file = template_files[0]
                    self.assertIsInstance(template_file, dict)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        self._template_manifest_text(
                            template_file,
                            replace_field=field,
                            replacement=value,
                        ),
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_compatible_profiles_schema(
        self,
    ) -> None:
        cases = (
            (
                'compatible_profiles = "windows-release"\n',
                "template.manifest field compatible_profiles must be a string array",
            ),
            (
                "compatible_profiles = [1]\n",
                "template.manifest field compatible_profiles[0] must be a string",
            ),
            (
                'compatible_profiles = ["windows-release", " "]\n',
                "template.manifest field compatible_profiles "
                "must not contain blank entries",
            ),
            (
                'compatible_profiles = ["windows-release", "windows-release"]\n',
                "template.manifest field compatible_profiles "
                "duplicate entry windows-release",
            ),
        )
        for replacement, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                    manifest.write_text(
                        self._template_manifest_text(
                            template_file,
                            replace_field="compatible_profiles",
                            replacement=replacement,
                        ),
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_padded_duplicate_compatible_profile_before_uniqueness(
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
            template_files = template["files"]
            self.assertIsInstance(template_files, list)
            template_file = template_files[0]
            self.assertIsInstance(template_file, dict)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                self._template_manifest_text(
                    template_file,
                    replace_field="compatible_profiles",
                    replacement=(
                        'compatible_profiles = [" windows-release ", '
                        '" windows-release "]\n'
                    ),
                ),
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
                    "PlatformBundle report template.manifest field "
                    "compatible_profiles[0] must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report template.manifest field "
                    "compatible_profiles duplicate entry"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_manifest_paths_host_schema(self) -> None:
        cases = (
            (
                "",
                "template.manifest table [paths] is required",
            ),
            (
                "[paths]\n",
                "template.manifest field paths.host_executable "
                "must be a non-empty string",
            ),
            (
                '[paths]\nhost_executable = " "\n',
                "template.manifest field paths.host_executable "
                "must be a non-empty string",
            ),
            (
                '[paths]\nhost_executable = "../zircon_runtime.exe"\n',
                "template.manifest field paths.host_executable "
                "must be a safe relative path",
            ),
        )
        for paths_text, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                    manifest.write_text(
                        self._template_manifest_text(
                            template_file,
                            replace_field="paths",
                            replacement=paths_text,
                        ),
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_bundle_field_schema(self) -> None:
        cases = (
            (
                "[bundle]\nroot = 1\n",
                "template.manifest field bundle.root must be a string",
            ),
            (
                '[bundle]\nhost_path = ""\n',
                "template.manifest field bundle.host_path must be a non-empty string",
            ),
            (
                '[bundle]\npack_path = "../assets.zrpack"\n',
                "template.manifest field bundle.pack_path must be a safe relative path",
            ),
        )
        for bundle_text, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                    manifest.write_text(
                        self._template_manifest_text(
                            template_file,
                            replace_field="bundle",
                            replacement=bundle_text,
                        ),
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_file_row_schema(self) -> None:
        cases = (
            (
                '[[files]]\npath = ""\n',
                "template.manifest [[files]] entry 0 needs a non-empty path",
            ),
            (
                '[[files]]\npath = "../Info.plist"\n',
                "template.manifest [[files]] entry 0 path must be a safe relative path",
            ),
            (
                '[[files]]\npath = "Info.plist"\n',
                "template.manifest file Info.plist must declare a SHA-256 hex digest",
            ),
            (
                '[[files]]\npath = "Info.plist"\nsha256 = "not-a-hash"\n',
                "template.manifest file Info.plist must declare a SHA-256 hex digest",
            ),
            (
                (
                    '[[files]]\npath = "Info.plist"\n'
                    f'sha256 = "{"a" * 64}"\n'
                    'bundle_path = ""\n'
                ),
                "template.manifest file Info.plist has an invalid bundle_path",
            ),
            (
                (
                    '[[files]]\npath = "Info.plist"\n'
                    f'sha256 = "{"a" * 64}"\n'
                    'bundle_path = "../Info.plist"\n'
                ),
                "template.manifest file Info.plist bundle_path "
                "must be a safe relative path",
            ),
            (
                (
                    '[[files]]\npath = "Info.plist"\n'
                    f'sha256 = "{"a" * 64}"\n'
                    'purpose = 1\n'
                ),
                "template.manifest file Info.plist purpose must be a string",
            ),
            (
                (
                    '[[files]]\npath = "Info.plist"\n'
                    f'sha256 = "{"a" * 64}"\n'
                    'purpose = ""\n'
                ),
                "template.manifest file Info.plist purpose "
                "must be non-empty when present",
            ),
        )
        for files_text, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
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
                    manifest_text = self._template_manifest_text(
                        template_file,
                        replace_field="files",
                        replacement="",
                    )
                    if files_text:
                        manifest_text = manifest_text.replace(
                            "\n[paths]\n",
                            f"{files_text}\n[paths]\n",
                            1,
                        )
                    manifest.write_text(
                        manifest_text,
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_missing_file_rows(self) -> None:
        cases = (
            ("", "missing [[files]]"),
            ("files = []\n", "empty files array"),
        )
        for files_text, label in cases:
            with self.subTest(label=label):
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
                    manifest_text = self._template_manifest_text(
                        template_file,
                        replace_field="files",
                        replacement="",
                    )
                    if files_text:
                        manifest_text = manifest_text.replace(
                            "\n[paths]\n",
                            f"{files_text}\n[paths]\n",
                            1,
                        )
                    manifest.write_text(
                        manifest_text,
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
                            "PlatformBundle report template.manifest must declare "
                            "at least one [[files]] entry" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_template_report_manifest_file_row_duplicates(self) -> None:
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
            second_source = template_dir / "LaunchScreen.storyboard"
            second_source.write_text("launch", encoding="utf-8")
            second_sha256 = hashlib.sha256(second_source.read_bytes()).hexdigest()
            cases = (
                (
                    (
                        "\n[[files]]\n"
                        f'path = "{template_file["path"]}"\n'
                        f'bundle_path = "{template_file["bundle_path"]}"\n'
                        f'sha256 = "{template_file["sha256"]}"\n'
                        f'purpose = "{template_file["purpose"]}"\n'
                        "\n[[files]]\n"
                        f'path = "{template_file["path"]}"\n'
                        'bundle_path = "Contents/OtherInfo.plist"\n'
                        f'sha256 = "{template_file["sha256"]}"\n'
                        f'purpose = "{template_file["purpose"]}"\n'
                    ),
                    f'template.manifest template file {template_file["path"]} '
                    "is declared more than once",
                ),
                (
                    (
                        "\n[[files]]\n"
                        f'path = "{template_file["path"]}"\n'
                        f'bundle_path = "{template_file["bundle_path"]}"\n'
                        f'sha256 = "{template_file["sha256"]}"\n'
                        f'purpose = "{template_file["purpose"]}"\n'
                        "\n[[files]]\n"
                        f'path = "{second_source.name}"\n'
                        f'bundle_path = "{template_file["bundle_path"]}"\n'
                        f'sha256 = "{second_sha256}"\n'
                        'purpose = "platform_metadata"\n'
                    ),
                    (
                        "template.manifest template bundle path "
                        f'{template_file["bundle_path"]} '
                        "is declared more than once"
                    ),
                ),
            )
            for files_text, expected_diagnostic in cases:
                with self.subTest(expected_diagnostic=expected_diagnostic):
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        self._template_manifest_text(
                            template_file,
                            replace_field="files",
                            replacement=files_text,
                        ),
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
                            f"PlatformBundle report {expected_diagnostic}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    @staticmethod
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
        files = (
            "\n[[files]]\n"
            f'path = "{template_file["path"]}"\n'
            f'bundle_path = "{template_file["bundle_path"]}"\n'
            f'sha256 = "{template_file["sha256"]}"\n'
            f'purpose = "{template_file["purpose"]}"\n'
        )
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

if __name__ == "__main__":
    unittest.main()
