"""NativeDynamic package materialization helpers."""

from __future__ import annotations

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
from .native_dynamic_manifest_index import (
    NativePackageManifestIndex,
    PackageManifestRead,
    find_native_package_dir as _find_native_package_dir,
    populate_native_package_manifest_index as _populate_native_package_manifest_index,
    read_package_manifest_id as _read_package_manifest_id,
)


def read_package_manifest_id(path: Path) -> PackageManifestRead:
    return _read_package_manifest_id(path)


def populate_native_package_manifest_index(
    plugin_root: Path,
    manifest_index: NativePackageManifestIndex,
) -> None:
    _populate_native_package_manifest_index(
        plugin_root,
        manifest_index,
        list_directory=list_native_dynamic_dir,
        read_manifest=read_package_manifest_id,
    )


def find_native_package_dir(
    plugin_root: Path,
    package_id: str,
    diagnostics: list[str],
    *,
    manifest_index: NativePackageManifestIndex | None = None,
) -> Path | None:
    return _find_native_package_dir(
        plugin_root,
        package_id,
        diagnostics,
        manifest_index=manifest_index,
        populate_index=populate_native_package_manifest_index,
        read_manifest=read_package_manifest_id,
    )


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
