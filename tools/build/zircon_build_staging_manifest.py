"""Source-bound staged-product manifest support for zircon_build."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import sys
from pathlib import Path
from typing import Iterator, Mapping

from .zircon_build_asset_staging import (
    ENGINE_ASSET_ROOTS,
    UI_COMPILED_ARTIFACT_STAGE_ROOT,
    ui_compiled_artifact_cache_root,
)
from .zircon_build_size_report import classify_product_path


STAGING_MANIFEST_FILE_NAME = "staging_manifest.json"
STAGING_MANIFEST_SCHEMA_VERSION = 1
_FILE_ATTRIBUTE_REPARSE_POINT = 0x00000400


def write_staging_manifest(
    config: object,
    *,
    product_report_reference: Mapping[str, object] | None = None,
) -> Path:
    """Write a hash inventory for every staged file with verified provenance."""

    manifest_path = config.engine_root / STAGING_MANIFEST_FILE_NAME
    if config.dry_run:
        print(f"DRY-RUN write {manifest_path}")
        return manifest_path

    artifacts = [
        _manifest_entry(config, staged_path)
        for staged_path in _iter_staged_files(config.engine_root)
        if staged_path != manifest_path
    ]
    product_profile = getattr(config, "product_profile", None)
    payload = {
        "schema_version": STAGING_MANIFEST_SCHEMA_VERSION,
        # A shipping manifest is part of the published tree; retaining the
        # developer checkout path would leak an absolute host path.
        "source_repository": (
            "<redacted>" if product_profile is not None else str(config.repo_root.resolve())
        ),
        "build": {
            "mode": config.mode,
            "targets": list(config.targets),
            "runtime_features": list(config.runtime_features),
        },
        "artifacts": artifacts,
    }
    if product_profile is not None:
        product_build_set_id = getattr(config, "build_set_id", None)
        if not isinstance(product_build_set_id, str) or not product_build_set_id:
            product_build_set_id = product_profile.build_set_id
        cargo_profile = getattr(
            config, "cargo_profile_name", product_profile.cargo_profile
        )
        if callable(cargo_profile):
            cargo_profile = cargo_profile()
        product_payload: dict[str, object] = {
            "profile": product_profile.name,
            "build_set_id": product_build_set_id,
            "target_triple": product_profile.target_triple,
            "cargo_profile": cargo_profile,
            "app_binary": product_profile.app_binary,
            "cargo_app_binary": product_profile.cargo_app_binary,
            "runtime_library": product_profile.runtime_library,
            "features": {
                "runtime": list(product_profile.runtime_features),
                "app": list(product_profile.app_features),
            },
            "budget": {
                "warning_bytes": product_profile.warning_bytes,
                "hard_limit_bytes": product_profile.hard_limit_bytes,
            },
            "size_report": "../reports/"
            + product_profile.name
            + "/product_size_report.json",
            "symbols": "../symbols/" + product_profile.name,
            "required_exports": ["zircon_runtime_get_api_v8"],
            "ffi_no_unwind_guard": True,
        }
        if product_report_reference is not None:
            product_payload["size_report_reference"] = dict(product_report_reference)
        payload["product"] = product_payload
    temporary_path = manifest_path.with_suffix(".json.tmp")
    temporary_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    os.replace(temporary_path, manifest_path)
    print(f"Wrote {manifest_path}")
    return manifest_path


def _manifest_entry(config: object, staged_path: Path) -> dict[str, object]:
    if _is_reparse_point(staged_path):
        raise SystemExit(
            "Staged product contains a symlink/reparse point; refuse to emit a portable manifest: "
            f"{staged_path}"
        )
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
        "category": _entry_category(config, target_path),
        "bytes": staged_path.stat().st_size,
        "sha256": staged_hash,
    }


def _entry_category(config: object, target_path: str) -> str:
    profile = getattr(config, "product_profile", None)
    return classify_product_path(
        target_path,
        product_kind=profile.product_kind if profile is not None else "runtime",
        forbidden_path_tokens=(
            profile.forbidden_path_tokens if profile is not None else ()
        ),
    )


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
    target_triple = getattr(config, "effective_target_triple", None)
    if callable(target_triple):
        target_triple = target_triple()

    def profile_candidates(root: Path) -> Iterator[Path]:
        if target_triple:
            yield root / str(target_triple) / profile_dir
        yield root / profile_dir

    if relative.parent == Path("."):
        artifact_names = [relative.name]
        product_profile = getattr(config, "product_profile", None)
        if product_profile is not None:
            cargo_name = getattr(product_profile, "cargo_app_binary", relative.stem)
            if os.name == "nt" and not cargo_name.lower().endswith(".exe"):
                cargo_name += ".exe"
            if cargo_name.casefold() != relative.name.casefold():
                artifact_names.append(cargo_name)
        for target_root in (
            config.targets_root / "editor",
            config.targets_root / "runtime" / "bin",
            config.targets_root / "runtime" / "lib",
        ):
            for profile_root in profile_candidates(target_root):
                for artifact_name in artifact_names:
                    yield "build_artifact", profile_root / artifact_name

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
                plugin_root = config.targets_root / "plugins" / package_name
                for profile_root in profile_candidates(plugin_root):
                    yield "build_artifact", profile_root / package_relative.name

    if relative.as_posix() in {"plugins.toml", "plugins/native_plugins.toml"} or (
        relative.parts and relative.parts[0] == "cache"
    ):
        yield "generated", config.repo_root / "tools" / "zircon_build.py"

    if relative.name == f"{_runtime_library_name()}.manifest.json":
        yield "generated", config.repo_root / "tools" / "build" / "zircon_build_runtime_manifest.py"


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
    if mode in {"shipping", "shipping-symbols"}:
        return mode
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
    if target_path in {"plugins.toml", "plugins/native_plugins.toml"}:
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
    return "<external>" if getattr(config, "product_profile", None) is not None else str(source_path.resolve())


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


def _is_reparse_point(path: Path) -> bool:
    if path.is_symlink():
        return True
    try:
        attributes = os.lstat(path).st_file_attributes
    except (OSError, AttributeError):
        return False
    return bool(attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", _FILE_ATTRIBUTE_REPARSE_POINT))


def _iter_staged_files(root: Path) -> tuple[Path, ...]:
    """Walk the published tree without traversing links or Windows junctions."""

    if not root.is_dir() or _is_reparse_point(root):
        raise SystemExit(
            "Staged product root must be a regular directory without a reparse point: "
            f"{root}"
        )
    files: list[Path] = []

    def visit(directory: Path) -> None:
        try:
            entries = sorted(os.scandir(directory), key=lambda entry: entry.name.lower())
        except OSError as error:
            raise SystemExit(f"Could not inspect staged product directory {directory}: {error}") from error
        for entry in entries:
            candidate = Path(entry.path)
            if _is_reparse_point(candidate):
                raise SystemExit(
                    "Staged product contains a symlink/reparse point: "
                    f"{candidate}"
                )
            try:
                if entry.is_dir(follow_symlinks=False):
                    visit(candidate)
                elif entry.is_file(follow_symlinks=False):
                    files.append(candidate)
            except OSError as error:
                raise SystemExit(f"Could not inspect staged product path {candidate}: {error}") from error

    visit(root)
    return tuple(sorted(files, key=lambda path: path.relative_to(root).as_posix()))
