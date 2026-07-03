"""Top-level layout validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path

from .native_build_workspace import read_toml
from .plugin_validate_layout_coordinates import validate_plugin_layout_coordinates
from .plugin_validate_layout_public_metadata import (
    validate_plugin_layout_public_metadata,
)
from .plugin_validate_layout_roots import validate_plugin_layout_roots
from .plugin_validate_layout_targets import validate_plugin_layout_targets


Diagnostics = list[str]


def validate_plugin_layout(
    *,
    plugin_manifest_path: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    validate_plugin_layout_public_metadata(manifest, package_id, diagnostics)
    validate_plugin_layout_targets(manifest, package_id, diagnostics)
    validate_plugin_layout_roots(manifest, package_id, diagnostics)
    validate_plugin_layout_coordinates(manifest, package_id, diagnostics)
