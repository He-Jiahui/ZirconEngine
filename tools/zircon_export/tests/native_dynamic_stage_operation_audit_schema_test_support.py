from __future__ import annotations


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


def _native_operation_audit_without(field: str) -> dict[str, object]:
    audit = _native_operation_audit()
    audit.pop(field, None)
    return audit


def _disabled_native_operation_audit_without(field: str) -> dict[str, object]:
    audit = _native_operation_audit(
        enabled=False,
        profile=None,
        allowed_platforms=[],
        package_count=0,
        packages=[],
    )
    audit.pop(field, None)
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
        "artifact": "E:/tmp/out/stages/native_dynamic/plugins/animation/native/plugin.dll",
        "package_relative_artifact": "native/plugin.dll",
        "command": ["signtool", "sign", "native/plugin.dll"],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    }
    artifact.update(overrides)
    return artifact