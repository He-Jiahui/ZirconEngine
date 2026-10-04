"""All-target discovery for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_package_identity import feature_provider_package_id
from .plugin_validate_common import plugin_validate_manifest_target_id


def plugin_validate_discover_target_ids(
    plugin_root: Path,
    diagnostics: list[str],
) -> list[str]:
    targets: list[str] = []
    seen: dict[str, str] = {}
    if not plugin_root.exists():
        diagnostics.append(f"plugin root {plugin_root} does not exist")
        return targets
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        root_id = plugin_validate_manifest_target_id(
            manifest,
            f"{manifest_path} id",
            diagnostics,
        )
        root_distribution = manifest.get("distribution")
        if root_distribution is not None and not isinstance(root_distribution, dict):
            diagnostics.append(f"{manifest_path} distribution must be a table")
        elif isinstance(root_distribution, dict) and root_id is not None:
            plugin_validate_append_target(
                targets,
                seen,
                root_id,
                str(manifest_path),
                diagnostics,
            )
        optional_features = manifest.get("optional_features", [])
        if optional_features in (None, []):
            continue
        if not isinstance(optional_features, list):
            diagnostics.append(f"{manifest_path} optional_features must be an array")
            continue
        for index, feature in enumerate(optional_features):
            feature_label = f"{manifest_path} optional_features[{index}]"
            if not isinstance(feature, dict):
                diagnostics.append(f"{feature_label} must be a table")
                continue
            feature_distribution = feature.get("distribution")
            if feature_distribution is None:
                continue
            if not isinstance(feature_distribution, dict):
                diagnostics.append(f"{feature_label}.distribution must be a table")
                continue
            feature_id = plugin_validate_manifest_target_id(
                feature,
                f"{feature_label}.id",
                diagnostics,
                field="id",
            )
            if feature_id is None:
                continue
            provider_id = feature_provider_package_id(feature, feature_id)
            if provider_id is None:
                diagnostics.append(
                    f"{feature_label}.provider_package_id must be a non-empty trimmed string"
                )
                continue
            plugin_validate_append_target(
                targets,
                seen,
                provider_id,
                feature_label,
                diagnostics,
            )
    return targets


def plugin_validate_append_target(
    targets: list[str],
    seen: dict[str, str],
    target_id: str,
    source: str,
    diagnostics: list[str],
) -> None:
    previous = seen.get(target_id)
    if previous is not None:
        diagnostics.append(
            f"plugin validate target {target_id} is duplicated by {previous} and {source}"
        )
        return
    seen[target_id] = source
    targets.append(target_id)
