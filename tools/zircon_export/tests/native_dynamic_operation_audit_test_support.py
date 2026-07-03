from __future__ import annotations

from pathlib import Path

from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_native_dynamic_exports,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)


def _write_native_dynamic_reports(out: Path) -> Path:
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
        "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    }
    artifact.update(overrides)
    return artifact


def _native_operation_audit_artifact_without(field: str) -> dict[str, object]:
    artifact = _native_operation_audit_artifact()
    artifact.pop(field, None)
    return artifact

