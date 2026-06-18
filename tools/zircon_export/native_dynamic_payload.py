"""NativeDynamic payload manifests and report summary validation."""

from __future__ import annotations

import hashlib
import tomllib
from pathlib import Path
from typing import Any

from .native_signing import native_dynamic_signing_platform_allowed
from .pipeline_report_native_dynamic_payload_schema import (
    native_dynamic_file_manifest_schema_diagnostics,
    native_dynamic_materialized_packages_schema_diagnostics,
    native_dynamic_operation_audit_stage_schema_diagnostics,
)

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    NATIVE_DYNAMIC_STAGE,
    REPORT_FILE_NAME,
)
from .stage_handoff import load_stage_report_object, stage_report_metadata_diagnostic


def native_dynamic_file_manifest(
    stage_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload stage directory",
        stage_dir,
        diagnostics,
    )
    if manifest_root is None:
        return []
    plugins_dir = manifest_root / "plugins"
    return native_dynamic_plugins_file_manifest(
        manifest_root,
        plugins_dir,
        diagnostics=diagnostics,
    )


def native_dynamic_plugins_bundle_file_manifest(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload source",
        plugins_dir,
        diagnostics,
    )
    if manifest_root is None:
        return []
    return native_dynamic_plugins_file_manifest(
        manifest_root,
        plugins_dir,
        root_prefix="plugins",
        diagnostics=diagnostics,
    )


def native_dynamic_plugins_file_manifest(
    manifest_root: Path,
    plugins_dir: Path,
    *,
    root_prefix: str | None = None,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    manifest_root = resolve_native_dynamic_payload_path(
        "NativeDynamic payload manifest root",
        manifest_root,
        diagnostics,
    )
    plugins_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic payload directory",
        plugins_dir,
        diagnostics,
    )
    if manifest_root is None or plugins_dir is None:
        return []
    if not plugins_dir.exists():
        return []

    file_manifest: list[dict[str, object]] = []
    entries = native_dynamic_payload_tree_entries(plugins_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file():
            continue
        relative_path = file_path.relative_to(manifest_root).as_posix()
        if root_prefix is not None:
            relative_path = f"{root_prefix}/{file_path.relative_to(plugins_dir).as_posix()}"
        payload = read_native_dynamic_payload_file(file_path, diagnostics)
        if payload is None:
            continue
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def native_dynamic_package_payload_file_manifest(
    package_dir: Path,
    diagnostics: list[str] | None = None,
) -> list[dict[str, object]]:
    package_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic package payload directory",
        package_dir,
        diagnostics,
    )
    if package_dir is None:
        return []
    file_manifest: list[dict[str, object]] = []
    entries = native_dynamic_payload_tree_entries(package_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file() or file_path.name == NATIVE_DYNAMIC_PACKAGE_REPORT_FILE:
            continue
        relative_path = file_path.relative_to(package_dir).as_posix()
        payload = read_native_dynamic_payload_file(file_path, diagnostics)
        if payload is None:
            continue
        file_manifest.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest(),
            }
        )
    return sorted(file_manifest, key=lambda entry: str(entry["path"]))


def read_native_dynamic_payload_file(
    file_path: Path,
    diagnostics: list[str] | None,
) -> bytes | None:
    try:
        return file_path.read_bytes()
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic payload file {file_path} could not be read: {error}")
        return None


def native_dynamic_package_loadable_artifacts(
    stage_dir: Path,
    package_dir: Path,
    loadable_artifact_extensions: set[str],
    diagnostics: list[str] | None = None,
) -> list[str]:
    stage_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic payload stage directory",
        stage_dir,
        diagnostics,
    )
    package_dir = resolve_native_dynamic_payload_path(
        "NativeDynamic package payload directory",
        package_dir,
        diagnostics,
    )
    if stage_dir is None or package_dir is None:
        return []
    artifacts: list[str] = []
    entries = native_dynamic_payload_tree_entries(package_dir, diagnostics)
    if entries is None:
        return []
    for file_path in entries:
        if not file_path.is_file():
            continue
        if file_path.suffix.lower() not in loadable_artifact_extensions:
            continue
        artifacts.append(file_path.relative_to(stage_dir).as_posix())
    return artifacts


def native_dynamic_payload_tree_entries(
    directory: Path,
    diagnostics: list[str] | None,
) -> list[Path] | None:
    try:
        return sorted(directory.rglob("*"))
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic payload directory {directory} could not be listed: {error}"
            )
        return None


def resolve_native_dynamic_payload_path(
    label: str,
    path: Path,
    diagnostics: list[str] | None,
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def native_dynamic_content_hash(file_manifest: list[dict[str, object]]) -> str:
    hasher = hashlib.sha256()
    for entry in file_manifest:
        hasher.update(str(entry["path"]).encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(str(entry["bytes"]).encode("ascii"))
        hasher.update(b"\0")
        hasher.update(str(entry["sha256"]).lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def native_dynamic_stage_payload_summary(
    out_root: Path,
    profile: str,
    plugins_dir: Path | None,
    diagnostics: list[str] | None = None,
) -> dict[str, Any] | None:
    if plugins_dir is None:
        return None

    report_path = out_root / "stages" / NATIVE_DYNAMIC_STAGE / REPORT_FILE_NAME
    if not report_path.exists():
        return native_dynamic_directory_payload_summary(plugins_dir, diagnostics)
    report, report_diagnostic = load_stage_report_object(report_path, "NativeDynamic")
    if report_diagnostic:
        if diagnostics is not None:
            diagnostics.append(report_diagnostic)
        return None
    metadata_diagnostic = stage_report_metadata_diagnostic(
        report,
        NATIVE_DYNAMIC_STAGE,
        profile,
    )
    if metadata_diagnostic:
        if diagnostics is not None:
            diagnostics.append(metadata_diagnostic)
        return None

    reported_plugins_dir = report.get("plugins_dir")
    if not isinstance(reported_plugins_dir, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report plugins_dir is missing or invalid")
        return None
    reported_plugins_path = resolve_native_dynamic_payload_path(
        "NativeDynamic report plugins_dir",
        Path(reported_plugins_dir).expanduser(),
        diagnostics,
    )
    current_plugins_path = resolve_native_dynamic_payload_path(
        "NativeDynamic current plugins_dir",
        plugins_dir,
        diagnostics,
    )
    if reported_plugins_path is None or current_plugins_path is None:
        return None
    if reported_plugins_path != current_plugins_path:
        return native_dynamic_directory_payload_summary(plugins_dir, diagnostics)

    report_profile = report.get("profile")
    if report.get("fatal"):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report is fatal")
        return None
    if report_profile != profile:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report profile {report_profile} does not match requested profile {profile}"
            )
        return None

    content_hash = report.get("content_hash")
    payload_schema_diagnostics = native_dynamic_file_manifest_schema_diagnostics(
        "NativeDynamic report",
        report,
    )
    payload_schema_diagnostics.extend(
        native_dynamic_materialized_packages_schema_diagnostics(
            "NativeDynamic report",
            report,
        )
    )
    if payload_schema_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(payload_schema_diagnostics)
        return None
    file_manifest = normalized_file_manifest(report.get("file_manifest"))
    materialized_packages = normalized_materialized_packages(
        report.get("materialized_packages")
    )
    if not isinstance(content_hash, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report content_hash is missing or invalid")
        return None
    if file_manifest is None:
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report file_manifest is malformed")
        return None
    if materialized_packages is None:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report materialized_packages are malformed"
            )
        return None

    manifest_diagnostics: list[str] = []
    actual_file_manifest = native_dynamic_plugins_file_manifest(
        plugins_dir.parent,
        plugins_dir,
        diagnostics=manifest_diagnostics,
    )
    if manifest_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(manifest_diagnostics)
        return None
    actual_content_hash = native_dynamic_content_hash(actual_file_manifest)
    if actual_content_hash != content_hash:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report content_hash "
                f"{content_hash} does not match current plugins directory "
                f"{plugins_dir} content_hash {actual_content_hash}"
            )
        return None

    if not materialized_package_loadable_artifacts_match_manifest(
        materialized_packages,
        file_manifest,
        plugins_dir,
        diagnostics,
    ):
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report loadable_artifacts are not present in file_manifest"
            )
        return None

    payload_summary = {
        "stage_report": str(report_path),
        "source": str(plugins_dir),
        "loader_manifest": str(plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST),
        "content_hash": content_hash,
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": len(materialized_packages),
        "materialized_packages": materialized_packages,
    }
    signing_summary = normalized_native_dynamic_stage_operation_audit(
        report,
        "native_signing",
        expected_package_count=len(materialized_packages),
        diagnostics=diagnostics,
    )
    if report.get("native_signing") is not None and signing_summary is None:
        return None
    if signing_summary is not None:
        payload_summary["native_signing"] = signing_summary
    notarization_summary = normalized_native_dynamic_stage_operation_audit(
        report,
        "native_notarization",
        expected_package_count=len(materialized_packages),
        diagnostics=diagnostics,
    )
    if report.get("native_notarization") is not None and notarization_summary is None:
        return None
    if notarization_summary is not None:
        payload_summary["native_notarization"] = notarization_summary
    return payload_summary


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
            diagnostics.append(f"native dynamic package report {package_report} is not a file")
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


def normalized_native_dynamic_operation_audit(
    value: object,
) -> dict[str, object] | None:
    if value is None:
        return None
    if not isinstance(value, dict):
        return None
    enabled = value.get("enabled")
    profile = value.get("profile")
    target_platform = value.get("target_platform")
    allowed_platforms = value.get("allowed_platforms")
    platform_allowed = value.get("platform_allowed")
    fatal = value.get("fatal")
    package_count = value.get("package_count")
    if (
        type(enabled) is not bool
        or (profile is not None and not isinstance(profile, str))
        or (target_platform is not None and not isinstance(target_platform, str))
        or not isinstance(allowed_platforms, list)
        or any(not isinstance(platform, str) for platform in allowed_platforms)
        or type(platform_allowed) is not bool
        or type(fatal) is not bool
        or type(package_count) is not int
    ):
        return None
    return {
        "enabled": enabled,
        "profile": profile,
        "target_platform": target_platform,
        "allowed_platforms": list(allowed_platforms),
        "platform_allowed": platform_allowed,
        "fatal": fatal,
        "package_count": package_count,
    }


def normalized_native_dynamic_stage_operation_audit(
    report: dict[str, Any],
    field: str,
    *,
    expected_package_count: int,
    diagnostics: list[str] | None,
) -> dict[str, object] | None:
    value = report.get(field)
    if value is None:
        return None
    if not isinstance(value, dict):
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report {field} must be an object")
        return None
    schema_diagnostics = native_dynamic_operation_audit_stage_schema_diagnostics(
        f"NativeDynamic report {field}",
        value,
    )
    if schema_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(schema_diagnostics)
        return None
    summary = normalized_native_dynamic_operation_audit(value)
    if summary is None:
        if diagnostics is not None:
            diagnostics.append(f"NativeDynamic report {field} is malformed")
        return None
    if not native_dynamic_operation_audit_is_consistent(
        summary,
        report_is_fatal=bool(report.get("fatal")),
        field=field,
        diagnostics=diagnostics,
    ):
        return None
    enabled = summary["enabled"]
    package_count = summary["package_count"]
    if enabled is True and package_count != expected_package_count:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} package_count {package_count} "
                f"does not match materialized_packages {expected_package_count}"
            )
        return None
    return summary


def native_dynamic_operation_audit_is_consistent(
    summary: dict[str, object],
    *,
    report_is_fatal: bool,
    field: str,
    diagnostics: list[str] | None,
) -> bool:
    target_platform_value = summary["target_platform"]
    target_platform = (
        target_platform_value if isinstance(target_platform_value, str) else None
    )
    allowed_platforms = list(summary["allowed_platforms"])
    if summary["enabled"] is True:
        computed_platform_allowed = native_dynamic_signing_platform_allowed(
            target_platform,
            [str(platform) for platform in allowed_platforms],
        )
        if summary["platform_allowed"] != computed_platform_allowed:
            if diagnostics is not None:
                diagnostics.append(
                    f"NativeDynamic report {field} platform_allowed "
                    "does not match target platform"
                )
            return False
    if summary["fatal"] is True and not report_is_fatal:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} is fatal but report is non-fatal"
            )
        return False
    if summary["enabled"] is True and summary["platform_allowed"] is False:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report {field} disallows target platform"
            )
        return False
    return True


def normalized_file_manifest(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        path = entry.get("path")
        byte_count = entry.get("bytes")
        sha256 = entry.get("sha256")
        if not isinstance(path, str) or type(byte_count) is not int or not isinstance(sha256, str):
            return None
        normalized.append(
            {
                "path": path,
                "bytes": byte_count,
                "sha256": sha256,
            }
        )
    return normalized


def normalized_materialized_packages(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        package_id = entry.get("package_id")
        destination = entry.get("destination")
        loadable_artifact_count = entry.get("loadable_artifact_count")
        loadable_artifacts = entry.get("loadable_artifacts")
        if (
            not isinstance(package_id, str)
            or not isinstance(destination, str)
            or type(loadable_artifact_count) is not int
            or not isinstance(loadable_artifacts, list)
        ):
            return None
        if any(not isinstance(path, str) for path in loadable_artifacts):
            return None
        if loadable_artifact_count != len(loadable_artifacts):
            return None
        package_summary: dict[str, object] = {
            "package_id": package_id,
            "destination": destination,
            "loadable_artifact_count": loadable_artifact_count,
            "loadable_artifacts": list(loadable_artifacts),
        }
        source = entry.get("source")
        if source is not None:
            if not isinstance(source, str):
                return None
            package_summary["source"] = source
        package_report = entry.get("package_report")
        if package_report is not None:
            if not isinstance(package_report, str):
                return None
            package_summary["package_report"] = package_report
        normalized.append(package_summary)
    return normalized


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
    for index, package in enumerate(materialized_packages):
        destination = str(package["destination"])
        destination_path = resolve_native_dynamic_payload_path(
            f"NativeDynamic payload materialized_packages[{index}] destination",
            Path(destination).expanduser(),
            diagnostics,
        )
        plugins_root = resolve_native_dynamic_payload_path(
            "NativeDynamic payload plugins_dir",
            plugins_dir,
            diagnostics,
        )
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
        expected_artifacts = sorted(
            path
            for path in loadable_manifest_paths
            if path.startswith(package_prefix)
        )
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
