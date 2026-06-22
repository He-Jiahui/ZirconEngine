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
from .pipeline_report_validate_profile_summary_schema import (
    VALIDATE_PROFILE_SUMMARY_TARGET_PLATFORMS,
    validate_known_trimmed_string_schema_diagnostics,
)
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
    if not isinstance(packages, list):
        diagnostics.append("validate report native_dynamic_packages must be a string array")
        return []
    normalized_packages: list[str] = []
    package_id_indexes: dict[str, int] = {}
    for index, package_id in enumerate(packages):
        if not isinstance(package_id, str):
            diagnostics.append(
                f"native_dynamic_packages entry {index} must be a string"
            )
            continue
        if not native_dynamic_plan_non_empty_trimmed_string(package_id):
            diagnostics.append(
                f"native_dynamic_packages entry {index} "
                "must be a non-empty trimmed string"
            )
            continue
        previous_index = package_id_indexes.get(package_id)
        if previous_index is not None:
            diagnostics.append(
                f"native_dynamic_packages entry {package_id} duplicates entry {previous_index}"
            )
        else:
            package_id_indexes[package_id] = index
            normalized_packages.append(package_id)
    return normalized_packages


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
        if native_dynamic_plan_non_empty_trimmed_string(package_id):
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


def native_dynamic_target_platform(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> str | None:
    if validate_payload is None:
        return None
    profile_summary = validate_payload.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    if "target_platform" in profile_summary:
        label = "validate report profile_summary.target_platform"
        target_platform = profile_summary.get("target_platform")
    elif "platform" in profile_summary:
        label = "validate report profile_summary.platform"
        target_platform = profile_summary.get("platform")
    else:
        return None
    if not isinstance(target_platform, str):
        diagnostics.append(f"{label} must be a string")
        return None
    platform_diagnostics = validate_known_trimmed_string_schema_diagnostics(
        label,
        target_platform,
        VALIDATE_PROFILE_SUMMARY_TARGET_PLATFORMS,
        "export target platform",
    )
    diagnostics.extend(platform_diagnostics)
    if platform_diagnostics:
        return None
    return target_platform


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
        if not isinstance(value, str):
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field {field} must be a string"
            )
        elif not value.strip():
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field {field} must be a non-empty string"
            )
        elif value.strip() != value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field {field} "
                "must be a non-empty trimmed string"
            )
    package_id = package_export.get("package_id")
    directory = package_export.get("directory")
    path = package_export.get("path")
    manifest = package_export.get("manifest")
    package_report = package_export.get("package_report")
    package_id_clean = native_dynamic_plan_non_empty_trimmed_string(package_id)
    directory_clean = native_dynamic_plan_non_empty_trimmed_string(directory)
    path_clean = native_dynamic_plan_non_empty_trimmed_string(path)
    manifest_clean = native_dynamic_plan_non_empty_trimmed_string(manifest)
    if (
        package_id_clean
        and directory_clean
    ):
        expected_directory = native_dynamic_package_directory(package_id)
        if directory != expected_directory:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} directory must be {expected_directory} for package_id {package_id}"
            )
    if directory_clean:
        expected_path = f"plugins/{directory}"
        expected_manifest = f"{expected_path}/plugin.toml"
        expected_package_report = f"{expected_path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}"
        if path_clean and path != expected_path:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} path must be {expected_path} for directory {directory}"
            )
        if manifest_clean and manifest != expected_manifest:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} manifest must be {expected_manifest} for directory {directory}"
            )
        if package_report is None:
            package_export["package_report"] = expected_package_report
        elif not isinstance(package_report, str):
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field package_report must be a string"
            )
        elif not package_report.strip():
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field package_report must be a non-empty string"
            )
        elif package_report.strip() != package_report:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} field package_report "
                "must be a non-empty trimmed string"
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
        if not isinstance(value, str):
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be a string"
            )
            continue
        if not value.strip():
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be a non-empty string"
            )
            continue
        if value.strip() != value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} "
                "must be a non-empty trimmed string"
            )
            continue
        expected_value = NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]
        if value != expected_value:
            diagnostics.append(
                f"native_dynamic_package_exports entry {index} abi.{field} must be {expected_value}"
            )


def native_dynamic_plan_non_empty_trimmed_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
