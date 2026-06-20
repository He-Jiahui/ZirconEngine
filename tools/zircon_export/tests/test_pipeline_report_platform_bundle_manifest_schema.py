from __future__ import annotations

import json
import tempfile
import unittest
from copy import deepcopy
from pathlib import Path
from typing import Callable

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _write_platform_bundle_fixture,
    _write_bundle_manifest_from_platform_report,
    _write_stage_report,
)


class PlatformBundleManifestSchemaTests(unittest.TestCase):
    def _assert_platform_bundle_report_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
        *,
        with_template_file: bool = False,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=with_template_file,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report[field] = value
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

    def _assert_bundle_manifest_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            manifest = json.loads(
                fixture["bundle_manifest"].read_text(encoding="utf-8")
            )
            manifest[field] = value
            fixture["bundle_manifest"].write_text(
                json.dumps(manifest, indent=2),
                encoding="utf-8",
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

    def _assert_bundle_manifest_nested_diagnostic(
        self,
        mutate_manifest: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
        *,
        unexpected_diagnostic: str | None = None,
        with_template_file: bool = False,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(
                out,
                with_template_file=with_template_file,
            )
            manifest = json.loads(
                fixture["bundle_manifest"].read_text(encoding="utf-8")
            )
            mutate_manifest(manifest)
            fixture["bundle_manifest"].write_text(
                json.dumps(manifest, indent=2),
                encoding="utf-8",
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
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def _assert_platform_bundle_report_nested_diagnostic(
        self,
        mutate_report: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
        *,
        unexpected_diagnostic: str | None = None,
        with_template_file: bool = False,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(
                out,
                with_template_file=with_template_file,
            )
            platform_report = _read_stage_report(out, "platform_bundle")
            mutate_report(platform_report)
            _write_stage_report(out, "platform_bundle", platform_report)

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
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

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

    def test_report_rejects_platform_bundle_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report["unsigned_sidecar"] = {"path": "sidecar.bin"}
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
                    "PlatformBundle report unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_platform_bundle_string_fields_non_string(self) -> None:
        for field in (
            "bundle",
            "host_executable",
            "host_source",
            "host_source_origin",
            "pack",
            "pack_source",
            "pack_source_origin",
            "delta_pack",
            "delta_pack_source",
            "delta_pack_source_origin",
            "native_plugins",
            "bundle_manifest",
        ):
            with self.subTest(field=field):
                self._assert_platform_bundle_report_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle report {field} must be a string",
                )

    def test_report_rejects_platform_bundle_object_fields_non_object(self) -> None:
        for field in ("template", "native_plugins_payload"):
            with self.subTest(field=field):
                self._assert_platform_bundle_report_field_diagnostic(
                    field,
                    "not-an-object",
                    f"PlatformBundle report {field} must be an object",
                    with_template_file=field == "template",
                )

    def test_report_rejects_platform_bundle_template_files_non_object_array(
        self,
    ) -> None:
        self._assert_platform_bundle_report_field_diagnostic(
            "template_files",
            "not-an-array",
            "PlatformBundle report template_files must be an object array",
        )
        self._assert_platform_bundle_report_field_diagnostic(
            "template_files",
            ["not-an-object"],
            "PlatformBundle report template_files[0] must be an object",
        )

    def test_report_rejects_bundle_manifest_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            manifest = json.loads(
                fixture["bundle_manifest"].read_text(encoding="utf-8")
            )
            manifest["unsigned_sidecar"] = {"path": "sidecar.bin"}
            fixture["bundle_manifest"].write_text(
                json.dumps(manifest, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "bundle_manifest unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_bundle_manifest_string_fields_non_string(self) -> None:
        for field in (
            "profile",
            "host_executable",
            "host_source",
            "host_source_origin",
            "pack",
            "pack_source",
            "pack_source_origin",
            "delta_pack",
            "delta_pack_source",
            "delta_pack_source_origin",
            "native_plugins",
        ):
            with self.subTest(field=field):
                self._assert_bundle_manifest_field_diagnostic(
                    field,
                    42,
                    f"PlatformBundle bundle_manifest {field} must be a string",
                )

    def test_report_rejects_bundle_manifest_object_fields_non_object(self) -> None:
        for field in ("template_resolution", "template", "native_plugins_payload"):
            with self.subTest(field=field):
                self._assert_bundle_manifest_field_diagnostic(
                    field,
                    "not-an-object",
                    f"PlatformBundle bundle_manifest {field} must be an object",
                )

    def test_report_rejects_bundle_manifest_template_files_non_object_array(
        self,
    ) -> None:
        self._assert_bundle_manifest_field_diagnostic(
            "template_files",
            "not-an-array",
            "PlatformBundle bundle_manifest template_files must be an object array",
        )
        self._assert_bundle_manifest_field_diagnostic(
            "template_files",
            ["not-an-object"],
            "PlatformBundle bundle_manifest template_files[0] must be an object",
        )

    def test_report_rejects_bundle_manifest_native_plugins_payload_nested_schema(
        self,
    ) -> None:
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["native_plugins_payload"].__setitem__(
                "unsigned_sidecar",
                {"path": "plugins/sidecar.bin"},
            ),
            "PlatformBundle bundle_manifest native_plugins_payload "
            "unknown field unsigned_sidecar",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest native_plugins_payload "
                "does not match stage report"
            ),
        )
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["native_plugins_payload"][
                "file_manifest"
            ][0].__setitem__("bytes", "1"),
            "PlatformBundle bundle_manifest native_plugins_payload "
            "file_manifest[0].bytes must be an integer",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest native_plugins_payload "
                "does not match stage report"
            ),
        )

    def test_report_rejects_bundle_manifest_template_nested_schema(self) -> None:
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["template"].__setitem__(
                "unsigned_sidecar",
                "sidecar.bin",
            ),
            "PlatformBundle bundle_manifest template unknown field unsigned_sidecar",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template does not match stage report"
            ),
            with_template_file=True,
        )
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["template"]["files"][0].__setitem__(
                "path",
                42,
            ),
            "PlatformBundle bundle_manifest template.files[0].path must be a string",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template does not match stage report"
            ),
            with_template_file=True,
        )

    def test_report_rejects_bundle_manifest_template_resolution_nested_schema(
        self,
    ) -> None:
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest.__setitem__(
                "template_resolution",
                {"unsigned_sidecar": "sidecar.bin"},
            ),
            "PlatformBundle bundle_manifest template_resolution "
            "unknown field unsigned_sidecar",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template_resolution "
                "does not match stage report"
            ),
            with_template_file=True,
        )
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest.__setitem__(
                "template_resolution",
                {
                    "candidates": [
                        {"compatible_profiles": ["windows-release", 42]}
                    ]
                },
            ),
            "PlatformBundle bundle_manifest template_resolution candidates"
            "[0].compatible_profiles must be a string array",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template_resolution "
                "does not match stage report"
            ),
            with_template_file=True,
        )

    def test_report_rejects_bundle_manifest_template_resolution_template_dir_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report["template_resolution"] = _template_resolution(out)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )
            manifest = json.loads(
                fixture["bundle_manifest"].read_text(encoding="utf-8")
            )
            alternate_template_dir = out / "alternate-template"
            alternate_template_dir.mkdir(parents=True)
            (alternate_template_dir / "Info.plist").write_text(
                "<plist>zircon</plist>",
                encoding="utf-8",
            )
            template = manifest["template"]
            self.assertIsInstance(template, dict)
            template["template_dir"] = str(alternate_template_dir)
            fixture["bundle_manifest"].write_text(
                json.dumps(manifest, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle bundle_manifest template_resolution.template_dir "
                    "must match template.template_dir" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle bundle_manifest template does not match stage report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_bundle_manifest_template_resolution_profile_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out, with_template_file=True)
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report["template_resolution"] = _template_resolution(out)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )
            manifest = json.loads(
                fixture["bundle_manifest"].read_text(encoding="utf-8")
            )
            resolution = manifest["template_resolution"]
            self.assertIsInstance(resolution, dict)
            resolution["profile"] = "other-profile"
            candidate = resolution["candidates"][0]
            self.assertIsInstance(candidate, dict)
            candidate["compatible_profiles"] = ["other-profile"]
            fixture["bundle_manifest"].write_text(
                json.dumps(manifest, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle bundle_manifest template_resolution.profile "
                    "must match PlatformBundle bundle_manifest profile windows-release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle bundle_manifest template_resolution "
                    "does not match stage report" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_bundle_manifest_template_resolution_null_expected_identity(
        self,
    ) -> None:
        for field in ("expected_engine_version", "expected_target_platform"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(
                        out,
                        with_template_file=True,
                    )
                    platform_report = _read_stage_report(out, "platform_bundle")
                    platform_report["template_resolution"] = _template_resolution(out)
                    _write_stage_report(out, "platform_bundle", platform_report)
                    _write_bundle_manifest_from_platform_report(
                        fixture["bundle_manifest"],
                        platform_report,
                    )
                    manifest = json.loads(
                        fixture["bundle_manifest"].read_text(encoding="utf-8")
                    )
                    resolution = manifest["template_resolution"]
                    self.assertIsInstance(resolution, dict)
                    resolution[field] = None
                    fixture["bundle_manifest"].write_text(
                        json.dumps(manifest, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            "PlatformBundle bundle_manifest template_resolution "
                            f"non-fatal resolution must include {field}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "PlatformBundle bundle_manifest template_resolution "
                            "does not match stage report" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_bundle_manifest_template_files_nested_schema(
        self,
    ) -> None:
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["template_files"][0].__setitem__(
                "unsigned_sidecar",
                "sidecar.bin",
            ),
            "PlatformBundle bundle_manifest template_files[0] "
            "unknown field unsigned_sidecar",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template_files "
                "does not match stage report"
            ),
            with_template_file=True,
        )
        self._assert_bundle_manifest_nested_diagnostic(
            lambda manifest: manifest["template_files"][0].__setitem__(
                "destination",
                42,
            ),
            "PlatformBundle bundle_manifest template_files[0].destination "
            "must be a string",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template_files "
                "does not match stage report"
            ),
            with_template_file=True,
        )

    def test_report_rejects_platform_bundle_report_template_nested_schema_before_manifest_compare(
        self,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            template = platform_report["template"]
            self.assertIsInstance(template, dict)
            template["unsigned_sidecar"] = "sidecar.bin"
            platform_report["template_files"] = []

        self._assert_platform_bundle_report_nested_diagnostic(
            mutate,
            "PlatformBundle report template unknown field unsigned_sidecar",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest template does not match stage report"
            ),
            with_template_file=True,
        )

    def test_report_rejects_platform_bundle_report_native_payload_schema_before_manifest_compare(
        self,
    ) -> None:
        def mutate(platform_report: dict[str, object]) -> None:
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            file_entry = file_manifest[0]
            self.assertIsInstance(file_entry, dict)
            file_entry["bytes"] = "1"

        self._assert_platform_bundle_report_nested_diagnostic(
            mutate,
            "PlatformBundle report native_plugins_payload "
            "file_manifest[0].bytes must be an integer",
            unexpected_diagnostic=(
                "PlatformBundle bundle_manifest native_plugins_payload "
                "does not match stage report"
            ),
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
        for field in ("compatible_profiles", "diagnostics"):
            with self.subTest(field=field):
                self._assert_template_report_field_diagnostic(
                    field,
                    ["windows-release", 42],
                    f"PlatformBundle report template.{field} must be a string array",
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
            "PlatformBundle report template_resolution.diagnostics must be a string array",
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
            "PlatformBundle report template_resolution candidates[0].compatible_profiles "
            "must be a string array",
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
            "PlatformBundle report template_resolution skipped_candidates[0].diagnostics "
            "must be a string array",
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


def _template_resolution(out: Path) -> dict[str, object]:
    template_root = out
    template_dir = out / "template"
    candidate = {
        "template_dir": str(template_dir),
        "template_id": "windows-template",
        "engine_version": "0.1.0",
        "target_platform": "windows-x86_64",
        "compatible_profiles": ["windows-release"],
        "bundle_format": "directory",
    }
    return {
        "template_root": str(template_root),
        "profile": "windows-release",
        "expected_engine_version": "0.1.0",
        "expected_target_platform": "windows-x86_64",
        "fatal": False,
        "diagnostics": [],
        "candidates": [deepcopy(candidate)],
        "skipped_candidates": [
            {
                "template_dir": str(out / "broken-template"),
                "diagnostics": ["template format_version 999 is not supported"],
            }
        ],
        "template_dir": str(template_dir),
    }
