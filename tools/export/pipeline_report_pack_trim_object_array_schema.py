"""Pack trim report object-array schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_manifest_path_hash_schema_helpers import (
    pack_asset_path_is_schema_clean,
    pack_asset_path_schema_diagnostics,
)

PACK_TRIMMED_ASSET_FIELDS = ("path", "reason")
PACK_MISSING_DEPENDENCY_FIELDS = ("dependency", "owner")
PACK_MISSING_DEPENDENCY_STRING_FIELDS = ("dependency", "owner")
PACK_TRIM_REASON_OBJECT_FIELDS = (
    "AssetFilterMismatch",
    "UnreferencedAndAssetFilterMismatch",
)

SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_trimmed_assets_are_schema_clean(value: list[Any]) -> bool:
    for trimmed_asset in value:
        if not isinstance(trimmed_asset, dict):
            return False
        if any(field not in PACK_TRIMMED_ASSET_FIELDS for field in trimmed_asset):
            return False
        path = trimmed_asset.get("path")
        if "path" in trimmed_asset and (
            not isinstance(path, str) or not pack_asset_path_is_schema_clean(path)
        ):
            return False
        reason = trimmed_asset.get("reason")
        if "reason" in trimmed_asset and not trim_reason_is_schema_clean(reason):
            return False
    return True


def trim_reason_is_schema_clean(value: Any) -> bool:
    if isinstance(value, str):
        return True
    if not isinstance(value, dict):
        return False
    return not any(field not in PACK_TRIM_REASON_OBJECT_FIELDS for field in value) and all(
        isinstance(value.get(field), str)
        for field in PACK_TRIM_REASON_OBJECT_FIELDS
        if field in value
    )


def trim_report_missing_dependencies_are_schema_clean(value: list[Any]) -> bool:
    for missing_dependency in value:
        if not isinstance(missing_dependency, dict):
            return False
        if any(field not in PACK_MISSING_DEPENDENCY_FIELDS for field in missing_dependency):
            return False
        for field in PACK_MISSING_DEPENDENCY_STRING_FIELDS:
            if field not in missing_dependency:
                continue
            path = missing_dependency.get(field)
            if not isinstance(path, str) or not pack_asset_path_is_schema_clean(path):
                return False
    return True


def pack_trimmed_assets_schema_diagnostics(
    label: str,
    trimmed_assets: list[Any],
    *,
    validate_string_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(PACK_TRIMMED_ASSET_FIELDS)
    for index, trimmed_asset in enumerate(trimmed_assets):
        if not isinstance(trimmed_asset, dict):
            continue
        asset_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{asset_label} unknown field {field}"
            for field in sorted(trimmed_asset)
            if field not in known_fields
        )
        if "path" in trimmed_asset:
            path_label = f"{asset_label}.path"
            path_value = trimmed_asset.get("path")
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    path_label,
                    path_value,
                )
            )
            diagnostics.extend(pack_optional_asset_path_schema_diagnostics(path_label, path_value))
        if "reason" in trimmed_asset:
            diagnostics.extend(
                validate_trim_reason_schema_diagnostics(
                    f"{asset_label}.reason",
                    trimmed_asset.get("reason"),
                    validate_string_schema_diagnostics=(
                        validate_string_schema_diagnostics
                    ),
                )
            )
    return diagnostics


def pack_missing_dependencies_schema_diagnostics(
    label: str,
    missing_dependencies: list[Any],
    *,
    validate_string_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(PACK_MISSING_DEPENDENCY_FIELDS)
    for index, missing_dependency in enumerate(missing_dependencies):
        if not isinstance(missing_dependency, dict):
            continue
        dependency_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{dependency_label} unknown field {field}"
            for field in sorted(missing_dependency)
            if field not in known_fields
        )
        for field in PACK_MISSING_DEPENDENCY_STRING_FIELDS:
            if field in missing_dependency:
                field_label = f"{dependency_label}.{field}"
                field_value = missing_dependency.get(field)
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        field_label,
                        field_value,
                    )
                )
                diagnostics.extend(
                    pack_optional_asset_path_schema_diagnostics(field_label, field_value)
                )
    return diagnostics


def pack_optional_asset_path_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str):
        return []
    if not value.strip():
        return [f"{label} must be a non-empty string"]
    return pack_asset_path_schema_diagnostics(label, value)


def validate_trim_reason_schema_diagnostics(
    label: str,
    value: Any,
    *,
    validate_string_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    if isinstance(value, str):
        return []
    if not isinstance(value, dict):
        return [f"{label} must be a string or object"]
    diagnostics = [
        f"{label} unknown field {field}"
        for field in sorted(value)
        if field not in PACK_TRIM_REASON_OBJECT_FIELDS
    ]
    for field in PACK_TRIM_REASON_OBJECT_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    return diagnostics
