from __future__ import annotations

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


if __name__ == "__main__":
    unittest.main()
