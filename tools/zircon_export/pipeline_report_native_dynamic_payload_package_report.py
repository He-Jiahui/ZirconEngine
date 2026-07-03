"""NativeDynamic payload package-report diagnostics for final reports."""

from __future__ import annotations

import tomllib
from pathlib import Path

from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from .native_dynamic_payload import normalized_file_manifest
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_package_payload_file_manifest,
)
from .pipeline_report_native_dynamic_package_report_schema import (
    platform_bundle_native_plugins_package_report_abi_schema_diagnostics,
    platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics,
    platform_bundle_native_plugins_package_report_payload_schema_diagnostics,
    platform_bundle_native_plugins_package_report_schema_diagnostics,
)


def is_non_empty_safe_relative_path(value: str) -> bool:
    return bool(value.strip()) and is_safe_relative_path(normalize_relative_path(value))


def platform_bundle_native_plugins_package_report_content_diagnostics(
    index: int,
    package: dict[str, object],
    plugins_root: Path,
    package_dir: Path,
    package_report_path: Path,
    *,
    label: str | None = None,
) -> list[str]:
    diagnostics: list[str] = []
    if label is None:
        label = (
            "PlatformBundle report native_plugins_payload "
            f"materialized_packages[{index}] package_report"
        )
    try:
        with package_report_path.open("rb") as report_file:
            package_report = tomllib.load(report_file)
    except OSError as error:
        return [f"{label} {package_report_path} could not be read: {error}"]
    except tomllib.TOMLDecodeError as error:
        return [f"{label} {package_report_path} is not valid TOML: {error}"]

    package_report_schema_diagnostics = (
        platform_bundle_native_plugins_package_report_schema_diagnostics(
            label,
            package_report,
        )
    )
    diagnostics.extend(package_report_schema_diagnostics)
    package_report_id = package_report.get("package_id")
    if package_report_id is None:
        diagnostics.append(f"{label} package_id must be a non-empty string")
        return diagnostics
    if isinstance(package_report_id, str) and not package_report_id.strip():
        return diagnostics
    if not isinstance(package_report_id, str):
        return diagnostics
    format_version = package_report.get("format_version")
    if type(format_version) is int and format_version != 1:
        diagnostics.append(
            f"{label} format_version {format_version} is not supported; expected 1"
        )
    package_id = str(package["package_id"])
    if (
        isinstance(package_report_id, str)
        and package_report_id.strip() == package_report_id
        and package_report_id != package_id
    ):
        diagnostics.append(
            f"{label} package_id {package_report_id} "
            f"does not match materialized package {package_id}"
        )
    directory = package_report.get("directory")
    expected_directory = package_dir.relative_to(plugins_root).as_posix()
    if directory is not None:
        if isinstance(directory, str) and not directory.strip():
            pass
        elif isinstance(directory, str) and directory.strip() != directory:
            pass
        elif isinstance(directory, str) and not is_non_empty_safe_relative_path(
            directory
        ):
            pass
        elif isinstance(directory, str) and directory != expected_directory:
            diagnostics.append(
                f"{label} directory {directory} "
                f"does not match materialized package directory {expected_directory}"
            )
    expected_path = f"plugins/{expected_directory}"
    expected_manifest = f"{expected_path}/plugin.toml"
    for field, expected_value in (
        ("path", expected_path),
        ("manifest", expected_manifest),
    ):
        value = package_report.get(field)
        if value is None:
            continue
        if isinstance(value, str) and not value.strip():
            continue
        if isinstance(value, str) and value.strip() != value:
            continue
        if isinstance(value, str) and not is_non_empty_safe_relative_path(value):
            continue
        if isinstance(value, str) and value != expected_value:
            diagnostics.append(
                f"{label} {field} {value} does not match {expected_value}"
            )
    payload = package_report.get("payload")
    abi = package_report.get("abi")
    if isinstance(abi, dict):
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_abi_diagnostics(
                label,
                abi,
            )
        )
    if isinstance(payload, dict):
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_payload_diagnostics(
                label,
                package_dir,
                payload,
            )
        )
    return diagnostics


def platform_bundle_native_plugins_package_report_abi_diagnostics(
    label: str,
    abi: object,
) -> list[str]:
    if not isinstance(abi, dict):
        return [f"{label} abi must be an object"]
    diagnostics: list[str] = []
    abi_schema_diagnostics = (
        platform_bundle_native_plugins_package_report_abi_schema_diagnostics(
            label,
            abi,
        )
    )
    diagnostics.extend(abi_schema_diagnostics)
    abi_version = abi.get("abi_version")
    if abi_version is None:
        diagnostics.append(f"{label} abi.abi_version must be an integer")
    elif type(abi_version) is int and abi_version != 3:
        diagnostics.append(f"{label} abi.abi_version must be 3")
    for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
        value = abi.get(field)
        if value is None:
            diagnostics.append(f"{label} abi.{field} must be a non-empty string")
            continue
        if isinstance(value, str) and not value.strip():
            continue
        if isinstance(value, str) and value.strip() != value:
            continue
        if not isinstance(value, str):
            continue
        expected_value = NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]
        if value != expected_value:
            diagnostics.append(f"{label} abi.{field} must be {expected_value}")
    return diagnostics


def platform_bundle_native_plugins_package_report_payload_diagnostics(
    label: str,
    package_dir: Path,
    payload: object,
) -> list[str]:
    if not isinstance(payload, dict):
        return [f"{label} payload must be a table"]
    diagnostics = platform_bundle_native_plugins_package_report_payload_schema_diagnostics(
        label,
        payload,
    )
    file_count = payload.get("file_count")
    content_hash = payload.get("content_hash")
    payload_files = payload.get("files")
    payload_files_schema_diagnostics = (
        platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics(
            label,
            payload,
        )
        if payload_files is not None
        else []
    )
    if file_count is None:
        diagnostics.append(f"{label} payload file_count must be an integer")
    if content_hash is None:
        diagnostics.append(f"{label} payload content_hash must be a non-empty string")
    elif isinstance(content_hash, str) and not content_hash.strip():
        pass
    if payload_files is not None and not payload_files_schema_diagnostics:
        if normalized_file_manifest(payload_files) is None:
            diagnostics.append(f"{label} payload files are malformed")
            return diagnostics
    if diagnostics:
        return diagnostics

    current_file_manifest = native_dynamic_package_payload_file_manifest(
        package_dir,
        diagnostics=diagnostics,
    )
    if diagnostics:
        return diagnostics
    current_content_hash = native_dynamic_content_hash(current_file_manifest)
    if file_count != len(current_file_manifest):
        diagnostics.append(
            f"{label} payload file_count {file_count} "
            f"does not match current package payload {len(current_file_manifest)}"
        )
    if content_hash != current_content_hash:
        diagnostics.append(
            f"{label} payload content_hash {content_hash} "
            f"does not match current package payload {current_content_hash}"
        )
    if payload_files is not None:
        normalized_payload_files = normalized_file_manifest(payload_files)
        if normalized_payload_files != current_file_manifest:
            diagnostics.append(
                f"{label} payload files do not match current package payload"
            )
    return diagnostics
