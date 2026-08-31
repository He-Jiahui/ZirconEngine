"""NativeDynamic package materialization helpers."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    NATIVE_DYNAMIC_RESOURCE_DIRS,
)
from .native_dynamic_payload_file_manifest import native_dynamic_package_loadable_artifacts
from .native_dynamic_materialize_io import (
    copy_native_dynamic_file,
    copy_native_dynamic_tree,
    list_native_dynamic_dir,
    remove_native_dynamic_dir,
    resolve_stage_child,
)


@dataclass(frozen=True)
class PackageManifestRead:
    manifest_id: str | None = None
    error: str | None = None


@dataclass
class NativePackageManifestIndex:
    entries: tuple[tuple[Path, Path, PackageManifestRead], ...] | None = None
    listing_failure_suffix: str | None = None


def materialize_native_dynamic_packages(
    package_exports: list[dict[str, Any]],
    plugin_root: Path,
    stage_dir: Path,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    source_packages: dict[str, Path],
    diagnostics: list[str],
    *,
    require_source_native_artifacts: bool,
) -> list[dict[str, object]]:
    materialized_packages: list[dict[str, object]] = []
    package_root = stage_dir / "plugins"
    copied_directories: set[str] = set()
    manifest_index = NativePackageManifestIndex()
    for package_export in package_exports:
        package_id = str(package_export["package_id"])
        directory = str(package_export["directory"])
        if directory in copied_directories:
            diagnostics.append(
                f"native dynamic package {package_id} resolves to duplicate output directory plugins/{directory}"
            )
            continue
        copied_directories.add(directory)

        diagnostics_before_source_lookup = len(diagnostics)
        source = find_native_package_dir(
            plugin_root,
            package_id,
            diagnostics,
            manifest_index=manifest_index,
        )
        if source is None:
            if len(diagnostics) == diagnostics_before_source_lookup:
                diagnostics.append(
                    f"native dynamic package {package_id} was selected but no plugin.toml was found under {plugin_root}"
                )
            continue
        source_packages[package_id] = source
        destination = resolve_stage_child(package_root, directory, diagnostics)
        if destination is None:
            continue
        if not copy_native_dynamic_package(
            source,
            destination,
            package_id,
            artifact_extensions,
            loadable_artifact_extensions,
            diagnostics,
            require_source_native_artifacts=require_source_native_artifacts,
        ):
            if destination.exists():
                label = f"NativeDynamic package {package_id} partial package"
                remove_native_dynamic_dir(label, destination, diagnostics)
            continue
        package_report = destination / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            destination,
            loadable_artifact_extensions,
            diagnostics,
        )
        materialized_packages.append(
            {
                "package_id": package_id,
                "source": str(source),
                "destination": str(destination),
                "package_report": str(package_report),
                "loadable_artifact_count": len(loadable_artifacts),
                "loadable_artifacts": loadable_artifacts,
            }
        )
    return materialized_packages


def find_native_package_dir(
    plugin_root: Path,
    package_id: str,
    diagnostics: list[str],
    *,
    manifest_index: NativePackageManifestIndex | None = None,
) -> Path | None:
    if not plugin_root.exists() or not plugin_root.is_dir():
        return None

    direct = plugin_root / package_id
    direct_manifest = direct / "plugin.toml"
    if direct_manifest.exists():
        manifest_read = read_package_manifest_id(direct_manifest)
        if manifest_read.error is not None:
            diagnostics.append(
                f"native dynamic package {package_id} direct manifest {manifest_read.error}"
            )
            return None
        manifest_id = manifest_read.manifest_id
        if manifest_id == package_id:
            return direct
        if manifest_id is not None:
            diagnostics.append(
                f"native dynamic package {package_id} direct manifest id {manifest_id} does not match selected package {package_id}"
            )
            return None
        diagnostics.append(
            f"native dynamic package {package_id} direct manifest id must be a non-empty string"
        )
        return None

    if manifest_index is None:
        manifest_index = NativePackageManifestIndex()
    populate_native_package_manifest_index(plugin_root, manifest_index)
    if manifest_index.listing_failure_suffix is not None:
        diagnostics.append(
            f"native dynamic package {package_id} source search directory "
            f"{manifest_index.listing_failure_suffix}"
        )
        return None

    matches: list[Path] = []
    manifest_diagnostics: list[str] = []
    for child, manifest_path, manifest_read in manifest_index.entries or ():
        if manifest_read.error is not None:
            manifest_diagnostics.append(
                f"native dynamic package {package_id} source manifest {manifest_path} {manifest_read.error}"
            )
        elif manifest_read.manifest_id == package_id:
            matches.append(child)
        elif manifest_read.manifest_id is None:
            manifest_diagnostics.append(
                f"native dynamic package {package_id} source manifest {manifest_path} id must be a non-empty string"
            )
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        diagnostics.append(
            f"native dynamic package {package_id} has multiple source package manifests: "
            + ", ".join(str(match) for match in sorted(matches))
        )
    if manifest_diagnostics:
        diagnostics.extend(manifest_diagnostics)
    return None


def populate_native_package_manifest_index(
    plugin_root: Path,
    manifest_index: NativePackageManifestIndex,
) -> None:
    if (
        manifest_index.entries is not None
        or manifest_index.listing_failure_suffix is not None
    ):
        return

    entries: list[tuple[Path, Path, PackageManifestRead]] = []
    stack = [plugin_root]
    while stack:
        current = stack.pop()
        listing_label = (
            "native dynamic package __manifest_index__ source search directory"
        )
        listing_diagnostics: list[str] = []
        children = list_native_dynamic_dir(
            listing_label,
            current,
            listing_diagnostics,
        )
        if children is None:
            prefix = f"{listing_label} "
            manifest_index.listing_failure_suffix = listing_diagnostics[-1].removeprefix(
                prefix
            )
            manifest_index.entries = ()
            return
        for child in children:
            if not child.is_dir():
                continue
            manifest_path = child / "plugin.toml"
            if manifest_path.exists():
                entries.append(
                    (child, manifest_path, read_package_manifest_id(manifest_path))
                )
            stack.append(child)
    manifest_index.entries = tuple(entries)


def read_package_manifest_id(path: Path) -> PackageManifestRead:
    if not path.exists():
        return PackageManifestRead()
    if not path.is_file():
        return PackageManifestRead(error=f"{path} is not a file")
    try:
        with path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        return PackageManifestRead(error=f"could not be parsed: {error}")
    except OSError as error:
        return PackageManifestRead(error=f"could not be read: {error}")
    if "id" not in manifest:
        return PackageManifestRead()
    manifest_id = manifest.get("id")
    if isinstance(manifest_id, str):
        if not manifest_id:
            return PackageManifestRead()
        if manifest_id.strip() != manifest_id:
            return PackageManifestRead(error="id must be a non-empty trimmed string")
        return PackageManifestRead(manifest_id=manifest_id)
    return PackageManifestRead(error="id must be a string")


def copy_native_dynamic_package(
    source: Path,
    destination: Path,
    package_id: str,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
    *,
    require_source_native_artifacts: bool,
) -> bool:
    if destination.exists():
        if not remove_native_dynamic_dir(
            f"NativeDynamic package {package_id} destination",
            destination,
            diagnostics,
        ):
            return False
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic package {package_id} destination {destination} could not be created: {error}"
        )
        return False

    saw_native_dir = False
    copied_native_artifacts = 0
    copied_loadable_artifacts = 0
    children = list_native_dynamic_dir(
        f"NativeDynamic package {package_id} source directory",
        source,
        diagnostics,
    )
    if children is None:
        return False
    for child in children:
        destination_child = destination / child.name
        if child.is_dir():
            if child.name == "native":
                saw_native_dir = True
                copy_result = copy_native_artifacts(
                    child, destination_child, package_id,
                    artifact_extensions, loadable_artifact_extensions, diagnostics
                )
                copied_native_artifacts += copy_result["copied"]
                copied_loadable_artifacts += copy_result["loadable"]
                if copy_result["fatal"]:
                    return False
                if copy_result["copied"] == 0:
                    if require_source_native_artifacts:
                        diagnostics.append(
                            f"native dynamic package {package_id} has no dynamic library artifacts under {child}"
                        )
                elif copy_result["loadable"] == 0:
                    if require_source_native_artifacts:
                        diagnostics.append(
                            f"native dynamic package {package_id} has no loadable native library artifacts under {child}"
                        )
            elif child.name in NATIVE_DYNAMIC_RESOURCE_DIRS:
                if not copy_native_dynamic_tree(
                    child,
                    destination_child,
                    diagnostics,
                    f"NativeDynamic package {package_id} resource directory",
                ):
                    return False
        elif child.name == "plugin.toml":
            if not copy_native_dynamic_file(
                child,
                destination_child,
                diagnostics,
                f"NativeDynamic package {package_id} manifest",
            ):
                return False
    if not saw_native_dir:
        if require_source_native_artifacts:
            diagnostics.append(
                f"native dynamic package {package_id} has no native artifact directory under {source}"
            )
            return False
        return True
    if not require_source_native_artifacts:
        return True
    return copied_native_artifacts > 0 and copied_loadable_artifacts > 0


def copy_native_artifacts(
    source: Path,
    destination: Path,
    package_id: str,
    artifact_extensions: set[str],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
) -> dict[str, int | bool]:
    copied = 0
    copied_loadable = 0
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic package {package_id} artifact directory {destination} could not be created: {error}"
        )
        return {"copied": copied, "loadable": copied_loadable, "fatal": True}
    children = list_native_dynamic_dir(
        f"NativeDynamic package {package_id} artifact directory",
        source,
        diagnostics,
    )
    if children is None:
        return {"copied": copied, "loadable": copied_loadable, "fatal": True}
    for child in children:
        extension = child.suffix.lower()
        if child.is_dir():
            if (
                extension not in artifact_extensions
                or extension not in NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS
            ):
                continue
            if not copy_native_dynamic_tree(
                child,
                destination / child.name,
                diagnostics,
                f"NativeDynamic package {package_id} artifact",
            ):
                return {"copied": copied, "loadable": copied_loadable, "fatal": True}
            copied += 1
            continue
        if not child.is_file():
            continue
        if extension not in artifact_extensions:
            continue
        if not copy_native_dynamic_file(
            child,
            destination / child.name,
            diagnostics,
            f"NativeDynamic package {package_id} artifact",
        ):
            return {"copied": copied, "loadable": copied_loadable, "fatal": True}
        copied += 1
        if extension in loadable_artifact_extensions:
            copied_loadable += 1
    return {"copied": copied, "loadable": copied_loadable, "fatal": False}
