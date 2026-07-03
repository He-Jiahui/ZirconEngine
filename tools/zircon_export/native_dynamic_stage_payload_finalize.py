"""NativeDynamic stage payload finalization helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_file_manifest,
    native_dynamic_package_loadable_artifacts,
    native_dynamic_package_payload_file_manifest,
)
from .native_dynamic_templates import (
    native_dynamic_package_report_template,
    native_plugin_load_manifest_template,
)


def finalize_native_dynamic_stage_payload(
    package_exports: list[dict[str, Any]],
    stage_dir: Path,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
) -> tuple[list[dict[str, object]], str | None]:
    finalize_native_dynamic_package_reports(
        package_exports,
        stage_dir,
        materialized_packages,
        loadable_artifact_extensions,
        diagnostics,
    )
    if diagnostics:
        return [], None

    loader_manifest = stage_dir / "plugins" / NATIVE_DYNAMIC_LOADER_MANIFEST
    try:
        loader_manifest.parent.mkdir(parents=True, exist_ok=True)
        loader_manifest.write_text(
            native_plugin_load_manifest_template(package_exports),
            encoding="utf-8",
        )
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic loader manifest {loader_manifest} could not be written: {error}"
        )
        return [], None

    file_manifest = native_dynamic_file_manifest(stage_dir, diagnostics)
    if diagnostics:
        return file_manifest, None
    return file_manifest, native_dynamic_content_hash(file_manifest)


def finalize_native_dynamic_package_reports(
    package_exports: list[dict[str, Any]],
    stage_dir: Path,
    materialized_packages: list[dict[str, object]],
    loadable_artifact_extensions: set[str],
    diagnostics: list[str],
) -> None:
    package_exports_by_id = {
        str(package_export["package_id"]): package_export
        for package_export in package_exports
    }
    for materialized_package in materialized_packages:
        package_id = materialized_package.get("package_id")
        destination = materialized_package.get("destination")
        if not isinstance(package_id, str) or not isinstance(destination, str):
            diagnostics.append("NativeDynamic materialized package entry is malformed")
            continue
        package_export = package_exports_by_id.get(package_id)
        if package_export is None:
            diagnostics.append(
                f"NativeDynamic materialized package {package_id} has no package export"
            )
            continue
        package_dir = Path(destination)
        package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        payload_file_manifest = native_dynamic_package_payload_file_manifest(
            package_dir,
            diagnostics,
        )
        if diagnostics:
            continue
        try:
            package_report.write_text(
                native_dynamic_package_report_template(
                    package_export,
                    payload_file_manifest,
                ),
                encoding="utf-8",
            )
        except OSError as error:
            diagnostics.append(
                f"NativeDynamic package {package_id} report {package_report} could not be written: {error}"
            )
            continue
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            stage_dir,
            package_dir,
            loadable_artifact_extensions,
            diagnostics,
        )
        if diagnostics:
            continue
        materialized_package["package_report"] = str(package_report)
        materialized_package["loadable_artifact_count"] = len(loadable_artifacts)
        materialized_package["loadable_artifacts"] = loadable_artifacts
