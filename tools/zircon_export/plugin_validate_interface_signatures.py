"""Bridge interface method signature validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_trimmed_string

Diagnostics = list[str]

ALLOWED_SCRIPT_HOST_VALUE_KINDS = (
    "null",
    "bool",
    "int",
    "float",
    "string",
    "bytes",
    "host_handle",
)
PLUGIN_VALIDATE_INTERFACE_METHOD_PARAMETER_FIELDS = frozenset((
    "name", "type_ref", "value_kind",
))
PLUGIN_VALIDATE_INTERFACE_METHOD_TYPE_REF_FIELDS = frozenset(("type_name", "value_kind"))


def validate_plugin_interface_method_return_kind(
    method: dict[str, Any],
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if "return_value_kind" not in method:
        return
    validate_plugin_interface_signature_value_kind(
        method.get("return_value_kind"), label, diagnostics
    )


def validate_plugin_interface_method_parameter_signature(
    parameter: dict[str, Any],
    label: str,
    diagnostics: Diagnostics,
) -> None:
    validate_plugin_interface_signature_known_fields(
        parameter, PLUGIN_VALIDATE_INTERFACE_METHOD_PARAMETER_FIELDS,
        label, "interface method parameter", diagnostics,
    )
    validate_plugin_interface_signature_required_value_kind(
        parameter, "value_kind", f"{label}.value_kind", diagnostics
    )
    validate_plugin_interface_method_type_ref(
        parameter.get("type_ref"), f"{label}.type_ref", diagnostics
    )


def validate_plugin_interface_method_type_ref(
    type_ref: Any,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if type_ref is None:
        return
    if not isinstance(type_ref, dict):
        diagnostics.append(f"{label} must be a table")
        return
    validate_plugin_interface_signature_known_fields(
        type_ref, PLUGIN_VALIDATE_INTERFACE_METHOD_TYPE_REF_FIELDS,
        label, "interface method type_ref", diagnostics,
    )
    validate_plugin_interface_signature_required_value_kind(
        type_ref, "value_kind", f"{label}.value_kind", diagnostics
    )
    plugin_validate_trimmed_string(type_ref, "type_name", f"{label}.type_name", diagnostics)


def validate_plugin_interface_signature_known_fields(
    table: dict[str, Any],
    fields: frozenset[str],
    label: str,
    field_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field in sorted(table):
        if field not in fields:
            diagnostics.append(f"{label}.{field} is not a known {field_label} field")


def validate_plugin_interface_signature_required_value_kind(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    if field not in table:
        diagnostics.append(f"{label} is required")
        return None
    return validate_plugin_interface_signature_value_kind(
        table.get(field), label, diagnostics
    )


def validate_plugin_interface_signature_value_kind(
    value: Any,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed string")
        return None
    if value not in ALLOWED_SCRIPT_HOST_VALUE_KINDS:
        expected = ", ".join(ALLOWED_SCRIPT_HOST_VALUE_KINDS)
        diagnostics.append(f"{label} {value} is unsupported; expected one of {expected}")
    return value
