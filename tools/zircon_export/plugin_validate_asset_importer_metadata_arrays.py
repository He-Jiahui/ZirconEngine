"""asset_importers metadata array duplicate checks for plugin validation."""

from __future__ import annotations

from typing import Any

Importer = dict[str, Any]
Diagnostics = list[str]
ASSET_IMPORTER_METADATA_ARRAYS = (
    "additional_output_kinds",
    "required_capabilities",
)


def validate_plugin_asset_importer_metadata_arrays(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    for field_name in ASSET_IMPORTER_METADATA_ARRAYS:
        validate_plugin_asset_importer_metadata_array_unique_entries(
            importer, field_name, importer_label, diagnostics
        )


def validate_plugin_asset_importer_metadata_array_unique_entries(
    importer: Importer,
    field_name: str,
    importer_label: str,
    diagnostics: Diagnostics,
) -> None:
    values = importer.get(field_name)
    if not isinstance(values, list):
        return
    label = f"{importer_label}.{field_name}"
    if not values:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        duplicate_index = seen.get(value)
        if duplicate_index is not None:
            diagnostics.append(f"{label}[{index}] duplicates entry {duplicate_index}")
            continue
        seen[value] = index
