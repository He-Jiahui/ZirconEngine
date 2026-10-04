"""Module row validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_module_crates import plugin_validate_optional_feature_root
from .plugin_validate_module_rows import (
    Diagnostics,
    ModuleRowContext,
    validate_plugin_module_rows,
)

Manifest = dict[str, Any]


def validate_plugin_modules(
    *, plugin_manifest_path: Path | None, plugin_root: Path | None, package_id: str,
    workspace_crate_index: dict[str, dict[str, Any]], diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    supported_targets = plugin_validate_root_supported_targets(manifest)
    seen_names: dict[str, str] = {}
    row_context: ModuleRowContext = (plugin_root, workspace_crate_index, supported_targets, seen_names, diagnostics)
    validate_plugin_module_rows(
        manifest.get("modules"), f"plugin {package_id} modules", package_id,
        plugin_manifest_path.parent, *row_context,
    )
    optional_features = manifest.get("optional_features")
    if isinstance(optional_features, list):
        for feature_index, feature in enumerate(optional_features):
            if not isinstance(feature, dict):
                continue
            feature_id = feature.get("id")
            if not isinstance(feature_id, str) or not feature_id.strip():
                continue
            if feature_id.strip() != feature_id:
                continue
            feature_root = plugin_validate_optional_feature_root(
                plugin_manifest_path.parent, package_id, feature_id
            )
            validate_plugin_module_rows(
                feature.get("modules"),
                f"plugin {package_id} optional_features[{feature_index}].modules",
                feature_id, feature_root, *row_context,
            )
    validate_plugin_feature_extension_modules(
        manifest.get("feature_extensions"), plugin_manifest_path.parent, package_id, row_context,
    )


def validate_plugin_feature_extension_modules(
    feature_extensions: Any, package_root: Path, package_id: str, row_context: ModuleRowContext,
) -> None:
    if not isinstance(feature_extensions, list):
        return
    for feature_index, feature in enumerate(feature_extensions):
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not isinstance(feature_id, str) or not feature_id.strip():
            continue
        if feature_id.strip() != feature_id:
            continue
        validate_plugin_module_rows(
            feature.get("modules"),
            f"plugin {package_id} feature_extensions[{feature_index}].modules",
            feature_id, package_root, *row_context,
        )


def plugin_validate_root_supported_targets(manifest: Manifest) -> set[str]:
    supported_targets = manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }
