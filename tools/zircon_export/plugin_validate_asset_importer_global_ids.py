"""Global asset_importers id uniqueness checks for plugin validate --all."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml

Manifest = dict[str, Any]
Diagnostics = list[str]


def validate_plugin_asset_importer_global_ids(
    plugin_root: Path,
    diagnostics: Diagnostics,
) -> None:
    seen: dict[str, str] = {}
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        validate_plugin_asset_importer_manifest_global_ids(
            manifest, manifest_path, seen, diagnostics
        )


def validate_plugin_asset_importer_manifest_global_ids(
    manifest: Manifest,
    manifest_path: Path,
    seen: dict[str, str],
    diagnostics: Diagnostics,
) -> None:
    package_id = plugin_validate_asset_importer_global_package_label(manifest)
    asset_importers = manifest.get("asset_importers")
    if not isinstance(asset_importers, list):
        return
    for index, importer in enumerate(asset_importers):
        if not isinstance(importer, dict):
            continue
        importer_id = importer.get("id")
        if (
            not isinstance(importer_id, str)
            or not importer_id.strip()
            or importer_id.strip() != importer_id
        ):
            continue
        label = f"{package_id or manifest_path} asset_importers[{index}].id"
        previous = seen.get(importer_id)
        if previous is not None:
            diagnostics.append(
                f"plugin validate asset_importers id {importer_id} "
                f"is duplicated by {previous} and {label}"
            )
            continue
        seen[importer_id] = label


def plugin_validate_asset_importer_global_package_label(
    manifest: Manifest,
) -> str | None:
    package_id = manifest.get("id")
    if not isinstance(package_id, str) or not package_id.strip():
        return None
    if package_id.strip() != package_id:
        return None
    return f"plugin {package_id}"
