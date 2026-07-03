"""PlatformBundle NativeDynamic payload handoff diagnostics."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_payload import (
    normalized_file_manifest,
    normalized_materialized_packages,
)
from .pipeline_report_native_dynamic_payload_bundle_evidence import (
    platform_bundle_native_plugins_bundle_path_diagnostics,
    platform_bundle_native_plugins_current_bundle_evidence_diagnostics,
)
from .pipeline_report_native_dynamic_payload_loader_manifest import (
    platform_bundle_native_plugins_loader_manifest_diagnostics,
    platform_bundle_native_plugins_loader_manifest_package_diagnostics,
)
from .pipeline_report_native_dynamic_payload_package_path import (
    _resolve_user_path_or_diagnostic,
    platform_bundle_native_plugins_package_path_diagnostics,
)
from .pipeline_report_native_dynamic_payload_file_manifest_schema import (
    platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_materialized_packages_schema import (
    platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_schema import (
    platform_bundle_native_plugins_payload_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_stage_report import (
    platform_bundle_native_plugins_operation_audit_diagnostics,
    platform_bundle_native_plugins_stage_package_diagnostics,
    platform_bundle_native_plugins_stage_payload_diagnostics,
)
from .pipeline_report_native_dynamic_payload_platform_bundle_stage import (
    platform_bundle_native_plugins_stage_report_handoff,
)


def platform_bundle_native_plugins_payload_diagnostics(
    report: dict[str, Any],
    native_dynamic_report_path: Path | None,
    *,
    native_dynamic_stage_report_failed: bool = False,
) -> list[str]:
    native_plugins = report.get("native_plugins")
    payload = report.get("native_plugins_payload")
    if native_plugins is None:
        if payload is None:
            return []
        return [
            "PlatformBundle report native_plugins_payload is present but native_plugins is missing"
        ]
    diagnostics: list[str] = []
    if not isinstance(native_plugins, str) or not native_plugins.strip():
        return ["PlatformBundle report native_plugins must be a non-empty string"]
    if not isinstance(payload, dict):
        return [
            "PlatformBundle report native_plugins_payload is required when native_plugins is present"
        ]
    plugins_dir = _resolve_user_path_or_diagnostic(
        native_plugins,
        diagnostics,
        "PlatformBundle report native_plugins",
    )
    if plugins_dir is None:
        return diagnostics
    if not plugins_dir.exists():
        return [f"PlatformBundle report native_plugins {plugins_dir} does not exist"]
    if not plugins_dir.is_dir():
        return [f"PlatformBundle report native_plugins {plugins_dir} is not a directory"]
    payload_schema_diagnostics = platform_bundle_native_plugins_payload_schema_diagnostics(
        payload
    )
    diagnostics.extend(payload_schema_diagnostics)
    if payload_schema_diagnostics:
        return diagnostics

    stage_report_handoff = platform_bundle_native_plugins_stage_report_handoff(
        payload,
        plugins_dir,
        native_dynamic_report_path,
        native_dynamic_stage_report_failed=native_dynamic_stage_report_failed,
    )
    diagnostics.extend(stage_report_handoff.diagnostics)
    payload_stage_report_matches = stage_report_handoff.payload_stage_report_matches
    suppress_unbacked_stage_audits = (
        stage_report_handoff.suppress_unbacked_stage_audits
    )
    effective_native_dynamic_report_path = (
        stage_report_handoff.effective_native_dynamic_report_path
    )
    diagnostics.extend(
        platform_bundle_native_plugins_bundle_path_diagnostics(
            payload,
            plugins_dir,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_loader_manifest_diagnostics(
            payload,
            plugins_dir,
        )
    )
    payload_content_hash = payload.get("content_hash")
    if payload_content_hash is None or payload_content_hash == "":
        diagnostics.append(
            "PlatformBundle report native_plugins_payload content_hash must be a non-empty string"
        )
    payload_file_manifest_value = payload.get("file_manifest")
    payload_file_manifest_schema_diagnostics = (
        platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics(
            payload
        )
    )
    payload_file_manifest = (
        normalized_file_manifest(payload_file_manifest_value)
        if not payload_file_manifest_schema_diagnostics
        else None
    )
    if not payload_file_manifest_schema_diagnostics and payload_file_manifest is None:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_manifest is malformed"
        )
    payload_file_count = payload.get("file_count")
    if payload_file_count is None:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_count must be an integer"
        )
    payload_materialized_packages = payload.get("materialized_packages")
    payload_materialized_packages_schema_diagnostics = (
        platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics(
            payload
        )
    )
    payload_packages = (
        normalized_materialized_packages(payload_materialized_packages)
        if not payload_materialized_packages_schema_diagnostics
        else None
    )
    if (
        not payload_materialized_packages_schema_diagnostics
        and payload_packages is None
    ):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload materialized_packages are malformed"
        )
    if payload_packages is not None:
        diagnostics.extend(
            platform_bundle_native_plugins_stage_package_diagnostics(
                payload_packages,
                effective_native_dynamic_report_path,
                profile=report.get("profile"),
                stage_backed_payload=payload_stage_report_matches,
            )
        )
        diagnostics.extend(
            platform_bundle_native_plugins_loader_manifest_package_diagnostics(
                payload,
                payload_packages,
                stage_backed_payload=payload_stage_report_matches,
            )
        )
    if not suppress_unbacked_stage_audits:
        diagnostics.extend(
            platform_bundle_native_plugins_operation_audit_diagnostics(
                payload,
                effective_native_dynamic_report_path,
                profile=report.get("profile"),
                payload_packages=payload_packages,
                stage_backed_payload=payload_stage_report_matches,
            )
        )
    payload_package_count = payload.get("package_count")
    if payload_package_count is None:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload package_count must be an integer"
        )
    if diagnostics:
        return diagnostics

    diagnostics.extend(
        platform_bundle_native_plugins_current_bundle_evidence_diagnostics(
            payload,
            plugins_dir,
            payload_file_manifest,
            payload_packages,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_package_path_diagnostics(
            payload_packages,
            plugins_dir,
            stage_backed_payload=payload_stage_report_matches,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_stage_payload_diagnostics(
            payload,
            effective_native_dynamic_report_path,
            profile=report.get("profile"),
            stage_backed_payload=payload_stage_report_matches,
        )
    )
    return diagnostics
