"""asset_importers field schema checks for plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_asset_importer_resource_kinds import validate_plugin_asset_importer_output_kinds

Importer = dict[str, Any]
Diagnostics = list[str]

PLUGIN_VALIDATE_ASSET_IMPORTER_FIELDS = frozenset(
    "additional_output_kinds full_suffixes id importer_version output_kind "
    "plugin_id priority required_capabilities source_extensions".split()
)


def validate_plugin_asset_importer_schema(
    importer: Importer, importer_label: str, package_id: str, diagnostics: Diagnostics
) -> None:
    validate_plugin_asset_importer_known_fields(importer, importer_label, diagnostics)
    for field_name in ("id", "plugin_id"):
        validate_plugin_asset_importer_required_string(
            importer, field_name, importer_label, diagnostics
        )
    validate_plugin_asset_importer_plugin_id_match(importer, importer_label, package_id, diagnostics)
    validate_plugin_asset_importer_required_integer(
        importer, "priority", importer_label, diagnostics
    )
    validate_plugin_asset_importer_required_string(
        importer, "output_kind", importer_label, diagnostics
    )
    validate_plugin_asset_importer_positive_integer(
        importer, "importer_version", importer_label, diagnostics
    )
    for field_name in ("source_extensions", "additional_output_kinds", "required_capabilities"):
        validate_plugin_asset_importer_string_array(
            importer, field_name, importer_label, diagnostics
        )
    validate_plugin_asset_importer_source_extensions(importer, importer_label, diagnostics)
    validate_plugin_asset_importer_output_kinds(importer, importer_label, diagnostics)


def validate_plugin_asset_importer_known_fields(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    for field in sorted(importer):
        if field not in PLUGIN_VALIDATE_ASSET_IMPORTER_FIELDS:
            diagnostics.append(f"{importer_label}.{field} is not a known asset_importer field")


def validate_plugin_asset_importer_required_string(
    importer: Importer, field_name: str, importer_label: str, diagnostics: Diagnostics
) -> None:
    value = importer.get(field_name)
    label = f"{importer_label}.{field_name}"
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(f"{label} must be a non-empty string")
        return
    if value.strip() != value:
        diagnostics.append(f"{label} must be trimmed")


def validate_plugin_asset_importer_required_integer(
    importer: Importer, field_name: str, importer_label: str, diagnostics: Diagnostics
) -> None:
    value = importer.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool):
        diagnostics.append(f"{importer_label}.{field_name} must be an integer")


def validate_plugin_asset_importer_positive_integer(
    importer: Importer, field_name: str, importer_label: str, diagnostics: Diagnostics
) -> None:
    value = importer.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        diagnostics.append(f"{importer_label}.{field_name} must be a positive integer")


def validate_plugin_asset_importer_string_array(
    importer: Importer, field_name: str, importer_label: str, diagnostics: Diagnostics
) -> None:
    values = importer.get(field_name)
    if values is None:
        return
    label = f"{importer_label}.{field_name}"
    if not isinstance(values, list):
        diagnostics.append(f"{label} must be an array")
        return
    for index, value in enumerate(values):
        item_label = f"{label}[{index}]"
        if not isinstance(value, str) or not value.strip():
            diagnostics.append(f"{item_label} must be a non-empty string")
            continue
        if value.strip() != value:
            diagnostics.append(f"{item_label} must be trimmed")


def validate_plugin_asset_importer_source_extensions(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    values = importer.get("source_extensions")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.source_extensions"
    seen: dict[str, int] = {}
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        item_label = f"{label}[{index}]"
        duplicate_index = seen.get(value)
        if duplicate_index is not None:
            diagnostics.append(f"{item_label} duplicates entry {duplicate_index}")
            continue
        seen[value] = index
        if "." in value:
            diagnostics.append(
                f"{item_label} must be a lowercase extension without dots; "
                "use full_suffixes for dotted suffixes"
            )
            continue
        if value != value.lower():
            diagnostics.append(f"{item_label} must be lowercase")


def validate_plugin_asset_importer_plugin_id_match(
    importer: Importer, importer_label: str, package_id: str, diagnostics: Diagnostics
) -> None:
    value = importer.get("plugin_id")
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    if value != package_id:
        diagnostics.append(f"{importer_label}.plugin_id must match package id {package_id}")
