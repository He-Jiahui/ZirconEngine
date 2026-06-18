"""NativeDynamic loader manifest parsing diagnostics."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any


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
        if not isinstance(plugin_id, str) or not plugin_id:
            diagnostics.append(
                f"{label} plugins[{index}].id must be a non-empty string"
            )
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
            if expected_value is not None and plugin_value is not None:
                if plugin_value != expected_value:
                    diagnostics.append(
                        f"{label} plugin {plugin_id} {field} {plugin_value} "
                        f"does not match {expected_label} {field} "
                        f"{expected_value}"
                    )
        plugin_abi = plugin.get("abi")
        expected_abi = expected_plugin.get("abi")
        if not isinstance(plugin_abi, dict) or not isinstance(expected_abi, dict):
            continue
        for field, expected_value in expected_abi.items():
            if field not in plugin_abi:
                continue
            plugin_value = plugin_abi.get(field)
            if plugin_value != expected_value:
                diagnostics.append(
                    f"{label} plugin {plugin_id} abi.{field} {plugin_value} "
                    f"does not match {expected_label} abi.{field} "
                    f"{expected_value}"
                )
    return diagnostics
