"""Validate report LibraryEmbed CompileHost plan schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_string_array_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_linkage_schema import (
    validate_linked_runtime_crate_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_array_schema_diagnostics,
)

VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS = (
    "app_features",
    "binary",
    "cargo_profile",
    "command",
    "expected_runtime_plugins",
    "linked_runtime_crates",
    "manifest_path",
    "package",
    "release",
    "runtime_features",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS = (
    "binary",
    "cargo_profile",
    "manifest_path",
    "package",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_ARRAY_FIELDS = (
    "app_features",
    "command",
    "runtime_features",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PROJECT_PLUGIN_ID_ARRAY_FIELDS = (
    "expected_runtime_plugins",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BOOL_FIELDS = (
    "release",
)


def validate_library_embed_compile_host_schema_diagnostics(value: Any) -> list[str]:
    label = "validate report plan_summary.library_embed_compile_host"
    if not isinstance(value, dict):
        return [f"{label} must be an object"]

    diagnostics: list[str] = []
    known_compile_host_fields = set(VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS)
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(value)
        if field not in known_compile_host_fields
    )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_ARRAY_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BOOL_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    linked_runtime_crates = value.get("linked_runtime_crates")
    if "linked_runtime_crates" in value:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.linked_runtime_crates",
                linked_runtime_crates,
            )
        )
    if isinstance(linked_runtime_crates, list):
        diagnostics.extend(
            validate_linked_runtime_crate_schema_diagnostics(linked_runtime_crates)
        )
    return diagnostics
