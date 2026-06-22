"""Pack trim report schema and release-evidence diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_manifest_schema import (
    is_safe_asset_package_path,
    normalized_asset_package_path,
    pack_asset_path_is_schema_clean,
    pack_asset_path_schema_diagnostics,
    pack_document_manifest_is_schema_clean,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
)

PACK_TRIM_REPORT_FIELDS = (
    "diagnostics",
    "duplicate_assets",
    "included_assets",
    "missing_dependencies",
    "trimmed_assets",
)
PACK_TRIM_REPORT_STRING_ARRAY_FIELDS = (
    "diagnostics",
    "duplicate_assets",
    "included_assets",
)
PACK_TRIM_REPORT_ASSET_PATH_ARRAY_FIELDS = (
    "duplicate_assets",
    "included_assets",
)
PACK_TRIM_REPORT_OBJECT_ARRAY_FIELDS = ("missing_dependencies", "trimmed_assets")
PACK_TRIMMED_ASSET_FIELDS = ("path", "reason")
PACK_MISSING_DEPENDENCY_FIELDS = ("dependency", "owner")
PACK_MISSING_DEPENDENCY_STRING_FIELDS = ("dependency", "owner")
PACK_TRIM_REASON_OBJECT_FIELDS = (
    "AssetFilterMismatch",
    "UnreferencedAndAssetFilterMismatch",
)

SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_trim_report_non_fatal_preflight_diagnostics(
    label: str,
    trim_report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    duplicate_assets = trim_report.get("duplicate_assets")
    if (
        isinstance(duplicate_assets, list)
        and duplicate_assets
        and trim_report_duplicate_assets_are_schema_clean(duplicate_assets)
    ):
        diagnostics.append(
            f"{label}.duplicate_assets must be empty for a non-fatal Pack report"
        )
    missing_dependencies = trim_report.get("missing_dependencies")
    if (
        isinstance(missing_dependencies, list)
        and missing_dependencies
        and trim_report_missing_dependencies_are_schema_clean(missing_dependencies)
    ):
        diagnostics.append(
            f"{label}.missing_dependencies must be empty for a non-fatal Pack report"
        )
    return diagnostics


def pack_trim_report_is_schema_clean(trim_report: dict[str, Any]) -> bool:
    if any(field not in PACK_TRIM_REPORT_FIELDS for field in trim_report):
        return False
    for field in PACK_TRIM_REPORT_STRING_ARRAY_FIELDS:
        if field not in trim_report:
            continue
        value = trim_report.get(field)
        if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
            return False
        if field == "diagnostics" and any(not item.strip() for item in value):
            return False
        if (
            field in PACK_TRIM_REPORT_ASSET_PATH_ARRAY_FIELDS
            and not pack_asset_path_list_is_schema_clean(value)
        ):
            return False
    for field in PACK_TRIM_REPORT_OBJECT_ARRAY_FIELDS:
        if field in trim_report and (
            not isinstance(trim_report.get(field), list)
            or any(not isinstance(item, dict) for item in trim_report.get(field))
        ):
            return False
    trimmed_assets = trim_report.get("trimmed_assets")
    if isinstance(trimmed_assets, list) and not pack_trimmed_assets_are_schema_clean(
        trimmed_assets
    ):
        return False
    missing_dependencies = trim_report.get("missing_dependencies")
    if (
        isinstance(missing_dependencies, list)
        and not trim_report_missing_dependencies_are_schema_clean(missing_dependencies)
    ):
        return False
    return True


def pack_asset_path_list_is_schema_clean(value: list[str]) -> bool:
    seen_paths: set[str] = set()
    for path in value:
        if not path.strip() or not pack_asset_path_is_schema_clean(path):
            return False
        normalized_path = normalized_asset_package_path(path)
        if normalized_path in seen_paths:
            return False
        seen_paths.add(normalized_path)
    return True


def trim_report_duplicate_assets_are_schema_clean(value: list[Any]) -> bool:
    return all(
        isinstance(path, str) and pack_asset_path_is_schema_clean(path)
        for path in value
    )


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


def pack_report_trim_manifest_consistency_diagnostics(
    trim_report: dict[str, Any],
    manifest: dict[str, Any],
) -> list[str]:
    assets = manifest.get("assets")
    included_assets = trim_report.get("included_assets")
    if not isinstance(assets, list) or not isinstance(included_assets, list):
        return []
    manifest_asset_paths: list[str] = []
    for asset in assets:
        if not isinstance(asset, dict) or not isinstance(asset.get("path"), str):
            return []
        if not pack_asset_path_is_schema_clean(asset["path"]):
            return []
        manifest_asset_paths.append(asset["path"])
    if any(
        not isinstance(asset, str) or not pack_asset_path_is_schema_clean(asset)
        for asset in included_assets
    ):
        return []
    if sorted(included_assets) == sorted(manifest_asset_paths):
        return []
    return [
        "pack report trim_report.included_assets does not match "
        "manifest.assets paths"
    ]


def pack_trim_report_schema_diagnostics(
    label: str,
    trim_report: dict[str, Any],
    *,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(trim_report)
        if field not in PACK_TRIM_REPORT_FIELDS
    )
    for field in PACK_TRIM_REPORT_STRING_ARRAY_FIELDS:
        if field in trim_report:
            diagnostics.extend(
                pack_string_array_entry_type_schema_diagnostics(
                    f"{label}.{field}",
                    trim_report.get(field),
                )
            )
            if field == "diagnostics":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        f"{label}.{field}",
                        trim_report.get(field),
                    )
                )
            elif field in PACK_TRIM_REPORT_ASSET_PATH_ARRAY_FIELDS:
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        f"{label}.{field}",
                        trim_report.get(field),
                    )
                )
                diagnostics.extend(
                    pack_asset_path_array_schema_diagnostics(
                        f"{label}.{field}",
                        trim_report.get(field),
                    )
                )
    for field in PACK_TRIM_REPORT_OBJECT_ARRAY_FIELDS:
        if field in trim_report:
            diagnostics.extend(
                validate_object_array_schema_diagnostics(
                    f"{label}.{field}",
                    trim_report.get(field),
                )
            )
    trimmed_assets = trim_report.get("trimmed_assets")
    if isinstance(trimmed_assets, list):
        diagnostics.extend(
            pack_trimmed_assets_schema_diagnostics(
                f"{label}.trimmed_assets",
                trimmed_assets,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
            )
        )
    missing_dependencies = trim_report.get("missing_dependencies")
    if isinstance(missing_dependencies, list):
        diagnostics.extend(
            pack_missing_dependencies_schema_diagnostics(
                f"{label}.missing_dependencies",
                missing_dependencies,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
            )
        )
    return diagnostics


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


def pack_asset_path_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    seen_paths: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            continue
        item_label = f"{label}[{index}]"
        path_diagnostics = pack_asset_path_schema_diagnostics(item_label, item)
        diagnostics.extend(path_diagnostics)
        if path_diagnostics:
            continue
        if not is_safe_asset_package_path(item):
            continue
        normalized_path = normalized_asset_package_path(item)
        if normalized_path in seen_paths:
            diagnostics.append(
                f"{label} path {normalized_path} is declared more than once"
            )
        else:
            seen_paths.add(normalized_path)
    return diagnostics


def pack_string_array_entry_type_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, item in enumerate(value)
        if not isinstance(item, str)
    ]


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
