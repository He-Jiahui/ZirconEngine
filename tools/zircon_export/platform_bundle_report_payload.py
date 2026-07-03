"""PlatformBundle bundle manifest and stage report payload assembly."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def platform_bundle_stage_directory_failure_report(
    *,
    profile: str,
    bundle_dir: Path,
    diagnostics: list[str],
    template_resolution: dict[str, Any] | None,
) -> dict[str, Any]:
    report = {
        "stage": "PlatformBundle",
        "profile": profile,
        "bundle": str(bundle_dir),
        "fatal": True,
        "diagnostics": diagnostics,
        "template_resolution": template_resolution,
        "template": None,
        "host_executable": None,
        "host_source": None,
        "host_source_origin": None,
        "pack": None,
        "pack_source": None,
        "pack_source_origin": None,
        "delta_pack": None,
        "delta_pack_source": None,
        "delta_pack_source_origin": None,
        "native_plugins": None,
        "native_plugins_payload": None,
        "template_files": [],
        "bundle_manifest": None,
    }
    return report


def platform_bundle_manifest_payload(
    *,
    profile: str,
    template_resolution: dict[str, Any] | None,
    template_report: dict[str, Any] | None,
    copied_host: Path | None,
    host_executable: Path | None,
    host_source_origin: str | None,
    copied_pack: Path | None,
    pack_path: Path | None,
    pack_source_origin: str,
    copied_delta_pack: Path | None,
    delta_pack_path: Path | None,
    delta_pack_source_origin: str,
    copied_native_plugins: Path | None,
    copied_native_plugins_payload: dict[str, Any] | None,
    copied_template_files: list[dict[str, str]],
) -> dict[str, Any]:
    manifest = {
        "profile": profile,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "host_source": str(host_executable) if copied_host else None,
        "host_source_origin": host_source_origin if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "pack_source": str(pack_path) if copied_pack else None,
        "pack_source_origin": pack_source_origin if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "delta_pack_source": str(delta_pack_path) if copied_delta_pack else None,
        "delta_pack_source_origin": (
            delta_pack_source_origin if copied_delta_pack else None
        ),
        "native_plugins": str(copied_native_plugins) if copied_native_plugins else None,
        "native_plugins_payload": copied_native_plugins_payload,
        "template_files": copied_template_files,
    }
    return manifest


def platform_bundle_stage_report_payload(
    *,
    profile: str,
    bundle_dir: Path,
    fatal: bool,
    diagnostics: list[str],
    template_resolution: dict[str, Any] | None,
    template_report: dict[str, Any] | None,
    copied_host: Path | None,
    host_executable: Path | None,
    host_source_origin: str | None,
    copied_pack: Path | None,
    pack_path: Path | None,
    pack_source_origin: str,
    copied_delta_pack: Path | None,
    delta_pack_path: Path | None,
    delta_pack_source_origin: str,
    copied_native_plugins: Path | None,
    copied_native_plugins_payload: dict[str, Any] | None,
    copied_template_files: list[dict[str, str]],
    bundle_manifest: Path | None,
) -> dict[str, Any]:
    report = {
        "stage": "PlatformBundle",
        "profile": profile,
        "bundle": str(bundle_dir),
        "fatal": fatal,
        "diagnostics": diagnostics,
        "template_resolution": template_resolution,
        "template": template_report,
        "host_executable": str(copied_host) if copied_host else None,
        "host_source": str(host_executable) if copied_host else None,
        "host_source_origin": host_source_origin if copied_host else None,
        "pack": str(copied_pack) if copied_pack else None,
        "pack_source": str(pack_path) if copied_pack else None,
        "pack_source_origin": pack_source_origin if copied_pack else None,
        "delta_pack": str(copied_delta_pack) if copied_delta_pack else None,
        "delta_pack_source": str(delta_pack_path) if copied_delta_pack else None,
        "delta_pack_source_origin": (
            delta_pack_source_origin if copied_delta_pack else None
        ),
        "native_plugins": str(copied_native_plugins) if copied_native_plugins else None,
        "native_plugins_payload": copied_native_plugins_payload,
        "template_files": copied_template_files,
        "bundle_manifest": str(bundle_manifest) if bundle_manifest else None,
    }
    return report
