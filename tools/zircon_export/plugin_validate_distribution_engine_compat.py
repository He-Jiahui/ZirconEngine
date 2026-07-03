"""Engine compatibility range checks for plugin distribution validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_trimmed_string


def plugin_validate_engine_compat(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    engine_version: str | None,
    *,
    distribution_label: str | None = None,
) -> None:
    distribution_label = distribution_label or f"plugin {package_id} distribution"
    label = f"{distribution_label}.engine_compat"
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
