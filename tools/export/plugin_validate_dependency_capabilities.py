"""Dependency capability resolution checks for plugin validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml

Diagnostics = list[str]
Manifest = dict[str, Any]
CapabilityIndex = dict[str, set[str]]


def plugin_validate_dependency_capability_target_index(
    plugin_root: Path | None,
    diagnostics: Diagnostics,
) -> CapabilityIndex:
    targets: CapabilityIndex = {}
    if plugin_root is None or not plugin_root.exists():
        return targets
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        package_id = manifest.get("id")
        if not isinstance(package_id, str) or not package_id.strip():
            continue
        if package_id.strip() != package_id:
            continue
        capabilities = targets.setdefault(package_id, set())
        plugin_validate_collect_dependency_capabilities(manifest, capabilities)
    return targets


def validate_plugin_dependency_capability_gate(
    dependency: Manifest,
    dependency_label: str,
    capability_targets: CapabilityIndex,
    diagnostics: Diagnostics,
) -> None:
    capability = dependency.get("capability")
    if not isinstance(capability, str) or not capability.strip():
        return
    if capability.strip() != capability:
        return
    dependency_id = dependency.get("id")
    if not isinstance(dependency_id, str) or not dependency_id.strip():
        return
    if dependency_id.strip() != dependency_id:
        return

    target_capabilities = capability_targets.get(dependency_id)
    if target_capabilities is not None:
        if capability in target_capabilities:
            return
        diagnostics.append(
            f"{dependency_label}.capability {capability} should be declared by "
            "the referenced static plugin package or one of its feature rows"
        )
        return
    if plugin_validate_dependency_capability_is_host_owned(capability):
        return
    diagnostics.append(
        f"{dependency_label}.capability {capability} references no static "
        "plugin package and should use a runtime.module.* or runtime.capability.* "
        "host namespace"
    )


def plugin_validate_collect_dependency_capabilities(
    manifest: Manifest,
    capabilities: set[str],
) -> None:
    plugin_validate_collect_capability_rows(manifest, capabilities)
    for feature in plugin_validate_dependency_table_rows(
        manifest.get("optional_features")
    ):
        plugin_validate_collect_capability_rows(feature, capabilities)
    for extension in plugin_validate_dependency_table_rows(
        manifest.get("feature_extensions")
    ):
        plugin_validate_collect_capability_rows(extension, capabilities)


def plugin_validate_collect_capability_rows(
    table: Manifest,
    capabilities: set[str],
) -> None:
    values = table.get("capabilities")
    if not isinstance(values, list):
        return
    for value in values:
        if isinstance(value, str) and value.strip() and value.strip() == value:
            capabilities.add(value)


def plugin_validate_dependency_table_rows(value: Any) -> list[Manifest]:
    if not isinstance(value, list):
        return []
    return [row for row in value if isinstance(row, dict)]


def plugin_validate_dependency_capability_is_host_owned(capability: str) -> bool:
    return capability.startswith("runtime.module.") or capability.startswith(
        "runtime.capability."
    )
