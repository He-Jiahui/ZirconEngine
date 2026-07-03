"""NativeDynamic payload materialized_packages schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_payload_schema_helpers import (
    object_array_non_negative_integer_schema_diagnostics,
    object_array_required_non_empty_string_schema_diagnostics,
    object_array_required_trimmed_non_empty_string_schema_diagnostics,
    object_array_unique_string_field_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_string_array_schema_helpers import (
    object_array_integer_matches_string_array_length_schema_diagnostics,
    object_array_loadable_artifacts_schema_diagnostics,
    object_array_string_array_no_blank_entries_schema_diagnostics,
    object_array_string_array_safe_relative_path_schema_diagnostics,
    object_array_string_array_trimmed_non_empty_entries_schema_diagnostics,
    object_array_string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_schema_table import object_array_schema_diagnostics

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS = (
    "destination",
    "loadable_artifact_count",
    "loadable_artifacts",
    "package_id",
    "package_report",
    "source",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS = (
    "destination",
    "package_id",
    "package_report",
    "source",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS = (
    "loadable_artifact_count",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS = (
    "loadable_artifacts",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS = (
    "destination",
    "package_id",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS = (
    "loadable_artifact_count",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_ARRAY_FIELDS = (
    "loadable_artifacts",
)


def platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    return native_dynamic_materialized_packages_schema_diagnostics(
        label,
        payload,
    )


def native_dynamic_materialized_packages_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    diagnostics = object_array_schema_diagnostics(
        label,
        payload,
        "materialized_packages",
        NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS,
        string_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS,
        required_string_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS
        ),
        required_integer_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS
        ),
        require_present=True,
    )
    diagnostics.extend(
        object_array_loadable_artifacts_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
        )
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_unique_string_field_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            "package_id",
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_no_blank_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_trimmed_non_empty_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_safe_relative_path_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_unique_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_integer_matches_string_array_length_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            "loadable_artifact_count",
            "loadable_artifacts",
        )
    )
    return diagnostics
