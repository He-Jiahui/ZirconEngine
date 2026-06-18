"""Validate report runtime_plugin_availability schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_non_empty_trimmed_string_schema_diagnostics,
    validate_project_plugin_package_id_schema_diagnostics,
)

VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_FIELDS = (
    "available",
    "blocked_by_maturity",
    "blocked_by_target",
    "externalized_missing",
    "linked",
    "missing_required",
    "native_dynamic",
    "stub",
)
VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_FIELDS = (
    "id",
    "maturity",
    "reason",
    "required",
    "runtime_id",
)
VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_STRING_FIELDS = (
    "id",
    "maturity",
    "reason",
    "runtime_id",
)
VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_BOOL_FIELDS = ("required",)
VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_RUNTIME_IDS = {
    "ai",
    "animation",
    "audio_importer",
    "gltf_importer",
    "hybrid_gi",
    "navigation",
    "net",
    "obj_importer",
    "particles",
    "physics",
    "prefab_tools",
    "rendering",
    "shader_wgsl_importer",
    "solari",
    "sound",
    "terrain",
    "texture",
    "texture_importer",
    "tilemap_2d",
    "ui",
    "ui_document_importer",
    "virtual_geometry",
    "zr_vm_language",
}
VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_MATURITIES = {
    "beta",
    "core",
    "deprecated",
    "experimental",
    "externalized",
    "stable",
    "stub",
}


def validate_runtime_plugin_availability_schema_diagnostics(
    runtime_plugin_availability: Any,
) -> list[str]:
    if not isinstance(runtime_plugin_availability, dict):
        return [
            "validate report plan_summary.runtime_plugin_availability must be an object"
        ]

    diagnostics: list[str] = []
    known_availability_fields = set(VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_FIELDS)
    diagnostics.extend(
        "validate report plan_summary.runtime_plugin_availability "
        f"unknown field {field}"
        for field in sorted(runtime_plugin_availability)
        if field not in known_availability_fields
    )
    diagnostics.extend(
        "validate report plan_summary.runtime_plugin_availability "
        f"missing field {field}"
        for field in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_FIELDS
        if field not in runtime_plugin_availability
    )
    known_entry_fields = set(VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_FIELDS)
    for category in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_FIELDS:
        entries = runtime_plugin_availability.get(category)
        if category not in runtime_plugin_availability:
            continue
        category_label = (
            "validate report plan_summary.runtime_plugin_availability."
            f"{category}"
        )
        if not isinstance(entries, list):
            diagnostics.append(f"{category_label} must be an array")
            continue
        for index, entry in enumerate(entries):
            entry_label = f"{category_label}[{index}]"
            if not isinstance(entry, dict):
                diagnostics.append(f"{entry_label} must be an object")
                continue
            diagnostics.extend(
                f"{entry_label} unknown field {field}"
                for field in sorted(entry)
                if field not in known_entry_fields
            )
            for field in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_STRING_FIELDS:
                if field in entry:
                    diagnostics.extend(
                        validate_string_schema_diagnostics(
                            f"{entry_label}.{field}",
                            entry.get(field),
                        )
                    )
            id_schema_diagnostics: list[str] = []
            runtime_id_schema_diagnostics: list[str] = []
            if "id" in entry:
                id_schema_diagnostics = (
                    validate_project_plugin_package_id_schema_diagnostics(
                        f"{entry_label}.id",
                        entry.get("id"),
                    )
                )
                diagnostics.extend(id_schema_diagnostics)
            if "runtime_id" in entry:
                runtime_id_schema_diagnostics = (
                    validate_runtime_plugin_id_schema_diagnostics(
                        f"{entry_label}.runtime_id",
                        entry.get("runtime_id"),
                    )
                )
                diagnostics.extend(runtime_id_schema_diagnostics)
            if (
                "id" in entry
                and "runtime_id" in entry
                and not id_schema_diagnostics
                and not runtime_id_schema_diagnostics
                and entry.get("id") != entry.get("runtime_id")
            ):
                diagnostics.append(f"{entry_label}.id must match runtime_id")
            if "maturity" in entry:
                diagnostics.extend(
                    validate_plugin_maturity_schema_diagnostics(
                        f"{entry_label}.maturity",
                        entry.get("maturity"),
                    )
                )
            if "reason" in entry:
                diagnostics.extend(
                    validate_non_empty_trimmed_string_schema_diagnostics(
                        f"{entry_label}.reason",
                        entry.get("reason"),
                    )
                )
            for field in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_ENTRY_BOOL_FIELDS:
                if field in entry:
                    diagnostics.extend(
                        validate_bool_schema_diagnostics(
                            f"{entry_label}.{field}",
                            entry.get(field),
                        )
                    )
            if category == "missing_required" and entry.get("required") is False:
                diagnostics.append(f"{entry_label}.required must be true")
    return diagnostics


def validate_runtime_plugin_id_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    diagnostics = validate_non_empty_trimmed_string_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    if value not in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_RUNTIME_IDS:
        return [f"{label} must be a known runtime plugin id"]
    return []


def validate_plugin_maturity_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    diagnostics = validate_non_empty_trimmed_string_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    if value not in VALIDATE_RUNTIME_PLUGIN_AVAILABILITY_MATURITIES:
        return [f"{label} must be a known plugin maturity"]
    return []
