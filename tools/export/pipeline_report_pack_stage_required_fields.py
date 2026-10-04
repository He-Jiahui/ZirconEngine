"""Required-field diagnostics for Pack stage reports."""

from __future__ import annotations

from typing import Any, Callable

PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = ("asset_manifest", "pack", "stage_output")
PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = ("asset_count", "chunk_count")
PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("deduplicated_assets",)
PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = ("deterministic_double_run",)
PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = ("manifest", "trim_report")
PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS = ("delta_asset_count", "delta_chunk_count")
PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS = ("delta_pack", "previous_pack")
PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS = ("delta_removed_assets", "delta_reused_assets")
PACK_REPORT_REQUIRED_DELTA_TRUE_BOOL_FIELDS = ("delta_apply_verified",)
SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_report_required_field_schema_diagnostics(
    report: dict[str, Any],
    *,
    no_blank_string_fields: tuple[str, ...],
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    if report.get("fatal") is not False:
        return []
    diagnostics: list[str] = []
    diagnostics.extend(
        _required_string_diagnostics(
            report,
            PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS,
            no_blank_string_fields,
            validate_string_schema_diagnostics,
        )
    )
    for fields, validator in (
        (PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS, validate_integer_schema_diagnostics),
        (
            PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS,
            validate_string_array_schema_diagnostics,
        ),
        (PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS, validate_bool_schema_diagnostics),
    ):
        diagnostics.extend(_missing_field_diagnostics(report, fields, validator))
    diagnostics.extend(
        _missing_field_diagnostics(
            report,
            PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS,
            validate_object_schema_diagnostics,
            require_non_none=True,
        )
    )
    if _publishes_delta(report):
        diagnostics.extend(
            _required_string_diagnostics(
                report,
                PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS,
                (),
                validate_string_schema_diagnostics,
            )
        )
        for fields, validator in (
            (PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS, validate_integer_schema_diagnostics),
            (
                PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS,
                validate_string_array_schema_diagnostics,
            ),
        ):
            diagnostics.extend(_missing_field_diagnostics(report, fields, validator))
        diagnostics.extend(
            _required_true_bool_diagnostics(
                report,
                PACK_REPORT_REQUIRED_DELTA_TRUE_BOOL_FIELDS,
                validate_bool_schema_diagnostics,
            )
        )
    return diagnostics


def _required_string_diagnostics(
    report: dict[str, Any],
    fields: tuple[str, ...],
    no_blank_string_fields: tuple[str, ...],
    validate_string_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = report.get(field)
        if field not in report or value is None:
            diagnostics.extend(
                validate_string_schema_diagnostics(f"pack report {field}", value)
            )
        elif (
            field not in no_blank_string_fields
            and isinstance(value, str)
            and not value.strip()
        ):
            diagnostics.append(f"pack report {field} must be a non-empty string")
    return diagnostics


def _missing_field_diagnostics(
    report: dict[str, Any],
    fields: tuple[str, ...],
    validate_schema_diagnostics: SchemaDiagnostic,
    *,
    require_non_none: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        if field not in report or (require_non_none and report.get(field) is None):
            diagnostics.extend(
                validate_schema_diagnostics(f"pack report {field}", report.get(field))
            )
    return diagnostics


def _required_true_bool_diagnostics(
    report: dict[str, Any],
    fields: tuple[str, ...],
    validate_bool_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = report.get(field)
        if field not in report or value is None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(f"pack report {field}", value)
            )
        elif isinstance(value, bool) and value is not True:
            diagnostics.append(
                f"pack report {field} must be true when delta_pack is published"
            )
    return diagnostics


def _publishes_delta(report: dict[str, Any]) -> bool:
    delta_pack = report.get("delta_pack")
    return (
        isinstance(delta_pack, str)
        and delta_pack.strip()
        and delta_pack.strip() == delta_pack
        and isinstance(report.get("delta_manifest"), dict)
    )
