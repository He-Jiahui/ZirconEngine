"""Distribution module binding checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build import read_toml
from .plugin_validate_common import (
    PLUGIN_VALIDATE_ROOT_SOURCE,
    plugin_validate_allowed_string_values,
    plugin_validate_append_once,
    plugin_validate_module_crate_names,
    plugin_validate_modules_array,
    plugin_validate_selected_feature,
    plugin_validate_string_array,
)


PLUGIN_VALIDATE_RUNTIME_TARGET_MODES = ("client_runtime", "server_runtime")
PLUGIN_VALIDATE_EDITOR_TARGET_MODE = "editor_host"
PLUGIN_VALIDATE_TARGET_MODES = (
    *PLUGIN_VALIDATE_RUNTIME_TARGET_MODES,
    PLUGIN_VALIDATE_EDITOR_TARGET_MODE,
)


def validate_plugin_distribution_modules(
    *,
    plugin_manifest_path: Path | None,
    requested_plugin_id: str,
    package_id: str,
    source_kind: str,
    dist_crate: str | None,
    runtime_entry: str | None,
    editor_entry: str | None,
    diagnostics: list[str],
) -> None:
    if plugin_manifest_path is None or dist_crate is None:
        return
    plugin_manifest = read_toml(plugin_manifest_path, diagnostics)
    if plugin_manifest is None:
        return
    modules = plugin_validate_distribution_modules(
        plugin_manifest=plugin_manifest,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        source_kind=source_kind,
        diagnostics=diagnostics,
    )
    module_crates = plugin_validate_module_crate_names(
        modules,
        package_id,
        diagnostics,
    )
    if dist_crate not in module_crates:
        diagnostics.append(
            f"plugin {package_id} distribution.dist_crate {dist_crate} "
            "is not declared by any module crate_name"
        )
    plugin_validate_distribution_module_target_modes(
        modules=modules,
        package_id=package_id,
        dist_crate=dist_crate,
        runtime_entry=runtime_entry,
        editor_entry=editor_entry,
        diagnostics=diagnostics,
    )


def plugin_validate_distribution_modules(
    *,
    plugin_manifest: dict[str, Any],
    requested_plugin_id: str,
    package_id: str,
    source_kind: str,
    diagnostics: list[str],
) -> list[Any]:
    if source_kind == PLUGIN_VALIDATE_ROOT_SOURCE:
        return plugin_validate_modules_array(
            plugin_manifest.get("modules", []),
            f"plugin {package_id} modules",
            diagnostics,
        )
    feature = plugin_validate_selected_feature(
        plugin_manifest,
        requested_plugin_id,
        package_id,
    )
    if feature is None:
        diagnostics.append(
            f"plugin {package_id} feature source was not found in manifest modules"
        )
        return []
    return plugin_validate_modules_array(
        feature.get("modules", []),
        f"plugin {package_id} optional feature modules",
        diagnostics,
    )


def plugin_validate_distribution_module_target_modes(
    *,
    modules: list[Any],
    package_id: str,
    dist_crate: str,
    runtime_entry: str | None,
    editor_entry: str | None,
    diagnostics: list[str],
) -> None:
    if runtime_entry is None and editor_entry is None:
        return
    dist_module_found = False
    dist_target_modes: list[str] = []
    for index, module in enumerate(modules):
        if not isinstance(module, dict):
            continue
        if module.get("crate_name") != dist_crate:
            continue
        dist_module_found = True
        target_modes = plugin_validate_string_array(
            module,
            "target_modes",
            (
                f"plugin {package_id} distribution.dist_crate {dist_crate} "
                f"modules[{index}].target_modes"
            ),
            diagnostics,
        )
        if target_modes is not None:
            plugin_validate_allowed_string_values(
                target_modes,
                (
                    f"plugin {package_id} distribution.dist_crate {dist_crate} "
                    f"modules[{index}].target_modes"
                ),
                PLUGIN_VALIDATE_TARGET_MODES,
                diagnostics,
            )
            dist_target_modes.extend(target_modes)
    if not dist_module_found or not dist_target_modes:
        return
    if runtime_entry is not None and not any(
        target_mode in PLUGIN_VALIDATE_RUNTIME_TARGET_MODES
        for target_mode in dist_target_modes
    ):
        plugin_validate_append_once(
            diagnostics,
            f"plugin {package_id} distribution.runtime_entry requires "
            "dist module target_modes to include client_runtime or server_runtime",
        )
    if editor_entry is not None and PLUGIN_VALIDATE_EDITOR_TARGET_MODE not in set(
        dist_target_modes
    ):
        plugin_validate_append_once(
            diagnostics,
            f"plugin {package_id} distribution.editor_entry requires "
            "dist module target_modes to include editor_host",
        )
