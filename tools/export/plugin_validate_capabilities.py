"""Root capability declaration validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_string_array


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_CAPABILITIES_DUPLICATE_MESSAGE = "duplicates capabilities"


def validate_plugin_capabilities(
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
    label = f"plugin {package_id} capabilities"
    capabilities = plugin_validate_string_array(
        manifest, "capabilities", label, diagnostics
    )
    if capabilities is None:
        return
    validate_plugin_capability_values(capabilities, label, diagnostics)


def validate_plugin_capability_values(
    capabilities: list[str],
    label: str,
    diagnostics: Diagnostics,
) -> None:
    seen: dict[str, int] = {}
    for index, capability in enumerate(capabilities):
        item_label = f"{label}[{index}]"
        validate_plugin_capability_namespace(capability, item_label, diagnostics)
        previous_index = seen.get(capability)
        if previous_index is not None:
            diagnostics.append(
                f"{item_label} {capability} "
                f"{PLUGIN_VALIDATE_CAPABILITIES_DUPLICATE_MESSAGE} "
                f"capabilities[{previous_index}]"
            )
            continue
        seen[capability] = index


def validate_plugin_capability_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} {value} should use at least two dot-separated namespace segments"
        )
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} {value} should not contain empty namespace segments")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )
