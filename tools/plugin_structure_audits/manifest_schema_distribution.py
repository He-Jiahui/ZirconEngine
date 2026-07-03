from __future__ import annotations

from typing import Any

from .manifest_schema import (
    PACKAGING_VALUES,
    collect_required_field_violation,
    is_non_empty_trimmed_string,
)


DISTRIBUTION_FIELDS = frozenset(
    """
    abi_version assets default_packaging descriptor_symbol dist_crate editor_entry
    engine_compat forms runtime_entry
    """.split()
)
DISTRIBUTION_REQUIRED_FIELDS = (
    "forms",
    "default_packaging",
    "abi_version",
    "engine_compat",
    "dist_crate",
    "descriptor_symbol",
)
DISTRIBUTION_OPTIONAL_ENTRY_FIELDS = ("runtime_entry", "editor_entry")
DISTRIBUTION_FORM_VALUES = ("dist", "embed")
DISTRIBUTION_DIST_FORM = "dist"
DISTRIBUTION_DIST_PACKAGING = "native_dynamic"
DISTRIBUTION_DESCRIPTOR_SYMBOL = "zircon_native_plugin_descriptor_v3"
DISTRIBUTION_ABI_VERSION = 3
DISTRIBUTION_RETIRED_UI_ASSET_SUFFIXES = (".v2.ui.toml", ".ui.toml")


def collect_root_distribution_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    if "distribution" not in manifest:
        return
    collect_distribution_schema_violations(
        display_path,
        "distribution",
        manifest["distribution"],
        violations,
    )


def collect_feature_distribution_schema_violations(
    display_path: str,
    field_label: str,
    distribution: object,
    violations: list[str],
) -> None:
    collect_distribution_schema_violations(
        display_path,
        field_label,
        distribution,
        violations,
    )


def collect_distribution_schema_violations(
    display_path: str,
    field_label: str,
    distribution: object,
    violations: list[str],
) -> None:
    if not isinstance(distribution, dict):
        violations.append(f"{display_path}: {field_label} must be a table")
        return
    collect_distribution_known_field_violations(
        display_path, field_label, distribution, violations
    )
    for field in DISTRIBUTION_REQUIRED_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            distribution,
            violations,
            field_name=field,
        )
    for field in DISTRIBUTION_OPTIONAL_ENTRY_FIELDS:
        if field in distribution:
            collect_required_field_violation(
                display_path,
                f"{field_label}.{field}",
                distribution,
                violations,
                field_name=field,
            )
    collect_distribution_engine_compat_violations(
        display_path, field_label, distribution, violations
    )
    collect_distribution_abi_violations(
        display_path, field_label, distribution, violations
    )
    collect_distribution_descriptor_symbol_violations(
        display_path, field_label, distribution, violations
    )
    collect_distribution_entry_violations(
        display_path, field_label, distribution, violations
    )
    collect_distribution_string_set_violations(
        display_path,
        f"{field_label}.forms",
        distribution.get("forms"),
        DISTRIBUTION_FORM_VALUES,
        required_value=DISTRIBUTION_DIST_FORM,
        required_message="must include dist",
        duplicate_message="duplicates distribution.forms",
        violations=violations,
    )
    collect_distribution_string_set_violations(
        display_path,
        f"{field_label}.default_packaging",
        distribution.get("default_packaging"),
        PACKAGING_VALUES,
        required_value=DISTRIBUTION_DIST_PACKAGING,
        required_message="must include native_dynamic",
        duplicate_message="duplicates distribution.default_packaging",
        violations=violations,
    )
    collect_distribution_assets_violations(
        display_path, field_label, distribution, violations
    )


def collect_distribution_known_field_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    violations.extend(
        f"{display_path}: {field_label}.{field} is not a known distribution field"
        for field in sorted(set(distribution) - DISTRIBUTION_FIELDS)
    )


def collect_distribution_engine_compat_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    value = distribution.get("engine_compat")
    if not is_non_empty_trimmed_string(value):
        return
    try:
        parse_distribution_engine_compat(value)
    except ValueError as error:
        violations.append(
            f'{display_path}: {field_label}.engine_compat "{value}" '
            f"is invalid: {error}"
        )


def parse_distribution_engine_compat(value: str) -> None:
    for raw_clause in value.split(","):
        clause = raw_clause.strip()
        if not clause:
            raise ValueError("empty comparator")
        parse_distribution_engine_comparator(clause)


def parse_distribution_engine_comparator(clause: str) -> None:
    for prefix in (">=", "<=", ">", "<", "="):
        if clause.startswith(prefix):
            parse_distribution_engine_version(clause[len(prefix) :].strip())
            return
    parse_distribution_engine_version(clause)


def parse_distribution_engine_version(value: str) -> None:
    release = value.split("-", 1)[0].split("+", 1)[0].strip()
    if not release:
        raise ValueError("version is empty")
    parts = release.split(".")
    if len(parts) < 2 or len(parts) > 3:
        raise ValueError(f'version "{value}" must be major.minor[.patch]')
    for component in parts:
        if not component.isdigit():
            raise ValueError(
                f'version "{value}" contains non-numeric component "{component}"'
            )


def collect_distribution_abi_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    value = distribution.get("abi_version")
    if type(value) is int and value != DISTRIBUTION_ABI_VERSION:
        violations.append(
            f"{display_path}: {field_label}.abi_version "
            f"must be {DISTRIBUTION_ABI_VERSION}"
        )


def collect_distribution_descriptor_symbol_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    value = distribution.get("descriptor_symbol")
    if is_non_empty_trimmed_string(value) and value != DISTRIBUTION_DESCRIPTOR_SYMBOL:
        violations.append(
            f"{display_path}: {field_label}.descriptor_symbol "
            f"must equal {DISTRIBUTION_DESCRIPTOR_SYMBOL}"
        )


def collect_distribution_entry_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    if any(
        is_non_empty_trimmed_string(distribution.get(field))
        for field in DISTRIBUTION_OPTIONAL_ENTRY_FIELDS
    ):
        return
    violations.append(
        f"{display_path}: {field_label} must declare runtime_entry or editor_entry"
    )


def collect_distribution_string_set_violations(
    display_path: str,
    field_label: str,
    values: object,
    allowed_values: tuple[str, ...],
    *,
    required_value: str,
    required_message: str,
    duplicate_message: str,
    violations: list[str],
) -> None:
    parsed_values = distribution_valid_string_array_values(values)
    if parsed_values is None:
        return
    if not any(value == required_value for _index, value in parsed_values):
        violations.append(f"{display_path}: {field_label} {required_message}")
    allowed = set(allowed_values)
    expected = ", ".join(allowed_values)
    seen: dict[str, int] = {}
    for index, value in parsed_values:
        if value not in allowed:
            violations.append(
                f'{display_path}: {field_label}[{index}] "{value}" '
                f"is unsupported; expected one of {expected}"
            )
            continue
        previous_index = seen.get(value)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {field_label}[{index}] {value} "
                f"{duplicate_message}[{previous_index}]"
            )
        else:
            seen[value] = index


def distribution_valid_string_array_values(
    values: object,
) -> list[tuple[int, str]] | None:
    if not isinstance(values, list) or not values:
        return None
    parsed: list[tuple[int, str]] = []
    for index, value in enumerate(values):
        if is_non_empty_trimmed_string(value):
            parsed.append((index, value))
    return parsed


def collect_distribution_assets_violations(
    display_path: str,
    field_label: str,
    distribution: dict[str, Any],
    violations: list[str],
) -> None:
    if "assets" not in distribution:
        return
    assets = distribution["assets"]
    assets_label = f"{field_label}.assets"
    if not isinstance(assets, list):
        violations.append(f"{display_path}: {assets_label} must be an array")
        return
    for index, raw_pattern in enumerate(assets):
        item_label = f"{assets_label}[{index}]"
        if not isinstance(raw_pattern, str) or not raw_pattern.strip():
            violations.append(f"{display_path}: {item_label} must be a non-empty string")
            continue
        if raw_pattern.strip() != raw_pattern:
            violations.append(f"{display_path}: {item_label} must be trimmed")
            continue
        if distribution_asset_pattern_escapes_plugin_root(raw_pattern):
            violations.append(
                f"{display_path}: {item_label} must be a plugin-relative glob"
            )
            continue
        if retired_suffix := retired_ui_asset_pattern(raw_pattern):
            violations.append(
                f"{display_path}: {item_label} targets retired UI asset suffix "
                f"{retired_suffix}; use .zui"
            )


def distribution_asset_pattern_escapes_plugin_root(pattern: str) -> bool:
    if pattern.startswith("/") or pattern.startswith("\\"):
        return True
    if len(pattern) >= 2 and pattern[1] == ":":
        return True
    segments = pattern.replace("\\", "/").split("/")
    return ".." in segments


def retired_ui_asset_pattern(pattern: str) -> str | None:
    normalized = pattern.replace("\\", "/")
    for suffix in DISTRIBUTION_RETIRED_UI_ASSET_SUFFIXES:
        if normalized.endswith(suffix):
            return normalized
    return None
