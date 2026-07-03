"""NativeDynamic materialization IO and path helpers."""

from __future__ import annotations

import shutil
from pathlib import Path


def reset_native_dynamic_plugins_dir(
    stage_dir: Path,
    diagnostics: list[str],
) -> bool:
    plugins_dir = stage_dir / "plugins"
    if plugins_dir.exists():
        if not remove_native_dynamic_dir(
            "NativeDynamic plugins directory",
            plugins_dir,
            diagnostics,
        ):
            return False
    try:
        plugins_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic plugins directory {plugins_dir} could not be created: {error}"
        )
        return False
    return True


def remove_native_dynamic_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> bool:
    try:
        shutil.rmtree(directory)
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be removed: {error}")
        return False
    return True


def list_native_dynamic_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> list[Path] | None:
    try:
        return list(directory.iterdir())
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be listed: {error}")
        return None


def copy_native_dynamic_file(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def copy_native_dynamic_tree(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
) -> bool:
    try:
        shutil.copytree(source, destination)
    except OSError as error:
        diagnostics.append(f"{label} {source} could not be copied to {destination}: {error}")
        return False
    return True


def resolve_stage_child(
    stage_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"native dynamic package directory {relative_path} must be relative")
        return None
    try:
        resolved_root = stage_root.resolve()
        resolved = (resolved_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(
            f"native dynamic package directory {relative_path} could not be resolved: {error}"
        )
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(
            f"native dynamic package directory {relative_path} escapes the NativeDynamic stage"
        )
        return None
    return resolved
