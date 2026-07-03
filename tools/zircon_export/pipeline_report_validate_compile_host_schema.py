"""Validate report LibraryEmbed CompileHost plan schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .pipeline_report_validate_compile_host_command_semantics import (
    library_embed_compile_host_command_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_semantics import (
    compile_host_target_selector_schema_diagnostics,
    library_embed_compile_host_profile_release_diagnostics,
)
from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_schema_string_array import (
    string_array_duplicate_entry_index_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_linkage_schema import (
    linked_runtime_crates_cover_expected_plugins_diagnostics,
    linked_runtime_crates_only_expected_plugins_diagnostics,
    validate_linked_runtime_crate_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_unique_project_plugin_package_id_array_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
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
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_REQUIRED_FIELDS = VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS = (
    "binary",
    "cargo_profile",
    "manifest_path",
    "package",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PATH_FIELDS = (
    "manifest_path",
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
    diagnostics.extend(
        f"{label}.{field} is required"
        for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_REQUIRED_FIELDS
        if field not in value
    )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
            diagnostics.extend(
                validate_non_empty_trimmed_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
            if field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PATH_FIELDS:
                diagnostics.extend(
                    validate_safe_relative_path_schema_diagnostics(
                        f"{label}.{field}",
                        value.get(field),
                    )
                )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_ARRAY_FIELDS:
        if field in value:
            field_label = f"{label}.{field}"
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            diagnostics.extend(
                string_array_no_blank_entries_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            if field in ("app_features", "command", "runtime_features"):
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        value.get(field),
                    )
                )
            if field in ("app_features", "runtime_features"):
                diagnostics.extend(
                    string_array_duplicate_entry_index_schema_diagnostics(
                        field_label,
                        value.get(field),
                    )
                )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in value:
            field_label = f"{label}.{field}"
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            diagnostics.extend(
                validate_unique_project_plugin_package_id_array_schema_diagnostics(
                    field_label,
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
    diagnostics.extend(
        linked_runtime_crates_cover_expected_plugins_diagnostics(
            value.get("expected_runtime_plugins"),
            linked_runtime_crates,
            label=label,
        )
    )
    diagnostics.extend(
        linked_runtime_crates_only_expected_plugins_diagnostics(
            value.get("expected_runtime_plugins"),
            linked_runtime_crates,
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_target_selector_schema_diagnostics(
            value,
            package_label=f"{label}.package",
            binary_label=f"{label}.binary",
        )
    )
    diagnostics.extend(library_embed_compile_host_profile_release_diagnostics(value))
    diagnostics.extend(library_embed_compile_host_command_schema_diagnostics(value))
    return diagnostics


def validate_non_empty_trimmed_string_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    if isinstance(value, str) and (not value.strip() or value != value.strip()):
        return [f"{label} must be a non-empty trimmed string"]
    return []


def validate_safe_relative_path_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    if not isinstance(value, str) or not value.strip() or value != value.strip():
        return []
    if not is_safe_relative_path(normalize_relative_path(value)):
        return [f"{label} must be a safe relative path"]
    return []
