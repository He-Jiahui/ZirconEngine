from __future__ import annotations

import tempfile
import unittest
from collections.abc import Callable
from pathlib import Path

from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from tools.zircon_export.native_dynamic_payload import (
    native_dynamic_content_hash,
    native_dynamic_package_payload_file_manifest,
)
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_platform_bundle_fixture,
)


class NativeDynamicPackageReportTrimmedSchemaTests(unittest.TestCase):
    def _assert_package_report_diagnostic(
        self,
        mutate_lines: Callable[[list[str], list[dict[str, object]]], None],
        expected_diagnostic: str,
        unexpected_diagnostic: str | None = None,
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
            file_manifest = native_dynamic_package_payload_file_manifest(
                package_report.parent,
            )
            lines = self._package_report_lines(file_manifest)
            mutate_lines(lines, file_manifest)
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
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        "native_plugins_payload materialized_packages[0] package_report"
                        in diagnostic
                        and unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )
            self.assertNotIn("native_plugins_payload", report)

    def _package_report_lines(
        self,
        file_manifest: list[dict[str, object]],
    ) -> list[str]:
        content_hash = native_dynamic_content_hash(file_manifest)
        lines = [
            "format_version = 1",
            'package_id = "animation"',
            'directory = "animation"',
            'path = "plugins/animation"',
            'manifest = "plugins/animation/plugin.toml"',
            "",
            "[payload]",
            f"file_count = {len(file_manifest)}",
            f'content_hash = "{content_hash}"',
        ]
        for entry in file_manifest:
            lines.extend(
                [
                    "",
                    "[[payload.files]]",
                    f'path = "{entry["path"]}"',
                    f'bytes = {entry["bytes"]}',
                    f'sha256 = "{entry["sha256"]}"',
                ]
            )
        lines.extend(["", "[abi]", "abi_version = 3"])
        for field, value in NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items():
            lines.append(f'{field} = "{value}"')
        return lines

    def test_report_rejects_native_plugins_package_report_padded_top_level_string(
        self,
    ) -> None:
        values = {
            "package_id": "animation",
            "directory": "animation",
            "path": "plugins/animation",
            "manifest": "plugins/animation/plugin.toml",
        }
        for field, value in values.items():
            with self.subTest(field=field):

                def mutate(
                    lines: list[str],
                    _file_manifest: list[dict[str, object]],
                    field: str = field,
                    value: str = value,
                ) -> None:
                    _replace_line(lines, field, f'{field} = " {value} "')

                self._assert_package_report_diagnostic(
                    mutate,
                    f"package_report.{field} "
                    "must be a non-empty trimmed string",
                    unexpected_diagnostic="does not match",
                )

    def test_report_rejects_native_plugins_package_report_payload_padded_content_hash(
        self,
    ) -> None:
        def mutate(
            lines: list[str],
            file_manifest: list[dict[str, object]],
        ) -> None:
            content_hash = native_dynamic_content_hash(file_manifest)
            _replace_line(
                lines,
                "content_hash",
                f'content_hash = " {content_hash} "',
            )

        self._assert_package_report_diagnostic(
            mutate,
            "package_report payload.content_hash "
            "must be a non-empty trimmed string",
            unexpected_diagnostic=(
                "package_report payload.content_hash "
                "must be a SHA-256 hex digest"
            ),
        )

    def test_report_rejects_native_plugins_package_report_payload_file_padded_string(
        self,
    ) -> None:
        for field in ("path", "sha256"):
            with self.subTest(field=field):

                def mutate(
                    lines: list[str],
                    file_manifest: list[dict[str, object]],
                    field: str = field,
                ) -> None:
                    value = file_manifest[0][field]
                    self.assertIsInstance(value, str)
                    _replace_line_after(
                        lines,
                        "[[payload.files]]",
                        field,
                        f'{field} = " {value} "',
                    )

                self._assert_package_report_diagnostic(
                    mutate,
                    f"package_report payload files[0].{field} "
                    "must be a non-empty trimmed string",
                    unexpected_diagnostic=(
                        "package_report payload files[0].sha256 "
                        "must be a SHA-256 hex digest"
                    )
                    if field == "sha256"
                    else None,
                )

    def test_report_rejects_native_plugins_package_report_payload_file_padded_duplicate_path_before_uniqueness(
        self,
    ) -> None:
        def mutate(
            lines: list[str],
            file_manifest: list[dict[str, object]],
        ) -> None:
            path = file_manifest[0]["path"]
            bytes_count = file_manifest[0]["bytes"]
            sha256 = file_manifest[0]["sha256"]
            self.assertIsInstance(path, str)
            self.assertIsInstance(bytes_count, int)
            self.assertIsInstance(sha256, str)
            padded_path = f" {path} "
            _replace_line_after(
                lines,
                "[[payload.files]]",
                "path",
                f'path = "{padded_path}"',
            )
            _replace_line(lines, "file_count", "file_count = 2")
            first_file_index = lines.index("[[payload.files]]")
            second_file = [
                "",
                "[[payload.files]]",
                f'path = "{padded_path}"',
                f"bytes = {bytes_count}",
                f'sha256 = "{sha256}"',
            ]
            lines[first_file_index + 4:first_file_index + 4] = second_file

        self._assert_package_report_diagnostic(
            mutate,
            "package_report payload files[0].path "
            "must be a non-empty trimmed string",
            unexpected_diagnostic=(
                "package_report payload files.path "
                "must not contain duplicate entries"
            ),
        )

    def test_report_rejects_native_plugins_package_report_abi_padded_string(
        self,
    ) -> None:
        field, value = next(iter(NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items()))

        def mutate(
            lines: list[str],
            _file_manifest: list[dict[str, object]],
        ) -> None:
            _replace_line(lines, field, f'{field} = " {value} "')

        self._assert_package_report_diagnostic(
            mutate,
            f"package_report abi.{field} must be a non-empty trimmed string",
            unexpected_diagnostic=f"package_report abi.{field} must be {value}",
        )


def _replace_line(lines: list[str], field: str, replacement: str) -> None:
    prefix = f"{field} = "
    for index, line in enumerate(lines):
        if line.startswith(prefix):
            lines[index] = replacement
            return
    raise AssertionError(f"missing line for {field}")


def _replace_line_after(
    lines: list[str],
    marker: str,
    field: str,
    replacement: str,
) -> None:
    try:
        start = lines.index(marker) + 1
    except ValueError as error:
        raise AssertionError(f"missing marker {marker}") from error
    prefix = f"{field} = "
    for index in range(start, len(lines)):
        if lines[index].startswith(prefix):
            lines[index] = replacement
            return
    raise AssertionError(f"missing line for {field} after {marker}")


if __name__ == "__main__":
    unittest.main()
