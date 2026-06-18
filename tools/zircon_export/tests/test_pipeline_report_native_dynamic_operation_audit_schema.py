from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_native_dynamic_exports,
)


class PipelineReportNativeDynamicOperationAuditSchemaTests(unittest.TestCase):
    def _write_native_dynamic_reports(self, out: Path) -> Path:
        _write_validate_report_with_native_dynamic_exports(out)
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "native_dynamic" / "report.json"

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field(
        self,
    ) -> None:
        cases = (
            ("exit_code", "must be an integer"),
            ("before_sha256", "must be a string"),
            ("after_sha256", "must be a string"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(
                                artifacts=[
                                    _native_operation_audit_artifact_without(field)
                                ]
                            )
                        ]
                    )
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            "native_dynamic report native_signing packages[0] "
                            f"artifacts[0].{field} {expected_type}" in diagnostic
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


def _native_operation_audit(**overrides: object) -> dict[str, object]:
    audit = {
        "enabled": True,
        "profile": "windows-store",
        "target_platform": "windows-x86_64",
        "allowed_platforms": ["windows"],
        "platform_allowed": True,
        "fatal": False,
        "package_count": 1,
        "diagnostics": [],
        "packages": [_native_operation_audit_package()],
    }
    audit.update(overrides)
    return audit


def _native_operation_audit_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "artifact_count": 1,
        "artifacts": [_native_operation_audit_artifact()],
    }
    package.update(overrides)
    return package


def _native_operation_audit_artifact(**overrides: object) -> dict[str, object]:
    artifact = {
        "artifact": "plugins/animation/native/zircon_plugin_animation.dll",
        "package_relative_artifact": "native/zircon_plugin_animation.dll",
        "command": ["signtool", "sign", "native/zircon_plugin_animation.dll"],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "before_sha256": "before-hash",
        "after_sha256": "after-hash",
    }
    artifact.update(overrides)
    return artifact


def _native_operation_audit_artifact_without(field: str) -> dict[str, object]:
    artifact = _native_operation_audit_artifact()
    artifact.pop(field, None)
    return artifact


if __name__ == "__main__":
    unittest.main()
