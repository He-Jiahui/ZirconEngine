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
            "[0].compatible_profiles[1] must be a string",
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
