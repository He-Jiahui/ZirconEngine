"""Global option key uniqueness checks for plugin validate --all."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml

Diagnostics = list[str]
Manifest = dict[str, Any]


def validate_plugin_option_global_keys(
    plugin_root: Path,
    diagnostics: Diagnostics,
) -> None:
    seen: dict[str, str] = {}
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        validate_plugin_option_manifest_global_keys(
            manifest, manifest_path, seen, diagnostics
        )


def validate_plugin_option_manifest_global_keys(
    manifest: Manifest,
    manifest_path: Path,
    seen: dict[str, str],
    diagnostics: Diagnostics,
) -> None:
    package_id = plugin_validate_option_global_package_label(manifest)
    options = manifest.get("options")
    if not isinstance(options, list):
        return
    for index, option in enumerate(options):
        if not isinstance(option, dict):
            continue
        key = option.get("key")
        if not isinstance(key, str) or not key.strip() or key.strip() != key:
            continue
        label = f"{package_id or manifest_path} options[{index}].key"
        previous = seen.get(key)
        if previous is not None:
            diagnostics.append(
                f"plugin validate options key {key} "
                f"is duplicated by {previous} and {label}"
            )
            continue
        seen[key] = label


def plugin_validate_option_global_package_label(manifest: Manifest) -> str | None:
    package_id = manifest.get("id")
    if not isinstance(package_id, str) or not package_id.strip():
        return None
    if package_id.strip() != package_id:
        return None
    return f"plugin {package_id}"
