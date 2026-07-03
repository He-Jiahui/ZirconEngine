from __future__ import annotations

import tempfile
import tomllib
import unittest
from pathlib import Path
from typing import Callable

from tools.zircon_export.native_dynamic_contract import NATIVE_DYNAMIC_ABI_STRING_FIELDS
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _native_plugins_content_hash,
    _native_plugins_file_manifest,
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class NativeDynamicPayloadPackageReportSchemaTests(unittest.TestCase):
    def _assert_package_report_schema_diagnostic(
        self,
        mutate_package_report: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            package_report_path = (
                out
                / "bundle"
                / "windows-release"
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            )
            package_report = _read_toml(package_report_path)
            mutate_package_report(package_report)
            _write_toml(package_report_path, package_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            native_plugins = out / "bundle" / "windows-release" / "plugins"
            file_manifest = _native_plugins_file_manifest(native_plugins)
            payload["file_manifest"] = file_manifest
            payload["file_count"] = len(file_manifest)
            payload["content_hash"] = _native_plugins_content_hash(file_manifest)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                out / "bundle" / "windows-release" / "bundle.json",
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("format_version", "must be an integer"),
            ("package_id", "must be a string"),
            ("directory", "must be a string"),
            ("path", "must be a string"),
            ("manifest", "must be a string"),
            ("abi", "must be an object"),
            ("payload", "must be an object"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(
                    package_report: dict[str, object],
                    field=field,
                ) -> None:
                    package_report.pop(field)

                self._assert_package_report_schema_diagnostic(
                    mutate,
                    "PlatformBundle report native_plugins_payload "
                    "materialized_packages[0] package_report."
                    f"{field} {expected_type}",
                )

    def test_report_rejects_native_plugins_payload_package_report_payload_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("file_count", "must be an integer"),
            ("content_hash", "must be a string"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(
                    package_report: dict[str, object],
                    field=field,
                ) -> None:
                    payload = package_report["payload"]
                    self.assertIsInstance(payload, dict)
                    payload.pop(field)

                self._assert_package_report_schema_diagnostic(
                    mutate,
                    "PlatformBundle report native_plugins_payload "
                    "materialized_packages[0] package_report "
                    f"payload.{field} {expected_type}",
                )

    def test_report_rejects_native_plugins_payload_package_report_abi_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("abi_version", "must be an integer"),
            *((field, "must be a string") for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(
                    package_report: dict[str, object],
                    field=field,
                ) -> None:
                    abi = package_report["abi"]
                    self.assertIsInstance(abi, dict)
                    abi.pop(field)

                self._assert_package_report_schema_diagnostic(
                    mutate,
                    "PlatformBundle report native_plugins_payload "
                    "materialized_packages[0] package_report "
                    f"abi.{field} {expected_type}",
                )


def _read_toml(path: Path) -> dict[str, object]:
    with path.open("rb") as file:
        return dict(tomllib.load(file))


def _write_toml(path: Path, document: dict[str, object]) -> None:
    lines: list[str] = []
    for field in (
        "format_version",
        "package_id",
        "directory",
        "path",
        "manifest",
    ):
        if field not in document:
            continue
        value = document[field]
        if isinstance(value, int):
            lines.append(f"{field} = {value}")
        else:
            lines.append(f'{field} = "{value}"')
    abi = document.get("abi")
    if isinstance(abi, dict):
        lines.extend(["", "[abi]"])
        if "abi_version" in abi:
            lines.append(f"abi_version = {abi['abi_version']}")
        for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
            if field in abi:
                lines.append(f'{field} = "{abi[field]}"')
    payload = document.get("payload")
    if isinstance(payload, dict):
        lines.extend(["", "[payload]"])
        if "file_count" in payload:
            lines.append(f"file_count = {payload['file_count']}")
        if "content_hash" in payload:
            lines.append(f'content_hash = "{payload["content_hash"]}"')
        files = payload.get("files")
        if isinstance(files, list):
            for entry in files:
                if not isinstance(entry, dict):
                    continue
                lines.extend(["", "[[payload.files]]"])
                if "path" in entry:
                    lines.append(f'path = "{entry["path"]}"')
                if "bytes" in entry:
                    lines.append(f'bytes = {entry["bytes"]}')
                if "sha256" in entry:
                    lines.append(f'sha256 = "{entry["sha256"]}"')
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")