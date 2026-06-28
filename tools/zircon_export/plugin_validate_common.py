"""Shared helpers for standalone plugin validation."""

from __future__ import annotations

from typing import Any

from .plugin_build import feature_provider_package_id


PLUGIN_VALIDATE_ROOT_SOURCE = "plugin"
PLUGIN_VALIDATE_FEATURE_SOURCE = "feature_extension"
PLUGIN_VALIDATE_DIST_PACKAGING = "native_dynamic"
PLUGIN_VALIDATE_DISTRIBUTION_FORMS = ("dist", "embed")
PLUGIN_VALIDATE_DEFAULT_PACKAGING = (
    "source_template",
    "library_embed",
    "native_dynamic",
)


def plugin_validate_manifest_target_id(
    table: dict[str, Any],
    label: str,
    diagnostics: list[str],
    *,
    field: str = "id",
) -> str | None:
    value = table.get(field)
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed string")
        return None
    return value


def plugin_validate_selected_feature(
    plugin_manifest: dict[str, Any],
    requested_plugin_id: str,
    package_id: str,
) -> dict[str, Any] | None:
    optional_features = plugin_manifest.get("optional_features", [])
    if not isinstance(optional_features, list):
        return None
    for feature in optional_features:
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not isinstance(feature_id, str) or not feature_id.strip():
            continue
        provider_package_id = feature_provider_package_id(feature, feature_id)
        if requested_plugin_id in {feature_id, provider_package_id}:
            return feature
        if package_id == provider_package_id:
            return feature
    return None


def plugin_validate_modules_array(
    modules: Any,
    label: str,
    diagnostics: list[str],
) -> list[Any]:
    if modules in (None, []):
        return []
    if not isinstance(modules, list):
        diagnostics.append(f"{label} must be an array")
        return []
    return modules


def plugin_validate_module_crate_names(
    modules: list[Any],
    package_id: str,
    diagnostics: list[str],
) -> list[str]:
    crate_names: list[str] = []
    for index, module in enumerate(modules):
        if not isinstance(module, dict):
            diagnostics.append(f"plugin {package_id} modules[{index}] must be a table")
            continue
        crate_name = module.get("crate_name")
        if (
            isinstance(crate_name, str)
            and crate_name.strip()
            and crate_name.strip() == crate_name
        ):
            crate_names.append(crate_name)
    return crate_names


def plugin_validate_string_array(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: list[str],
) -> list[str] | None:
    if field not in table:
        diagnostics.append(f"{label} is required")
        return None
    value = table[field]
    if not isinstance(value, list) or not value:
        diagnostics.append(f"{label} must be a non-empty string array")
        return None
    values: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            diagnostics.append(
                f"{label}[{index}] must be a non-empty trimmed string"
            )
            continue
        values.append(item)
    return values


def plugin_validate_int(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: list[str],
) -> int | None:
    if field not in table:
        diagnostics.append(f"{label} is required")
        return None
    value = table[field]
    if type(value) is not int:
        diagnostics.append(f"{label} must be an integer")
        return None
    return value


def plugin_validate_trimmed_string(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: list[str],
) -> str | None:
    if field not in table:
        diagnostics.append(f"{label} is required")
        return None
    return plugin_validate_optional_trimmed_string(table, field, label, diagnostics)


def plugin_validate_optional_trimmed_string(
    table: dict[str, Any],
    field: str,
    label: str,
    diagnostics: list[str],
) -> str | None:
    if field not in table:
        return None
    value = table[field]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed string")
        return None
    return value


def plugin_validate_append_once(diagnostics: list[str], diagnostic: str) -> None:
    if diagnostic not in diagnostics:
        diagnostics.append(diagnostic)


def plugin_validate_allowed_string_values(
    values: list[str],
    label: str,
    allowed_values: tuple[str, ...],
    diagnostics: list[str],
) -> None:
    allowed = set(allowed_values)
    expected = ", ".join(allowed_values)
    for index, value in enumerate(values):
        if value not in allowed:
            diagnostics.append(
                f'{label}[{index}] "{value}" is unsupported; expected one of {expected}'
            )
