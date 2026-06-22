"""NativeDynamic loader manifest parsing diagnostics."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any


NATIVE_DYNAMIC_LOADER_MANIFEST_FIELDS = frozenset(("plugins",))
NATIVE_DYNAMIC_LOADER_MANIFEST_PLUGIN_FIELDS = frozenset(
    ("id", "path", "manifest", "package_report", "abi")
)
NATIVE_DYNAMIC_LOADER_MANIFEST_PLUGIN_STRING_FIELDS = (
    "path",
    "manifest",
    "package_report",
)


def native_dynamic_loader_manifest_plugins_or_diagnostics(
    loader_manifest: Path,
    *,
    label: str,
) -> tuple[list[dict[str, Any]] | None, list[str]]:
    try:
        with loader_manifest.open("rb") as manifest_file:
            document = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        return None, [f"{label} {loader_manifest} could not be parsed: {error}"]
    except OSError as error:
        return None, [f"{label} {loader_manifest} could not be read: {error}"]

    top_level_diagnostics = [
        f"{label} {field} is not supported"
        for field in document
        if field not in NATIVE_DYNAMIC_LOADER_MANIFEST_FIELDS
    ]
    if top_level_diagnostics:
        return None, top_level_diagnostics

    plugins = document.get("plugins")
    if not isinstance(plugins, list):
        return None, [f"{label} plugins must be an array"]

    diagnostics: list[str] = []
    normalized_plugins: list[dict[str, Any]] = []
    for index, plugin in enumerate(plugins):
        if not isinstance(plugin, dict):
            diagnostics.append(f"{label} plugins[{index}] must be a table")
            continue
        plugin_id = plugin.get("id")
        if not isinstance(plugin_id, str) or not plugin_id.strip():
            diagnostics.append(
                f"{label} plugins[{index}].id must be a non-empty string"
            )
            continue
        if plugin_id.strip() != plugin_id:
            diagnostics.append(
                f"{label} plugins[{index}].id must be a non-empty trimmed string"
            )
            continue
        plugin_has_diagnostics = False
        for field in plugin:
            if field not in NATIVE_DYNAMIC_LOADER_MANIFEST_PLUGIN_FIELDS:
                diagnostics.append(
                    f"{label} plugins[{index}].{field} is not supported"
                )
                plugin_has_diagnostics = True
        for field in NATIVE_DYNAMIC_LOADER_MANIFEST_PLUGIN_STRING_FIELDS:
            if field in plugin and not isinstance(plugin.get(field), str):
                diagnostics.append(f"{label} plugins[{index}].{field} must be a string")
                plugin_has_diagnostics = True
            elif (
                field in plugin
                and isinstance(plugin.get(field), str)
                and not str(plugin.get(field)).strip()
            ):
                diagnostics.append(
                    f"{label} plugins[{index}].{field} must be a non-empty string"
                )
                plugin_has_diagnostics = True
            elif (
                field in plugin
                and isinstance(plugin.get(field), str)
                and str(plugin.get(field)).strip() != str(plugin.get(field))
            ):
                diagnostics.append(
                    f"{label} plugins[{index}].{field} "
                    "must be a non-empty trimmed string"
                )
                plugin_has_diagnostics = True
        plugin_abi = plugin.get("abi")
        if plugin_abi is not None and not isinstance(plugin_abi, dict):
            diagnostics.append(f"{label} plugins[{index}].abi must be a table")
            plugin_has_diagnostics = True
        if plugin_has_diagnostics:
            continue
        normalized_plugins.append(plugin)
    if diagnostics:
        return None, diagnostics
    return normalized_plugins, []


def native_dynamic_loader_manifest_row_field_diagnostics(
    loader_plugins: list[dict[str, Any]],
    expected_plugins_by_id: dict[str, dict[str, Any]],
    *,
    label: str,
    expected_label: str,
    fields: tuple[str, ...] = ("path", "manifest", "package_report"),
    require_fields: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    for plugin in loader_plugins:
        plugin_id = str(plugin["id"])
        expected_plugin = expected_plugins_by_id.get(plugin_id)
        if expected_plugin is None:
            continue
        for field in fields:
            expected_value = expected_plugin.get(field)
            plugin_value = plugin.get(field)
            if require_fields and expected_value is not None and field not in plugin:
                diagnostics.append(
                    f"{label} plugin {plugin_id} {field} "
                    f"is required by {expected_label}"
                )
                continue
            if expected_value is not None and plugin_value is not None:
                if plugin_value != expected_value:
                    diagnostics.append(
                        f"{label} plugin {plugin_id} {field} {plugin_value} "
                        f"does not match {expected_label} {field} "
                        f"{expected_value}"
                    )
        plugin_abi = plugin.get("abi")
        expected_abi = expected_plugin.get("abi")
        if require_fields and isinstance(expected_abi, dict) and "abi" not in plugin:
            diagnostics.append(
                f"{label} plugin {plugin_id} abi is required by {expected_label}"
            )
            continue
        if not isinstance(plugin_abi, dict) or not isinstance(expected_abi, dict):
            continue
        for field in plugin_abi:
            if field not in expected_abi:
                diagnostics.append(
                    f"{label} plugin {plugin_id} abi.{field} "
                    f"is not supported by {expected_label}"
                )
        invalid_abi_fields = native_dynamic_loader_manifest_abi_field_type_diagnostics(
            plugin_id,
            plugin_abi,
            expected_abi,
            label=label,
        )
        diagnostics.extend(invalid_abi_fields.values())
        for field, expected_value in expected_abi.items():
            if field not in plugin_abi or field in invalid_abi_fields:
                continue
            plugin_value = plugin_abi.get(field)
            if plugin_value != expected_value:
                diagnostics.append(
                    f"{label} plugin {plugin_id} abi.{field} {plugin_value} "
                    f"does not match {expected_label} abi.{field} "
                    f"{expected_value}"
                )
    return diagnostics


def native_dynamic_loader_manifest_abi_field_type_diagnostics(
    plugin_id: str,
    plugin_abi: dict[str, Any],
    expected_abi: dict[str, Any],
    *,
    label: str,
) -> dict[str, str]:
    diagnostics: dict[str, str] = {}
    for field, expected_value in expected_abi.items():
        if field not in plugin_abi:
            diagnostics[field] = (
                f"{label} plugin {plugin_id} abi.{field} "
                "is required when abi is present"
            )
            continue
        plugin_value = plugin_abi.get(field)
        if isinstance(expected_value, int):
            if not isinstance(plugin_value, int) or isinstance(plugin_value, bool):
                diagnostics[field] = (
                    f"{label} plugin {plugin_id} abi.{field} must be an integer"
                )
        elif isinstance(expected_value, str):
            if not isinstance(plugin_value, str):
                diagnostics[field] = (
                    f"{label} plugin {plugin_id} abi.{field} must be a string"
                )
            elif not plugin_value.strip():
                diagnostics[field] = (
                    f"{label} plugin {plugin_id} abi.{field} "
                    "must be a non-empty string"
                )
            elif plugin_value.strip() != plugin_value:
                diagnostics[field] = (
                    f"{label} plugin {plugin_id} abi.{field} "
                    "must be a non-empty trimmed string"
                )
    return diagnostics
