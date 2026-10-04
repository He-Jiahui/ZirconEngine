"""Global identity checks shared by the standalone plugin validator."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_asset_importer_global_ids import (
    validate_plugin_asset_importer_global_ids,
)
from .plugin_validate_option_global_keys import validate_plugin_option_global_keys


def validate_plugin_global_identities(
    plugin_root: Path,
    diagnostics: list[str],
) -> None:
    manifests: list[tuple[Path, dict[str, Any]]] = []
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is not None:
            manifests.append((manifest_path, manifest))
    validate_plugin_option_global_keys(plugin_root, diagnostics, manifests)
    validate_plugin_asset_importer_global_ids(plugin_root, diagnostics, manifests)
