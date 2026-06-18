from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _export_args,
    _run_pipeline_quiet,
    _write_compile_host_report,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
    json_loads,
)


class PlatformBundleNativeDynamicOperationAuditTests(unittest.TestCase):
    def test_pipeline_platform_bundle_rejects_malformed_native_dynamic_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(
                    allowed_platforms="windows",
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_signing.allowed_platforms must be a string array"
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
            self.assertFalse((out / "bundle" / "windows-release").exists())

    def test_pipeline_platform_bundle_rejects_native_dynamic_signing_package_count_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(
                    package_count=2,
                    packages=[
                        _native_dynamic_operation_audit_package(),
                        _native_dynamic_operation_audit_package(package_id="extra"),
                    ],
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_signing package_count 2 "
                    "does not match materialized_packages 1" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())

    def test_pipeline_platform_bundle_accepts_disabled_native_dynamic_signing_placeholder(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(
                    enabled=False,
                    profile=None,
                    allowed_platforms=[],
                    package_count=0,
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(
                report["native_plugins_payload"]["native_signing"]["package_count"],
                0,
            )

    def test_pipeline_platform_bundle_rejects_fatal_native_dynamic_signing_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(fatal=True),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_signing is fatal but report is non-fatal"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())

    def test_pipeline_platform_bundle_rejects_disallowed_native_dynamic_signing_platform(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(
                    profile="macos-store",
                    allowed_platforms=["macos"],
                    platform_allowed=False,
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_signing disallows target platform"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())

    def test_pipeline_platform_bundle_rejects_spoofed_native_dynamic_signing_platform_allowed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_signing=_native_dynamic_stage_operation_audit(
                    profile="macos-store",
                    allowed_platforms=["macos"],
                    platform_allowed=True,
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_signing platform_allowed "
                    "does not match target platform" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse((out / "bundle" / "windows-release").exists())

    def test_pipeline_platform_bundle_rejects_malformed_native_dynamic_notarization_audit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out, native_plugins = _write_pipeline_handoff_fixture(Path(temp_dir))
            _write_native_dynamic_report(
                out,
                native_plugins,
                native_notarization=_native_dynamic_stage_operation_audit(
                    profile="windows-attestation",
                    platform_allowed="yes",
                ),
            )

            exit_code, report = _run_platform_bundle_from_native_dynamic(out)

            self.assertEqual(exit_code, 2)
            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertTrue(
                any(
                    "NativeDynamic report native_notarization.platform_allowed must be a boolean"
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
            self.assertFalse((out / "bundle" / "windows-release").exists())


def _write_pipeline_handoff_fixture(root: Path) -> tuple[Path, Path]:
    out = root / "out"
    host = root / "compile" / "zircon_runtime.exe"
    host.parent.mkdir(parents=True)
    host.write_text("host placeholder", encoding="utf-8")
    pack = root / "pack-output" / "assets.zrpack"
    pack.parent.mkdir(parents=True)
    pack.write_text("pack placeholder", encoding="utf-8")
    native_plugins = _write_native_dynamic_stage_plugins(
        out / "stages" / "native_dynamic"
    )
    _write_validate_report_with_strategies(out, ["native_dynamic"])
    _write_compile_host_report(out, host)
    _write_stage_report(out, "cook_assets", fatal=False)
    _write_pack_report(out, pack)
    return out, native_plugins


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
                "before_sha256": "before-hash",
                "after_sha256": "after-hash",
            }
        ],
    }


def _run_platform_bundle_from_native_dynamic(out: Path) -> tuple[int, dict[str, object]]:
    exit_code = _run_pipeline_quiet(
        _export_args(out=out, stage="platform_bundle", dry_run=False),
        "platform_bundle",
    )
    report = json_loads(
        (out / "stages" / "platform_bundle" / "report.json").read_text(
            encoding="utf-8"
        )
    )
    return exit_code, report


if __name__ == "__main__":
    unittest.main()
