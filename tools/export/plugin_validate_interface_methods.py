"""Bridge interface method validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_optional_trimmed_string, plugin_validate_trimmed_string
from .plugin_validate_interface_signatures import validate_plugin_interface_method_parameter_signature, validate_plugin_interface_method_return_kind

Diagnostics = list[str]

PLUGIN_VALIDATE_INTERFACE_METHOD_FIELDS = frozenset(("documentation", "method_slot", "name", "parameters", "required_capabilities", "return_value_kind"))


def validate_plugin_provided_interface_methods(
    interface: dict[str, Any],
    row_label: str,
    diagnostics: Diagnostics,
) -> None:
    methods = interface.get("methods")
    if methods is None:
        return
    label = f"{row_label}.methods"
    if not isinstance(methods, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not methods:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen_names: dict[str, int] = {}
    seen_slots: dict[int, int] = {}
    for index, method in enumerate(methods):
        method_label = f"{label}[{index}]"
        if not isinstance(method, dict):
            diagnostics.append(f"{method_label} must be a table")
            continue

        validate_plugin_interface_method_known_fields(method, method_label, diagnostics)
        method_name = validate_plugin_interface_method_token_field(method, "name", f"{method_label}.name", diagnostics)
        method_slot = validate_plugin_interface_method_slot(
            method.get("method_slot"), f"{method_label}.method_slot", diagnostics
        )
        if method_name is not None:
            previous = seen_names.get(method_name)
            if previous is not None:
                diagnostics.append(
                    f"{method_label}.name {method_name} "
                    f"duplicates provided interface method methods[{previous}]"
                )
            else:
                seen_names[method_name] = index
        if method_slot is not None:
            previous = seen_slots.get(method_slot)
            if previous is not None:
                diagnostics.append(
                    f"{method_label}.method_slot {method_slot} "
                    f"duplicates provided interface method_slot methods[{previous}]"
                )
            else:
                seen_slots[method_slot] = index

        validate_plugin_interface_method_return_kind(method, f"{method_label}.return_value_kind", diagnostics)
        validate_plugin_interface_method_parameters(method.get("parameters"), f"{method_label}.parameters", diagnostics)
        validate_plugin_interface_method_required_capabilities(method.get("required_capabilities"), f"{method_label}.required_capabilities", diagnostics)
        plugin_validate_optional_trimmed_string(method, "documentation", f"{method_label}.documentation", diagnostics)


def validate_plugin_interface_method_known_fields(method: dict[str, Any], row_label: str, diagnostics: Diagnostics) -> None:
    for field in sorted(method):
        if field not in PLUGIN_VALIDATE_INTERFACE_METHOD_FIELDS:
            diagnostics.append(f"{row_label}.{field} is not a known provided interface method field")


def validate_plugin_interface_method_parameters(parameters: Any, label: str, diagnostics: Diagnostics) -> None:
    if parameters is None:
        return
    if not isinstance(parameters, list):
        diagnostics.append(f"{label} must be an array")
        return
    seen_names: dict[str, int] = {}
    for index, parameter in enumerate(parameters):
        parameter_label = f"{label}[{index}]"
        if not isinstance(parameter, dict):
            diagnostics.append(f"{parameter_label} must be a table")
            continue
        validate_plugin_interface_method_parameter_signature(parameter, parameter_label, diagnostics)
        parameter_name = validate_plugin_interface_method_token_field(
            parameter, "name", f"{parameter_label}.name", diagnostics
        )
        if parameter_name is None:
            continue
        previous = seen_names.get(parameter_name)
        if previous is not None:
            diagnostics.append(
                f"{parameter_label}.name {parameter_name} "
                f"duplicates interface method parameter parameters[{previous}]"
            )
        else:
            seen_names[parameter_name] = index


def validate_plugin_interface_method_required_capabilities(capabilities: Any, label: str, diagnostics: Diagnostics) -> None:
    if capabilities is None:
        return
    if not isinstance(capabilities, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not capabilities:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, capability in enumerate(capabilities):
        capability_label = f"{label}[{index}]"
        if not isinstance(capability, str) or not capability.strip() or capability.strip() != capability:
            diagnostics.append(f"{capability_label} must be a non-empty trimmed string")
            continue
        validate_plugin_interface_method_namespace(capability, capability_label, diagnostics)
        previous = seen.get(capability)
        if previous is not None:
            diagnostics.append(
                f"{capability_label} {capability} duplicates required capability "
                f"required_capabilities[{previous}]"
            )
        else:
            seen[capability] = index


def validate_plugin_interface_method_token_field(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    value = plugin_validate_trimmed_string(table, field, label, diagnostics)
    if value is None:
        return None
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, and underscores"
        )
    return value


def validate_plugin_interface_method_slot(value: Any, label: str, diagnostics: Diagnostics) -> int | None:
    if type(value) is not int or value < 0:
        diagnostics.append(f"{label} must be a non-negative integer")
        return None
    return value


def validate_plugin_interface_method_namespace(value: str, label: str, diagnostics: Diagnostics) -> None:
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
