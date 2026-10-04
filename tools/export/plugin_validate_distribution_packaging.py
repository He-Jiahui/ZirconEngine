"""Distribution carrier and packaging checks for plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    PLUGIN_VALIDATE_DEFAULT_PACKAGING,
    PLUGIN_VALIDATE_DIST_FORM,
    PLUGIN_VALIDATE_DIST_PACKAGING,
    PLUGIN_VALIDATE_DISTRIBUTION_FORMS,
    plugin_validate_allowed_string_values,
    plugin_validate_append_once,
    plugin_validate_string_array,
)

PLUGIN_VALIDATE_DISTRIBUTION_FORMS_DUPLICATE_MESSAGE = "duplicates distribution.forms"
PLUGIN_VALIDATE_DEFAULT_PACKAGING_DUPLICATE_MESSAGE = "duplicates distribution.default_packaging"


def plugin_validate_distribution_packaging(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    distribution_label: str | None = None,
) -> None:
    label = distribution_label or f"plugin {package_id} distribution"
    forms = plugin_validate_string_array(
        distribution,
        "forms",
        f"{label}.forms",
        diagnostics,
    )
    if forms is not None and PLUGIN_VALIDATE_DIST_FORM not in forms:
        plugin_validate_append_once(
            diagnostics,
            f"{label}.forms must include {PLUGIN_VALIDATE_DIST_FORM}",
        )
    if forms is not None:
        plugin_validate_allowed_string_values(
            forms,
            f"{label}.forms",
            PLUGIN_VALIDATE_DISTRIBUTION_FORMS,
            diagnostics,
        )
        plugin_validate_distribution_unique_values(
            forms,
            f"{label}.forms",
            PLUGIN_VALIDATE_DISTRIBUTION_FORMS_DUPLICATE_MESSAGE,
            diagnostics,
        )
    default_packaging = plugin_validate_string_array(
        distribution,
        "default_packaging",
        f"{label}.default_packaging",
        diagnostics,
    )
    if (
        default_packaging is not None
        and PLUGIN_VALIDATE_DIST_PACKAGING not in default_packaging
    ):
        diagnostics.append(
            f"{label}.default_packaging must include {PLUGIN_VALIDATE_DIST_PACKAGING}"
        )
    if default_packaging is not None:
        plugin_validate_allowed_string_values(
            default_packaging,
            f"{label}.default_packaging",
            PLUGIN_VALIDATE_DEFAULT_PACKAGING,
            diagnostics,
        )
        plugin_validate_distribution_unique_values(
            default_packaging,
            f"{label}.default_packaging",
            PLUGIN_VALIDATE_DEFAULT_PACKAGING_DUPLICATE_MESSAGE,
            diagnostics,
        )


def plugin_validate_distribution_unique_values(
    values: list[str],
    label: str,
    duplicate_message: str,
    diagnostics: list[str],
) -> None:
    seen: dict[str, int] = {}
    for index, value in enumerate(values):
        previous_index = seen.get(value)
        if previous_index is not None:
            diagnostics.append(
                f"{label}[{index}] {value} {duplicate_message}[{previous_index}]"
            )
        else:
            seen[value] = index
