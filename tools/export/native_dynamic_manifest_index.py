"""NativeDynamic package manifest discovery and index helpers."""

from __future__ import annotations

import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

from .native_dynamic_materialize_io import list_native_dynamic_dir


ManifestDirectoryLister = Callable[
    [str, Path, list[str]],
    list[Path] | None,
]
ManifestReader = Callable[[Path], "PackageManifestRead"]
ManifestIndexPopulator = Callable[[Path, "NativePackageManifestIndex"], None]


@dataclass(frozen=True)
class PackageManifestRead:
    manifest_id: str | None = None
    error: str | None = None


@dataclass
class NativePackageManifestIndex:
    entries: tuple[tuple[Path, Path, PackageManifestRead], ...] | None = None
    listing_failure_suffix: str | None = None


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


def populate_native_package_manifest_index(
    plugin_root: Path,
    manifest_index: NativePackageManifestIndex,
    *,
    list_directory: ManifestDirectoryLister | None = None,
    read_manifest: ManifestReader | None = None,
) -> None:
    if (
        manifest_index.entries is not None
        or manifest_index.listing_failure_suffix is not None
    ):
        return

    list_directory = list_directory or list_native_dynamic_dir
    read_manifest = read_manifest or read_package_manifest_id
    entries: list[tuple[Path, Path, PackageManifestRead]] = []
    stack = [plugin_root]
    while stack:
        current = stack.pop()
        listing_label = (
            "native dynamic package __manifest_index__ source search directory"
        )
        listing_diagnostics: list[str] = []
        children = list_directory(
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
                    (child, manifest_path, read_manifest(manifest_path))
                )
            stack.append(child)
    manifest_index.entries = tuple(entries)


def find_native_package_dir(
    plugin_root: Path,
    package_id: str,
    diagnostics: list[str],
    *,
    manifest_index: NativePackageManifestIndex | None = None,
    populate_index: ManifestIndexPopulator | None = None,
    read_manifest: ManifestReader | None = None,
) -> Path | None:
    if not plugin_root.exists() or not plugin_root.is_dir():
        return None

    read_manifest = read_manifest or read_package_manifest_id
    direct = plugin_root / package_id
    direct_manifest = direct / "plugin.toml"
    if direct_manifest.exists():
        manifest_read = read_manifest(direct_manifest)
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
    populate_index = populate_index or populate_native_package_manifest_index
    populate_index(plugin_root, manifest_index)
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
