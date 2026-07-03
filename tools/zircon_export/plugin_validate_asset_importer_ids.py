"""asset_importers id namespace checks for plugin validation."""

from __future__ import annotations

from typing import Any

Importer = dict[str, Any]
Diagnostics = list[str]
ASSET_IMPORTER_ID_CHARSET_DIAGNOSTIC = (
    "must contain only lowercase ASCII letters, digits, underscores, and dots"
)


def validate_plugin_asset_importer_id(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    value = importer.get("id")
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return
    label = f"{importer_label}.id"
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} must use at least two dot-separated namespace segments"
        )
        return
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} must not contain empty namespace segments")
        return
    if any(not plugin_validate_asset_importer_id_segment(segment) for segment in segments):
        diagnostics.append(f"{label} {ASSET_IMPORTER_ID_CHARSET_DIAGNOSTIC}")


def plugin_validate_asset_importer_id_segment(segment: str) -> bool:
    return all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte == "_")
        for byte in segment
    )
