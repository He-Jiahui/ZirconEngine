"""NativeDynamic stage report payload assembly."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def native_dynamic_stage_report_payload(
    *,
    profile: str,
    fatal: bool,
    diagnostics: list[str],
    validate_report: Path | None,
    stage_dir: Path,
    plugin_root: Path | None,
    target_platform: str | None,
    artifact_extensions: set[str],
    loader_manifest: Path | None,
    native_dynamic_packages: list[str],
    package_exports: list[dict[str, Any]] | None,
    native_build_plan: dict[str, Any] | None,
    native_build_execution: dict[str, Any],
    native_signing: dict[str, Any],
    native_notarization: dict[str, Any],
    materialized_packages: list[dict[str, Any]],
    file_manifest: list[dict[str, Any]],
    content_hash: str | None,
    payload_cleaned: bool,
    cleanup_reason: str | None,
) -> dict[str, Any]:
    return {
        "stage": "NativeDynamic",
        "profile": profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report) if validate_report else None,
        "stage_output": str(stage_dir),
        "native_plugin_root": str(plugin_root) if plugin_root else None,
        "target_platform": target_platform,
        "artifact_extensions": sorted(artifact_extensions),
        "plugins_dir": str(stage_dir / "plugins") if not fatal else None,
        "loader_manifest": str(loader_manifest) if not fatal and loader_manifest else None,
        "native_dynamic_packages": native_dynamic_packages,
        "package_exports": package_exports or [],
        "package_count": len(package_exports or []),
        "native_build_plan": native_build_plan,
        "native_build_execution": native_build_execution,
        "native_signing": native_signing,
        "native_notarization": native_notarization,
        "materialized_packages": materialized_packages,
        "file_manifest": file_manifest,
        "content_hash": content_hash,
        "payload_cleaned": payload_cleaned,
        "cleanup_reason": cleanup_reason,
    }
