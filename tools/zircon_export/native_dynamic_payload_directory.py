"""NativeDynamic directory-backed payload summary helpers."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_package_loadable_artifacts,
    native_dynamic_plugins_bundle_file_manifest,
    resolve_native_dynamic_payload_path,
)


def native_dynamic_directory_payload_summary(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> dict[str, Any] | None:
    resolved_plugins_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic payload source",
        plugins_dir,
        diagnostics,
    )
    if resolved_plugins_dir is None:
        return None
    plugins_dir = resolved_plugins_dir
    if not plugins_dir.exists() or not plugins_dir.is_dir():
        return None
    manifest_diagnostics: list[str] = []
    file_manifest = native_dynamic_plugins_bundle_file_manifest(
        plugins_dir,
        diagnostics=manifest_diagnostics,
    )
    if manifest_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(manifest_diagnostics)
        return None
    materialized_packages = native_dynamic_directory_materialized_packages(
        plugins_dir,
        diagnostics,
    )
    if materialized_packages is None:
        return None
    return {
        "stage_report": None,
        "source": str(plugins_dir),
        "loader_manifest": str(plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST),
        "content_hash": native_dynamic_content_hash(file_manifest),
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": len(materialized_packages),
        "materialized_packages": materialized_packages,
    }


def native_dynamic_directory_materialized_packages(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]] | None:
    materialized_packages: list[dict[str, object]] = []
    children = native_dynamic_payload_directory_children(plugins_dir, diagnostics)
    if children is None:
        return None
    for package_dir in children:
        if not package_dir.is_dir():
            continue
        package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        diagnostic_count = len(diagnostics) if diagnostics is not None else 0
        package_id = native_dynamic_package_report_id(package_report, diagnostics)
        if diagnostics is not None and len(diagnostics) > diagnostic_count:
            return None
        package_summary: dict[str, object] = {
            "package_id": package_id or package_dir.name,
            "destination": str(package_dir),
            "loadable_artifact_count": 0,
            "loadable_artifacts": [],
        }
        if package_report.exists():
            package_summary["package_report"] = str(package_report)
        loadable_artifacts = native_dynamic_package_loadable_artifacts(
            plugins_dir,
            package_dir,
            NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
            diagnostics,
        )
        if diagnostics is not None and len(diagnostics) > diagnostic_count:
            return None
        loadable_artifacts = [f"plugins/{path}" for path in loadable_artifacts]
        package_summary["loadable_artifact_count"] = len(loadable_artifacts)
        package_summary["loadable_artifacts"] = loadable_artifacts
        materialized_packages.append(package_summary)
    return materialized_packages


def native_dynamic_payload_directory_children(
    directory: Path,
    diagnostics: list[str] | None,
) -> list[Path] | None:
    try:
        return sorted(directory.iterdir(), key=lambda path: path.name)
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic payload directory {directory} could not be listed: {error}"
            )
        return None


def native_dynamic_package_report_id(
    package_report: Path,
    diagnostics: list[str] | None = None,
) -> str | None:
    if not package_report.exists():
        return None
    if not package_report.is_file():
        if diagnostics is not None:
            diagnostics.append(
                f"native dynamic package report {package_report} is not a file"
            )
        return None
    try:
        with package_report.open("rb") as report_file:
            report = tomllib.load(report_file)
    except tomllib.TOMLDecodeError:
        return None
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"native dynamic package report {package_report} could not be read: {error}"
            )
        return None
    package_id = report.get("package_id")
    if isinstance(package_id, str) and package_id:
        return package_id
    return None


def materialized_package_loadable_artifacts_match_manifest(
    materialized_packages: list[dict[str, object]],
    file_manifest: list[dict[str, object]],
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> bool:
    manifest_paths = {str(entry["path"]) for entry in file_manifest}
    loadable_manifest_paths = {
        path
        for path in manifest_paths
        if Path(path).suffix.lower() in NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS
    }
    loadable_artifacts_by_prefix = native_dynamic_loadable_artifact_prefix_index(
        loadable_manifest_paths
    )
    plugins_root: Path | None = None
    plugins_root_resolved = False
    for index, package in enumerate(materialized_packages):
        destination = str(package["destination"])
        destination_path = resolve_native_dynamic_payload_path(
            f"NativeDynamic payload materialized_packages[{index}] destination",
            Path(destination).expanduser(),
            diagnostics,
        )
        if not plugins_root_resolved:
            plugins_root = resolve_native_dynamic_payload_path(
                "NativeDynamic payload plugins_dir",
                plugins_dir,
                diagnostics,
            )
            plugins_root_resolved = True
        if destination_path is None or plugins_root is None:
            return False
        try:
            relative_destination = destination_path.relative_to(plugins_root)
        except ValueError:
            if diagnostics is not None:
                diagnostics.append(
                    "NativeDynamic payload "
                    f"materialized_packages[{index}] destination {destination} "
                    f"is outside plugins_dir {plugins_root}"
                )
            return False
        package_prefix = f"plugins/{relative_destination.as_posix().rstrip('/')}/"
        loadable_artifacts = package["loadable_artifacts"]
        if not isinstance(loadable_artifacts, list):
            return False
        for artifact_path in loadable_artifacts:
            if not isinstance(artifact_path, str):
                return False
            if artifact_path not in manifest_paths:
                return False
            if not artifact_path.startswith(package_prefix):
                return False
        reported_artifacts = {
            artifact_path
            for artifact_path in loadable_artifacts
            if isinstance(artifact_path, str)
        }
        expected_artifacts = loadable_artifacts_by_prefix.get(package_prefix, [])
        for artifact_path in expected_artifacts:
            if artifact_path not in reported_artifacts:
                if diagnostics is not None:
                    diagnostics.append(
                        "NativeDynamic payload "
                        f"materialized_packages[{index}] loadable_artifacts "
                        "do not include current loadable artifact "
                        f"{artifact_path}"
                    )
                return False
    return True


def native_dynamic_loadable_artifact_prefix_index(
    loadable_manifest_paths: set[str],
) -> dict[str, list[str]]:
    artifacts_by_prefix: dict[str, list[str]] = {}
    for path in sorted(loadable_manifest_paths):
        parent, separator, _ = path.rpartition("/")
        while separator and parent:
            prefix = f"{parent.rstrip('/')}/"
            artifacts_by_prefix.setdefault(prefix, []).append(path)
            parent, separator, _ = parent.rpartition("/")
    return artifacts_by_prefix
