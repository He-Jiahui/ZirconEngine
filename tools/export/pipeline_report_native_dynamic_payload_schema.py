"""Schema diagnostics for NativeDynamic payload evidence in final reports."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
    native_dynamic_operation_audit_stage_schema_diagnostics,
)
from .pipeline_report_native_dynamic_operation_audit_summary_schema import (
    platform_bundle_native_plugins_operation_audit_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_file_manifest_schema import (
    platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_materialized_packages_schema import (
    platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_schema_helpers import (
    table_non_empty_string_schema_diagnostics,
    table_non_negative_integer_schema_diagnostics,
    table_sha256_hex_string_schema_diagnostics,
    table_trimmed_non_empty_string_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    optional_fields,
    table_integer_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)

NATIVE_DYNAMIC_PAYLOAD_FIELDS = (
    "bundle_path",
    "content_hash",
    "file_count",
    "file_manifest",
    "loader_manifest",
    "materialized_packages",
    "package_count",
    "source",
    "stage_report",
    *NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
)

NATIVE_DYNAMIC_PAYLOAD_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
    "stage_report",
)

NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
)
NATIVE_DYNAMIC_PAYLOAD_CONTENT_HASH_FIELDS = ("content_hash",)
NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
    "stage_report",
)

NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS = (
    "file_count",
    "package_count",
)
NATIVE_DYNAMIC_PAYLOAD_REQUIRED_INTEGER_FIELDS = NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS

def platform_bundle_native_plugins_payload_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        payload,
        NATIVE_DYNAMIC_PAYLOAD_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            payload,
            optional_fields(
                NATIVE_DYNAMIC_PAYLOAD_STRING_FIELDS,
                NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_non_empty_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_trimmed_non_empty_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_sha256_hex_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_CONTENT_HASH_FIELDS,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics(
            payload,
            label=label,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics(
            payload,
            label=label,
        )
    )
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        audit = payload.get(field)
        if audit is None:
            continue
        audit_label = f"{label} {field}"
        if not isinstance(audit, dict):
            diagnostics.append(f"{audit_label} must be an object")
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_operation_audit_schema_diagnostics(
                audit_label,
                audit,
            )
        )
    return diagnostics
