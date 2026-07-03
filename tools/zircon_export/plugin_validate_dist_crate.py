"""Dist crate Cargo manifest preflight checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import PLUGIN_VALIDATE_DIST_FEATURE
from .plugin_validate_dist_crate_dependency import (
    validate_plugin_dist_crate_sdk_dependency,
)


def plugin_validate_dist_crate_manifest(
    crate_index: dict[str, dict[str, Any]],
    package_id: str,
    dist_crate: str | None,
    diagnostics: list[str],
) -> Path | None:
    if not dist_crate:
        return None
    crate = crate_index.get(dist_crate)
    if crate is None:
        diagnostics.append(
            f"plugin {package_id} distribution dist_crate {dist_crate} "
            "is not a cdylib workspace member"
        )
        return None
    manifest_path = crate.get("manifest_path")
    if isinstance(manifest_path, Path):
        return manifest_path
    return None


def validate_plugin_dist_crate_workspace_member(
    crate_index: dict[str, dict[str, Any]],
    package_id: str,
    dist_crate: str | None,
    diagnostics: list[str],
) -> Path | None:
    dist_crate_manifest = plugin_validate_dist_crate_manifest(
        crate_index,
        package_id,
        dist_crate,
        diagnostics,
    )
    if dist_crate_manifest is None or dist_crate is None:
        return dist_crate_manifest
    validate_plugin_dist_crate_feature(
        dist_crate_manifest,
        package_id,
        dist_crate,
        diagnostics,
    )
    return dist_crate_manifest


def validate_plugin_dist_crate_feature(
    dist_crate_manifest: Path,
    package_id: str,
    dist_crate: str,
    diagnostics: list[str],
) -> None:
    manifest = read_toml(dist_crate_manifest, diagnostics)
    if manifest is None:
        return
    features = manifest.get("features")
    if not isinstance(features, dict):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} Cargo.toml features "
            "must be a table"
        )
        return
    feature = features.get(PLUGIN_VALIDATE_DIST_FEATURE)
    if feature is None:
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} "
            f"must declare Cargo feature {PLUGIN_VALIDATE_DIST_FEATURE}"
        )
        return
    if not isinstance(feature, list):
        diagnostics.append(
            f"plugin {package_id} dist crate {dist_crate} "
            f"Cargo feature {PLUGIN_VALIDATE_DIST_FEATURE} must be an array"
        )
        return
    dist_feature_entries: list[str] = []
    for index, item in enumerate(feature):
        if not isinstance(item, str) or not item.strip():
            diagnostics.append(
                f"plugin {package_id} dist crate {dist_crate} "
                f"Cargo feature {PLUGIN_VALIDATE_DIST_FEATURE}[{index}] "
                "must be a non-empty trimmed string"
            )
            continue
        if item.strip() != item:
            diagnostics.append(
                f"plugin {package_id} dist crate {dist_crate} "
                f"Cargo feature {PLUGIN_VALIDATE_DIST_FEATURE}[{index}] "
                "must be a non-empty trimmed string"
            )
            continue
        dist_feature_entries.append(item)
    validate_plugin_dist_crate_sdk_dependency(
        manifest,
        package_id,
        dist_crate,
        dist_feature_entries,
        diagnostics,
    )
