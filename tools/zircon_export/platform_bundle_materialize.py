"""PlatformBundle bundle materialization helpers."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path
from typing import Any

from .export_template_manifest import resolve_bundle_child
from .platform_bundle_native_plugins_materialize import (
    materialize_platform_bundle_native_plugins,
)
from .platform_bundle_template_files_materialize import (
    materialize_platform_bundle_template_files,
)


def materialize_platform_bundle(
    *,
    bundle_dir: Path,
    profile: str,
    host_executable: Path | None,
    pack_path: Path,
    delta_pack_path: Path | None,
    native_plugins_dir: Path | None,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> dict[str, Any]:
    fatal = False
    copied_host: Path | None = None
    copied_pack: Path | None = None
    copied_delta_pack: Path | None = None
    copied_native_plugins: Path | None = None
    copied_template_files: list[dict[str, str]] = []
    bundle_root = template_bundle_root(bundle_dir, template_report, diagnostics)
    try:
        bundle_root.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"PlatformBundle bundle root {bundle_root} could not be created: {error}"
        )
        return {
            "fatal": True,
            "profile": profile,
            "bundle_root": bundle_root,
            "host_executable": None,
            "pack": None,
            "delta_pack": None,
            "native_plugins": None,
            "template_files": [],
        }

    host_destination: Path | None = None
    if host_executable:
        host_destination = template_bundle_output_path(
            bundle_root,
            template_report,
            "host_path",
            host_executable.name,
            diagnostics,
        )
    else:
        diagnostics.append("host executable not supplied; bundle contains assets only")
        fatal = True

    pack_destination = template_bundle_output_path(
        bundle_root,
        template_report,
        "pack_path",
        pack_path.name,
        diagnostics,
    )
    delta_pack_destination = None
    if delta_pack_path:
        delta_pack_destination = template_bundle_output_path(
            bundle_root,
            template_report,
            "delta_pack_path",
            delta_pack_path.name,
            diagnostics,
        )
    if host_executable and not host_destination:
        fatal = True
    if not pack_destination:
        fatal = True
    if delta_pack_path and not delta_pack_destination:
        fatal = True

    if host_executable:
        host_diagnostic = platform_bundle_file_input_diagnostic(
            "host executable",
            host_executable,
        )
        if host_diagnostic:
            diagnostics.append(host_diagnostic)
            fatal = True

    pack_diagnostic = platform_bundle_file_input_diagnostic("pack file", pack_path)
    if pack_diagnostic:
        diagnostics.append(pack_diagnostic)
        fatal = True

    if delta_pack_path:
        delta_pack_diagnostic = platform_bundle_file_input_diagnostic(
            "delta pack file",
            delta_pack_path,
        )
        if delta_pack_diagnostic:
            diagnostics.append(delta_pack_diagnostic)
            fatal = True

    if template_report and not fatal:
        template_files_fatal, copied_template_files = (
            materialize_platform_bundle_template_files(
                bundle_root=bundle_root,
                template_report=template_report,
                host_executable=host_executable,
                host_destination=host_destination,
                diagnostics=diagnostics,
            )
        )
        if template_files_fatal:
            fatal = True

    if host_executable and host_destination and not fatal:
        if copy_platform_bundle_file(
            "host executable",
            host_executable,
            host_destination,
            diagnostics,
        ):
            copied_host = host_destination
        else:
            fatal = True

    if not fatal and pack_destination:
        if copy_platform_bundle_file(
            "pack file",
            pack_path,
            pack_destination,
            diagnostics,
        ):
            copied_pack = pack_destination
        else:
            fatal = True

    if delta_pack_path and not fatal and delta_pack_destination:
        if copy_platform_bundle_file(
            "delta pack file",
            delta_pack_path,
            delta_pack_destination,
            diagnostics,
        ):
            copied_delta_pack = delta_pack_destination
        else:
            fatal = True

    if native_plugins_dir:
        native_plugins_fatal, copied_native_plugins, copied_template_files = (
            materialize_platform_bundle_native_plugins(
                bundle_root=bundle_root,
                native_plugins_dir=native_plugins_dir,
                copied_template_files=copied_template_files,
                diagnostics=diagnostics,
            )
        )
        if native_plugins_fatal:
            fatal = True

    return {
        "fatal": fatal,
        "profile": profile,
        "bundle_root": bundle_root,
        "host_executable": copied_host,
        "pack": copied_pack,
        "delta_pack": copied_delta_pack,
        "native_plugins": copied_native_plugins,
        "template_files": copied_template_files,
    }


def platform_bundle_file_input_diagnostic(label: str, path: Path) -> str | None:
    try:
        metadata = path.stat()
    except FileNotFoundError:
        return f"{label} {path} does not exist"
    except OSError as error:
        return f"{label} {path} could not be inspected: {error}"
    if not stat.S_ISREG(metadata.st_mode):
        return f"{label} {path} is not a file"
    if metadata.st_size <= 0:
        return f"{label} {path} is empty"
    return None


def copy_platform_bundle_file(
    label: str,
    source: Path,
    destination: Path,
    diagnostics: list[str],
) -> bool:
    try:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
    except OSError as error:
        diagnostics.append(
            f"{label} {source} could not be copied to {destination}: {error}"
        )
        return False
    return True


def remove_platform_bundle_dir(
    label: str,
    directory: Path,
    diagnostics: list[str],
) -> bool:
    try:
        shutil.rmtree(directory)
    except OSError as error:
        diagnostics.append(f"{label} {directory} could not be removed: {error}")
        return False
    return True


def template_bundle_root(
    bundle_dir: Path,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> Path:
    if not template_report:
        return bundle_dir
    bundle = template_report.get("bundle")
    if not isinstance(bundle, dict):
        return bundle_dir
    root = bundle.get("root")
    if not isinstance(root, str) or not root or root == ".":
        return bundle_dir
    return resolve_bundle_child(bundle_dir, root, diagnostics) or bundle_dir


def template_bundle_output_path(
    bundle_root: Path,
    template_report: dict[str, Any] | None,
    field_name: str,
    fallback_name: str,
    diagnostics: list[str],
) -> Path | None:
    if template_report:
        bundle = template_report.get("bundle")
        if isinstance(bundle, dict):
            value = bundle.get(field_name)
            if isinstance(value, str) and value:
                return resolve_bundle_child(bundle_root, value, diagnostics)
    return bundle_root / fallback_name


def template_bundle_manifest_path(
    bundle_dir: Path,
    template_report: dict[str, Any] | None,
    diagnostics: list[str],
) -> Path | None:
    if not template_report:
        return None
    bundle = template_report.get("bundle")
    if not isinstance(bundle, dict):
        return None
    manifest_path = bundle.get("manifest_path")
    if not isinstance(manifest_path, str) or not manifest_path:
        return None
    return resolve_bundle_child(
        template_bundle_root(bundle_dir, template_report, diagnostics),
        manifest_path,
        diagnostics,
    )
