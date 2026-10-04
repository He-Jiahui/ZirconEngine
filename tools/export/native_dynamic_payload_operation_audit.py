"""NativeDynamic payload operation-audit summary normalization."""

from __future__ import annotations

from typing import Any

from .native_signing import native_dynamic_signing_platform_allowed
from .pipeline_report_native_dynamic_operation_audit_schema import (
    native_dynamic_operation_audit_stage_schema_diagnostics,
)


def normalized_native_dynamic_operation_audit(
    value: object,
) -> dict[str, object] | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        return None
    enabled = value.get("enabled")
    profile = value.get("profile")
    target_platform = value.get("target_platform")
    allowed_platforms = value.get("allowed_platforms")
    platform_allowed = value.get("platform_allowed")
    fatal = value.get("fatal")
    package_count = value.get("package_count")
    if (
        type(enabled) is not bool
        or (profile is not None and not isinstance(profile, str))
        or (target_platform is not None and not isinstance(target_platform, str))
        or not isinstance(allowed_platforms, list)
        or any(not isinstance(platform, str) for platform in allowed_platforms)
        or type(platform_allowed) is not bool
        or type(fatal) is not bool
        or type(package_count) is not int
    ):
        return None
    return {
        "enabled": enabled,
        "profile": profile,
        "target_platform": target_platform,
        "allowed_platforms": list(allowed_platforms),
        "platform_allowed": platform_allowed,
        "fatal": fatal,
        "package_count": package_count,
    }


def normalized_native_dynamic_stage_operation_audit(
    report: dict[str, Any],
    field: str,
    *,
    expected_package_count: int,
    diagnostics: list[str] | None,
) -> dict[str, object] | None:
    value = report.get(field)
    if value is None:
        return None
    if not isinstance(value, dict):
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report {field} must be an object")
        return None
    schema_diagnostics = native_dynamic_operation_audit_stage_schema_diagnostics(
        f"NativeDynamic report {field}",
        value,
    )
    if schema_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(schema_diagnostics)
        return None
    summary = normalized_native_dynamic_operation_audit(value)
    if summary is None:
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report {field} is malformed")
        return None
    if not native_dynamic_operation_audit_is_consistent(
        summary,
        report_is_fatal=bool(report.get("fatal")),
        field=field,
        diagnostics=diagnostics,
    ):
        return None
    enabled = summary["enabled"]
    package_count = summary["package_count"]
    if enabled is True and package_count != expected_package_count:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} package_count {package_count} "
                f"does not match materialized_packages {expected_package_count}"
            )
        return None
    return summary


def native_dynamic_operation_audit_is_consistent(
    summary: dict[str, object],
    *,
    report_is_fatal: bool,
    field: str,
    diagnostics: list[str] | None,
) -> bool:
    target_platform_value = summary["target_platform"]
    target_platform = (
        target_platform_value if isinstance(target_platform_value, str) else None
    )
    allowed_platforms = list(summary["allowed_platforms"])
    if summary["enabled"] is True:
        computed_platform_allowed = native_dynamic_signing_platform_allowed(
            target_platform,
            [str(platform) for platform in allowed_platforms],
        )
        if summary["platform_allowed"] != computed_platform_allowed:
            if diagnostics is not None:
                diagnostics.append(
                    f"NativeDynamic report {field} platform_allowed "
                    "does not match target platform"
                )
            return False
    if summary["fatal"] is True and not report_is_fatal:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} is fatal but report is non-fatal"
            )
        return False
    if summary["enabled"] is True and summary["platform_allowed"] is False:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} disallows target platform"
            )
        return False
    return True
