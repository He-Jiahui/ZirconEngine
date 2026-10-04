"""Top-level target and platform validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_string_array


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_LAYOUT_SUPPORTED_TARGETS = (
    "client_runtime",
    "server_runtime",
    "editor_host",
)
PLUGIN_VALIDATE_LAYOUT_SUPPORTED_TARGET_DUPLICATE_MESSAGE = "duplicates supported_targets"
PLUGIN_VALIDATE_LAYOUT_SUPPORTED_PLATFORMS = (
    "windows",
    "linux",
    "macos",
    "android",
    "ios",
    "web_gpu",
    "wasm",
    "headless",
    "windows-x86_64",
    "linux-x86_64",
    "macos-aarch64",
)
PLUGIN_VALIDATE_LAYOUT_SUPPORTED_PLATFORM_DUPLICATE_MESSAGE = (
    "duplicates supported_platforms"
)
PLUGIN_VALIDATE_LAYOUT_PLATFORM_ALIASES = {
    "windows-x86_64": "windows",
    "linux-x86_64": "linux",
    "macos-aarch64": "macos",
}


def validate_plugin_layout_targets(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    validate_plugin_layout_string_set(
        manifest,
        "supported_targets",
        f"plugin {package_id} supported_targets",
        PLUGIN_VALIDATE_LAYOUT_SUPPORTED_TARGETS,
        {},
        PLUGIN_VALIDATE_LAYOUT_SUPPORTED_TARGET_DUPLICATE_MESSAGE,
        diagnostics,
    )
    validate_plugin_layout_string_set(
        manifest,
        "supported_platforms",
        f"plugin {package_id} supported_platforms",
        PLUGIN_VALIDATE_LAYOUT_SUPPORTED_PLATFORMS,
        PLUGIN_VALIDATE_LAYOUT_PLATFORM_ALIASES,
        PLUGIN_VALIDATE_LAYOUT_SUPPORTED_PLATFORM_DUPLICATE_MESSAGE,
        diagnostics,
    )


def validate_plugin_layout_string_set(
    manifest: Manifest,
    field: str,
    label: str,
    allowed_values: tuple[str, ...],
    aliases: dict[str, str],
    duplicate_message: str,
    diagnostics: Diagnostics,
) -> None:
    values = plugin_validate_string_array(manifest, field, label, diagnostics)
    if values is None:
        return
    allowed = set(allowed_values)
    expected = ", ".join(allowed_values)
    seen: dict[str, int] = {}
    for index, value in enumerate(values):
        if value not in allowed:
            diagnostics.append(
                f'{label}[{index}] "{value}" is unsupported; expected one of {expected}'
            )
            continue
        canonical = aliases.get(value, value)
        previous_index = seen.get(canonical)
        if previous_index is not None:
            diagnostics.append(
                f"{label}[{index}] {value} {duplicate_message}[{previous_index}]"
            )
        else:
            seen[canonical] = index
