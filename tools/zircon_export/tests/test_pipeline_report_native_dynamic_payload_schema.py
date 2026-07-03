from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from typing import Callable

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _native_plugins_content_hash,
    _native_plugins_file_manifest,
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class NativeDynamicPayloadSchemaTests(unittest.TestCase):
    def _assert_payload_schema_diagnostic(
        self,
        mutate_payload: Callable[[dict[str, object]], None],
        expected_diagnostic: str,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            mutate_payload(payload)
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
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_unknown_top_level_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["unsigned_sidecar"] = {"path": "plugins/sidecar.bin"}
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
                    "native_plugins_payload unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_materialized_package_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["unsigned_sidecar"] = {
                "path": "plugins/animation/sidecar.bin"
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
                    "native_plugins_payload materialized_packages[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_file_manifest_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            entry = file_manifest[0]
            self.assertIsInstance(entry, dict)
            entry["unsigned_sidecar"] = "plugins/animation/sidecar.bin"
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
                    "native_plugins_payload file_manifest[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_operation_audit_unknown_field(
        self,
    ) -> None:
        for field in ("native_signing", "native_notarization"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out)
                    audit = {
                        "enabled": False,
                        "profile": None,
                        "target_platform": "windows-x86_64",
                        "allowed_platforms": [],
                        "platform_allowed": True,
                        "fatal": False,
                        "package_count": 0,
                    }
                    native_report = _read_stage_report(out, "native_dynamic")
                    native_report[field] = dict(audit)
                    _write_stage_report(out, "native_dynamic", native_report)
                    platform_report = _read_stage_report(out, "platform_bundle")
                    payload = platform_report["native_plugins_payload"]
                    self.assertIsInstance(payload, dict)
                    payload[field] = {
                        **audit,
                        "unsigned_sidecar": "plugins/animation/sidecar.bin",
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
                            f"native_plugins_payload {field} unknown field unsigned_sidecar"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_file_manifest_non_object_array(
        self,
    ) -> None:
        cases = (
            (
                "not-an-array",
                "native_plugins_payload file_manifest must be an object array",
            ),
            (
                ["not-an-object"],
                "native_plugins_payload file_manifest[0] must be an object",
            ),
        )
        for value, diagnostic in cases:
            with self.subTest(value=value):
                self._assert_payload_schema_diagnostic(
                    lambda payload, value=value: payload.__setitem__(
                        "file_manifest",
                        value,
                    ),
                    diagnostic,
                    "native_plugins_payload file_manifest is malformed",
                )

    def test_report_rejects_native_plugins_payload_missing_top_level_object_arrays(
        self,
    ) -> None:
        cases = (
            (
                "file_manifest",
                "native_plugins_payload file_manifest must be an object array",
                "native_plugins_payload file_manifest is malformed",
            ),
            (
                "materialized_packages",
                "native_plugins_payload materialized_packages must be an object array",
                "native_plugins_payload materialized_packages are malformed",
            ),
        )
        for field, diagnostic, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_payload_schema_diagnostic(
                    lambda payload, field=field: payload.pop(field),
                    diagnostic,
                    unexpected_diagnostic,
                )

    def test_report_rejects_native_plugins_payload_missing_required_top_level_scalars(
        self,
    ) -> None:
        cases = (
            ("bundle_path", "native_plugins_payload.bundle_path must be a string"),
            ("content_hash", "native_plugins_payload.content_hash must be a string"),
            ("source", "native_plugins_payload.source must be a string"),
            ("file_count", "native_plugins_payload.file_count must be an integer"),
            ("package_count", "native_plugins_payload.package_count must be an integer"),
        )
        for field, diagnostic in cases:
            with self.subTest(field=field):
                self._assert_payload_schema_diagnostic(
                    lambda payload, field=field: payload.pop(field),
                    diagnostic,
                )

    def test_report_rejects_native_plugins_payload_content_hash_non_string_without_semantic_fallback(
        self,
    ) -> None:
        self._assert_payload_schema_diagnostic(
            lambda payload: payload.__setitem__("content_hash", 42),
            "native_plugins_payload.content_hash must be a string",
            "native_plugins_payload content_hash must be a non-empty string",
        )

    def test_report_rejects_native_plugins_payload_content_hash_blank(
        self,
    ) -> None:
        self._assert_payload_schema_diagnostic(
            lambda payload: payload.__setitem__("content_hash", "   "),
            "native_plugins_payload.content_hash must be a non-empty string",
            "native_plugins_payload content_hash     does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_content_hash_malformed(
        self,
    ) -> None:
        self._assert_payload_schema_diagnostic(
            lambda payload: payload.__setitem__("content_hash", "not-a-hash"),
            "native_plugins_payload.content_hash must be a SHA-256 hex digest",
            "native_plugins_payload content_hash not-a-hash does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_blank_top_level_path_strings(
        self,
    ) -> None:
        cases = (
            ("bundle_path", "native_plugins_payload bundle_path"),
            ("loader_manifest", "native_plugins_payload loader_manifest"),
            ("source", "native_plugins_payload source"),
            ("stage_report", "native_plugins_payload stage_report"),
        )
        for field, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_payload_schema_diagnostic(
                    lambda payload, field=field: payload.__setitem__(field, "   "),
                    f"native_plugins_payload.{field} must be a non-empty string",
                    unexpected_diagnostic,
                )

    def test_report_rejects_native_plugins_payload_negative_top_level_counts(
        self,
    ) -> None:
        cases = (
            (
                "file_count",
                "native_plugins_payload.file_count must be non-negative",
                "native_plugins_payload file_count does not match current bundle plugins directory",
            ),
            (
                "package_count",
                "native_plugins_payload.package_count must be non-negative",
                "native_plugins_payload package_count does not match materialized_packages",
            ),
        )
        for field, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_payload_schema_diagnostic(
                    lambda payload, field=field: payload.__setitem__(field, -1),
                    expected_diagnostic,
                    unexpected_diagnostic,
                )

    def test_report_rejects_native_plugins_payload_file_manifest_field_types(
        self,
    ) -> None:
        cases = (
            ("path", 42, "must be a string"),
            ("sha256", 42, "must be a string"),
            ("bytes", "1", "must be an integer"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field, value=value) -> None:
                    file_manifest = payload["file_manifest"]
                    self.assertIsInstance(file_manifest, list)
                    entry = file_manifest[0]
                    self.assertIsInstance(entry, dict)
                    entry[field] = value

                self._assert_payload_schema_diagnostic(
                    mutate,
                    f"native_plugins_payload file_manifest[0].{field} {expected_type}",
                    "native_plugins_payload file_manifest is malformed",
                )

    def test_report_rejects_native_plugins_payload_file_manifest_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("path", "must be a string"),
            ("sha256", "must be a string"),
            ("bytes", "must be an integer"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field) -> None:
                    file_manifest = payload["file_manifest"]
                    self.assertIsInstance(file_manifest, list)
                    entry = file_manifest[0]
                    self.assertIsInstance(entry, dict)
                    entry.pop(field)

                self._assert_payload_schema_diagnostic(
                    mutate,
                    f"native_plugins_payload file_manifest[0].{field} {expected_type}",
                    "native_plugins_payload file_manifest is malformed",
                )

    def test_report_rejects_native_plugins_payload_file_manifest_blank_strings(
        self,
    ) -> None:
        for field in ("path", "sha256"):
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field) -> None:
                    file_manifest = payload["file_manifest"]
                    self.assertIsInstance(file_manifest, list)
                    entry = file_manifest[0]
                    self.assertIsInstance(entry, dict)
                    entry[field] = "   "

                self._assert_payload_schema_diagnostic(
                    mutate,
                    f"native_plugins_payload file_manifest[0].{field} "
                    "must be a non-empty string",
                    "native_plugins_payload file_manifest does not match "
                    "current bundle plugins directory",
                )

    def test_report_rejects_native_plugins_payload_file_manifest_malformed_sha256(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            entry = file_manifest[0]
            self.assertIsInstance(entry, dict)
            entry["sha256"] = "not-a-hash"

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload file_manifest[0].sha256 "
            "must be a SHA-256 hex digest",
            "native_plugins_payload file_manifest does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_file_manifest_unsafe_path(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            entry = file_manifest[0]
            self.assertIsInstance(entry, dict)
            entry["path"] = "../plugins/animation/zircon_plugin_animation.dll"

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload file_manifest[0].path "
            "must be a safe relative path",
            "native_plugins_payload file_manifest does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_file_manifest_negative_bytes(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            entry = file_manifest[0]
            self.assertIsInstance(entry, dict)
            entry["bytes"] = -1

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload file_manifest[0].bytes "
            "must be non-negative",
            "native_plugins_payload file_manifest does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_file_manifest_duplicate_path(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            file_manifest = payload["file_manifest"]
            self.assertIsInstance(file_manifest, list)
            entry = file_manifest[0]
            self.assertIsInstance(entry, dict)
            file_manifest.append(dict(entry))
            payload["file_count"] = len(file_manifest)
            payload["content_hash"] = _native_plugins_content_hash(file_manifest)

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload file_manifest[3].path "
            "must be unique",
            "native_plugins_payload file_manifest does not match "
            "current bundle plugins directory",
        )

    def test_report_rejects_native_plugins_payload_materialized_packages_non_object_array(
        self,
    ) -> None:
        cases = (
            (
                "not-an-array",
                "native_plugins_payload materialized_packages must be an object array",
            ),
            (
                ["not-an-object"],
                "native_plugins_payload materialized_packages[0] must be an object",
            ),
        )
        for value, diagnostic in cases:
            with self.subTest(value=value):
                self._assert_payload_schema_diagnostic(
                    lambda payload, value=value: payload.__setitem__(
                        "materialized_packages",
                        value,
                    ),
                    diagnostic,
                    "native_plugins_payload materialized_packages are malformed",
                )

    def test_report_rejects_native_plugins_payload_materialized_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("destination", 42, "must be a string"),
            ("package_report", 42, "must be a string"),
            ("source", 42, "must be a string"),
            ("loadable_artifact_count", "1", "must be an integer"),
            ("loadable_artifacts", "not-an-array", "must be a string array"),
            ("loadable_artifacts", [42], "must be a string array"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                expected_diagnostic = (
                    "native_plugins_payload materialized_packages[0]."
                    "loadable_artifacts[0] must be a string"
                    if field == "loadable_artifacts" and isinstance(value, list)
                    else (
                        "native_plugins_payload "
                        f"materialized_packages[0].{field} {expected_type}"
                    )
                )
                def mutate(payload: dict[str, object], field=field, value=value) -> None:
                    packages = payload["materialized_packages"]
                    self.assertIsInstance(packages, list)
                    package = packages[0]
                    self.assertIsInstance(package, dict)
                    package[field] = value

                self._assert_payload_schema_diagnostic(
                    mutate,
                    expected_diagnostic,
                    "native_plugins_payload materialized_packages are malformed",
                )

    def test_report_rejects_native_plugins_payload_materialized_package_non_string_loadable_artifact_entry_before_array_shape(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["loadable_artifacts"] = [
                "plugins/animation/native/zircon_plugin_animation.dll",
                42,
            ]
            package["loadable_artifact_count"] = 2

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload materialized_packages[0]."
            "loadable_artifacts[1] must be a string",
            "native_plugins_payload materialized_packages[0]."
            "loadable_artifacts must be a string array",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("package_id", "must be a string"),
            ("destination", "must be a string"),
            ("loadable_artifact_count", "must be an integer"),
            ("loadable_artifacts", "must be a string array"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field) -> None:
                    packages = payload["materialized_packages"]
                    self.assertIsInstance(packages, list)
                    package = packages[0]
                    self.assertIsInstance(package, dict)
                    package.pop(field)

                self._assert_payload_schema_diagnostic(
                    mutate,
                    "native_plugins_payload "
                    f"materialized_packages[0].{field} {expected_type}",
                    "native_plugins_payload materialized_packages are malformed",
                )

    def test_report_rejects_native_plugins_payload_materialized_package_blank_strings(
        self,
    ) -> None:
        for field in ("package_id", "destination", "package_report", "source"):
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field) -> None:
                    packages = payload["materialized_packages"]
                    self.assertIsInstance(packages, list)
                    package = packages[0]
                    self.assertIsInstance(package, dict)
                    package[field] = "   "

                self._assert_payload_schema_diagnostic(
                    mutate,
                    "native_plugins_payload "
                    f"materialized_packages[0].{field} "
                    "must be a non-empty string",
                    "native_plugins_payload materialized_packages are malformed",
                )

    def test_report_rejects_native_plugins_payload_materialized_package_negative_loadable_artifact_count(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["loadable_artifact_count"] = -1

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifact_count "
            "must be non-negative",
            "native_plugins_payload materialized_packages are malformed",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_duplicate_id(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            packages.append(dict(package))
            payload["package_count"] = len(packages)

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[1].package_id "
            "must be unique",
            "native_plugins_payload loader_manifest plugin ids",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_loadable_artifact_count_mismatch(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            artifacts = package["loadable_artifacts"]
            self.assertIsInstance(artifacts, list)
            package["loadable_artifact_count"] = len(artifacts) + 1

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifact_count "
            "must match loadable_artifacts length",
            "native_plugins_payload materialized_packages are malformed",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_duplicate_loadable_artifact(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            artifacts = package["loadable_artifacts"]
            self.assertIsInstance(artifacts, list)
            artifacts.append(artifacts[0])
            package["loadable_artifact_count"] = len(artifacts)

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifacts "
            "must not contain duplicate entries",
            "native_plugins_payload loadable_artifacts are not present",
        )

    def test_report_rejects_native_plugins_payload_materialized_package_blank_loadable_artifact(
        self,
    ) -> None:
        for value in ("", "   "):
            with self.subTest(value=repr(value)):
                def mutate(payload: dict[str, object], value=value) -> None:
                    packages = payload["materialized_packages"]
                    self.assertIsInstance(packages, list)
                    package = packages[0]
                    self.assertIsInstance(package, dict)
                    artifacts = package["loadable_artifacts"]
                    self.assertIsInstance(artifacts, list)
                    artifacts.append(value)

                self._assert_payload_schema_diagnostic(
                    mutate,
                    "native_plugins_payload "
                    "materialized_packages[0].loadable_artifacts "
                    "must not contain blank entries",
                    "native_plugins_payload loadable_artifacts are not present",
                )

    def test_report_rejects_native_plugins_payload_materialized_package_unsafe_loadable_artifact(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            artifacts = package["loadable_artifacts"]
            self.assertIsInstance(artifacts, list)
            artifacts[0] = "../zircon_plugin_animation.dll"

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload "
            "materialized_packages[0].loadable_artifacts[0] "
            "must be a safe relative path",
            "native_plugins_payload loadable_artifacts are not present",
        )

    def test_report_rejects_native_plugins_payload_operation_audit_non_object(
        self,
    ) -> None:
        for field in ("native_signing", "native_notarization"):
            with self.subTest(field=field):
                self._assert_payload_schema_diagnostic(
                    lambda payload, field=field: payload.__setitem__(
                        field,
                        "not-an-object",
                    ),
                    f"native_plugins_payload {field} must be an object",
                )

    def test_report_rejects_native_plugins_payload_operation_audit_field_types(
        self,
    ) -> None:
        cases = (
            ("enabled", "true", "must be a boolean"),
            ("profile", 42, "must be a string"),
            ("target_platform", 42, "must be a string"),
            ("allowed_platforms", "windows-x86_64", "must be a string array"),
            ("allowed_platforms", [42], "must be a string array"),
            ("platform_allowed", "true", "must be a boolean"),
            ("fatal", "false", "must be a boolean"),
            ("package_count", "1", "must be an integer"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                expected_diagnostic = (
                    "native_plugins_payload native_signing.allowed_platforms[0] "
                    "must be a string"
                    if field == "allowed_platforms" and isinstance(value, list)
                    else (
                        f"native_plugins_payload native_signing.{field} "
                        f"{expected_type}"
                    )
                )
                def mutate(payload: dict[str, object], field=field, value=value) -> None:
                    payload["native_signing"] = {
                        "enabled": False,
                        "profile": None,
                        "target_platform": "windows-x86_64",
                        "allowed_platforms": [],
                        "platform_allowed": True,
                        "fatal": False,
                        "package_count": 0,
                        field: value,
                    }

                self._assert_payload_schema_diagnostic(
                    mutate,
                    expected_diagnostic,
                    "native_plugins_payload native_signing is malformed",
                )

    def test_report_rejects_native_plugins_payload_operation_audit_negative_package_count(
        self,
    ) -> None:
        def mutate(payload: dict[str, object]) -> None:
            payload["native_signing"] = {
                "enabled": False,
                "profile": None,
                "target_platform": "windows-x86_64",
                "allowed_platforms": [],
                "platform_allowed": True,
                "fatal": False,
                "package_count": -1,
            }

        self._assert_payload_schema_diagnostic(
            mutate,
            "native_plugins_payload native_signing.package_count "
            "must be non-negative",
            "native_plugins_payload native_signing is malformed",
        )

    def test_report_rejects_native_plugins_payload_operation_audit_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("enabled", "must be a boolean"),
            ("allowed_platforms", "must be a string array"),
            ("platform_allowed", "must be a boolean"),
            ("fatal", "must be a boolean"),
            ("package_count", "must be an integer"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                def mutate(payload: dict[str, object], field=field) -> None:
                    payload["native_signing"] = {
                        "enabled": False,
                        "profile": None,
                        "target_platform": "windows-x86_64",
                        "allowed_platforms": [],
                        "platform_allowed": True,
                        "fatal": False,
                        "package_count": 0,
                    }
                    audit = payload["native_signing"]
                    self.assertIsInstance(audit, dict)
                    audit.pop(field)

                self._assert_payload_schema_diagnostic(
                    mutate,
                    f"native_plugins_payload native_signing.{field} {expected_type}",
                    "native_plugins_payload native_signing is malformed",
                )

    def test_report_rejects_native_plugins_payload_operation_audit_platform_allowed_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            native_report = _read_stage_report(out, "native_dynamic")

            for audit in (payload["native_signing"], native_report["native_signing"]):
                self.assertIsInstance(audit, dict)
                audit["enabled"] = True
                audit["allowed_platforms"] = ["macos"]
                audit["platform_allowed"] = True

            _write_stage_report(out, "native_dynamic", native_report)
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
                    "native_plugins_payload native_signing.platform_allowed "
                    "does not match target_platform and allowed_platforms"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_plugins_payload native_signing is malformed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
