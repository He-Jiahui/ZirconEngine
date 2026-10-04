"""Top-level root path validation for standalone plugin manifests."""

from __future__ import annotations

from typing import Any


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_LAYOUT_ROOT_FIELDS = ("asset_roots", "content_roots")
PLUGIN_VALIDATE_LAYOUT_ASSET_ROOT_DUPLICATE_MESSAGE = "duplicates asset_roots"
PLUGIN_VALIDATE_LAYOUT_CONTENT_ROOT_DUPLICATE_MESSAGE = "duplicates content_roots"


def validate_plugin_layout_roots(
    manifest: Manifest,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    for field in PLUGIN_VALIDATE_LAYOUT_ROOT_FIELDS:
        duplicate_message = (
            PLUGIN_VALIDATE_LAYOUT_ASSET_ROOT_DUPLICATE_MESSAGE
            if field == "asset_roots"
            else PLUGIN_VALIDATE_LAYOUT_CONTENT_ROOT_DUPLICATE_MESSAGE
        )
        validate_plugin_layout_root_array(
            manifest,
            field,
            f"plugin {package_id} {field}",
            duplicate_message,
            diagnostics,
        )


def validate_plugin_layout_root_array(
    manifest: Manifest,
    field: str,
    label: str,
    duplicate_message: str,
    diagnostics: Diagnostics,
) -> None:
    if field not in manifest:
        return
    roots = manifest[field]
    if not isinstance(roots, list):
        diagnostics.append(f"{label} must be an array")
        return
    seen: dict[str, int] = {}
    for index, root in enumerate(roots):
        item_label = f"{label}[{index}]"
        if not isinstance(root, str) or not root.strip() or root.strip() != root:
            diagnostics.append(f"{item_label} must be a non-empty trimmed string")
            continue
        previous_index = seen.get(root)
        if previous_index is not None:
            diagnostics.append(
                f"{item_label} {root} {duplicate_message}[{previous_index}]"
            )
        else:
            seen[root] = index
        validate_plugin_layout_root_path(item_label, root, diagnostics)


def validate_plugin_layout_root_path(
    label: str,
    root: str,
    diagnostics: Diagnostics,
) -> None:
    if root.startswith("/") or root.startswith("\\"):
        diagnostics.append(f"{label} {root} must be relative")
    if plugin_validate_layout_root_has_drive_separator(root):
        diagnostics.append(f"{label} {root} must not contain a drive separator")
    if "\\" in root:
        diagnostics.append(f"{label} {root} must use forward slashes")
    if any(segment in {"", ".", ".."} for segment in root.split("/")):
        diagnostics.append(
            f"{label} {root} must not contain empty, current, or parent path segments"
        )


def plugin_validate_layout_root_has_drive_separator(root: str) -> bool:
    return len(root) >= 2 and root[1] == ":" and root[0].isascii() and root[0].isalpha()
