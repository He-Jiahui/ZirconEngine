"""Pack stage report schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_delta_schema import (
    pack_delta_manifest_schema_diagnostics,
    pack_delta_manifest_is_schema_clean,
)
from .pipeline_report_pack_delta_asset_set_semantics import (
    pack_report_delta_asset_set_diagnostics,
)
from .pipeline_report_pack_delta_semantics import (
    pack_report_delta_manifest_count_diagnostics,
    pack_report_delta_publication_diagnostics,
    pack_report_delta_target_manifest_diagnostics,
)
from .pipeline_report_pack_file_evidence import (
    pack_report_binary_manifest_evidence_diagnostics,
    pack_report_file_evidence_diagnostics,
)
from .pipeline_report_pack_manifest_schema import (
    pack_document_manifest_schema_diagnostics,
    pack_document_manifest_is_schema_clean,
    pack_report_deduplicated_assets_diagnostics,
    pack_report_manifest_count_diagnostics,
)
from .pipeline_report_pack_manifest_schema_helpers import (
    non_negative_integer_diagnostics,
)
from .pipeline_report_pack_stage_required_fields import (
    pack_report_required_field_schema_diagnostics,
)
from .pipeline_report_pack_trim_schema import (
    pack_asset_path_array_schema_diagnostics,
    pack_report_trim_manifest_consistency_diagnostics,
    pack_trim_report_is_schema_clean,
    pack_trim_report_non_fatal_preflight_diagnostics,
    pack_trim_report_schema_diagnostics,
)
from .pipeline_report_pack_manifest_path_hash_schema_helpers import (
    normalized_asset_package_path,
    pack_asset_path_schema_diagnostics,
)
from .pipeline_report_schema_string_array import string_array_no_blank_entries_schema_diagnostics

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
PACK_REPORT_NO_BLANK_STRING_FIELDS = (
    "delta_pack",
    "previous_pack",
)
PACK_REPORT_TRIMMED_STRING_FIELDS = PACK_REPORT_STRING_FIELDS
PACK_REPORT_INTEGER_FIELDS = (
    "asset_count",
    "chunk_count",
    "delta_asset_count",
    "delta_chunk_count",
)
PACK_REPORT_NON_NEGATIVE_INTEGER_FIELDS = PACK_REPORT_INTEGER_FIELDS
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
PACK_REPORT_ASSET_PATH_ARRAY_FIELDS = (
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
SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_report_asset_path_array_projection(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    type_diagnostics: list[str] = []
    path_diagnostics: list[str] = []
    seen_paths: set[str] = set()
    all_strings = True
    has_blank = False
    for index, item in enumerate(value):
        if not isinstance(item, str):
            all_strings = False
            type_diagnostics.append(f"{label}[{index}] must be a string")
            continue
        if not item.strip():
            has_blank = True
            continue
        path_result = pack_asset_path_schema_diagnostics(
            f"{label}[{index}]", item
        )
        path_diagnostics.extend(path_result)
        if path_result:
            continue
        normalized_path = normalized_asset_package_path(item)
        if normalized_path in seen_paths:
            path_diagnostics.append(
                f"{label} path {normalized_path} is declared more than once"
            )
        else:
            seen_paths.add(normalized_path)
    blank_diagnostics = (
        [f"{label} must not contain blank entries"]
        if all_strings and has_blank
        else []
    )
    return type_diagnostics + blank_diagnostics + path_diagnostics


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
            value = report.get(field)
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"pack report {field}",
                    value,
                )
            )
            if (
                field in PACK_REPORT_NO_BLANK_STRING_FIELDS
                and isinstance(value, str)
                and not value.strip()
            ):
                diagnostics.append(f"pack report {field} must be a non-empty string")
            if (
                field in PACK_REPORT_TRIMMED_STRING_FIELDS
                and isinstance(value, str)
                and value.strip()
                and value.strip() != value
            ):
                diagnostics.append(
                    f"pack report {field} must be a non-empty trimmed string"
                )
    for field in PACK_REPORT_INTEGER_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
            if field in PACK_REPORT_NON_NEGATIVE_INTEGER_FIELDS:
                diagnostics.extend(
                    non_negative_integer_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
    for field in PACK_REPORT_STRING_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            label = f"pack report {field}"
            if field in PACK_REPORT_ASSET_PATH_ARRAY_FIELDS:
                diagnostics.extend(pack_report_asset_path_array_projection(label, report.get(field)))
            else:
                diagnostics.extend(pack_string_array_entry_type_schema_diagnostics(label, report.get(field)))
                if field in PACK_REPORT_NO_BLANK_STRING_ARRAY_FIELDS:
                    diagnostics.extend(string_array_no_blank_entries_schema_diagnostics(label, report.get(field)))
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
        diagnostics.extend(
            pack_report_required_field_schema_diagnostics(
                report,
                no_blank_string_fields=PACK_REPORT_NO_BLANK_STRING_FIELDS,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_integer_schema_diagnostics=validate_integer_schema_diagnostics,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
            )
        )
        diagnostics.extend(pack_report_file_evidence_diagnostics(report))
        diagnostics.extend(pack_report_binary_manifest_evidence_diagnostics(report))
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
                    pack_delta_asset_list_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        if pack_delta_manifest_is_schema_clean(delta_manifest):
            diagnostics.extend(
                pack_report_delta_manifest_count_diagnostics(report, delta_manifest)
            )
            if isinstance(manifest, dict):
                diagnostics.extend(
                    pack_report_delta_target_manifest_diagnostics(
                        manifest,
                        delta_manifest,
                    )
                )
            diagnostics.extend(
                pack_report_delta_asset_set_diagnostics(report, delta_manifest)
            )
    trim_report = report.get("trim_report")
    if isinstance(trim_report, dict):
        trim_report_schema_clean = pack_trim_report_is_schema_clean(trim_report)
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
        if report.get("fatal") is False and trim_report_schema_clean:
            diagnostics.extend(
                pack_trim_report_non_fatal_preflight_diagnostics(
                    "pack report trim_report",
                    trim_report,
                )
            )
        if (
            isinstance(manifest, dict)
            and trim_report_schema_clean
            and pack_document_manifest_is_schema_clean(manifest)
        ):
            diagnostics.extend(
                pack_report_trim_manifest_consistency_diagnostics(
                    trim_report,
                    manifest,
                )
            )
    return diagnostics


def pack_delta_asset_list_schema_diagnostics(label: str, value: Any) -> list[str]:
    return pack_string_array_entry_type_schema_diagnostics(label, value)


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
