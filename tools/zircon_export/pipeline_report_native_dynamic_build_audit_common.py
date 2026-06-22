"""Shared NativeDynamic build audit schema diagnostics."""

from __future__ import annotations

from typing import Any

def native_dynamic_fatal_report_diagnostics(
    label: str,
    report: dict[str, Any],
) -> list[str]:
    if report.get("fatal") is not True:
        return []
    diagnostics = report.get("diagnostics")
    if isinstance(diagnostics, list) and any(
        isinstance(diagnostic, str) and diagnostic.strip()
        for diagnostic in diagnostics
    ):
        return []
    return [f"{label} fatal report must include diagnostics"]


def native_dynamic_non_fatal_report_diagnostics(
    label: str,
    report: dict[str, Any],
) -> list[str]:
    if report.get("fatal") is not False:
        return []
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, list):
        return []
    if not any(
        isinstance(diagnostic, str) and diagnostic.strip()
        for diagnostic in diagnostics
    ):
        return []
    return [f"{label}.diagnostics must be empty when fatal is False"]


def native_dynamic_build_audit_package_count_diagnostics(
    label: str,
    report: dict[str, Any],
) -> list[str]:
    package_count = report.get("package_count")
    packages = report.get("packages")
    if not (type(package_count) is int and isinstance(packages, list)):
        return []
    if package_count < 0:
        return []
    if package_count == len(packages):
        return []
    return [
        f"{label}.package_count {package_count} does not match "
        f"{label}.packages {len(packages)}"
    ]


def native_dynamic_build_audit_package_id_uniqueness_diagnostics(
    label: str,
    packages: list[Any],
) -> list[str]:
    diagnostics: list[str] = []
    seen: set[str] = set()
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        if not isinstance(package_id, str) or not package_id.strip():
            continue
        if package_id.strip() != package_id:
            continue
        normalized_package_id = package_id
        if normalized_package_id in seen:
            diagnostics.append(
                f"{label} package_id {normalized_package_id} must be unique"
            )
            continue
        seen.add(normalized_package_id)
    return diagnostics


def table_non_negative_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, int) and value < 0:
            diagnostics.append(f"{label}.{field} must be non-negative")
    return diagnostics


def string_array_unique_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(entry, str) for entry in value):
        return []

    seen: set[str] = set()
    for entry in value:
        if not entry.strip() or entry.strip() != entry:
            continue
        if entry in seen:
            return [f"{label} must not contain duplicate entries"]
        seen.add(entry)
    return []

