"""Bridge interface validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_trimmed_string
from .plugin_validate_interface_methods import (
    validate_plugin_provided_interface_methods,
)

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_PROVIDED_INTERFACE_FIELDS = frozenset(("id", "methods"))


def validate_plugin_provided_interfaces(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    interfaces = manifest.get("provides_interfaces")
    label = f"plugin {package_id} provides_interfaces"
    if interfaces is None:
        return
    if not isinstance(interfaces, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not interfaces:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, interface in enumerate(interfaces):
        row_label = f"{label}[{index}]"
        if not isinstance(interface, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_provided_interface_known_fields(
            interface, row_label, diagnostics
        )
        validate_plugin_provided_interface_methods(interface, row_label, diagnostics)
        interface_id = plugin_validate_trimmed_string(
            interface, "id", f"{row_label}.id", diagnostics
        )
        if interface_id is None:
            continue
        validate_plugin_interface_namespace(
            interface_id, f"{row_label}.id", diagnostics
        )
        previous = seen.get(interface_id)
        if previous is not None:
            diagnostics.append(
                f"{row_label}.id {interface_id} duplicates provided interface id "
                f"provides_interfaces[{previous}]"
            )
        else:
            seen[interface_id] = index


def plugin_validate_dependency_interfaces(
    interfaces: Any,
    label: str,
    diagnostics: Diagnostics,
) -> list[str] | None:
    if not isinstance(interfaces, list) or not interfaces:
        diagnostics.append(f"{label} must be a non-empty string array")
        return None

    values: list[str] = []
    seen: dict[str, int] = {}
    valid = True
    for index, item in enumerate(interfaces):
        item_label = f"{label}[{index}]"
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            diagnostics.append(f"{item_label} must be a non-empty trimmed string")
            valid = False
            continue
        validate_plugin_interface_namespace(item, item_label, diagnostics)
        previous = seen.get(item)
        if previous is not None:
            diagnostics.append(
                f"{item_label} {item} duplicates dependency interface "
                f"interfaces[{previous}]"
            )
        else:
            seen[item] = index
        values.append(item)
    if not valid:
        return None
    return values


def validate_plugin_provided_interface_known_fields(
    interface: Manifest, row_label: str, diagnostics: Diagnostics
) -> None:
    for field in sorted(interface):
        if field not in PLUGIN_VALIDATE_PROVIDED_INTERFACE_FIELDS:
            diagnostics.append(
                f"{row_label}.{field} is not a known provided interface field"
            )


def validate_plugin_interface_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(f"{label} {value} should use package.module dot namespace form")
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} {value} should not contain empty namespace segments")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )
