"""Top-level public metadata validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_trimmed_string


Diagnostics = list[str]
Manifest = dict[str, Any]


def validate_plugin_layout_public_metadata(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    plugin_validate_trimmed_string(
        manifest,
        "category",
        f"plugin {package_id} category",
        diagnostics,
    )
    validate_plugin_layout_description(manifest, package_id, diagnostics)


def validate_plugin_layout_description(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if "description" not in manifest:
        return
    value = manifest["description"]
    if not isinstance(value, str):
        diagnostics.append(f"plugin {package_id} description must be a string when present")
        return
    if value and value.strip() != value:
        diagnostics.append(f"plugin {package_id} description must be trimmed when present")
