"""asset_importers required capability declared/host gates."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml

Diagnostics = list[str]
Importer = dict[str, Any]
Manifest = dict[str, Any]
PLUGIN_VALIDATE_REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC = (
    "should reference a declared static package/feature capability "
    "or an explicitly host-owned capability"
)


def plugin_validate_static_declared_capabilities(
    plugin_root: Path | None,
    diagnostics: Diagnostics,
) -> set[str]:
    capabilities: set[str] = set()
    if plugin_root is None or not plugin_root.exists():
        return capabilities
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        plugin_validate_collect_manifest_capabilities(manifest, capabilities)
    return capabilities


def validate_plugin_asset_importer_required_capability_gates(
    importer: Importer,
    importer_label: str,
    declared_capabilities: set[str],
    diagnostics: Diagnostics,
) -> None:
    values = importer.get("required_capabilities")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.required_capabilities"
    for index, value in enumerate(values):
        if not isinstance(value, str) or not value.strip() or value.strip() != value:
            continue
        if value in declared_capabilities:
            continue
        if plugin_validate_required_capability_is_host_owned(value):
            continue
        diagnostics.append(
            f"{label}[{index}] {value} "
            f"{PLUGIN_VALIDATE_REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC}"
        )


def plugin_validate_collect_manifest_capabilities(
    manifest: Manifest,
    capabilities: set[str],
) -> None:
    plugin_validate_collect_capabilities(manifest, capabilities)
    for feature in plugin_validate_table_rows(manifest.get("optional_features")):
        plugin_validate_collect_capabilities(feature, capabilities)
    for extension in plugin_validate_table_rows(manifest.get("feature_extensions")):
        plugin_validate_collect_capabilities(extension, capabilities)


def plugin_validate_collect_capabilities(
    table: Manifest,
    capabilities: set[str],
) -> None:
    values = table.get("capabilities")
    if not isinstance(values, list):
        return
    for value in values:
        if isinstance(value, str) and value.strip() and value.strip() == value:
            capabilities.add(value)


def plugin_validate_table_rows(value: Any) -> list[Manifest]:
    if not isinstance(value, list):
        return []
    return [row for row in value if isinstance(row, dict)]


def plugin_validate_required_capability_is_host_owned(capability: str) -> bool:
    return capability.startswith(
        "runtime.capability."
    ) or capability == "runtime.asset.importer.native"
