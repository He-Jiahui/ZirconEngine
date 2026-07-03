"""Plugin option validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_asset_importer_required_capability_gates import (
    PLUGIN_VALIDATE_REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC,
    plugin_validate_required_capability_is_host_owned,
    plugin_validate_static_declared_capabilities,
)
from .plugin_validate_option_schema import validate_plugin_option_schema

Diagnostics = list[str]
OptionRow = dict[str, Any]


def validate_plugin_options(
    *,
    plugin_manifest_path: Path | None,
    plugin_root: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    options = manifest.get("options")
    if options is None:
        return
    label = f"plugin {package_id} options"
    if not isinstance(options, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not options:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    declared_capabilities = plugin_validate_static_declared_capabilities(
        plugin_root, diagnostics
    )
    for index, option in enumerate(options):
        option_label = f"{label}[{index}]"
        if not isinstance(option, dict):
            diagnostics.append(f"{option_label} must be a table")
            continue
        validate_plugin_option_schema(option, option_label, diagnostics)
        validate_plugin_option_required_capability(
            option, option_label, declared_capabilities, diagnostics
        )


def validate_plugin_option_required_capability(
    option: OptionRow,
    option_label: str,
    declared_capabilities: set[str],
    diagnostics: Diagnostics,
) -> None:
    if "required_capability" not in option:
        return
    value = option["required_capability"]
    label = f"{option_label}.required_capability"
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        diagnostics.append(f"{label} must be a non-empty trimmed string")
        return
    if value in declared_capabilities:
        return
    if plugin_validate_required_capability_is_host_owned(value):
        return
    diagnostics.append(
        f"{label} {value} "
        f"{PLUGIN_VALIDATE_REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC}"
    )
