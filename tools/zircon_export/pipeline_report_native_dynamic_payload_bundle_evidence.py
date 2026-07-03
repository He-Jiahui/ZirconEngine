"""PlatformBundle NativeDynamic payload current bundle evidence diagnostics."""

from __future__ import annotations

from pathlib import Path

from .native_dynamic_payload_directory import (
    materialized_package_loadable_artifacts_match_manifest,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_plugins_bundle_file_manifest,
)
from .pipeline_report_native_dynamic_payload_package_path import (
    _resolve_user_path_or_diagnostic,
)


def platform_bundle_native_plugins_bundle_path_diagnostics(
    payload: dict[str, object],
    plugins_dir: Path,
) -> list[str]:
    diagnostics: list[str] = []
    payload_bundle_path = payload.get("bundle_path")
    if payload_bundle_path is None or payload_bundle_path == "":
        diagnostics.append(
            "PlatformBundle report native_plugins_payload bundle_path must be a non-empty string"
        )
    elif isinstance(payload_bundle_path, str):
        payload_bundle_dir = _resolve_user_path_or_diagnostic(
            payload_bundle_path,
            diagnostics,
            "PlatformBundle report native_plugins_payload bundle_path",
        )
        if payload_bundle_dir is not None and payload_bundle_dir != plugins_dir:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload bundle_path "
                f"{payload_bundle_dir} does not match native_plugins {plugins_dir}"
            )
    return diagnostics


def platform_bundle_native_plugins_current_bundle_evidence_diagnostics(
    payload: dict[str, object],
    plugins_dir: Path,
    payload_file_manifest: list[dict[str, object]],
    payload_packages: list[dict[str, object]],
) -> list[str]:
    diagnostics: list[str] = []
    actual_file_manifest = native_dynamic_plugins_bundle_file_manifest(
        plugins_dir,
        diagnostics=diagnostics,
    )
    if diagnostics:
        return diagnostics
    actual_content_hash = native_dynamic_content_hash(actual_file_manifest)
    if payload.get("content_hash") != actual_content_hash:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload content_hash "
            f"{payload.get('content_hash')} does not match current bundle plugins "
            f"directory {plugins_dir} content_hash {actual_content_hash}"
        )
    if payload_file_manifest != actual_file_manifest:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_manifest does not match current bundle plugins directory"
        )
    if payload.get("file_count") != len(actual_file_manifest):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_count does not match current bundle plugins directory"
        )
    if payload.get("package_count") != len(payload_packages):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload package_count does not match materialized_packages"
        )
    if not materialized_package_loadable_artifacts_match_manifest(
        payload_packages,
        actual_file_manifest,
        plugins_dir,
        diagnostics,
    ):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loadable_artifacts are not present in current bundle plugins directory"
        )
    return diagnostics
