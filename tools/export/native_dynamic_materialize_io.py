"""NativeDynamic materialization filesystem and path adapters."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Callable

from .native_dynamic_cas import (
    _materialize_native_dynamic_source,
    _materialize_native_dynamic_tree,
    _remove_native_dynamic_readonly,
    _validate_native_dynamic_source_file,
    _validate_native_dynamic_source_tree,
    native_dynamic_cas_max_bytes, native_dynamic_cas_scope,
    prune_native_dynamic_cas,
    resolve_native_dynamic_cas_root,
)
_CopyFunction = Callable[[Path, Path], object]


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
        shutil.rmtree(directory, onerror=_remove_native_dynamic_readonly)
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
    *,
    cas_root: str | Path | None = None,
    copy_function: _CopyFunction | None = None,
) -> bool:
    try:
        _validate_native_dynamic_source_file(source)
        destination.parent.mkdir(parents=True, exist_ok=True)
        copy_file = copy_function or shutil.copy2
        resolved_cas_root = resolve_native_dynamic_cas_root(cas_root)
        if resolved_cas_root is None:
            copy_file(source, destination)
        else:
            _materialize_native_dynamic_source(
                source,
                destination,
                resolved_cas_root,
                copy_function=copy_file,
            )
    except OSError as error:
        diagnostics.append(
            f"{label} {source} could not be copied to {destination}: {error}"
        )
        return False
    return True


def copy_native_dynamic_tree(
    source: Path,
    destination: Path,
    diagnostics: list[str],
    label: str,
    *,
    cas_root: str | Path | None = None,
) -> bool:
    try:
        _validate_native_dynamic_source_tree(source)
        resolved_cas_root = resolve_native_dynamic_cas_root(cas_root)
        if resolved_cas_root is None:
            shutil.copytree(source, destination)
        else:
            _materialize_native_dynamic_tree(source, destination, resolved_cas_root)
    except OSError as error:
        diagnostics.append(
            f"{label} {source} could not be copied to {destination}: {error}"
        )
        return False
    return True


def resolve_stage_child(
    stage_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(
            f"native dynamic package directory {relative_path} must be relative"
        )
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
