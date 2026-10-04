"""Plugin option row schema checks."""

from __future__ import annotations

import math
from typing import Any

Diagnostics = list[str]
OptionRow = dict[str, Any]
PLUGIN_VALIDATE_OPTION_VALUE_TYPES = ("bool", "integer", "number", "string", "enum")
PLUGIN_VALIDATE_OPTION_FIELDS = frozenset(("key", "display_name", "value_type", "default_value", "enum_values", "required_capability"))


def validate_plugin_option_schema(
    option: OptionRow,
    option_label: str,
    diagnostics: Diagnostics,
) -> None:
    validate_plugin_option_known_fields(option, option_label, diagnostics)
    key = plugin_validate_option_required_string(
        option, "key", option_label, diagnostics
    )
    if key is not None:
        plugin_validate_option_key(key, f"{option_label}.key", diagnostics)
    plugin_validate_option_required_string(
        option, "display_name", option_label, diagnostics
    )
    value_type = plugin_validate_option_required_string(
        option, "value_type", option_label, diagnostics
    )
    if value_type is not None and value_type not in PLUGIN_VALIDATE_OPTION_VALUE_TYPES:
        expected = ", ".join(PLUGIN_VALIDATE_OPTION_VALUE_TYPES)
        diagnostics.append(
            f'{option_label}.value_type "{value_type}" is unsupported; '
            f"expected one of {expected}"
        )
    default_value = plugin_validate_option_required_string(
        option, "default_value", option_label, diagnostics
    )
    if value_type in PLUGIN_VALIDATE_OPTION_VALUE_TYPES and default_value is not None:
        plugin_validate_option_default_value(
            value_type, default_value, f"{option_label}.default_value", diagnostics
        )
    plugin_validate_option_enum_values(
        option, value_type, default_value, option_label, diagnostics
    )


def validate_plugin_option_known_fields(
    option: OptionRow,
    option_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field in sorted(option):
        if field not in PLUGIN_VALIDATE_OPTION_FIELDS:
            diagnostics.append(f"{option_label}.{field} is not a known option field")


def plugin_validate_option_required_string(
    option: OptionRow,
    field_name: str,
    option_label: str,
    diagnostics: Diagnostics,
) -> str | None:
    value = option.get(field_name)
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        diagnostics.append(f"{option_label}.{field_name} must be a non-empty trimmed string")
        return None
    return value


def plugin_validate_option_key(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(f"{label} must use at least two dot-separated namespace segments")
        return
    if any(segment == "" for segment in segments):
        diagnostics.append(f"{label} must not contain empty namespace segments")
        return
    if any(not plugin_validate_option_key_segment(segment) for segment in segments):
        diagnostics.append(
            f"{label} must contain only lowercase ASCII letters, digits, underscores, and dots"
        )


def plugin_validate_option_key_segment(segment: str) -> bool:
    return all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte == "_")
        for byte in segment
    )


def plugin_validate_option_default_value(
    value_type: str,
    default_value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if value_type == "bool" and default_value not in {"true", "false"}:
        diagnostics.append(f"{label} bool value must be true or false")
    elif value_type == "integer":
        try:
            int(default_value)
        except ValueError:
            diagnostics.append(f"{label} integer value must parse as i64")
    elif value_type == "number":
        try:
            number = float(default_value)
        except ValueError:
            diagnostics.append(f"{label} number value must parse as f64")
            return
        if not math.isfinite(number):
            diagnostics.append(f"{label} number value must be finite")
    elif value_type == "enum":
        plugin_validate_option_enum_token(default_value, label, diagnostics)


def plugin_validate_option_enum_values(
    option: OptionRow,
    value_type: str | None,
    default_value: str | None,
    option_label: str,
    diagnostics: Diagnostics,
) -> None:
    if value_type != "enum":
        if "enum_values" in option:
            diagnostics.append(
                f"{option_label}.enum_values must only be declared for enum options"
            )
        return
    values = option.get("enum_values")
    label = f"{option_label}.enum_values"
    if not isinstance(values, list) or not values:
        diagnostics.append(f"{label} must be a non-empty string array")
        return
    seen: dict[str, int] = {}
    strings: list[str] = []
    for index, value in enumerate(values):
        item_label = f"{label}[{index}]"
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            diagnostics.append(f"{item_label} must be a non-empty trimmed string")
            continue
        strings.append(value)
        duplicate_index = seen.get(value)
        if duplicate_index is not None:
            diagnostics.append(f"{item_label} duplicates entry {duplicate_index}")
        else:
            seen[value] = index
        plugin_validate_option_enum_token(value, item_label, diagnostics)
    if default_value is not None and default_value not in strings:
        diagnostics.append(f"{option_label}.default_value must be declared in enum_values")


def plugin_validate_option_enum_token(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if not all(
        byte.isascii()
        and (byte.islower() or byte.isdigit() or byte == "_" or byte == "-")
        for byte in value
    ):
        diagnostics.append(
            f"{label} must contain only lowercase ASCII letters, digits, underscores, or hyphens"
        )
