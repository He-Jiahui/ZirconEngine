"""asset_importers required capability namespace checks for plugin validation."""

from __future__ import annotations

from typing import Any

Importer = dict[str, Any]
Diagnostics = list[str]
ASSET_IMPORTER_REQUIRED_CAPABILITY_CHARSET_DIAGNOSTIC = (
    "must contain only lowercase ASCII letters, digits, underscores, and dots"
)


def validate_plugin_asset_importer_required_capabilities(
    importer: Importer, importer_label: str, diagnostics: Diagnostics
) -> None:
    values = importer.get("required_capabilities")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.required_capabilities"
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        validate_plugin_asset_importer_required_capability_namespace(
            value, f"{label}[{index}]", diagnostics
        )


def validate_plugin_asset_importer_required_capability_namespace(
    value: str, label: str, diagnostics: Diagnostics
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} must use at least two dot-separated namespace segments"
        )
        return
    if any(
        not plugin_validate_asset_importer_required_capability_segment(segment)
        for segment in segments
    ):
        diagnostics.append(
            f"{label} {ASSET_IMPORTER_REQUIRED_CAPABILITY_CHARSET_DIAGNOSTIC}"
        )


def plugin_validate_asset_importer_required_capability_segment(segment: str) -> bool:
    return all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte == "_")
        for byte in segment
    )
