"""Engine and plugin asset staging helpers for zircon_build."""

from __future__ import annotations

import filecmp
import os
import shutil
from pathlib import Path

from .zircon_build_filesystem import assert_safe_destination, copy_tree_payload, is_reparse_point, iter_tree_entries, remove_tree_without_reparse
from .zircon_build_asset_policy import SHIPPING_RUNTIME_EXCLUDED_PREFIXES, SHIPPING_RUNTIME_EXCLUDED_TOP_LEVEL
from .zircon_build_zui_assets import validate_staged_engine_asset_suffix


ENGINE_ASSET_ROOTS = (
    Path("zircon_editor") / "assets",
    Path("zircon_runtime") / "assets",
)
ENGINE_RUNTIME_ASSET_ROOTS = (Path("zircon_runtime") / "assets",)
ENGINE_EDITOR_ASSET_ROOTS = (Path("zircon_editor") / "assets",)
UI_COMPILED_ARTIFACT_CACHE_ENV = "ZIRCON_UI_COMPILED_ARTIFACT_CACHE"
UI_COMPILED_ARTIFACT_CACHE_ROOT = Path(".zircon") / "ui" / "compiled_artifacts"
UI_COMPILED_ARTIFACT_STAGE_ROOT = Path("ui") / "compiled_artifacts"
UI_COMPILED_ARTIFACT_SUFFIXES = (".zuiart", ".zuicache")
def stage_engine_assets(config: object, scope: str | None = None) -> None:
    """Stage engine assets according to the product composition."""
    selected_scope = scope or getattr(config, "asset_scope", "all") or "all"
    shipping = bool(getattr(config, "is_shipping", False) or getattr(config, "mode", "") in {"shipping", "shipping-symbols"})
    if selected_scope == "none":
        destination_root = config.engine_root / "assets"
        assert_safe_destination(destination_root, "Engine asset destination")
        if config.dry_run:
            print(f"DRY-RUN reset {destination_root}")
        elif destination_root.exists():
            remove_tree_without_reparse(destination_root, "Engine asset destination")
        return
    if shipping and selected_scope == "editor":
        destination_root = getattr(config, "editor_asset_root", None) or config.out_root / "EditorAssets"
    else:
        destination_root = config.engine_root / "assets"
    assert_safe_destination(destination_root, "Engine asset destination")
    if config.dry_run:
        print(f"DRY-RUN reset {destination_root}")
    else:
        if destination_root.exists():
            remove_tree_without_reparse(destination_root, "Engine asset destination")
        destination_root.mkdir(parents=True, exist_ok=True)

    if selected_scope == "runtime":
        asset_roots = ENGINE_RUNTIME_ASSET_ROOTS
    elif selected_scope == "editor":
        asset_roots = ENGINE_EDITOR_ASSET_ROOTS
    elif selected_scope == "all":
        asset_roots = ENGINE_ASSET_ROOTS
    else:
        raise SystemExit(f"Unknown engine asset staging scope: {selected_scope}")
    runtime_shipping = shipping and selected_scope == "runtime"
    for relative_root in asset_roots:
        source_root = config.repo_root / relative_root
        if not source_root.exists() or not source_root.is_dir():
            raise SystemExit(f"Engine asset root is missing: {source_root}")
        print(f"Staging assets {source_root} -> {destination_root}")
        skipped = copy_tree_contents(
            source_root,
            destination_root,
            config,
            excluded_top_level=SHIPPING_RUNTIME_EXCLUDED_TOP_LEVEL if runtime_shipping else frozenset(),
            excluded_prefixes=SHIPPING_RUNTIME_EXCLUDED_PREFIXES if runtime_shipping else (),
        )
        if skipped:
            print(f"Skipped {skipped} staged asset(s)")
    if not (shipping and selected_scope in {"runtime", "editor"}):
        stage_ui_compiled_artifacts(config, destination_root)
def copy_tree_contents(
    source_root: Path,
    destination_root: Path,
    config: object,
    *,
    excluded_top_level: set[str] | frozenset[str] = frozenset(),
    excluded_prefixes: tuple[str, ...] = (),
) -> int:
    skipped = 0
    for source, is_directory in iter_tree_entries(source_root):
        relative = source.relative_to(source_root)
        relative_key = relative.as_posix().lower()
        excluded = relative.parts and relative.parts[0].lower() in excluded_top_level
        excluded = excluded or any(
            relative_key == prefix.replace("\\", "/").rstrip("/").lower()
            or relative_key.startswith(prefix.replace("\\", "/").rstrip("/").lower() + "/")
            for prefix in excluded_prefixes
        )
        if excluded:
            if config.dry_run and not is_directory:
                print(f"DRY-RUN skip shipping asset {source}")
            skipped += 1 if not is_directory else 0
            continue
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
    for source, is_directory in iter_tree_entries(source_root):
        if is_directory:
            continue
        if source.suffix.lower() not in UI_COMPILED_ARTIFACT_SUFFIXES:
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
    assert_safe_destination(destination, "Engine asset destination")
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
        assert_safe_destination(destination, "Plugin resource destination")
        if config.dry_run:
            print(f"DRY-RUN copytree {source} -> {destination}")
            continue
        if destination.exists():
            if is_reparse_point(destination):
                raise SystemExit(
                    f"Plugin resource staging refuses destination reparse point: {destination}"
                )
            remove_tree_without_reparse(destination, "Plugin resource destination")
        copy_tree_payload(source, destination, config, copy_asset_file)
        print(f"Copied {source} -> {destination}")
def _copy_file(source: Path, destination: Path, config: object) -> None:
    if config.dry_run:
        print(f"DRY-RUN copy {source} -> {destination}")
        return
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    print(f"Copied {source} -> {destination}")
