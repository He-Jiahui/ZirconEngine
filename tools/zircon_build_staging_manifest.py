"""Source-bound staged-product manifest support for zircon_build."""

from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path
from typing import Iterator

try:
    from .zircon_build_asset_staging import (
        ENGINE_ASSET_ROOTS,
        UI_COMPILED_ARTIFACT_STAGE_ROOT,
        ui_compiled_artifact_cache_root,
    )
except ImportError:  # pragma: no cover - exercised when zircon_build.py is run directly.
    from zircon_build_asset_staging import (
        ENGINE_ASSET_ROOTS,
        UI_COMPILED_ARTIFACT_STAGE_ROOT,
        ui_compiled_artifact_cache_root,
    )


STAGING_MANIFEST_FILE_NAME = "staging_manifest.json"
STAGING_MANIFEST_SCHEMA_VERSION = 1


def write_staging_manifest(config: object) -> Path:
    """Write a hash inventory for every staged file with verified provenance."""

    manifest_path = config.engine_root / STAGING_MANIFEST_FILE_NAME
    if config.dry_run:
        print(f"DRY-RUN write {manifest_path}")
        return manifest_path

    artifacts = [
        _manifest_entry(config, staged_path)
        for staged_path in sorted(config.engine_root.rglob("*"))
        if staged_path.is_file() and staged_path != manifest_path
    ]
    payload = {
        "schema_version": STAGING_MANIFEST_SCHEMA_VERSION,
        "source_repository": str(config.repo_root.resolve()),
        "build": {
            "mode": config.mode,
            "targets": list(config.targets),
            "runtime_features": list(config.runtime_features),
        },
        "artifacts": artifacts,
    }
    temporary_path = manifest_path.with_suffix(".json.tmp")
    temporary_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary_path, manifest_path)
    print(f"Wrote {manifest_path}")
    return manifest_path


def _manifest_entry(config: object, staged_path: Path) -> dict[str, object]:
    target_path = staged_path.relative_to(config.engine_root).as_posix()
    staged_hash = file_sha256(staged_path)
    source = _find_source(config, staged_path, staged_hash)
    if source is None:
        raise SystemExit(
            "Staged file has no source provenance: "
            f"{target_path}. Refuse to emit an incomplete staging manifest."
        )
    source_kind, source_path = source
    return {
        "logical_artifact": _logical_artifact(target_path),
        "source": {
            "kind": source_kind,
            "path": _source_path(config, source_path),
        },
        "target_path": target_path,
        "sha256": staged_hash,
    }


def _find_source(
    config: object, staged_path: Path, staged_hash: str
) -> tuple[str, Path] | None:
    for source_kind, source_path in _source_candidates(config, staged_path):
        if source_kind == "generated" and source_path.is_file():
            return source_kind, source_path
        if source_path.is_file() and file_sha256(source_path) == staged_hash:
            return source_kind, source_path
    return None


def _source_candidates(config: object, staged_path: Path) -> Iterator[tuple[str, Path]]:
    relative = staged_path.relative_to(config.engine_root)
    profile_dir = _profile_dir(config.mode)

    if relative.parent == Path("."):
        for target_root in (
            config.targets_root / "editor",
            config.targets_root / "runtime" / "bin",
            config.targets_root / "runtime" / "lib",
        ):
            yield "build_artifact", target_root / profile_dir / relative.name

    if relative.parts and relative.parts[0] == "assets":
        asset_relative = Path(*relative.parts[1:])
        for asset_root in ENGINE_ASSET_ROOTS:
            yield "source_asset", config.repo_root / asset_root / asset_relative
        compiled_relative = _compiled_ui_relative(asset_relative)
        if compiled_relative is not None:
            yield (
                "ui_compiled_artifact",
                ui_compiled_artifact_cache_root(config) / compiled_relative,
            )

    if len(relative.parts) >= 2 and relative.parts[0] == "plugins":
        package_name = relative.parts[1]
        package_relative = Path(*relative.parts[2:])
        for package in getattr(config, "plugins", ()):
            if _sanitize_path_component(package.plugin_id) != package_name:
                continue
            yield "plugin_source", package.package_root / package_relative
            if package_relative.parts and package_relative.parts[0] == "native":
                yield (
                    "build_artifact",
                    config.targets_root
                    / "plugins"
                    / package_name
                    / profile_dir
                    / package_relative.name,
                )

    if relative.as_posix() == "plugins.toml" or (
        relative.parts and relative.parts[0] == "cache"
    ):
        yield "generated", config.repo_root / "tools" / "zircon_build.py"

    if relative.name == f"{_runtime_library_name()}.manifest.json":
        yield "generated", config.repo_root / "tools" / "zircon_build_runtime_manifest.py"


def _compiled_ui_relative(asset_relative: Path) -> Path | None:
    stage_parts = UI_COMPILED_ARTIFACT_STAGE_ROOT.parts
    if asset_relative.parts[: len(stage_parts)] != stage_parts:
        return None
    return Path(*asset_relative.parts[len(stage_parts) :])


def _profile_dir(mode: str) -> str:
    if mode == "release":
        return "release"
    if mode == "profiling":
        return "profiling"
    return "debug"


def _logical_artifact(target_path: str) -> str:
    if target_path in {"zircon_editor", "zircon_editor.exe"}:
        return "editor.executable"
    if target_path in {"zircon_runtime", "zircon_runtime.exe"}:
        return "runtime.executable"
    if target_path in {
        "zircon_runtime.dll",
        "libzircon_runtime.dylib",
        "libzircon_runtime.so",
    }:
        return "runtime.library"
    if target_path == f"{_runtime_library_name()}.manifest.json":
        return "runtime.library.manifest"
    if target_path.startswith("assets/"):
        return f"engine_asset:{target_path.removeprefix('assets/')}"
    if target_path.startswith("plugins/"):
        return f"plugin_payload:{target_path.removeprefix('plugins/')}"
    if target_path == "plugins.toml":
        return "plugin_load_manifest"
    return f"staged_file:{target_path}"


def _runtime_library_name() -> str:
    if os.name == "nt":
        return "zircon_runtime.dll"
    if sys.platform == "darwin":
        return "libzircon_runtime.dylib"
    return "libzircon_runtime.so"


def _source_path(config: object, source_path: Path) -> str:
    for root in (config.out_root, config.repo_root):
        try:
            return source_path.resolve().relative_to(root.resolve()).as_posix()
        except ValueError:
            continue
    return str(source_path.resolve())


def _sanitize_path_component(value: str) -> str:
    sanitized = "".join(
        ch if ch.isascii() and (ch.isalnum() or ch in "-_") else "_" for ch in value
    )
    return sanitized or "_"


def file_sha256(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()
