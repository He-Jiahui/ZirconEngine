"""PlatformBundle native plugins directory materialization helpers."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from .export_template_manifest import resolve_bundle_child
from .platform_bundle_template_files_materialize import template_files_outside_directory


def materialize_platform_bundle_native_plugins(
    *,
    bundle_root: Path,
    native_plugins_dir: Path,
    copied_template_files: list[dict[str, str]],
    diagnostics: list[str],
) -> tuple[bool, Path | None, list[dict[str, str]]]:
    plugins_destination = resolve_bundle_child(bundle_root, "plugins", diagnostics)
    if not plugins_destination:
        return True, None, copied_template_files
    try:
        native_plugins_metadata = native_plugins_dir.stat()
    except FileNotFoundError:
        diagnostics.append(f"native plugins directory {native_plugins_dir} does not exist")
        return True, None, copied_template_files
    except OSError as error:
        diagnostics.append(
            f"native plugins directory {native_plugins_dir} could not be inspected: {error}"
        )
        return True, None, copied_template_files
    if not stat.S_ISDIR(native_plugins_metadata.st_mode):
        diagnostics.append(f"native plugins directory {native_plugins_dir} does not exist")
        return True, None, copied_template_files

    if plugins_destination.exists():
        if not remove_platform_bundle_native_plugins_destination(
            plugins_destination,
            diagnostics,
        ):
            return True, None, copied_template_files

    filtered_template_files = template_files_outside_directory(
        copied_template_files,
        plugins_destination,
        diagnostics,
    )
    if filtered_template_files is None:
        return True, None, copied_template_files
    if not copy_platform_bundle_native_plugins_dir(
        native_plugins_dir,
        plugins_destination,
        diagnostics,
    ):
        return True, None, filtered_template_files
    return False, plugins_destination, filtered_template_files


def remove_platform_bundle_native_plugins_destination(
    plugins_destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        shutil.rmtree(plugins_destination)
    except OSError as error:
        diagnostics.append(
            "native plugins destination "
            f"{plugins_destination} could not be removed: {error}"
        )
        return False
    return True


def copy_platform_bundle_native_plugins_dir(
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"native plugins directory {destination} could not be created: {error}"
        )
        return False
    copied = True
    try:
        for child in source.iterdir():
            target = destination / child.name
            if child.is_dir():
                if not copy_platform_bundle_native_plugins_dir(
                    child,
                    target,
                    diagnostics,
                ):
                    copied = False
            elif not copy_platform_bundle_native_plugins_file(
                child,
                target,
                diagnostics,
            ):
                copied = False
    except OSError as error:
        diagnostics.append(
            f"native plugins directory {source} could not be listed: {error}"
        )
        return False
    return copied


def copy_platform_bundle_native_plugins_file(
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(
            f"native plugins file {source} could not be copied to {destination}: {error}"
        )
        return False
    return True
