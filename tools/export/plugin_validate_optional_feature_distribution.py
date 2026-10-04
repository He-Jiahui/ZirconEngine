"""Root optional feature distribution validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_distribution_contract import validate_plugin_distribution

Diagnostics = list[str]
Manifest = dict[str, Any]


def validate_plugin_optional_feature_distribution(
    *,
    plugin_manifest_path: Path | None,
    package_id: str,
    engine_version: str | None,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    optional_features = manifest.get("optional_features")
    if optional_features is None or not isinstance(optional_features, list):
        return
    for index, feature in enumerate(optional_features):
        if not isinstance(feature, dict):
            continue
        validate_plugin_optional_feature_distribution_row(
            feature,
            index,
            plugin_manifest_path,
            package_id,
            engine_version,
            diagnostics,
        )


def validate_plugin_optional_feature_distribution_row(
    feature: Manifest,
    index: int,
    plugin_manifest_path: Path,
    package_id: str,
    engine_version: str | None,
    diagnostics: Diagnostics,
) -> None:
    distribution = feature.get("distribution")
    if distribution is None:
        return
    distribution_label = f"plugin {package_id} optional_features[{index}].distribution"
    if not isinstance(distribution, dict):
        diagnostics.append(f"{distribution_label} must be a table")
        return
    validate_plugin_distribution(
        distribution,
        package_id,
        diagnostics,
        plugin_manifest_path=plugin_manifest_path,
        engine_version=engine_version,
        distribution_label=distribution_label,
    )
