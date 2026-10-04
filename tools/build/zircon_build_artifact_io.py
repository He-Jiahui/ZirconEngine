"""Safe copying of Cargo artifacts and external symbol sidecars."""

from __future__ import annotations

import shutil
from pathlib import Path

from .zircon_build_filesystem import (
    assert_safe_destination,
    is_reparse_point,
    iter_tree_entries,
    remove_tree_without_reparse,
)


def copy_file(source: Path, destination: Path, config: object) -> None:
    if is_reparse_point(source):
        raise SystemExit(f"Refusing to copy a reparse-point artifact: {source}")
    assert_safe_destination(destination, "Artifact destination")
    if config.dry_run:
        print(f"DRY-RUN copy {source} -> {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    print(f"Copied {source} -> {destination}")


def copy_sidecars(
    source: Path,
    destination_dir: Path,
    config: object,
    *,
    published_name: str | None = None,
) -> None:
    """Copy symbols outside the product tree, preserving published names."""

    assert_safe_destination(destination_dir, "Symbol destination")
    published_stem = Path(published_name).stem if published_name else source.stem
    sidecars = (
        source.with_suffix(".pdb"),
        source.with_suffix(".dbg"),
        Path(str(source) + ".dSYM"),
    )
    for sidecar in sidecars:
        # Broken symlinks do not satisfy ``exists`` but are still unsafe
        # publication inputs and must reach the reparse-point guard.
        if not sidecar.exists() and not sidecar.is_symlink():
            continue
        if is_reparse_point(sidecar):
            raise SystemExit(f"Refusing to copy a reparse-point symbol: {sidecar}")
        if published_name and sidecar.suffix.lower() in {".pdb", ".dbg"}:
            destination_name = published_stem + sidecar.suffix
        elif published_name and sidecar.name.lower().endswith(".dsym"):
            destination_name = published_name + ".dSYM"
        else:
            destination_name = sidecar.name
        destination = destination_dir / destination_name
        assert_safe_destination(destination, "Symbol destination")
        if sidecar.is_dir():
            tuple(iter_tree_entries(sidecar))
            if config.dry_run:
                print(f"DRY-RUN copytree {sidecar} -> {destination}")
            else:
                if destination.exists():
                    remove_tree_without_reparse(destination, "Symbol destination")
                shutil.copytree(sidecar, destination)
                print(f"Copied {sidecar} -> {destination}")
        else:
            copy_file(sidecar, destination, config)
