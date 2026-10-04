"""Bevy-reference checks for plugin capability status manifests."""

from __future__ import annotations

from typing import Any

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_PREFIX = "dev/bevy/"
PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_PREFIX_MESSAGE = (
    "should start with dev/bevy/"
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_DUPLICATE_MESSAGE = (
    "duplicates capability_status bevy_references"
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_SEGMENT_MESSAGE = (
    "should not contain empty, current, or parent path segments"
)


def validate_plugin_capability_status_references(
    status: Manifest,
    row_label: str,
    diagnostics: Diagnostics,
) -> None:
    references = plugin_validate_capability_status_bevy_references(
        status, f"{row_label}.bevy_references", diagnostics
    )
    if references is None:
        return
    seen: dict[str, int] = {}
    for index, reference in enumerate(references):
        item_label = f"{row_label}.bevy_references[{index}]"
        validate_plugin_capability_status_bevy_reference(
            reference, item_label, diagnostics
        )
        previous_index = seen.get(reference)
        if previous_index is not None:
            diagnostics.append(
                f"{item_label} {reference} "
                f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_DUPLICATE_MESSAGE} "
                f"bevy_references[{previous_index}]"
            )
        else:
            seen[reference] = index


def plugin_validate_capability_status_bevy_references(
    status: Manifest,
    label: str,
    diagnostics: Diagnostics,
) -> list[str] | None:
    if "bevy_references" not in status:
        return None
    value = status["bevy_references"]
    if not isinstance(value, list):
        diagnostics.append(f"{label} must be an array")
        return None
    if not value:
        diagnostics.append(f"{label} must not be empty when declared")
        return None
    references: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            diagnostics.append(f"{label}[{index}] must be a non-empty trimmed string")
            continue
        references.append(item)
    return references


def validate_plugin_capability_status_bevy_reference(
    reference: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if not reference.startswith(PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_PREFIX):
        diagnostics.append(
            f"{label} {reference} "
            f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_PREFIX_MESSAGE}"
        )
    if "\\" in reference or ":" in reference:
        diagnostics.append(
            f"{label} {reference} should use repository-relative forward-slash paths"
        )
    segments = reference.split("/")
    if any(segment in {"", ".", ".."} for segment in segments):
        diagnostics.append(
            f"{label} {reference} "
            f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_BEVY_REFERENCE_SEGMENT_MESSAGE}"
        )
