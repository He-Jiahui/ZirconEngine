from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from tools.zircon_export.native_dynamic_payload import (
    native_dynamic_content_hash,
    native_dynamic_package_payload_file_manifest,
)
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_platform_bundle_fixture,
)


class NativeDynamicPackageReportSchemaTests(unittest.TestCase):
    def _assert_package_report_diagnostic(
        self,
        lines: list[str],
        expected_diagnostic: str,
        unexpected_diagnostic: str | tuple[str, ...] | None = None,
    ) -> None:
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
            package_report.write_text(
                "\n".join(lines) + "\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            if unexpected_diagnostic is None:
                unexpected_diagnostics: tuple[str, ...] = ()
            elif isinstance(unexpected_diagnostic, str):
                unexpected_diagnostics = (unexpected_diagnostic,)
            else:
                unexpected_diagnostics = unexpected_diagnostic
            for unexpected in unexpected_diagnostics:
                self.assertFalse(
                    any(
                        "native_plugins_payload materialized_packages[0] package_report"
                        in diagnostic
                        and unexpected in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_package_report_unknown_top_level_field(
        self,
    ) -> None:
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
            package_report.write_text(
                "\n".join(
                    [
                        'package_id = "animation"',
                        'unsigned_sidecar = "plugins/animation/sidecar.bin"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_package_report_top_level_blank_strings(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            abi_lines = ["[abi]", "abi_version = 3"]
            for field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
                abi_lines.append(f'{field} = "{value}"')
            base_lines = [
                'format_version = 1',
                'package_id = "animation"',
                'directory = "animation"',
                'path = "plugins/animation"',
                'manifest = "plugins/animation/plugin.toml"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
                "",
                *abi_lines,
            ]

        replacements = {
            "package_id": 'package_id = "   "',
            "directory": 'directory = "   "',
            "path": 'path = "   "',
            "manifest": 'manifest = "   "',
        }
        for field, replacement in replacements.items():
            with self.subTest(field=field):
                lines = [
                    replacement if line.startswith(f"{field} = ") else line
                    for line in base_lines
                ]
                self._assert_package_report_diagnostic(
                    lines,
                    f"package_report.{field} must be a non-empty string",
                    "does not match",
                )

    def test_report_rejects_native_plugins_package_report_top_level_unsafe_relative_paths(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            abi_lines = ["[abi]", "abi_version = 3"]
            for field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
                abi_lines.append(f'{field} = "{value}"')
            base_lines = [
                'format_version = 1',
                'package_id = "animation"',
                'directory = "animation"',
                'path = "plugins/animation"',
                'manifest = "plugins/animation/plugin.toml"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
                "",
                *abi_lines,
            ]

        replacements = {
            "directory": 'directory = "../animation"',
            "path": 'path = "../plugins/animation"',
            "manifest": 'manifest = "/plugins/animation/plugin.toml"',
        }
        for field, replacement in replacements.items():
            with self.subTest(field=field):
                lines = [
                    replacement if line.startswith(f"{field} = ") else line
                    for line in base_lines
                ]
                self._assert_package_report_diagnostic(
                    lines,
                    f"package_report.{field} must be a safe relative path",
                    "does not match",
                )

    def test_report_rejects_native_plugins_package_report_payload_unknown_field(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            package_report.write_text(
                "\n".join(
                    [
                        'package_id = "animation"',
                        "",
                        "[payload]",
                        f"file_count = {len(file_manifest)}",
                        f'content_hash = "{content_hash}"',
                        'unsigned_sidecar = "plugins/animation/sidecar.bin"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "payload unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_package_report_abi_unknown_field(
        self,
    ) -> None:
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
            abi_lines = ["[abi]", "abi_version = 3"]
            for field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
                abi_lines.append(f'{field} = "{value}"')
            abi_lines.append('unsigned_sidecar = "plugins/animation/sidecar.bin"')
            package_report.write_text(
                "\n".join(['package_id = "animation"', "", *abi_lines]) + "\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "abi unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_package_report_format_version_non_integer(
        self,
    ) -> None:
        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                'format_version = "1"',
            ],
            "format_version must be an integer",
            "format_version 1 is not supported; expected 1",
        )

    def test_report_rejects_native_plugins_package_report_abi_string_field_types(
        self,
    ) -> None:
        field = next(iter(NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS))
        abi_lines = ["[abi]", "abi_version = 3"]
        for abi_field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
            if abi_field == field:
                abi_lines.append(f"{abi_field} = 42")
            else:
                abi_lines.append(f'{abi_field} = "{value}"')

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                *abi_lines,
            ],
            f"abi.{field} must be a string",
        )

    def test_report_rejects_native_plugins_package_report_payload_content_hash_non_string(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                "content_hash = 42",
            ],
            "payload.content_hash must be a string",
        )

    def test_report_rejects_native_plugins_package_report_payload_content_hash_blank(
        self,
    ) -> None:
        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                "file_count = 1",
                'content_hash = "   "',
            ],
            "payload.content_hash must be a non-empty string",
            "does not match current package payload",
        )

    def test_report_rejects_native_plugins_package_report_payload_content_hash_malformed(
        self,
    ) -> None:
        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                "file_count = 1",
                'content_hash = "not-a-hash"',
            ],
            "payload.content_hash must be a SHA-256 hex digest",
            "does not match current package payload",
        )

    def test_report_rejects_native_plugins_package_report_payload_file_count_negative(
        self,
    ) -> None:
        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                "file_count = -1",
                'content_hash = "'
                "0000000000000000000000000000000000000000000000000000000000000000"
                '"',
            ],
            "payload.file_count must be non-negative",
            "does not match current package payload",
        )

    def test_report_rejects_native_plugins_package_report_abi_string_field_blank(
        self,
    ) -> None:
        field = next(iter(NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS))
        abi_lines = ["[abi]", "abi_version = 3"]
        for abi_field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
            if abi_field == field:
                abi_lines.append(f'{abi_field} = "   "')
            else:
                abi_lines.append(f'{abi_field} = "{value}"')

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                *abi_lines,
            ],
            f"abi.{field} must be a non-empty string",
            f"abi.{field} must be {NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]}",
        )

    def test_report_rejects_native_plugins_package_report_payload_file_unknown_field(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            payload_lines = [
                'package_id = "animation"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
            ]
            for entry in file_manifest:
                payload_lines.extend(
                    [
                        "",
                        "[[payload.files]]",
                        f'path = "{entry["path"]}"',
                        f'bytes = {entry["bytes"]}',
                        f'sha256 = "{entry["sha256"]}"',
                    ]
                )
            payload_lines.append('unsigned_sidecar = "plugins/animation/sidecar.bin"')
            package_report.write_text(
                "\n".join(payload_lines) + "\n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "payload files[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_package_report_payload_files_non_object_array(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)

            cases = (
                (
                    [
                        'package_id = "animation"',
                        "",
                        "[payload]",
                        f"file_count = {len(file_manifest)}",
                        f'content_hash = "{content_hash}"',
                        'files = "not-an-array"',
                    ],
                    "payload files must be an object array",
                ),
                (
                    [
                        'package_id = "animation"',
                        "",
                        "[payload]",
                        f"file_count = {len(file_manifest)}",
                        f'content_hash = "{content_hash}"',
                        'files = ["not-an-object"]',
                    ],
                    "payload files[0] must be an object",
                ),
            )

        for lines, diagnostic in cases:
            with self.subTest(diagnostic=diagnostic):
                self._assert_package_report_diagnostic(
                    lines,
                    diagnostic,
                    "payload files are malformed",
                )

    def test_report_rejects_native_plugins_package_report_payload_file_field_types(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)

            cases = (
                ("path", 42, "must be a string"),
                ("sha256", 42, "must be a string"),
                ("bytes", '"1"', "must be an integer"),
            )

        for field, value, expected_type in cases:
            with self.subTest(field=field):
                file_entry = dict(file_manifest[0])
                file_entry[field] = value
                self._assert_package_report_diagnostic(
                    [
                        'package_id = "animation"',
                        "",
                        "[payload]",
                        f"file_count = {len(file_manifest)}",
                        f'content_hash = "{content_hash}"',
                        "",
                        "[[payload.files]]",
                        f'path = "{file_entry["path"]}"'
                        if isinstance(file_entry["path"], str)
                        else f"path = {file_entry['path']}",
                        f"bytes = {file_entry['bytes']}",
                        f'sha256 = "{file_entry["sha256"]}"'
                        if isinstance(file_entry["sha256"], str)
                        else f"sha256 = {file_entry['sha256']}",
                    ],
                    f"payload files[0].{field} {expected_type}",
                    "payload files are malformed",
                )

    def test_report_rejects_native_plugins_package_report_payload_file_blank_strings(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)

        for field in ("path", "sha256"):
            with self.subTest(field=field):
                file_entry = dict(file_manifest[0])
                file_entry[field] = "   "
                self._assert_package_report_diagnostic(
                    [
                        'package_id = "animation"',
                        "",
                        "[payload]",
                        f"file_count = {len(file_manifest)}",
                        f'content_hash = "{content_hash}"',
                        "",
                        "[[payload.files]]",
                        f'path = "{file_entry["path"]}"',
                        f"bytes = {file_entry['bytes']}",
                        f'sha256 = "{file_entry["sha256"]}"',
                    ],
                    f"payload files[0].{field} must be a non-empty string",
                    "payload files do not match current package payload",
                )

    def test_report_rejects_native_plugins_package_report_payload_file_unsafe_path(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            file_entry = dict(file_manifest[0])
            file_entry["path"] = "../native/zircon_plugin_animation.dll"

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
                "",
                "[[payload.files]]",
                f'path = "{file_entry["path"]}"',
                f"bytes = {file_entry['bytes']}",
                f'sha256 = "{file_entry["sha256"]}"',
            ],
            "payload files[0].path must be a safe relative path",
            "payload files do not match current package payload",
        )

    def test_report_rejects_native_plugins_package_report_payload_file_duplicate_path(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            file_entry = dict(file_manifest[0])

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                "file_count = 2",
                f'content_hash = "{content_hash}"',
                "",
                "[[payload.files]]",
                f'path = "{file_entry["path"]}"',
                f"bytes = {file_entry['bytes']}",
                f'sha256 = "{file_entry["sha256"]}"',
                "",
                "[[payload.files]]",
                f'path = "{file_entry["path"]}"',
                f"bytes = {file_entry['bytes']}",
                f'sha256 = "{file_entry["sha256"]}"',
            ],
            "payload files.path must not contain duplicate entries",
        )

    def test_report_rejects_native_plugins_package_report_payload_file_negative_bytes(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            file_entry = dict(file_manifest[0])
            file_entry["bytes"] = -1

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
                "",
                "[[payload.files]]",
                f'path = "{file_entry["path"]}"',
                f"bytes = {file_entry['bytes']}",
                f'sha256 = "{file_entry["sha256"]}"',
            ],
            "payload files[0].bytes must be non-negative",
            "payload files do not match current package payload",
        )

    def test_report_rejects_native_plugins_package_report_payload_file_malformed_sha256(
        self,
    ) -> None:
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
            package_dir = package_report.parent
            file_manifest = native_dynamic_package_payload_file_manifest(package_dir)
            content_hash = native_dynamic_content_hash(file_manifest)
            file_entry = dict(file_manifest[0])
            file_entry["sha256"] = "not-a-hash"

        self._assert_package_report_diagnostic(
            [
                'package_id = "animation"',
                "",
                "[payload]",
                f"file_count = {len(file_manifest)}",
                f'content_hash = "{content_hash}"',
                "",
                "[[payload.files]]",
                f'path = "{file_entry["path"]}"',
                f"bytes = {file_entry['bytes']}",
                f'sha256 = "{file_entry["sha256"]}"',
            ],
            "payload files[0].sha256 must be a SHA-256 hex digest",
            "payload files do not match current package payload",
        )
