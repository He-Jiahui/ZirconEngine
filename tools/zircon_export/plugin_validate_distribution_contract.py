"""Distribution manifest contract checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .plugin_build import PLUGIN_BUILD_DIST_FORM
from .plugin_validate_common import (
    PLUGIN_VALIDATE_DEFAULT_PACKAGING,
    PLUGIN_VALIDATE_DIST_PACKAGING,
    PLUGIN_VALIDATE_DISTRIBUTION_FORMS,
    plugin_validate_allowed_string_values,
    plugin_validate_append_once,
    plugin_validate_optional_trimmed_string,
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)


PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 = "zircon_native_plugin_descriptor_v3"


def validate_plugin_distribution(
    distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
    *,
    engine_version: str | None,
) -> tuple[str | None, str | None]:
    if distribution is None:
        return None, None
    forms = plugin_validate_string_array(
        distribution,
        "forms",
        f"plugin {package_id} distribution.forms",
        diagnostics,
    )
    if forms is not None and PLUGIN_BUILD_DIST_FORM not in forms:
        plugin_validate_append_once(
            diagnostics,
            f"plugin {package_id} distribution.forms must include {PLUGIN_BUILD_DIST_FORM}",
        )
    if forms is not None:
        plugin_validate_allowed_string_values(
            forms,
            f"plugin {package_id} distribution.forms",
            PLUGIN_VALIDATE_DISTRIBUTION_FORMS,
            diagnostics,
        )
    default_packaging = plugin_validate_string_array(
        distribution,
        "default_packaging",
        f"plugin {package_id} distribution.default_packaging",
        diagnostics,
    )
    if (
        default_packaging is not None
        and PLUGIN_VALIDATE_DIST_PACKAGING not in default_packaging
    ):
        diagnostics.append(
            f"plugin {package_id} distribution.default_packaging must include "
            f"{PLUGIN_VALIDATE_DIST_PACKAGING}"
        )
    if default_packaging is not None:
        plugin_validate_allowed_string_values(
            default_packaging,
            f"plugin {package_id} distribution.default_packaging",
            PLUGIN_VALIDATE_DEFAULT_PACKAGING,
            diagnostics,
        )
    plugin_validate_engine_compat(distribution, package_id, diagnostics, engine_version)
    plugin_validate_descriptor_symbol(distribution, package_id, diagnostics)
    runtime_entry = plugin_validate_optional_trimmed_string(
        distribution,
        "runtime_entry",
        f"plugin {package_id} distribution.runtime_entry",
        diagnostics,
    )
    editor_entry = plugin_validate_optional_trimmed_string(
        distribution,
        "editor_entry",
        f"plugin {package_id} distribution.editor_entry",
        diagnostics,
    )
    plugin_validate_distribution_assets(distribution, package_id, diagnostics)
    if runtime_entry is None and editor_entry is None:
        diagnostics.append(
            f"plugin {package_id} distribution must declare runtime_entry or editor_entry"
        )
    return runtime_entry, editor_entry


def plugin_validate_descriptor_symbol(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} distribution.descriptor_symbol"
    descriptor_symbol = plugin_validate_trimmed_string(
        distribution,
        "descriptor_symbol",
        label,
        diagnostics,
    )
    if descriptor_symbol is None:
        return
    if descriptor_symbol != PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3:
        diagnostics.append(
            f"{label} must equal {PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3}"
        )


def plugin_validate_engine_compat(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    engine_version: str | None,
) -> None:
    label = f"plugin {package_id} distribution.engine_compat"
    engine_compat = plugin_validate_trimmed_string(
        distribution,
        "engine_compat",
        label,
        diagnostics,
    )
    if engine_compat is None or engine_version is None:
        return
    try:
        matches = plugin_validate_engine_compat_matches(
            engine_compat,
            engine_version,
        )
    except ValueError as error:
        diagnostics.append(f'{label} "{engine_compat}" is invalid: {error}')
        return
    if not matches:
        diagnostics.append(
            f'{label} "{engine_compat}" does not include engine {engine_version}'
        )


def plugin_validate_engine_compat_matches(
    compat_range: str,
    current_version: str,
) -> bool:
    current = plugin_validate_parse_engine_version(current_version)
    for raw_clause in compat_range.split(","):
        clause = raw_clause.strip()
        if not clause:
            raise ValueError("empty comparator")
        comparator, requested = plugin_validate_parse_engine_comparator(clause)
        if comparator == ">" and not current > requested:
            return False
        if comparator == ">=" and not current >= requested:
            return False
        if comparator == "=" and not current == requested:
            return False
        if comparator == "<" and not current < requested:
            return False
        if comparator == "<=" and not current <= requested:
            return False
    return True


def plugin_validate_parse_engine_comparator(
    clause: str,
) -> tuple[str, tuple[int, int, int]]:
    for prefix in (">=", "<=", ">", "<", "="):
        if clause.startswith(prefix):
            return prefix, plugin_validate_parse_engine_version(
                clause[len(prefix) :].strip()
            )
    return "=", plugin_validate_parse_engine_version(clause)


def plugin_validate_parse_engine_version(version: str) -> tuple[int, int, int]:
    release = version.split("-", 1)[0].split("+", 1)[0].strip()
    if not release:
        raise ValueError("version is empty")
    parts = release.split(".")
    if len(parts) < 2 or len(parts) > 3:
        raise ValueError(f'version "{version}" must be major.minor[.patch]')
    components: list[int] = []
    for component in parts:
        if not component.isdigit():
            raise ValueError(
                f'version "{version}" contains non-numeric component "{component}"'
            )
        components.append(int(component))
    if len(components) == 2:
        components.append(0)
    return components[0], components[1], components[2]


def plugin_validate_distribution_assets(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    assets = distribution.get("assets")
    if assets is None:
        return
    label = f"plugin {package_id} distribution.assets"
    if not isinstance(assets, list):
        diagnostics.append(f"{label} must be an array")
        return
    for index, raw_pattern in enumerate(assets):
        item_label = f"{label}[{index}]"
        if not isinstance(raw_pattern, str) or not raw_pattern.strip():
            diagnostics.append(f"{item_label} must be a non-empty string")
            continue
        if raw_pattern.strip() != raw_pattern:
            diagnostics.append(f"{item_label} must be trimmed")
            continue
        pattern_path = Path(raw_pattern)
        if pattern_path.is_absolute() or ".." in pattern_path.parts:
            diagnostics.append(f"{item_label} must be a plugin-relative glob")
