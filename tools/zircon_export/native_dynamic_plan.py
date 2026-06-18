"""Validate-report planning helpers for the NativeDynamic export stage."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    NATIVE_DYNAMIC_PLATFORM_ARTIFACT_EXTENSIONS,
    NATIVE_DYNAMIC_STAGE,
    native_dynamic_package_directory,
)
from .path_resolve import resolve_stage_optional_path
from .stage_handoff import (
    export_strategies_from_validate_report,
    export_strategy_diagnostics,
    load_stage_report_with_diagnostics,
    stage_report_metadata_diagnostic,
)


def resolve_native_dynamic_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(
        value,
        label,
        diagnostics,
        prefix="NativeDynamic",
    )


def load_validate_report(
    validate_report: Path,
    profile: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not validate_report.exists():
        diagnostics.append(f"validate report {validate_report} does not exist")
        return None
    report = load_stage_report_with_diagnostics(
        validate_report,
        "validate",
        diagnostics,
    )
    if report is None:
        return None
    metadata_diagnostic = stage_report_metadata_diagnostic(report, "validate", profile)
    if metadata_diagnostic:
        diagnostics.append(metadata_diagnostic)
        return None
    if report.get("fatal"):
        diagnostics.append("validate report is fatal; NativeDynamic will not export packages")
        return None
    if report.get("profile") != profile:
        diagnostics.append(
            f"validate report profile {report.get('profile')} does not match requested profile {profile}"
        )
        return None
    strategy_diagnostics = export_strategy_diagnostics(report)
    if strategy_diagnostics:
        diagnostics.extend(strategy_diagnostics)
        return None
    if validate_report_requires_strategy(report, NATIVE_DYNAMIC_STAGE):
        diagnostics.append(
            "NativeDynamic stage requires the native_dynamic strategy"
        )
        return None
    return report


def validate_report_requires_strategy(
    report: dict[str, Any],
    strategy: str,
) -> bool:
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return False
    if "strategies" not in profile_summary:
        return False
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    return strategy not in export_strategies_from_validate_report(report)


def native_dynamic_package_ids(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> list[str]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return []
    packages = plan_summary.get("native_dynamic_packages", [])
    if packages is None:
        return []
    if not isinstance(packages, list) or any(not isinstance(value, str) for value in packages):
        diagnostics.append("validate report native_dynamic_packages must be a string array")
        return []
    package_id_indexes: dict[str, int] = {}
    for index, package_id in enumerate(packages):
        previous_index = package_id_indexes.get(package_id)
        if previous_index is not None:
            diagnostics.append(
                f"native_dynamic_packages entry {package_id} duplicates entry {previous_index}"
            )
        else:
            package_id_indexes[package_id] = index
    return list(packages)


def native_dynamic_package_exports(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> list[dict[str, Any]] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    package_exports = plan_summary.get("native_dynamic_package_exports")
    if package_exports is None:
        diagnostics.append("validate report does not contain native_dynamic_package_exports")
        return None
    if not isinstance(package_exports, list):
        diagnostics.append("validate report native_dynamic_package_exports must be an array")
        return None

    normalized_exports: list[dict[str, Any]] = []
    package_id_indexes: dict[str, int] = {}
    for index, package_export in enumerate(package_exports):
        if not isinstance(package_export, dict):
            diagnostics.append(f"native_dynamic_package_exports entry {index} must be an object")
            return None
        validate_package_export_shape(index, package_export, diagnostics)
        package_id = package_export.get("package_id")
        if isinstance(package_id, str) and package_id:
            previous_index = package_id_indexes.get(package_id)
            if previous_index is not None:
                diagnostics.append(
                    f"native_dynamic_package_exports entry {index} package_id {package_id} duplicates entry {previous_index}"
                )
            else:
                package_id_indexes[package_id] = index
        normalized_exports.append(dict(package_export))
    if diagnostics:
        return None
    return normalized_exports


def validate_package_selection_matches_exports(
    package_ids: list[str],
    package_exports: list[dict[str, Any]],
    diagnostics: list[str],
) -> None:
    selected_ids = set(package_ids)
    exported_ids = {
        str(package_export["package_id"])
        for package_export in package_exports
    }
    for package_id in sorted(exported_ids - selected_ids):
        diagnostics.append(
            f"native_dynamic package_export {package_id} is not present in native_dynamic_packages"
        )
    for package_id in sorted(selected_ids - exported_ids):
        diagnostics.append(
            f"native_dynamic_packages entry {package_id} has no package_export"
        )


def native_dynamic_target_platform(validate_payload: dict[str, Any] | None) -> str | None:
    if validate_payload is None:
        return None
    profile_summary = validate_payload.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    target_platform = profile_summary.get("target_platform") or profile_summary.get("platform")
    if isinstance(target_platform, str) and target_platform:
        return target_platform
    return None


def native_dynamic_artifact_extensions(target_platform: str | None) -> set[str]:
    if not target_platform:
        return set(NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS)
    platform_key = target_platform.split("-", maxsplit=1)[0].lower()
    return set(
        NATIVE_DYNAMIC_PLATFORM_ARTIFACT_EXTENSIONS.get(
            platform_key,
            NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS,
        )
    )


def native_dynamic_loadable_artifact_extensions(artifact_extensions: set[str]) -> set[str]:
    return set(artifact_extensions) & NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS


def validate_package_export_shape(
    index: int,
    package_export: dict[str, Any],
    diagnostics: list[str],
) -> None:
    for field in ("package_id", "directory", "path", "manifest"):
        value = package_export.get(field)
        if not isinstance(value, str) or not value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field {field} must be a non-empty string"
            )
    package_id = package_export.get("package_id")
    directory = package_export.get("directory")
    path = package_export.get("path")
    manifest = package_export.get("manifest")
    package_report = package_export.get("package_report")
    if (
        isinstance(package_id, str)
        and package_id
        and isinstance(directory, str)
        and directory
    ):
        expected_directory = native_dynamic_package_directory(package_id)
        if directory != expected_directory:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} directory must be {expected_directory} for package_id {package_id}"
            )
    if isinstance(directory, str) and directory:
        expected_path = f"plugins/{directory}"
        expected_manifest = f"{expected_path}/plugin.toml"
        expected_package_report = f"{expected_path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}"
        if isinstance(path, str) and path and path != expected_path:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} path must be {expected_path} for directory {directory}"
            )
        if isinstance(manifest, str) and manifest and manifest != expected_manifest:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} manifest must be {expected_manifest} for directory {directory}"
            )
        if package_report is None:
            package_export["package_report"] = expected_package_report
        elif not isinstance(package_report, str) or not package_report:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field package_report must be a non-empty string"
            )
        elif package_report != expected_package_report:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} package_report must be {expected_package_report} for directory {directory}"
            )
    abi = package_export.get("abi")
    if not isinstance(abi, dict):
        diagnostics.append(f"native_dynamic_package_exports entry {index} field abi must be an object")
        return
    abi_version = abi.get("abi_version")
    if type(abi_version) is not int:
        diagnostics.append(
            f"native_dynamic_package_exports entry {index} abi.abi_version must be an integer"
        )
    elif abi_version != 3:
        diagnostics.append(
            f"native_dynamic_package_exports entry {index} abi.abi_version must be 3"
        )
    for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
        value = abi.get(field)
        if not isinstance(value, str) or not value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be a non-empty string"
            )
            continue
        expected_value = NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]
        if value != expected_value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be {expected_value}"
            )
