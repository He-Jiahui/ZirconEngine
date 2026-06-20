"""Validate report CompileHost linked runtime crate schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template import is_safe_relative_path, normalize_relative_path
from .pipeline_report_schema_primitives import validate_string_schema_diagnostics
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_schema_diagnostics,
    validate_project_runtime_crate_name_schema_diagnostics,
)

VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_FIELDS = (
    "crate_name",
    "path",
    "provider_package_id",
    "registration_kind",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_STRING_FIELDS = (
    "crate_name",
    "path",
    "provider_package_id",
    "registration_kind",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_RELATIVE_PATH_FIELDS = ("path",)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_NAME_FIELDS = ("crate_name",)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_PROVIDER_ID_FIELDS = (
    "provider_package_id",
)
VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_REGISTRATION_KINDS = {
    "runtime_plugin",
}


def validate_linked_runtime_crate_schema_diagnostics(
    linked_runtime_crates: list[Any],
    *,
    label: str = "validate report plan_summary.library_embed_compile_host.linked_runtime_crates",
) -> list[str]:
    diagnostics: list[str] = []
    known_linked_crate_fields = set(VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_FIELDS)
    seen_crate_names: dict[str, int] = {}
    for index, crate in enumerate(linked_runtime_crates):
        if not isinstance(crate, dict):
            continue
        diagnostics.extend(
            f"{label}[{index}] unknown field {field}"
            for field in sorted(crate)
            if field not in known_linked_crate_fields
        )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_STRING_FIELDS:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}[{index}].{field}",
                    crate.get(field),
                )
            )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_RELATIVE_PATH_FIELDS:
            value = crate.get(field)
            if isinstance(value, str) and not linked_crate_path_is_safe(value):
                diagnostics.append(
                    f"{label}[{index}].{field} must be a safe relative path"
                )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_NAME_FIELDS:
            value = crate.get(field)
            if isinstance(value, str):
                crate_name_diagnostics = (
                    validate_project_runtime_crate_name_schema_diagnostics(
                        f"{label}[{index}].{field}",
                        value,
                    )
                )
                diagnostics.extend(crate_name_diagnostics)
                if crate_name_diagnostics:
                    continue
                previous_index = seen_crate_names.get(value)
                if previous_index is None:
                    seen_crate_names[value] = index
                    continue
                diagnostics.append(
                    f"{label}[{index}].{field} duplicates entry {previous_index}"
                )
        for field in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_PROVIDER_ID_FIELDS:
            value = crate.get(field)
            if isinstance(value, str):
                diagnostics.extend(
                    validate_project_plugin_package_id_schema_diagnostics(
                        f"{label}[{index}].{field}",
                        value,
                    )
                )
        registration_kind = crate.get("registration_kind")
        if (
            isinstance(registration_kind, str)
            and registration_kind
            not in VALIDATE_LIBRARY_EMBED_LINKED_RUNTIME_CRATE_REGISTRATION_KINDS
        ):
            diagnostics.append(
                f"{label}[{index}].registration_kind must be runtime_plugin"
            )
    return diagnostics


def linked_crate_path_is_safe(value: str) -> bool:
    return (
        bool(value.strip())
        and value.strip() == value
        and is_safe_relative_path(normalize_relative_path(value))
    )
