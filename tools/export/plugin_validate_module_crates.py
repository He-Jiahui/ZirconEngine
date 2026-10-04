"""Workspace crate ownership checks for plugin module rows."""

from __future__ import annotations

from pathlib import Path
from typing import Any

Diagnostics = list[str]


def validate_plugin_module_workspace_crate(
    crate_name: str,
    label: str,
    package_root: Path,
    plugin_root: Path | None,
    workspace_crate_index: dict[str, dict[str, Any]],
    diagnostics: Diagnostics,
) -> None:
    crate = workspace_crate_index.get(crate_name)
    if crate is None:
        diagnostics.append(
            f"{label} {crate_name} must be a zircon_plugins workspace member"
        )
        return
    manifest_path = crate.get("manifest_path")
    if not isinstance(manifest_path, Path):
        return
    if plugin_validate_path_is_relative_to(manifest_path.parent, package_root):
        return
    member = crate.get("member")
    member_text = member if isinstance(member, str) else str(manifest_path.parent)
    expected_root = plugin_validate_workspace_relative_path(plugin_root, package_root)
    diagnostics.append(
        f"{label} {crate_name} workspace member {member_text} "
        f"must stay under {expected_root}"
    )


def plugin_validate_optional_feature_root(
    package_root: Path,
    package_id: str,
    feature_id: str,
) -> Path:
    feature_suffix = (
        feature_id.removeprefix(f"{package_id}.")
        if feature_id.startswith(f"{package_id}.")
        else feature_id
    )
    return package_root / "features" / feature_suffix.replace(".", "/")


def plugin_validate_path_is_relative_to(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
    except ValueError:
        return False
    return True


def plugin_validate_workspace_relative_path(
    plugin_root: Path | None,
    path: Path,
) -> str:
    if plugin_root is None:
        return path.as_posix()
    try:
        return path.resolve().relative_to(plugin_root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()
