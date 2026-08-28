"""NativeDynamic payload file manifests, path resolution, and content hashes."""

from __future__ import annotations

import hashlib
from pathlib import Path

from .native_dynamic_contract import NATIVE_DYNAMIC_PACKAGE_REPORT_FILE


def normalized_file_manifest(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        path = entry.get("path")
        byte_count = entry.get("bytes")
        sha256 = entry.get("sha256")
        if (
            not isinstance(path, str)
            or type(byte_count) is not int
            or not isinstance(sha256, str)
        ):
            return None
        normalized.append(
            {
                "path": path,
                "bytes": byte_count,
                "sha256": sha256,
            }
        )
    return normalized


def native_dynamic_file_manifest(
    stage_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload stage directory",
        stage_dir,
        diagnostics,
    )
    if manifest_root is None:
        return []
    plugins_dir = manifest_root / "plugins"
    return native_dynamic_plugins_file_manifest(
        manifest_root,
        plugins_dir,
        diagnostics=diagnostics,
    )


def native_dynamic_plugins_bundle_file_manifest(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload source",
        plugins_dir,
        diagnostics,
    )
    if manifest_root is None:
        return []
    return native_dynamic_plugins_file_manifest(
        manifest_root,
        plugins_dir,
        root_prefix="plugins",
        diagnostics=diagnostics,
    )


def native_dynamic_plugins_file_manifest(
    manifest_root: Path,
    plugins_dir: Path,
    *,
    root_prefix: str | None = None,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload manifest root",
        manifest_root,
        diagnostics,
    )
    plugins_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic payload directory",
        plugins_dir,
        diagnostics,
    )
    if manifest_root is None or plugins_dir is None:
        return []
    if not plugins_dir.exists():
        return []

    file_manifest: list[dict[str, object]] = []
    entries = native_dynamic_payload_tree_entries(plugins_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(manifest_root).as_posix()
        if root_prefix is not None:
            relative_path = f"{root_prefix}/{file_path.relative_to(plugins_dir).as_posix()}"
        payload = read_native_dynamic_payload_file(file_path, diagnostics)
        if payload is None:
            continue
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def native_dynamic_package_payload_file_manifest(
    package_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    package_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic package payload directory",
        package_dir,
        diagnostics,
    )
    if package_dir is None:
        return []
    file_manifest: list[dict[str, object]] = []
    entries = native_dynamic_payload_tree_entries(package_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file() or file_path.name == NATIVE_DYNAMIC_PACKAGE_REPORT_FILE:
            continue
        relative_path = file_path.relative_to(package_dir).as_posix()
        payload = read_native_dynamic_payload_file(file_path, diagnostics)
        if payload is None:
            continue
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def read_native_dynamic_payload_file(
    file_path: Path,
    diagnostics: list[str] | None,
) -> bytes | None:
    try:
        return file_path.read_bytes()
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic payload file {file_path} could not be read: {error}")
        return None


def native_dynamic_package_loadable_artifacts(
    stage_dir: Path,
    package_dir: Path,
    loadable_artifact_extensions: set[str],
    diagnostics: list[str] | None = None,
) -> list[str]:
    stage_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic payload stage directory",
        stage_dir,
        diagnostics,
    )
    package_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic package payload directory",
        package_dir,
        diagnostics,
    )
    if stage_dir is None or package_dir is None:
        return []
    artifacts: list[str] = []
    entries = native_dynamic_payload_tree_entries(package_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in loadable_artifact_extensions:
            continue
        artifacts.append(file_path.relative_to(stage_dir).as_posix())
    return artifacts


def native_dynamic_payload_tree_entries(
    directory: Path,
    diagnostics: list[str] | None,
) -> list[Path] | None:
    try:
        return sorted(directory.rglob("*"))
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic payload directory {directory} could not be listed: {error}"
            )
        return None


def resolve_native_dynamic_payload_path(
    label: str,
    path: Path,
    diagnostics: list[str] | None,
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def native_dynamic_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()
