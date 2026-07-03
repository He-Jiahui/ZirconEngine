"""Schema diagnostics for NativeDynamic operation audit evidence."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_operation_audit_schema_helpers import (
    operation_audit_platform_allowed_schema_diagnostics,
    string_array_unique_entries_schema_diagnostics,
    table_non_negative_integer_schema_diagnostics,
    table_required_non_empty_string_schema_diagnostics,
    table_required_trimmed_non_empty_string_schema_diagnostics,
)
from .pipeline_report_native_dynamic_operation_audit_summary_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
    NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS,
    operation_audit_allowed_platforms_schema_diagnostics,
)
from .pipeline_report_native_dynamic_operation_audit_stage_packages_schema import (
    native_dynamic_operation_audit_stage_packages_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    optional_fields,
    table_bool_schema_diagnostics,
    table_field_schema_diagnostics,
    table_integer_schema_diagnostics,
    table_string_array_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)
from .pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)

NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS = (
    "native_signing",
    "native_notarization",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_FIELDS = (
    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS,
    "diagnostics",
    "packages",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_STRING_ARRAY_FIELDS = (
    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
    "diagnostics",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS = (
    "diagnostics",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_OBJECT_ARRAY_FIELDS = (
    "packages",
)

def native_dynamic_operation_audit_stage_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        audit,
        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_FIELDS,
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        operation_audit_allowed_platforms_schema_diagnostics(
            label,
            audit,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_trimmed_non_empty_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_unique_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_trimmed_non_empty_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(operation_audit_platform_allowed_schema_diagnostics(label, audit))
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_STRING_ARRAY_FIELDS,
                (
                    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
                    *NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
                ),
            ),
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
            string_array_trimmed_non_empty_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        native_dynamic_operation_audit_stage_packages_schema_diagnostics(
            label,
            audit,
        )
    )
    return diagnostics
