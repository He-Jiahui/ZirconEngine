from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.native_dynamic_payload import (
    native_dynamic_content_hash,
    native_dynamic_package_payload_file_manifest,
)
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _native_dynamic_package_report_toml,
    _native_plugin_package_payload_file_manifest,
    _native_plugins_file_manifest,
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


def _native_dynamic_stage_operation_audit(**overrides: object) -> dict[str, object]:
    enabled = overrides.get("enabled", True)
    audit = {
        "enabled": True,
        "profile": "windows-store",
        "target_platform": "windows-x86_64",
        "allowed_platforms": ["windows"],
        "platform_allowed": True,
        "fatal": False,
        "package_count": 1,
        "diagnostics": [],
        "packages": [_native_dynamic_operation_audit_package()],
    }
    audit.update(overrides)
    if enabled is False and "packages" not in overrides:
        audit["packages"] = []
    return audit


def _native_dynamic_operation_audit_package(
    *,
    package_id: str = "animation",
) -> dict[str, object]:
    artifact_name = (
        "zircon_plugin_animation.dll"
        if package_id == "animation"
        else f"zircon_plugin_{package_id}.dll"
    )
    return {
        "package_id": package_id,
        "artifact_count": 1,
        "artifacts": [
            {
                "artifact": f"plugins/{package_id}/native/{artifact_name}",
                "package_relative_artifact": f"native/{artifact_name}",
                "command": ["native-operation"],
                "exit_code": 0,
                "stdout": "",
                "stderr": "",
                "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
            }
        ],
    }


def _native_dynamic_operation_audit_summary(
    audit: dict[str, object],
) -> dict[str, object]:
    return {
        "enabled": audit["enabled"],
        "profile": audit["profile"],
        "target_platform": audit["target_platform"],
        "allowed_platforms": audit["allowed_platforms"],
        "platform_allowed": audit["platform_allowed"],
        "fatal": audit["fatal"],
        "package_count": audit["package_count"],
    }


class NativeDynamicPayloadReportValidationTests(unittest.TestCase):
    def test_report_rejects_missing_native_plugins_payload_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit()
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload.pop("native_signing", None)
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
                    "native_plugins_payload native_signing"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_missing_native_plugins_payload_notarization_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_notarization"] = (
                _native_dynamic_stage_operation_audit(
                    profile="windows-attestation",
                )
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload.pop("native_notarization", None)
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
                    "native_plugins_payload native_notarization"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_malformed_native_dynamic_report_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = {
                "enabled": True,
                "profile": "windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": "windows",
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
            }
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
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
                    "native_dynamic report native_signing.allowed_platforms must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "NativeDynamic report native_signing is malformed" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_malformed_native_dynamic_report_notarization_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_notarization"] = {
                "enabled": True,
                "profile": "windows-attestation",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": "yes",
                "fatal": False,
                "package_count": 1,
            }
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
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
                    "native_dynamic report native_notarization.platform_allowed must be a boolean"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "NativeDynamic report native_notarization is malformed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_dynamic_report_signing_package_count_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit(
                package_count=2,
                packages=[_native_dynamic_operation_audit_package()],
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = _native_dynamic_operation_audit_summary(
                native_report["native_signing"]
            )
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
                    "native_dynamic report native_signing.package_count 2 "
                    "does not match native_signing.packages 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertNotIn("native_plugins_payload", report)

    def test_report_accepts_disabled_native_dynamic_report_signing_placeholder(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit(
                enabled=False,
                profile=None,
                allowed_platforms=[],
                package_count=0,
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = _native_dynamic_operation_audit_summary(
                native_report["native_signing"]
            )
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                report["native_plugins_payload"]["native_signing"]["package_count"],
                0,
            )

    def test_report_rejects_fatal_native_dynamic_report_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit(
                fatal=True,
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = _native_dynamic_operation_audit_summary(
                native_report["native_signing"]
            )
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
                    "NativeDynamic report native_signing is fatal but report is non-fatal"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_disallowed_native_dynamic_report_signing_platform(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit(
                profile="macos-store",
                allowed_platforms=["macos"],
                platform_allowed=False,
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = _native_dynamic_operation_audit_summary(
                native_report["native_signing"]
            )
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
                    "NativeDynamic report native_signing disallows target platform"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_spoofed_native_dynamic_report_signing_platform_allowed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["native_signing"] = _native_dynamic_stage_operation_audit(
                profile="macos-store",
                allowed_platforms=["macos"],
                platform_allowed=True,
            )
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = _native_dynamic_operation_audit_summary(
                native_report["native_signing"]
            )
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
                    "native_dynamic report native_signing.platform_allowed "
                    "does not match target_platform and allowed_platforms"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_spoofed_native_plugins_payload_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_signing"] = {
                "enabled": True,
                "profile": "forged-windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
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
                    "native_plugins_payload native_signing"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_spoofed_native_plugins_payload_notarization_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(
                out / "stages" / "native_dynamic" / "report.json"
            )
            payload["source"] = str(out / "stages" / "native_dynamic" / "plugins")
            payload["native_notarization"] = {
                "enabled": True,
                "profile": "forged-notary",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
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
                    "native_plugins_payload native_notarization"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_package_id_mismatch(
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
            package_report.write_text('package_id = "forged-animation"\n', encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "package_id forged-animation does not match"
                    in diagnostic
                    and "animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_stage_backed_native_plugins_payload_missing_package_report(
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
            package.pop("package_report")
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
                    "native_plugins_payload materialized_packages[0] "
                    "package_report is required for stage-backed payloads"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_stage_backed_native_plugins_payload_package_id_drift(
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
            package["package_id"] = "physics"
            native_plugins = fixture["native_plugins"]
            loader_manifest = native_plugins / "native_plugins.toml"
            loader_manifest.write_text(
                loader_manifest.read_text(encoding="utf-8").replace(
                    'id = "animation"',
                    'id = "physics"',
                ),
                encoding="utf-8",
            )
            package_report = Path(str(package["package_report"]))
            package_report.write_text(
                package_report.read_text(encoding="utf-8").replace(
                    'package_id = "animation"',
                    'package_id = "physics"',
                ),
                encoding="utf-8",
            )
            file_manifest = _native_plugins_file_manifest(native_plugins)
            payload["file_manifest"] = file_manifest
            payload["content_hash"] = native_dynamic_content_hash(file_manifest)
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
                    "native_plugins_payload materialized package ids ['physics'] "
                    "do not match NativeDynamic report materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_stage_backed_native_plugins_payload_file_manifest_drift(
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
            native_plugins = fixture["native_plugins"]
            package_dir = native_plugins / "animation"
            artifact = package_dir / "native" / "zircon_plugin_animation.dll"
            artifact.write_text("forged plugin dll", encoding="utf-8")
            package_report = Path(str(package["package_report"]))
            package_report.write_text(
                _native_dynamic_package_report_toml(
                    _native_plugin_package_payload_file_manifest(package_dir)
                ),
                encoding="utf-8",
            )
            file_manifest = _native_plugins_file_manifest(native_plugins)
            payload["file_manifest"] = file_manifest
            payload["file_count"] = len(file_manifest)
            payload["content_hash"] = native_dynamic_content_hash(file_manifest)
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
                    "native_plugins_payload file_manifest does not match "
                    "NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_legacy_native_dynamic_stage_report_schema_before_payload_semantics(
        self,
    ) -> None:
        cases = (
            (
                "file_manifest_path",
                lambda native_report: native_report["file_manifest"][0].__setitem__(
                    "path",
                    f' {native_report["file_manifest"][0]["path"]} ',
                ),
                "native_dynamic report file_manifest[0].path "
                "must be a non-empty trimmed string",
                "native_plugins_payload file_manifest does not match "
                "NativeDynamic report",
            ),
            (
                "materialized_package_id",
                lambda native_report: native_report["materialized_packages"][0].__setitem__(
                    "package_id",
                    f' {native_report["materialized_packages"][0]["package_id"]} ',
                ),
                "native_dynamic report materialized_packages[0].package_id "
                "must be a non-empty trimmed string",
                "native_plugins_payload materialized package ids",
            ),
            (
                "content_hash",
                lambda native_report: native_report.__setitem__(
                    "content_hash",
                    "not-a-hash",
                ),
                "native_dynamic report content_hash must be a SHA-256 hex digest",
                "native_plugins_payload content_hash does not match "
                "NativeDynamic report",
            ),
        )
        for name, mutate_native_report, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(name=name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out)
                    validate_report = _read_stage_report(out, "validate")
                    profile_summary = validate_report["profile_summary"]
                    self.assertIsInstance(profile_summary, dict)
                    profile_summary.pop("strategies")
                    _write_stage_report(out, "validate", validate_report)
                    native_report = _read_stage_report(out, "native_dynamic")
                    mutate_native_report(native_report)
                    _write_stage_report(out, "native_dynamic", native_report)
                    platform_report = _read_stage_report(out, "platform_bundle")
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
                    self.assertFalse(
                        any(
                            unexpected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_legacy_native_dynamic_operation_audit_schema_before_payload_semantics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            profile_summary = validate_report["profile_summary"]
            self.assertIsInstance(profile_summary, dict)
            profile_summary.pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            native_report = _read_stage_report(out, "native_dynamic")
            native_signing = native_report["native_signing"]
            self.assertIsInstance(native_signing, dict)
            native_signing["allowed_platforms"] = "windows"
            _write_stage_report(out, "native_dynamic", native_report)
            platform_report = _read_stage_report(out, "platform_bundle")
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_dynamic report native_signing.allowed_platforms "
                    "must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "NativeDynamic report native_signing is malformed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_payload_count_mismatch(
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
                        "",
                        "[payload]",
                        "file_count = 2",
                        f'content_hash = "{"0" * 64}"',
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
                    and "payload file_count 2 does not match current package payload 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_payload_file_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("path", "must be a string"),
            ("sha256", "must be a string"),
            ("bytes", "must be an integer"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
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
                        package_report.parent
                    )
                    self.assertEqual(len(file_manifest), 1)
                    payload_file = dict(file_manifest[0])
                    file_fields = {
                        "path": f'"{payload_file["path"]}"',
                        "bytes": str(payload_file["bytes"]),
                        "sha256": f'"{payload_file["sha256"]}"',
                    }
                    file_fields.pop(field)
                    payload_file_lines = [
                        "[[payload.files]]",
                        *(f"{key} = {value}" for key, value in file_fields.items()),
                    ]
                    package_report.write_text(
                        "\n".join(
                            [
                                "format_version = 1",
                                'package_id = "animation"',
                                'directory = "animation"',
                                'path = "plugins/animation"',
                                'manifest = "plugins/animation/plugin.toml"',
                                "",
                                "[payload]",
                                "file_count = 1",
                                f'content_hash = "{native_dynamic_content_hash(file_manifest)}"',
                                "",
                                *payload_file_lines,
                                "",
                            ]
                        ),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            "native_plugins_payload materialized_packages[0] package_report"
                            in diagnostic
                            and f"payload files[0].{field} {expected_type}"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_directory_mismatch(
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
                        'directory = "forged-animation"',
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
                    and "directory forged-animation does not match"
                    in diagnostic
                    and "animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_path_mismatch(
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
                        'path = "plugins/forged-animation"',
                        'manifest = "plugins/animation/plugin.toml"',
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
                    and "path plugins/forged-animation does not match"
                    in diagnostic
                    and "plugins/animation" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_format_version_mismatch(
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
                        "format_version = 999",
                        'package_id = "animation"',
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
                    and "format_version 999 is not supported; expected 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_abi_version_mismatch(
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
                        "",
                        "[abi]",
                        "abi_version = 2",
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
                    and "abi.abi_version must be 3"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)


if __name__ == "__main__":
    unittest.main()
