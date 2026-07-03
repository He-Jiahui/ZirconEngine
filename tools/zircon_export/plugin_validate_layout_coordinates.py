"""Top-level package coordinate validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_LAYOUT_COORDINATE_FIELDS = (
    "package_prefix",
    "package_company",
    "package_name",
)
PLUGIN_VALIDATE_LAYOUT_COORDINATE_COMPLETENESS_MESSAGE = "package coordinates must declare package_prefix, package_company, and package_name together or leave all empty"


def validate_plugin_layout_coordinates(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    values = {
        field: manifest.get(field, "")
        for field in PLUGIN_VALIDATE_LAYOUT_COORDINATE_FIELDS
    }
    declares_any = any(
        plugin_validate_layout_coordinate_declared(value)
        for value in values.values()
    )
    declares_all = all(
        isinstance(value, str) and bool(value)
        for value in values.values()
    )
    if declares_any and not declares_all:
        diagnostics.append(
            f"plugin {package_id} {PLUGIN_VALIDATE_LAYOUT_COORDINATE_COMPLETENESS_MESSAGE}"
        )
    if not declares_any:
        return
    validate_plugin_layout_coordinate_prefix(
        values["package_prefix"],
        f"plugin {package_id} package_prefix",
        diagnostics,
    )
    validate_plugin_layout_coordinate_segment(
        values["package_company"],
        f"plugin {package_id} package_company",
        diagnostics,
    )
    validate_plugin_layout_coordinate_segment(
        values["package_name"],
        f"plugin {package_id} package_name",
        diagnostics,
    )


def plugin_validate_layout_coordinate_declared(value: Any) -> bool:
    if isinstance(value, str):
        return bool(value)
    return value is not None


def validate_plugin_layout_coordinate_prefix(
    value: Any,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if (
        not isinstance(value, str)
        or not value.strip()
        or value.strip() != value
        or any(
            not plugin_validate_layout_lowercase_token(segment)
            for segment in value.split(".")
        )
    ):
        diagnostics.append(
            f"{label} {plugin_validate_layout_coordinate_display(value)} "
            "must contain only non-empty lowercase coordinate segments"
        )


def validate_plugin_layout_coordinate_segment(
    value: Any,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if (
        not isinstance(value, str)
        or not value.strip()
        or value.strip() != value
        or not plugin_validate_layout_lowercase_token(value)
    ):
        diagnostics.append(
            f"{label} {plugin_validate_layout_coordinate_display(value)} "
            "must be a non-empty lowercase coordinate segment"
        )


def plugin_validate_layout_lowercase_token(value: str) -> bool:
    return bool(value) and all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    )


def plugin_validate_layout_coordinate_display(value: Any) -> str:
    return value if isinstance(value, str) else str(value)
