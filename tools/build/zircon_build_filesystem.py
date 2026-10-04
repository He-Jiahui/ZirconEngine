"""Small secure filesystem primitives shared by product-build tooling."""

from __future__ import annotations

import os
import shutil
import stat
from pathlib import Path
from typing import Callable, Iterator


_FILE_ATTRIBUTE_REPARSE_POINT = 0x00000400


def is_reparse_point(path: Path) -> bool:
    """Return whether *path* is a symlink, junction, or other reparse point."""

    if path.is_symlink():
        return True
    try:
        attributes = os.lstat(path).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(
        attributes
        & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", _FILE_ATTRIBUTE_REPARSE_POINT)
    )


def assert_safe_destination(path: Path, label: str = "destination") -> None:
    """Reject an output path or existing ancestor that redirects through a link."""

    current = path if path.is_absolute() else Path.cwd() / path
    while True:
        if is_reparse_point(current):
            raise SystemExit(f"{label} contains a symlink/reparse point: {current}")
        if current.parent == current:
            return
        current = current.parent


def _entry_is_reparse_point(entry: os.DirEntry[str]) -> bool:
    if entry.is_symlink():
        return True
    try:
        attributes = entry.stat(follow_symlinks=False).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(
        attributes
        & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", _FILE_ATTRIBUTE_REPARSE_POINT)
    )


def iter_tree_entries(source_root: Path) -> Iterator[tuple[Path, bool]]:
    """Yield a deterministic tree frontier without following links."""

    if is_reparse_point(source_root):
        raise SystemExit(f"Asset staging rejects reparse point: {source_root}")
    with os.scandir(source_root) as iterator:
        entries = sorted(iterator, key=lambda entry: entry.name.lower())
    for entry in entries:
        source = Path(entry.path)
        if _entry_is_reparse_point(entry):
            raise SystemExit(f"Asset staging rejects reparse point: {source}")
        if entry.is_dir(follow_symlinks=False):
            yield source, True
            yield from iter_tree_entries(source)
        elif entry.is_file(follow_symlinks=False):
            yield source, False


def copy_tree_payload(
    source_root: Path,
    destination_root: Path,
    config: object,
    copy_file: Callable[[Path, Path, object], None],
) -> None:
    """Copy a tree through a caller-owned file copier, never following links."""

    if is_reparse_point(source_root):
        raise SystemExit(f"Plugin resource staging rejects reparse point: {source_root}")
    assert_safe_destination(destination_root, "Plugin resource destination")
    if config.dry_run:
        print(f"DRY-RUN mkdir {destination_root}")
    else:
        destination_root.mkdir(parents=True, exist_ok=True)
    for source, is_directory in iter_tree_entries(source_root):
        destination = destination_root / source.relative_to(source_root)
        if is_directory:
            if config.dry_run:
                print(f"DRY-RUN mkdir {destination}")
            else:
                destination.mkdir(parents=True, exist_ok=True)
        else:
            copy_file(source, destination, config)


def remove_tree_without_reparse(path: Path, label: str = "destination") -> None:
    """Remove an owned tree only after checking every descendant for links."""

    assert_safe_destination(path, label)
    # ``Path.exists`` is false for a broken symlink.  Treat links as present
    # so a stale/broken reparse point cannot silently survive a clean.
    if not path.exists() and not path.is_symlink() and not is_reparse_point(path):
        return
    if is_reparse_point(path):
        raise SystemExit(f"{label} is a symlink/reparse point: {path}")
    if not path.is_dir():
        path.unlink()
        return

    def preflight(directory: Path) -> None:
        try:
            with os.scandir(directory) as iterator:
                entries = list(iterator)
        except OSError as error:
            raise SystemExit(f"Could not inspect {label} {directory}: {error}") from error
        for entry in entries:
            candidate = Path(entry.path)
            if _entry_is_reparse_point(entry):
                raise SystemExit(f"{label} contains a symlink/reparse point: {candidate}")
            if entry.is_dir(follow_symlinks=False):
                preflight(candidate)

    preflight(path)
    shutil.rmtree(path)
