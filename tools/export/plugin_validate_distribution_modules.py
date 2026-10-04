"""Distribution module binding checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import (
    PLUGIN_VALIDATE_ROOT_SOURCE,
    plugin_validate_module_crate_names,
    plugin_validate_modules_array,
    plugin_validate_selected_feature,
)
from .plugin_validate_distribution_module_target_modes import (
    plugin_validate_distribution_module_target_modes,
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
