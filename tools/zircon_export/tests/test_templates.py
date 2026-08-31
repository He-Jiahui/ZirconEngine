from __future__ import annotations

import hashlib
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from tools.zircon_export.export_template import validate_export_template
from tools.zircon_export.tests.export_test_support import (
    LINUX_TEMPLATE,
    MACOS_TEMPLATE,
    TEMPLATE_ROOT,
    VALID_TEMPLATE,
    _file_sha256,
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    _template_content_hash,
    json_dumps,
    json_loads,
)


class ExportTemplateValidationTests(unittest.TestCase):
    def test_template_rejects_unknown_manifest_fields(self) -> None:
        cases = (
            (
                lambda text: text.replace(
                    "\n[paths]\n",
                    '\nunsigned_sidecar = "sidecar.bin"\n\n[paths]\n',
                ),
                "template.toml unknown field unsigned_sidecar",
            ),
            (
                lambda text: text.replace(
                    'host_executable = "bin/zircon_runtime.host-placeholder"\n',
                    'host_executable = "bin/zircon_runtime.host-placeholder"\n'
                    'future_path = "sidecar.bin"\n',
                ),
                "template.toml paths unknown field future_path",
            ),
            (
                lambda text: text.replace(
                    'delta_pack_path = "patches/assets.delta.zrpd"\n',
                    'delta_pack_path = "patches/assets.delta.zrpd"\n'
                    'future_path = "sidecar.bin"\n',
                ),
                "template.toml bundle unknown field future_path",
            ),
            (
                lambda text: text.replace(
                    'sha256 = "63a26218c731a8b79da125da1e59a6a4e67ac2212ce6a2ee3f3016dde237dd97"\n',
                    'sha256 = "63a26218c731a8b79da125da1e59a6a4e67ac2212ce6a2ee3f3016dde237dd97"\n'
                    'future_field = "sidecar.bin"\n',
                ),
                "template.toml [[files]] entry 0 unknown field future_field",
            ),
        )
        for mutate_manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        mutate_manifest(manifest.read_text(encoding="utf-8")),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        expected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_blank_compatible_profile_entries(self) -> None:
        cases = (
            'compatible_profiles = ["windows-release", ""]',
            'compatible_profiles = ["windows-release", "   "]',
        )
        for compatible_profiles in cases:
            with self.subTest(compatible_profiles=compatible_profiles):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        manifest.read_text(encoding="utf-8").replace(
                            'compatible_profiles = ["windows-release"]',
                            compatible_profiles,
                        ),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        "template.toml field compatible_profiles must not contain blank entries"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_non_string_compatible_profile_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'compatible_profiles = ["windows-release"]',
                    'compatible_profiles = [1, "windows-release"]',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertIn(
            "template.toml field compatible_profiles[0] must be a string",
            report["diagnostics"],
        )
        self.assertNotIn(
            "template.toml field compatible_profiles must be a string array",
            report["diagnostics"],
        )
        self.assertFalse(
            any(
                "template compatible_profiles does not include requested profile"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_padded_compatible_profile_entries(self) -> None:
        cases = (
            (
                'compatible_profiles = [" windows-release "]',
                "template.toml field compatible_profiles[0] "
                "must be a non-empty trimmed string",
            ),
            (
                'compatible_profiles = ["windows-release", " linux-release "]',
                "template.toml field compatible_profiles[1] "
                "must be a non-empty trimmed string",
            ),
        )
        for compatible_profiles, expected_diagnostic in cases:
            with self.subTest(compatible_profiles=compatible_profiles):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        manifest.read_text(encoding="utf-8").replace(
                            'compatible_profiles = ["windows-release"]',
                            compatible_profiles,
                        ),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        expected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )
                self.assertFalse(
                    any(
                        "template compatible_profiles does not include requested profile"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_padded_top_level_string_fields(self) -> None:
        cases = {
            "template_id": "windows-x86_64-library_embed-debug",
            "engine_version": "0.1.0",
            "target_platform": "windows-x86_64",
            "host_kind": "desktop",
            "host_artifact": "placeholder",
            "resource_strategy": "filesystem_bundle",
            "plugin_strategy": "native_dynamic_allowed",
            "bundle_format": "directory",
            "content_hash": (
                "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"
            ),
        }
        for field, value in cases.items():
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        manifest.read_text(encoding="utf-8").replace(
                            f'{field} = "{value}"',
                            f'{field} = " {value} "',
                        ),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        f"template.toml field {field} must be a non-empty trimmed string"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_padded_top_level_string_before_allowed_value_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'host_kind = "desktop"',
                    'host_kind = " desktopx "',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "template.toml field host_kind must be a non-empty trimmed string"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )
        self.assertFalse(
            any(
                "template.toml field host_kind='desktopx' is not one of"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_duplicate_compatible_profile_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'compatible_profiles = ["windows-release"]',
                    'compatible_profiles = ["windows-release", "windows-release"]',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "template.toml field compatible_profiles duplicate entry "
                "windows-release"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_blank_bundle_path_fields(self) -> None:
        for field in (
            "root",
            "host_path",
            "pack_path",
            "delta_pack_path",
            "manifest_path",
        ):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    original_lines = manifest.read_text(encoding="utf-8").splitlines()
                    field_exists = any(
                        line.startswith(f"{field} = ") for line in original_lines
                    )
                    lines = []
                    replaced = False
                    for line in original_lines:
                        if line.startswith(f"{field} = "):
                            lines.append(f'{field} = "   "')
                            replaced = True
                        else:
                            lines.append(line)
                        if line == "[bundle]" and not field_exists and not replaced:
                            lines.append(f'{field} = "   "')
                            replaced = True
                    manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        f"template.toml field bundle.{field} must be a non-empty string"
                        in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_padded_path_fields(self) -> None:
        def replace_or_insert_bundle_field(text: str, field: str, value: str) -> str:
            original_lines = text.splitlines()
            field_exists = any(
                line.startswith(f"{field} = ") for line in original_lines
            )
            lines = []
            replaced = False
            for line in original_lines:
                if line.startswith(f"{field} = "):
                    lines.append(f'{field} = "{value}"')
                    replaced = True
                else:
                    lines.append(line)
                if line == "[bundle]" and not field_exists and not replaced:
                    lines.append(f'{field} = "{value}"')
                    replaced = True
            return "\n".join(lines) + "\n"

        cases = [
            (
                "paths.host_executable",
                lambda text: text.replace(
                    'host_executable = "bin/zircon_runtime.host-placeholder"',
                    'host_executable = " bin/zircon_runtime.host-placeholder "',
                ),
                "template.toml field paths.host_executable "
                "must be a non-empty trimmed string",
            ),
            (
                "[[files]].path",
                lambda text: text.replace(
                    'path = "bin/zircon_runtime.host-placeholder"',
                    'path = " bin/zircon_runtime.host-placeholder "',
                ),
                "template.toml [[files]] entry 0 path "
                "must be a non-empty trimmed string",
            ),
            (
                "[[files]].bundle_path",
                lambda text: text.replace(
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    'bundle_path = " bin/zircon_runtime.host-placeholder "\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                ),
                "template file bin/zircon_runtime.host-placeholder bundle_path "
                "must be a non-empty trimmed string",
            ),
        ]
        for field in (
            "root",
            "host_path",
            "pack_path",
            "delta_pack_path",
            "manifest_path",
        ):
            cases.append(
                (
                    f"bundle.{field}",
                    lambda text, field=field: replace_or_insert_bundle_field(
                        text,
                        field,
                        f" {field}.out ",
                    ),
                    f"template.toml field bundle.{field} "
                    "must be a non-empty trimmed string",
                )
            )

        for label, mutate_manifest, expected_diagnostic in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        mutate_manifest(manifest.read_text(encoding="utf-8")),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        expected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_rejects_duplicate_bundle_path_entries(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            extra_file = template_dir / "bin" / "alternate-host-placeholder"
            extra_file.write_text("alternate host", encoding="utf-8")
            host_hash = _file_sha256(
                template_dir / "bin" / "zircon_runtime.host-placeholder"
            )
            extra_hash = _file_sha256(extra_file)
            bundle_path = "bin/zircon_runtime.host-placeholder"
            hasher = hashlib.sha256()
            for path, declared_bundle_path, sha256 in (
                (
                    "bin/alternate-host-placeholder",
                    bundle_path,
                    extra_hash,
                ),
                (
                    "bin/zircon_runtime.host-placeholder",
                    bundle_path,
                    host_hash,
                ),
            ):
                hasher.update(path.encode("utf-8"))
                hasher.update(b"\0")
                hasher.update(declared_bundle_path.encode("utf-8"))
                hasher.update(b"\0")
                hasher.update(sha256.lower().encode("ascii"))
                hasher.update(b"\n")
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8")
                .replace(
                    'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
                    f'content_hash = "{hasher.hexdigest()}"',
                )
                .replace(
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    f'bundle_path = "{bundle_path}"\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                )
                + "\n[[files]]\n"
                'path = "bin/alternate-host-placeholder"\n'
                f'bundle_path = "{bundle_path}"\n'
                'purpose = "duplicate output path test"\n'
                f'sha256 = "{extra_hash}"\n',
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                f"template bundle path {bundle_path} is declared more than once"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_padded_bundle_path_before_bundle_path_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            extra_file = template_dir / "bin" / "alternate-host-placeholder"
            extra_file.write_text("alternate host", encoding="utf-8")
            extra_hash = _file_sha256(extra_file)
            duplicate_bundle_path = "bin/zircon_runtime.host-placeholder"
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                    'path = "bin/zircon_runtime.host-placeholder"\n'
                    'bundle_path = " bin/custom-host "\n'
                    'purpose = "M3-T1 placeholder host path for template contract validation"\n',
                )
                + "\n[[files]]\n"
                'path = "bin/alternate-host-placeholder"\n'
                f'bundle_path = "{duplicate_bundle_path}"\n'
                'purpose = "duplicate output path test"\n'
                f'sha256 = "{extra_hash}"\n',
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "template file bin/zircon_runtime.host-placeholder bundle_path "
                "must be a non-empty trimmed string"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )
        self.assertFalse(
            any(
                f"template bundle path {duplicate_bundle_path} is declared more than once"
                in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_invalid_file_purpose(self) -> None:
        cases = (
            ("purpose = 123", "template file bin/zircon_runtime.host-placeholder purpose must be a string"),
            ('purpose = "   "', "template file bin/zircon_runtime.host-placeholder purpose must be non-empty when present"),
            (
                'purpose = " M3-T1 placeholder host path for template contract validation "',
                "template file bin/zircon_runtime.host-placeholder purpose "
                "must be a non-empty trimmed string",
            ),
        )
        for replacement, expected_diagnostic in cases:
            with self.subTest(replacement=replacement):
                with tempfile.TemporaryDirectory() as temp_dir:
                    template_dir = Path(temp_dir) / "template"
                    shutil.copytree(VALID_TEMPLATE, template_dir)
                    manifest = template_dir / "template.toml"
                    manifest.write_text(
                        manifest.read_text(encoding="utf-8").replace(
                            'purpose = "M3-T1 placeholder host path for template contract validation"',
                            replacement,
                        ),
                        encoding="utf-8",
                    )

                    report = validate_export_template(
                        template_dir=template_dir,
                        expected_engine_version="0.1.0",
                        profile="windows-release",
                        expected_target_platform="windows-x86_64",
                    )

                self.assertTrue(report["fatal"], report["diagnostics"])
                self.assertTrue(
                    any(
                        expected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_template_version_mismatch_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    "format_version = 1",
                    "format_version = 999",
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("format_version 999" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

    def test_valid_template_resolves_declared_host(self) -> None:
        report = validate_export_template(
            template_dir=VALID_TEMPLATE,
            expected_engine_version="0.1.0",
            profile="windows-release",
            expected_target_platform="windows-x86_64",
        )

        self.assertFalse(report["fatal"], report["diagnostics"])
        self.assertEqual(report["format_version"], 1)
        self.assertEqual(report["target_platform"], "windows-x86_64")
        self.assertEqual(
            Path(report["host_executable"]),
            VALID_TEMPLATE / "bin" / "zircon_runtime.host-placeholder",
        )
        self.assertEqual(
            report["computed_content_hash"],
            _template_content_hash(
                "bin/zircon_runtime.host-placeholder",
                report["files"][0]["sha256"],
            ),
        )
        self.assertEqual(report["host_artifact"], "placeholder")

    def test_template_rejects_unknown_host_artifact_status(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.write_text(
                manifest.read_text(encoding="utf-8").replace(
                    'host_artifact = "placeholder"',
                    'host_artifact = "generated"',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"], report["diagnostics"])
        self.assertTrue(
            any(
                "template.toml field host_artifact='generated' is not one of "
                "placeholder, precompiled" in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )

    def test_template_rejects_declared_directory_file(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            host = template_dir / "bin" / "zircon_runtime.host-placeholder"
            host.unlink()
            host.mkdir()

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template file bin/zircon_runtime.host-placeholder is not a file"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_declared_file_read_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            unreadable_file = (
                template_dir / "bin" / "zircon_runtime.host-placeholder"
            ).resolve()
            original_open = Path.open

            def open_or_fail(path: Path, *args: object, **kwargs: object):
                if path.resolve() == unreadable_file:
                    raise OSError("simulated template read failure")
                return original_open(path, *args, **kwargs)

            with mock.patch.object(Path, "open", open_or_fail):
                report = validate_export_template(
                    template_dir=template_dir,
                    expected_engine_version="0.1.0",
                    profile="windows-release",
                    expected_target_platform="windows-x86_64",
                )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template file bin/zircon_runtime.host-placeholder could not be read"
                    in diagnostic
                    and "simulated template read failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_declared_file_path_resolve_error(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            failing_file = template_dir / "bin" / "zircon_runtime.host-placeholder"
            original_resolve = Path.resolve

            def resolve_or_fail(path: Path, *args: object, **kwargs: object) -> Path:
                if Path(path) == failing_file:
                    raise OSError("simulated template path resolve failure")
                return original_resolve(path, *args, **kwargs)

            with mock.patch.object(Path, "resolve", resolve_or_fail):
                report = validate_export_template(
                    template_dir=template_dir,
                    expected_engine_version="0.1.0",
                    profile="windows-release",
                    expected_target_platform="windows-x86_64",
                )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "template path bin/zircon_runtime.host-placeholder could not be resolved"
                    in diagnostic
                    and "simulated template path resolve failure" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_manifest_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            manifest.unlink()
            manifest.mkdir()

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    f"export template manifest {manifest} is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_template_rejects_aliasing_file_and_host_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            template_dir = Path(temp_dir) / "template"
            shutil.copytree(VALID_TEMPLATE, template_dir)
            manifest = template_dir / "template.toml"
            aliased_path = "bin/./zircon_runtime.host-placeholder"
            bundle_path = "bin/zircon_runtime.host-placeholder"
            aliased_hash = _template_content_hash(
                aliased_path,
                _file_sha256(template_dir / "bin" / "zircon_runtime.host-placeholder"),
                bundle_path=bundle_path,
            )
            manifest.write_text(
                manifest.read_text(encoding="utf-8")
                .replace(
                    'host_executable = "bin/zircon_runtime.host-placeholder"',
                    f'host_executable = "{aliased_path}"',
                )
                .replace(
                    'path = "bin/zircon_runtime.host-placeholder"',
                    f'path = "{aliased_path}"\nbundle_path = "{bundle_path}"',
                )
                .replace(
                    'content_hash = "e5acc99c1ccc705e08793501ff1226adcc8e181c6d1d9ffbff7cef2270a99304"',
                    f'content_hash = "{aliased_hash}"',
                ),
                encoding="utf-8",
            )

            report = validate_export_template(
                template_dir=template_dir,
                expected_engine_version="0.1.0",
                profile="windows-release",
                expected_target_platform="windows-x86_64",
            )

        self.assertTrue(report["fatal"])
        self.assertTrue(
            any("paths.host_executable must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )
        self.assertTrue(
            any("[[files]] entry 0 path must be a safe relative path" in diagnostic for diagnostic in report["diagnostics"]),
            report["diagnostics"],
        )

if __name__ == "__main__":
    unittest.main()
