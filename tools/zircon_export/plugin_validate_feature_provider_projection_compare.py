"""Feature-provider distribution projection field comparison helpers."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    plugin_validate_int,
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)


def plugin_validate_compare_string_array_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_string_array(
        owner_table,
        field,
        owner_label,
        diagnostics,
    )
    generated_value = plugin_validate_string_array(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )


def plugin_validate_compare_int_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_int(owner_table, field, owner_label, diagnostics)
    generated_value = plugin_validate_int(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )


def plugin_validate_compare_required_string_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_trimmed_string(
        owner_table,
        field,
        owner_label,
        diagnostics,
    )
    generated_value = plugin_validate_trimmed_string(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )

