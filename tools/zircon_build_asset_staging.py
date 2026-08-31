"""Engine and plugin asset staging helpers for zircon_build."""

from __future__ import annotations

import filecmp
import os
import shutil
from pathlib import Path
from typing import Iterator

try:
    from .zircon_build_zui_assets import validate_staged_engine_asset_suffix
except ImportError:  # pragma: no cover - exercised when run as a script.
    from zircon_build_zui_assets import validate_staged_engine_asset_suffix


ENGINE_ASSET_ROOTS = (
    Path("zircon_editor") / "assets",
    Path("zircon_runtime") / "assets",
)
UI_COMPILED_ARTIFACT_CACHE_ENV = "ZIRCON_UI_COMPILED_ARTIFACT_CACHE"
UI_COMPILED_ARTIFACT_CACHE_ROOT = Path(".zircon") / "ui" / "compiled_artifacts"
UI_COMPILED_ARTIFACT_STAGE_ROOT = Path("ui") / "compiled_artifacts"
UI_COMPILED_ARTIFACT_SUFFIXES = (".zuiart", ".zuicache")


def stage_engine_assets(config: object) -> None:
    destination_root = config.engine_root / "assets"
    if config.dry_run:
        print(f"DRY-RUN reset {destination_root}")
    else:
        if destination_root.exists():
            shutil.rmtree(destination_root)
        destination_root.mkdir(parents=True, exist_ok=True)

    for relative_root in ENGINE_ASSET_ROOTS:
        source_root = config.repo_root / relative_root
        if not source_root.exists() or not source_root.is_dir():
            raise SystemExit(f"Engine asset root is missing: {source_root}")
        print(f"Staging assets {source_root} -> {destination_root}")
        skipped = copy_tree_contents(source_root, destination_root, config)
        if skipped:
            print(f"Skipped {skipped} staged asset(s)")
    stage_ui_compiled_artifacts(config, destination_root)


def _iter_tree_entries(source_root: Path) -> Iterator[tuple[Path, bool]]:
    with os.scandir(source_root) as iterator:
        entries = sorted(iterator, key=lambda entry: Path(entry.path))
    for entry in entries:
        source = Path(entry.path)
        if entry.is_dir():
            yield source, True
            if not entry.is_symlink():
                yield from _iter_tree_entries(source)
            continue
        if entry.is_file():
            yield source, False


def copy_tree_contents(source_root: Path, destination_root: Path, config: object) -> int:
    skipped = 0
    for source, is_directory in _iter_tree_entries(source_root):
        relative = source.relative_to(source_root)
        destination = destination_root / relative
        if is_directory:
            if config.dry_run:
                print(f"DRY-RUN mkdir {destination}")
            else:
                destination.mkdir(parents=True, exist_ok=True)
            continue
        validate_staged_engine_asset_suffix(relative, source)
        copy_asset_file(source, destination, config)
    return skipped


def stage_ui_compiled_artifacts(config: object, destination_root: Path) -> None:
    source_root = ui_compiled_artifact_cache_root(config)
    destination = destination_root / UI_COMPILED_ARTIFACT_STAGE_ROOT
    if not source_root.exists():
        if config.dry_run:
            print(f"DRY-RUN no UI compiled artifact cache found at {source_root}")
        return
    if not source_root.is_dir():
        raise SystemExit(f"UI compiled artifact cache root is not a directory: {source_root}")
    print(f"Staging UI compiled artifacts {source_root} -> {destination}")
    copied = 0
    skipped = 0
    for source, is_directory in _iter_tree_entries(source_root):
        if is_directory:
            continue
        if source.suffix not in UI_COMPILED_ARTIFACT_SUFFIXES:
            skipped += 1
            if config.dry_run:
                print(f"DRY-RUN skip non-compiled UI cache payload {source}")
            continue
        relative = source.relative_to(source_root)
        copy_asset_file(source, destination / relative, config)
        copied += 1
    if copied:
        print(f"Staged {copied} UI compiled artifact cache file(s)")
    if skipped:
        print(f"Skipped {skipped} non-compiled UI cache file(s)")


def ui_compiled_artifact_cache_root(config: object) -> Path:
    override = os.environ.get(UI_COMPILED_ARTIFACT_CACHE_ENV)
    if override:
        return Path(override).expanduser()
    return config.repo_root / UI_COMPILED_ARTIFACT_CACHE_ROOT


def copy_asset_file(source: Path, destination: Path, config: object) -> None:
    if destination.exists():
        if destination.is_file() and filecmp.cmp(source, destination, shallow=False):
            return
        raise SystemExit(
            "Engine asset staging collision: "
            f"{source} cannot overwrite existing {destination} with different content."
        )
    _copy_file(source, destination, config)


def copy_resource_dirs(source_root: Path, package_out: Path, config: object) -> None:
    for name in ("assets", "asset", "resources", "resource"):
        source = source_root / name
        if not source.exists() or not source.is_dir():
            continue
        destination = package_out / name
        if config.dry_run:
            print(f"DRY-RUN copytree {source} -> {destination}")
            continue
        if destination.exists():
            shutil.rmtree(destination)
        shutil.copytree(source, destination)
        print(f"Copied {source} -> {destination}")


def _copy_file(source: Path, destination: Path, config: object) -> None:
    if config.dry_run:
        print(f"DRY-RUN copy {source} -> {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    print(f"Copied {source} -> {destination}")
