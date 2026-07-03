"""PlatformBundle template file materialization helpers."""

from __future__ import annotations

import shutil
from pathlib import Path
from typing import Any

from .export_template_manifest import resolve_bundle_child


def materialize_platform_bundle_template_files(
    *,
    bundle_root: Path,
    template_report: dict[str, Any] | None,
    host_executable: Path | None,
    host_destination: Path | None,
    diagnostics: list[str],
) -> tuple[bool, list[dict[str, str]]]:
    if not template_report:
        return False, []

    fatal = False
    copied_template_files: list[dict[str, str]] = []
    for entry in template_report.get("files", []):
        if not isinstance(entry, dict):
            continue
        source = Path(template_report["template_dir"]) / entry["path"]
        destination = resolve_bundle_child(
            bundle_root,
            entry.get("bundle_path", entry["path"]),
            diagnostics,
        )
        if not destination:
            fatal = True
            continue
        if host_destination and host_executable:
            resolved_source = resolve_platform_bundle_template_file_copy_path(
                "template file",
                source,
                diagnostics,
            )
            resolved_host = resolve_platform_bundle_template_file_copy_path(
                "host executable",
                host_executable,
                diagnostics,
            )
            if resolved_source is None or resolved_host is None:
                fatal = True
                continue
            if resolved_source == resolved_host:
                continue
        if not source.exists():
            diagnostics.append(
                f"template file {source} does not exist during bundle copy"
            )
            fatal = True
            continue
        if not source.is_file():
            diagnostics.append(
                f"template file {source} is not a file during bundle copy"
            )
            fatal = True
            continue
        if not copy_platform_bundle_template_file(source, destination, diagnostics):
            fatal = True
            continue
        copied_template_files.append(
            {
                "source": str(source),
                "destination": str(destination),
            }
        )
    return fatal, copied_template_files


def resolve_platform_bundle_template_file_copy_path(
    label: str,
    path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        diagnostics.append(
            f"{label} {path} could not be resolved during bundle copy: {error}"
        )
        return None


def copy_platform_bundle_template_file(
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(
            f"template file {source} could not be copied to {destination}: {error}"
        )
        return False
    return True


def template_files_outside_directory(
    template_files: list[dict[str, str]],
    removed_directory: Path,
    diagnostics: list[str],
) -> list[dict[str, str]] | None:
    try:
        resolved_removed_directory = removed_directory.resolve()
    except OSError as error:
        diagnostics.append(
            "PlatformBundle template_files removed directory "
            f"{removed_directory} could not be resolved: {error}"
        )
        return None
    retained: list[dict[str, str]] = []
    for entry in template_files:
        destination = entry.get("destination")
        if not destination:
            retained.append(entry)
            continue
        try:
            resolved_destination = Path(destination).expanduser().resolve()
        except OSError as error:
            diagnostics.append(
                "PlatformBundle template_files destination "
                f"{destination} could not be resolved: {error}"
            )
            return None
        try:
            resolved_destination.relative_to(resolved_removed_directory)
        except ValueError:
            retained.append(entry)
    return retained
