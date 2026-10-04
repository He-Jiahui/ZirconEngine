"""Shared plugin package source resolution for build and validate commands."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml, resolve_native_build_path
from .plugin_package_identity import feature_provider_package_id
from .plugin_package_template import feature_provider_package_manifest_template


PLUGIN_PACKAGE_DIST_FORM = "dist"


@dataclass(frozen=True)
class PluginPackageSource:
    package_id: str
    plugin_manifest_path: Path
    distribution: dict[str, Any] | None
    package_manifest_text: str | None = None


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_plugin_package_path(
    label: str,
    path: Path | None,
    diagnostics: list[str],
) -> Path | None:
    if path is None:
        diagnostics.append(f"{label} is required")
        return None
    return resolve_native_build_path(label, path.expanduser(), diagnostics)


def resolve_plugin_package_source(
    plugin_root: Path,
    plugin_id: str,
    diagnostics: list[str],
) -> PluginPackageSource | None:
    direct = plugin_root / plugin_id / "plugin.toml"
    if direct.exists():
        manifest = read_toml(direct, diagnostics)
        return root_plugin_package_source(direct, manifest, plugin_id, diagnostics)
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        if manifest.get("id") == plugin_id:
            return root_plugin_package_source(
                manifest_path,
                manifest,
                plugin_id,
                diagnostics,
            )
        feature_source = feature_provider_plugin_package_source(
            manifest_path,
            manifest,
            plugin_id,
            diagnostics,
        )
        if feature_source is not None:
            return feature_source
    diagnostics.append(f"plugin {plugin_id} plugin.toml was not found under {plugin_root}")
    return None


def root_plugin_package_source(
    plugin_manifest_path: Path,
    plugin_manifest: dict[str, Any] | None,
    requested_plugin_id: str,
    diagnostics: list[str],
) -> PluginPackageSource | None:
    package_id = plugin_package_id(plugin_manifest, requested_plugin_id, diagnostics)
    distribution = plugin_distribution(plugin_manifest, package_id, diagnostics)
    return PluginPackageSource(
        package_id=package_id,
        plugin_manifest_path=plugin_manifest_path,
        distribution=distribution,
    )


def feature_provider_plugin_package_source(
    plugin_manifest_path: Path,
    plugin_manifest: dict[str, Any],
    requested_plugin_id: str,
    diagnostics: list[str],
) -> PluginPackageSource | None:
    owner_plugin_id = plugin_manifest.get("id")
    optional_features = plugin_manifest.get("optional_features", [])
    if not isinstance(owner_plugin_id, str) or not owner_plugin_id.strip():
        return None
    if not isinstance(optional_features, list):
        return None
    for feature in optional_features:
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not isinstance(feature_id, str) or not feature_id.strip():
            continue
        provider_package_id = feature_provider_package_id(feature, feature_id)
        if requested_plugin_id not in {feature_id, provider_package_id}:
            continue
        if not provider_package_id:
            diagnostics.append(
                f"plugin feature {feature_id} provider_package_id must be a string"
            )
            return PluginPackageSource(
                package_id=requested_plugin_id,
                plugin_manifest_path=plugin_manifest_path,
                distribution=None,
            )
        distribution = feature_provider_distribution(
            feature,
            provider_package_id,
            diagnostics,
        )
        package_manifest_text = (
            feature_provider_package_manifest_template(
                owner_manifest=plugin_manifest,
                feature=feature,
                provider_package_id=provider_package_id,
                distribution=distribution,
            )
            if distribution is not None
            else None
        )
        return PluginPackageSource(
            package_id=provider_package_id,
            plugin_manifest_path=plugin_manifest_path,
            distribution=distribution,
            package_manifest_text=package_manifest_text,
        )
    return None


def feature_provider_distribution(
    feature: dict[str, Any],
    provider_package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    feature_id = feature.get("id")
    distribution = feature.get("distribution")
    if not isinstance(distribution, dict):
        diagnostics.append(
            f"plugin feature {feature_id} provider {provider_package_id} has no distribution table"
        )
        return None
    return plugin_distribution_contract(distribution, provider_package_id, diagnostics)


def plugin_package_id(
    plugin_manifest: dict[str, Any] | None,
    requested_plugin_id: str,
    diagnostics: list[str],
) -> str:
    if plugin_manifest is None:
        return requested_plugin_id
    package_id = plugin_manifest.get("id")
    if not isinstance(package_id, str) or not package_id.strip():
        diagnostics.append(f"plugin {requested_plugin_id} plugin.toml id must be a string")
        return requested_plugin_id
    if package_id.strip() != package_id:
        diagnostics.append(f"plugin {requested_plugin_id} plugin.toml id must be trimmed")
        return requested_plugin_id
    if package_id != requested_plugin_id:
        diagnostics.append(
            f"plugin manifest id {package_id} does not match requested id {requested_plugin_id}"
        )
    return package_id


def plugin_distribution(
    plugin_manifest: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if plugin_manifest is None:
        return None
    distribution = plugin_manifest.get("distribution")
    if not isinstance(distribution, dict):
        diagnostics.append(f"plugin {package_id} has no [distribution] table")
        return None
    return plugin_distribution_contract(distribution, package_id, diagnostics)


def plugin_distribution_contract(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any]:
    forms = distribution.get("forms", [])
    if not isinstance(forms, list) or PLUGIN_PACKAGE_DIST_FORM not in forms:
        diagnostics.append(f"plugin {package_id} distribution.forms must include dist")
    return distribution
