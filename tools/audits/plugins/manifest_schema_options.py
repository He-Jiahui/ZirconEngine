from __future__ import annotations

import math
from typing import Any


OPTION_VALUE_TYPES = ("bool", "integer", "number", "string", "enum")
OPTION_FIELDS = frozenset(
    (
        "key",
        "display_name",
        "value_type",
        "default_value",
        "enum_values",
        "required_capability",
    )
)


def collect_options_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    options = manifest.get("options")
    if options is None:
        return
    if not isinstance(options, list):
        violations.append(f"{display_path}: options must be an array")
        return
    if not options:
        violations.append(f"{display_path}: options must not be empty when declared")
        return

    for option_index, option in enumerate(options):
        option_label = f"options[{option_index}]"
        if not isinstance(option, dict):
            violations.append(f"{display_path}: {option_label} must be a table")
            continue
        collect_option_schema_violations(
            display_path,
            option_label,
            option,
            violations,
        )


def collect_option_schema_violations(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    violations: list[str],
) -> None:
    collect_option_known_field_violations(
        display_path,
        option_label,
        option,
        violations,
    )
    key = collect_option_required_string_violation(
        display_path,
        option_label,
        option,
        "key",
        violations,
    )
    if key is not None:
        collect_option_key_violations(
            display_path,
            f"{option_label}.key",
            key,
            violations,
        )
    collect_option_required_string_violation(
        display_path,
        option_label,
        option,
        "display_name",
        violations,
    )
    value_type = collect_option_required_string_violation(
        display_path,
        option_label,
        option,
        "value_type",
        violations,
    )
    if value_type is not None and value_type not in OPTION_VALUE_TYPES:
        expected = ", ".join(OPTION_VALUE_TYPES)
        violations.append(
            f'{display_path}: {option_label}.value_type "{value_type}" '
            f"is unsupported; expected one of {expected}"
        )
    default_value = collect_option_required_string_violation(
        display_path,
        option_label,
        option,
        "default_value",
        violations,
    )
    if value_type in OPTION_VALUE_TYPES and default_value is not None:
        collect_option_default_value_violations(
            display_path,
            f"{option_label}.default_value",
            value_type,
            default_value,
            violations,
        )
    collect_option_enum_values_violations(
        display_path,
        option_label,
        option,
        value_type,
        default_value,
        violations,
    )
    collect_option_optional_required_capability_violations(
        display_path,
        option_label,
        option,
        violations,
    )


def collect_option_known_field_violations(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(option):
        if field not in OPTION_FIELDS:
            violations.append(
                f"{display_path}: {option_label}.{field} "
                "is not a known option field"
            )


def collect_option_key_violations(
    display_path: str,
    field_label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {field_label} must use at least two "
            "dot-separated namespace segments"
        )
        return
    if any(segment == "" for segment in segments):
        violations.append(
            f"{display_path}: {field_label} must not contain empty namespace "
            "segments"
        )
        return
    if any(not option_key_segment_is_valid(segment) for segment in segments):
        violations.append(
            f"{display_path}: {field_label} must contain only lowercase ASCII "
            "letters, digits, underscores, and dots"
        )


def collect_option_default_value_violations(
    display_path: str,
    field_label: str,
    value_type: str,
    default_value: str,
    violations: list[str],
) -> None:
    if value_type == "bool" and default_value not in {"true", "false"}:
        violations.append(
            f"{display_path}: {field_label} bool value must be true or false"
        )
    elif value_type == "integer":
        try:
            int(default_value)
        except ValueError:
            violations.append(
                f"{display_path}: {field_label} integer value must parse as i64"
            )
    elif value_type == "number":
        try:
            number = float(default_value)
        except ValueError:
            violations.append(
                f"{display_path}: {field_label} number value must parse as f64"
            )
            return
        if not math.isfinite(number):
            violations.append(
                f"{display_path}: {field_label} number value must be finite"
            )
    elif value_type == "enum":
        collect_option_enum_token_violations(
            display_path,
            field_label,
            default_value,
            violations,
        )


def collect_option_enum_values_violations(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    value_type: str | None,
    default_value: str | None,
    violations: list[str],
) -> None:
    if value_type != "enum":
        if "enum_values" in option:
            violations.append(
                f"{display_path}: {option_label}.enum_values must only be "
                "declared for enum options"
            )
        return
    values = option.get("enum_values")
    enum_values_label = f"{option_label}.enum_values"
    if not isinstance(values, list) or not values:
        violations.append(
            f"{display_path}: {enum_values_label} must be a non-empty string array"
        )
        return

    seen: dict[str, int] = {}
    strings: list[str] = []
    for value_index, value in enumerate(values):
        item_label = f"{enum_values_label}[{value_index}]"
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            violations.append(
                f"{display_path}: {item_label} must be a non-empty trimmed string"
            )
            continue
        strings.append(value)
        previous_index = seen.get(value)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} duplicates entry {previous_index}"
            )
        else:
            seen[value] = value_index
        collect_option_enum_token_violations(
            display_path,
            item_label,
            value,
            violations,
        )
    if default_value is not None and default_value not in strings:
        violations.append(
            f"{display_path}: {option_label}.default_value must be declared "
            "in enum_values"
        )


def collect_option_enum_token_violations(
    display_path: str,
    field_label: str,
    value: str,
    violations: list[str],
) -> None:
    if not all(
        byte.isascii()
        and (byte.islower() or byte.isdigit() or byte == "_" or byte == "-")
        for byte in value
    ):
        violations.append(
            f"{display_path}: {field_label} must contain only lowercase ASCII "
            "letters, digits, underscores, or hyphens"
        )


def collect_option_optional_required_capability_violations(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    violations: list[str],
) -> None:
    if "required_capability" not in option:
        return
    collect_option_required_string_violation(
        display_path,
        option_label,
        option,
        "required_capability",
        violations,
    )


def collect_option_required_string_violation(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    value = option.get(field_name)
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {option_label}.{field_name} "
            "must be a non-empty trimmed string"
        )
        return None
    return value


def option_key_segment_is_valid(segment: str) -> bool:
    return all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte == "_")
        for byte in segment
    )
