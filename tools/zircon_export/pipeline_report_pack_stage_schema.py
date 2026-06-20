"""Pack stage report schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_delta_schema import (
    pack_delta_manifest_schema_diagnostics,
    pack_report_delta_asset_set_diagnostics,
    pack_report_delta_manifest_count_diagnostics,
    pack_report_delta_publication_diagnostics,
    pack_report_delta_target_manifest_diagnostics,
)
from .pipeline_report_pack_manifest_schema import (
    pack_document_manifest_schema_diagnostics,
    pack_report_deduplicated_assets_diagnostics,
    pack_report_manifest_count_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
)

PACK_REPORT_FIELDS = (
    "asset_count",
    "asset_manifest",
    "chunk_count",
    "deduplicated_assets",
    "delta_apply_verified",
    "delta_asset_count",
    "delta_chunk_count",
    "delta_manifest",
    "delta_pack",
    "delta_removed_assets",
    "delta_reused_assets",
    "deterministic_double_run",
    "diagnostics",
    "fatal",
    "manifest",
    "pack",
    "previous_pack",
    "profile",
    "stage",
    "stage_output",
    "trim_report",
)
PACK_REPORT_STRING_FIELDS = (
    "asset_manifest",
    "delta_pack",
    "pack",
    "previous_pack",
    "stage_output",
)
PACK_REPORT_INTEGER_FIELDS = (
    "asset_count",
    "chunk_count",
    "delta_asset_count",
    "delta_chunk_count",
)
PACK_REPORT_STRING_ARRAY_FIELDS = (
    "deduplicated_assets",
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_NO_BLANK_STRING_ARRAY_FIELDS = (
    "deduplicated_assets",
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_BOOL_FIELDS = (
    "delta_apply_verified",
    "deterministic_double_run",
)
PACK_REPORT_OBJECT_FIELDS = (
    "delta_manifest",
    "manifest",
    "trim_report",
)
PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "asset_manifest",
    "pack",
    "stage_output",
)
PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "asset_count",
    "chunk_count",
)
PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("deduplicated_assets",)
PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = ("deterministic_double_run",)
PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    "manifest",
    "trim_report",
)
PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS = (
    "delta_asset_count",
    "delta_chunk_count",
)
PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS = ("delta_pack", "previous_pack")
PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS = (
    "delta_removed_assets",
    "delta_reused_assets",
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
PACK_TRIM_REPORT_OBJECT_ARRAY_FIELDS = ("missing_dependencies", "trimmed_assets")
PACK_TRIMMED_ASSET_FIELDS = ("path", "reason")
PACK_MISSING_DEPENDENCY_FIELDS = ("dependency", "owner")
PACK_MISSING_DEPENDENCY_STRING_FIELDS = ("dependency", "owner")
PACK_TRIM_REASON_OBJECT_FIELDS = (
    "AssetFilterMismatch",
    "UnreferencedAndAssetFilterMismatch",
)

SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_report_schema_diagnostics(
    report: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    for field in PACK_REPORT_STRING_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    for field in PACK_REPORT_INTEGER_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    for field in PACK_REPORT_STRING_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            label = f"pack report {field}"
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    label,
                    report.get(field),
                )
            )
            if field in PACK_REPORT_NO_BLANK_STRING_ARRAY_FIELDS:
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        label,
                        report.get(field),
                    )
                )
    for field in PACK_REPORT_BOOL_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    for field in PACK_REPORT_OBJECT_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
            elif isinstance(report.get(field), str) and not report.get(field).strip():
                diagnostics.append(f"pack report {field} must be a non-empty string")
        for field in PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
    manifest = report.get("manifest")
    if isinstance(manifest, dict):
        diagnostics.extend(
            pack_document_manifest_schema_diagnostics(
                "pack report manifest",
                manifest,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(pack_report_manifest_count_diagnostics(report, manifest))
        diagnostics.extend(pack_report_deduplicated_assets_diagnostics(report, manifest))
    diagnostics.extend(pack_report_delta_publication_diagnostics(report))
    delta_manifest = report.get("delta_manifest")
    if (
        report.get("fatal") is False
        and isinstance(report.get("delta_pack"), str)
        and report.get("delta_pack") is not None
        and isinstance(delta_manifest, dict)
    ):
        for field in PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
            elif isinstance(report.get(field), str) and not report.get(field).strip():
                diagnostics.append(f"pack report {field} must be a non-empty string")
        for field in PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
    if isinstance(delta_manifest, dict):
        diagnostics.extend(
            pack_delta_manifest_schema_diagnostics(
                "pack report delta_manifest",
                delta_manifest,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(
            pack_report_delta_manifest_count_diagnostics(report, delta_manifest)
        )
        if isinstance(manifest, dict):
            diagnostics.extend(
                pack_report_delta_target_manifest_diagnostics(manifest, delta_manifest)
            )
        diagnostics.extend(
            pack_report_delta_asset_set_diagnostics(report, delta_manifest)
        )
    trim_report = report.get("trim_report")
    if isinstance(trim_report, dict):
        diagnostics.extend(
            pack_trim_report_schema_diagnostics(
                "pack report trim_report",
                trim_report,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        if report.get("fatal") is False:
            diagnostics.extend(
                pack_trim_report_non_fatal_preflight_diagnostics(
                    "pack report trim_report",
                    trim_report,
                )
            )
        if isinstance(manifest, dict):
            diagnostics.extend(
                pack_report_trim_manifest_consistency_diagnostics(
                    trim_report,
                    manifest,
                )
            )
    return diagnostics


def pack_trim_report_non_fatal_preflight_diagnostics(
    label: str,
    trim_report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in ("duplicate_assets", "missing_dependencies"):
        value = trim_report.get(field)
        if isinstance(value, list) and value:
            diagnostics.append(
                f"{label}.{field} must be empty for a non-fatal Pack report"
            )
    return diagnostics


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
        manifest_asset_paths.append(asset["path"])
    if any(not isinstance(asset, str) for asset in included_assets):
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
                validate_string_array_schema_diagnostics(
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
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{asset_label}.path",
                    trimmed_asset.get("path"),
                )
            )
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
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{dependency_label}.{field}",
                        missing_dependency.get(field),
                    )
                )
    return diagnostics


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
