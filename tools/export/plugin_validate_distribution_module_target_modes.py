"""Target-mode checks for distribution module validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    plugin_validate_allowed_string_values,
    plugin_validate_append_once,
    plugin_validate_string_array,
)


PLUGIN_VALIDATE_RUNTIME_TARGET_MODES = ("client_runtime", "server_runtime")
PLUGIN_VALIDATE_EDITOR_TARGET_MODE = "editor_host"
PLUGIN_VALIDATE_TARGET_MODES = (
    *PLUGIN_VALIDATE_RUNTIME_TARGET_MODES,
    PLUGIN_VALIDATE_EDITOR_TARGET_MODE,
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
