"""Identifier schema diagnostics for Zircon export Validate reports."""

from __future__ import annotations

from typing import Any


def validate_non_empty_trimmed_string_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return [f"{label} must be a non-empty trimmed string"]
    return []


def validate_project_plugin_package_id_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    diagnostics = validate_string_array_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    return [
        diagnostic
        for index, package_id in enumerate(value)
        for diagnostic in validate_project_plugin_package_id_schema_diagnostics(
            f"{label}[{index}]",
            package_id,
        )
    ]


def validate_project_runtime_crate_name_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    diagnostics = validate_string_array_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    return [
        diagnostic
        for index, crate_name in enumerate(value)
        for diagnostic in validate_project_runtime_crate_name_schema_diagnostics(
            f"{label}[{index}]",
            crate_name,
        )
    ]


def validate_native_dynamic_package_id_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    diagnostics = validate_string_array_schema_diagnostics(label, value)
    if diagnostics:
        return diagnostics
    return [
        diagnostic
        for index, package_id in enumerate(value)
        for diagnostic in validate_native_dynamic_package_id_schema_diagnostics(
            f"{label}[{index}]",
            package_id,
        )
    ]


def validate_string_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        return [f"{label} must be a string array"]
    return []


def validate_project_plugin_package_id_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str):
        return [f"{label} must be a project plugin id string"]
    diagnostics: list[str] = []
    if not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed project plugin id")
    if not value or not value[0].isascii() or not value[0].islower():
        diagnostics.append(f"{label} must start with a lowercase ASCII letter")
    if not is_lowercase_ascii_identifier_token(value):
        diagnostics.append(
            f"{label} must contain only lowercase ASCII letters, digits, and underscores"
        )
    if value.endswith("_") or "__" in value:
        diagnostics.append(
            f"{label} must not end with an underscore or contain repeated underscores"
        )
    return diagnostics


def validate_project_plugin_feature_id_schema_diagnostics(
    label: str,
    owner_plugin_id: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str):
        return [f"{label} must be a project plugin feature id string"]
    diagnostics: list[str] = []
    if not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed feature id")
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} must use owner.feature dot namespace form"
        )
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} must not contain empty namespace segments")
    if any(
        segment and not is_lowercase_ascii_identifier_token(segment)
        for segment in segments
    ):
        diagnostics.append(
            f"{label} must contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )
    if project_plugin_package_id_is_valid(owner_plugin_id):
        owner_prefix = f"{owner_plugin_id}."
        if not value.startswith(owner_prefix):
            diagnostics.append(
                f"{label} must be prefixed by project plugin {owner_plugin_id}"
            )
    return diagnostics


def validate_project_runtime_crate_name_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str):
        return [f"{label} must be a runtime crate name string"]
    diagnostics: list[str] = []
    if not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed runtime crate name")
    if not (
        value.startswith("zircon_plugin_") or value.startswith("builtin_")
    ) or not is_lowercase_ascii_identifier_token(value):
        diagnostics.append(
            f"{label} must use zircon_plugin_ crate prefix or builtin_ "
            "runtime-domain prefix and contain only lowercase ASCII letters, "
            "digits, and underscores"
        )
    if value.endswith("_") or "__" in value:
        diagnostics.append(
            f"{label} must not end with an underscore or contain repeated underscores"
        )
    return diagnostics


def validate_native_dynamic_package_id_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return [f"{label} must be a non-empty trimmed native dynamic package id"]
    return []


def project_plugin_package_id_is_valid(value: str) -> bool:
    return (
        bool(value)
        and bool(value.strip())
        and value.strip() == value
        and value[0].isascii()
        and value[0].islower()
        and is_lowercase_ascii_identifier_token(value)
        and not value.endswith("_")
        and "__" not in value
    )


def is_lowercase_ascii_identifier_token(value: str) -> bool:
    return bool(value) and all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    )
