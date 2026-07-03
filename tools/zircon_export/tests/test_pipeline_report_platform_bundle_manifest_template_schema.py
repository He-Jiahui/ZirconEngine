from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from typing import Callable

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_platform_bundle_fixture,
    _write_bundle_manifest_from_platform_report,
    _write_stage_report,
)
from tools.zircon_export.tests.platform_bundle_manifest_schema_test_support import (
    _template_resolution,
)


class PlatformBundleManifestTemplateSchemaTests(unittest.TestCase):
    def _assert_template_evidence_diagnostic(
        self,
        mutate: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            mutate(platform_report)
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
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def _assert_template_report_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template[field] = value

        self._assert_template_evidence_diagnostic(mutate, expected_diagnostic)

    def _assert_template_bundle_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["bundle"] = {
                "root": platform_report["bundle"],
                "manifest_path": platform_report["bundle_manifest"],
                "host_path": platform_report["host_executable"],
                "pack_path": platform_report["pack"],
            }
            bundle = template["bundle"]
            self.assertIsInstance(bundle, dict)
            bundle[field] = value

        self._assert_template_evidence_diagnostic(mutate, expected_diagnostic)

    def _assert_template_file_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            files = template["files"]
            self.assertIsInstance(files, list)
            template_file = files[0]
            self.assertIsInstance(template_file, dict)
            template_file[field] = value

        self._assert_template_evidence_diagnostic(mutate, expected_diagnostic)

    def _assert_template_copied_file_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            template_files = platform_report["template_files"]
            self.assertIsInstance(template_files, list)
            template_file = template_files[0]
            self.assertIsInstance(template_file, dict)
            template_file[field] = value

        self._assert_template_evidence_diagnostic(mutate, expected_diagnostic)

    def _assert_template_resolution_diagnostic(
        self,
        mutate: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            resolution = _template_resolution(out)
            mutate(resolution)
            platform_report["template_resolution"] = resolution
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
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_string_fields_non_string(self) -> None:
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
                self._assert_template_report_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle report template.{field} must be a string",
                )

    def test_report_rejects_template_report_count_fields_non_integer(self) -> None:
        for field in ("expected_format_version", "format_version"):
            with self.subTest(field=field):
                self._assert_template_report_field_diagnostic(
                    field,
                    "1",
                    f"PlatformBundle report template.{field} must be an integer",
                )

    def test_report_rejects_template_report_bool_fields_non_bool(self) -> None:
        self._assert_template_report_field_diagnostic(
            "fatal",
            "false",
            "PlatformBundle report template.fatal must be a boolean",
        )

    def test_report_rejects_template_report_string_array_fields_non_string_array(
        self,
    ) -> None:
        cases = (
            (
                "compatible_profiles",
                "PlatformBundle report template.compatible_profiles[1] must be a string",
            ),
            (
                "diagnostics",
                "PlatformBundle report template.diagnostics[1] must be a string",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_template_report_field_diagnostic(
                    field,
                    ["windows-release", 42],
                    expected_diagnostic,
                )

    def test_report_rejects_template_report_object_fields_non_object(self) -> None:
        self._assert_template_report_field_diagnostic(
            "bundle",
            "not-an-object",
            "PlatformBundle report template.bundle must be an object",
        )

    def test_report_rejects_template_report_files_non_object_array(self) -> None:
        self._assert_template_report_field_diagnostic(
            "files",
            "not-an-array",
            "PlatformBundle report template.files must be an object array",
        )
        self._assert_template_report_field_diagnostic(
            "files",
            ["not-an-object"],
            "PlatformBundle report template.files[0] must be an object",
        )

    def test_report_rejects_template_bundle_string_fields_non_string(self) -> None:
        for field in (
            "delta_pack_path",
            "host_path",
            "manifest_path",
            "pack_path",
            "root",
        ):
            with self.subTest(field=field):
                self._assert_template_bundle_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle report template.bundle.{field} must be a string",
                )

    def test_report_rejects_template_bundle_string_fields_blank(self) -> None:
        for field in (
            "delta_pack_path",
            "host_path",
            "manifest_path",
            "pack_path",
            "root",
        ):
            with self.subTest(field=field):
                self._assert_template_bundle_field_diagnostic(
                    field,
                    " ",
                    f"PlatformBundle report template.bundle.{field} must be a non-empty string",
                )

    def test_report_rejects_template_file_string_fields_non_string(self) -> None:
        for field in ("bundle_path", "path", "purpose", "sha256"):
            with self.subTest(field=field):
                self._assert_template_file_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle report template.files[0].{field} must be a string",
                )

    def test_report_rejects_template_copied_file_string_fields_non_string(
        self,
    ) -> None:
        for field in ("destination", "source"):
            with self.subTest(field=field):
                self._assert_template_copied_file_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle report template_files[0].{field} must be a string",
                )

    def test_report_rejects_template_file_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            template_files = platform_report["template_files"]
            self.assertIsInstance(template_files, list)
            template_file = template_files[0]
            self.assertIsInstance(template_file, dict)
            template_file["unsigned_sidecar"] = "sidecar.bin"
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
                    "template_files[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["unsigned_sidecar"] = "sidecar.bin"
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
                    "template unknown field unsigned_sidecar" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_report_file_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            files = template["files"]
            self.assertIsInstance(files, list)
            template_file = files[0]
            self.assertIsInstance(template_file, dict)
            template_file["unsigned_sidecar"] = "sidecar.bin"
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
                    "template.files[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_bundle_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["bundle"] = {
                "root": platform_report["bundle"],
                "manifest_path": str(fixture["bundle_manifest"]),
                "host_path": platform_report["host_executable"],
                "pack_path": platform_report["pack"],
                "unsigned_sidecar": "sidecar.bin",
            }
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
                    "template.bundle unknown field unsigned_sidecar" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_template_resolution_string_fields_non_string(
        self,
    ) -> None:
        for field in (
            "expected_engine_version",
            "expected_target_platform",
            "profile",
            "template_dir",
            "template_root",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(field, 42),
                    f"PlatformBundle report template_resolution.{field} must be a string",
                )

    def test_report_rejects_template_resolution_bool_fields_non_bool(self) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__("fatal", "false"),
            "PlatformBundle report template_resolution.fatal must be a boolean",
        )

    def test_report_rejects_template_resolution_string_array_fields_non_string_array(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution.__setitem__(
                "diagnostics",
                ["template skipped", 42],
            ),
            "PlatformBundle report template_resolution.diagnostics[1] must be a string",
        )

    def test_report_rejects_template_resolution_candidate_entries_non_object(
        self,
    ) -> None:
        for field in ("candidates", "skipped_candidates"):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution.__setitem__(
                        field,
                        ["not-an-object"],
                    ),
                    f"PlatformBundle report template_resolution {field}[0] must be an object",
                )

    def test_report_rejects_template_resolution_candidate_string_fields_non_string(
        self,
    ) -> None:
        for field in (
            "bundle_format",
            "engine_version",
            "host_artifact",
            "target_platform",
            "template_dir",
            "template_id",
        ):
            with self.subTest(field=field):
                self._assert_template_resolution_diagnostic(
                    lambda resolution, field=field: resolution["candidates"][
                        0
                    ].__setitem__(field, 42),
                    "PlatformBundle report template_resolution candidates"
                    f"[0].{field} must be a string",
                )

    def test_report_rejects_template_resolution_candidate_string_arrays_non_string_array(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["candidates"][0].__setitem__(
                "compatible_profiles",
                ["windows-release", 42],
            ),
            "PlatformBundle report template_resolution candidates[0].compatible_profiles"
            "[1] must be a string",
        )

    def test_report_rejects_template_resolution_skipped_candidate_fields_non_string(
        self,
    ) -> None:
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "template_dir",
                42,
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].template_dir "
            "must be a string",
        )
        self._assert_template_resolution_diagnostic(
            lambda resolution: resolution["skipped_candidates"][0].__setitem__(
                "diagnostics",
                ["template skipped", 42],
            ),
            "PlatformBundle report template_resolution skipped_candidates[0].diagnostics"
            "[1] must be a string",
        )

    def test_report_rejects_template_resolution_unknown_field(self) -> None:
        for mutate, expected in (
            (
                lambda resolution: resolution.update(
                    {"unsigned_sidecar": "sidecar.bin"}
                ),
                "template_resolution unknown field unsigned_sidecar",
            ),
            (
                lambda resolution: resolution["candidates"][0].update(
                    {"unsigned_sidecar": "sidecar.bin"}
                ),
                "template_resolution candidates[0] unknown field unsigned_sidecar",
            ),
            (
                lambda resolution: resolution["skipped_candidates"][0].update(
                    {"unsigned_sidecar": "sidecar.bin"}
                ),
                "template_resolution skipped_candidates[0] unknown field unsigned_sidecar",
            ),
        ):
            with self.subTest(expected=expected):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    resolution = _template_resolution(out)
                    mutate(resolution)
                    platform_report["template_resolution"] = resolution
                    _write_stage_report(out, "platform_bundle", platform_report)
                    _write_bundle_manifest_from_platform_report(
                        fixture["bundle_manifest"],
                        platform_report,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(expected in diagnostic for diagnostic in report["diagnostics"]),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()