"""NativeDynamic package export projection diagnostics for final reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_contract import NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
from .pipeline_report_native_dynamic_package_export_schema import (
    native_dynamic_package_export_schema_diagnostics,
)


def validate_native_dynamic_package_export_ids(
    validate_payload: dict[str, Any] | None,
) -> list[str] | None:
    package_exports = validate_native_dynamic_package_exports(validate_payload)
    if package_exports is None:
        return None
    return [
        str(package_export["package_id"]) for package_export in package_exports
    ]


def validate_native_dynamic_package_exports(
    validate_payload: dict[str, Any] | None,
) -> list[dict[str, Any]] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    return schema_clean_native_dynamic_package_exports(
        plan_summary.get("native_dynamic_package_exports"),
        "validate report plan_summary.native_dynamic_package_exports",
    )


def schema_clean_native_dynamic_package_exports(
    package_exports: object,
    label: str,
) -> list[dict[str, Any]] | None:
    normalized_package_exports = normalized_native_dynamic_package_exports(
        package_exports
    )
    if normalized_package_exports is None:
        return None
    if native_dynamic_package_export_schema_diagnostics(
        label,
        normalized_package_exports,
    ):
        return None
    return normalized_package_exports


def normalized_native_dynamic_package_exports(
    package_exports: object,
) -> list[dict[str, Any]] | None:
    if not isinstance(package_exports, list) or not all(
        isinstance(package_export, dict)
        and isinstance(package_export.get("package_id"), str)
        for package_export in package_exports
    ):
        return None
    return [dict(package_export) for package_export in package_exports]


def native_dynamic_package_export_materialization_diagnostics(
    package_exports: list[dict[str, Any]],
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    label: str,
) -> list[str]:
    expected_exports_by_id = materialized_package_exports_by_id(
        materialized_packages,
        plugins_dir,
    )
    diagnostics: list[str] = []
    for package_export in package_exports:
        package_id = str(package_export["package_id"])
        expected_export = expected_exports_by_id.get(package_id)
        if expected_export is None:
            continue
        for field, expected_value in expected_export.items():
            value = package_export.get(field)
            if isinstance(value, str) and value != expected_value:
                diagnostics.append(
                    f"{label} package {package_id} {field} {value} "
                    f"does not match materialized package {field} "
                    f"{expected_value}"
                )
    return diagnostics


def materialized_package_exports_by_id(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, dict[str, str]]:
    package_exports: dict[str, dict[str, str]] = {}
    try:
        plugins_root = plugins_dir.resolve()
    except OSError:
        return package_exports

    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(plugins_root)
        except (OSError, ValueError):
            continue
        directory = relative_destination.as_posix().rstrip("/")
        path = f"plugins/{directory}"
        package_exports[package_id] = {
            "directory": directory,
            "path": path,
            "manifest": f"{path}/plugin.toml",
            "package_report": f"{path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}",
        }
    return package_exports
