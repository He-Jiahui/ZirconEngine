"""NativeDynamic payload final report diagnostics."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from .native_dynamic_payload import (
    materialized_package_loadable_artifacts_match_manifest,
    native_dynamic_content_hash,
    native_dynamic_package_payload_file_manifest,
    native_dynamic_plugins_bundle_file_manifest,
    normalized_file_manifest,
    normalized_materialized_packages,
)
from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_LOADER_MANIFEST,
)
from .export_template import is_safe_relative_path, normalize_relative_path
from .pipeline_report_native_dynamic_package_report_schema import (
    platform_bundle_native_plugins_package_report_abi_schema_diagnostics,
    platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics,
    platform_bundle_native_plugins_package_report_payload_schema_diagnostics,
    platform_bundle_native_plugins_package_report_schema_diagnostics,
)
from .pipeline_report_native_dynamic_loader_manifest import (
    native_dynamic_loader_manifest_plugins_or_diagnostics,
    native_dynamic_loader_manifest_row_field_diagnostics,
)
from .pipeline_report_native_dynamic_payload_schema import (
    platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics,
    platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics,
    platform_bundle_native_plugins_payload_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_stage_report import (
    platform_bundle_native_plugins_operation_audit_diagnostics,
    platform_bundle_native_plugins_stage_package_diagnostics,
    platform_bundle_native_plugins_stage_payload_diagnostics,
)


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


def resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return resolve_user_path(path)
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def is_non_empty_safe_relative_path(value: str) -> bool:
    return bool(value.strip()) and is_safe_relative_path(normalize_relative_path(value))


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
    plugins_dir = resolve_user_path_or_diagnostic(
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
    payload_schema_diagnostics = platform_bundle_native_plugins_payload_schema_diagnostics(payload)
    diagnostics.extend(payload_schema_diagnostics)
    if payload_schema_diagnostics:
        return diagnostics

    payload_stage_report_matches = False
    suppress_unbacked_stage_audits = False
    effective_native_dynamic_report_path = native_dynamic_report_path
    payload_stage_report = payload.get("stage_report")
    if payload_stage_report is None:
        if native_dynamic_report_path is not None:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report is required "
                "when a non-fatal NativeDynamic stage report is present"
            )
    else:
        if not isinstance(payload_stage_report, str) or not payload_stage_report:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload stage_report must be null or a non-empty string"
            )
        elif native_dynamic_report_path is None:
            payload_stage_report_path = resolve_user_path_or_diagnostic(
                payload_stage_report,
                diagnostics,
                "PlatformBundle report native_plugins_payload stage_report",
            )
            if payload_stage_report_path is None:
                payload_stage_report_path = None
            payload_source = payload.get("source")
            expected_stage_report_path = current_output_native_dynamic_report_path(
                plugins_dir,
                diagnostics,
            )
            if (
                payload_stage_report_path is not None
                and expected_stage_report_path is not None
                and payload_stage_report_path != expected_stage_report_path
            ):
                diagnostics.append(
                    "PlatformBundle report native_plugins_payload stage_report "
                    f"{payload_stage_report_path} does not match NativeDynamic report "
                    f"{expected_stage_report_path}"
                )
            elif native_dynamic_stage_report_failed:
                suppress_unbacked_stage_audits = True
            elif payload_stage_report_path is None:
                pass
            elif not payload_stage_report_path.exists():
                diagnostics.append(
                    "PlatformBundle report native_plugins_payload stage_report is present "
                    f"but NativeDynamic report {payload_stage_report_path} is missing"
                )
            else:
                payload_stage_report_matches = True
                effective_native_dynamic_report_path = payload_stage_report_path
                expected_source = payload_stage_report_path.parent / "plugins"
                if not isinstance(payload_source, str) or not payload_source:
                    diagnostics.append(
                        "PlatformBundle report native_plugins_payload source must be a non-empty string for stage-backed payloads"
                    )
                else:
                    payload_source_path = resolve_user_path_or_diagnostic(
                        payload_source,
                        diagnostics,
                        "PlatformBundle report native_plugins_payload source",
                    )
                    expected_source_path = resolve_user_path_or_diagnostic(
                        expected_source,
                        diagnostics,
                        "PlatformBundle expected NativeDynamic plugins source",
                    )
                    if (
                        payload_source_path is not None
                        and expected_source_path is not None
                        and payload_source_path != expected_source_path
                    ):
                        diagnostics.append(
                            "PlatformBundle report native_plugins_payload source "
                            f"{payload_source_path} does not match NativeDynamic plugins "
                            f"{expected_source_path}"
                        )
        else:
            payload_stage_report_path = resolve_user_path_or_diagnostic(
                payload_stage_report,
                diagnostics,
                "PlatformBundle report native_plugins_payload stage_report",
            )
            expected_report_path = resolve_user_path_or_diagnostic(
                native_dynamic_report_path,
                diagnostics,
                "NativeDynamic report path",
            )
            if payload_stage_report_path is None or expected_report_path is None:
                pass
            elif payload_stage_report_path != expected_report_path:
                diagnostics.append(
                    "PlatformBundle report native_plugins_payload stage_report "
                    f"{payload_stage_report_path} does not match NativeDynamic report "
                    f"{expected_report_path}"
                )
            else:
                payload_stage_report_matches = True
                payload_source = payload.get("source")
                expected_source = payload_stage_report_path.parent / "plugins"
                if not isinstance(payload_source, str) or not payload_source:
                    diagnostics.append(
                        "PlatformBundle report native_plugins_payload source must be a non-empty string for stage-backed payloads"
                    )
                else:
                    payload_source_path = resolve_user_path_or_diagnostic(
                        payload_source,
                        diagnostics,
                        "PlatformBundle report native_plugins_payload source",
                    )
                    expected_source_path = resolve_user_path_or_diagnostic(
                        expected_source,
                        diagnostics,
                        "PlatformBundle expected NativeDynamic plugins source",
                    )
                    if (
                        payload_source_path is not None
                        and expected_source_path is not None
                        and payload_source_path != expected_source_path
                    ):
                        diagnostics.append(
                            "PlatformBundle report native_plugins_payload source "
                            f"{payload_source_path} does not match NativeDynamic plugins "
                            f"{expected_source_path}"
                        )
    payload_bundle_path = payload.get("bundle_path")
    if payload_bundle_path is None or payload_bundle_path == "":
        diagnostics.append(
            "PlatformBundle report native_plugins_payload bundle_path must be a non-empty string"
        )
    elif isinstance(payload_bundle_path, str):
        payload_bundle_dir = resolve_user_path_or_diagnostic(
            payload_bundle_path,
            diagnostics,
            "PlatformBundle report native_plugins_payload bundle_path",
        )
        if payload_bundle_dir is not None and payload_bundle_dir != plugins_dir:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload bundle_path "
                f"{payload_bundle_dir} does not match native_plugins {plugins_dir}"
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

    actual_file_manifest = native_dynamic_plugins_bundle_file_manifest(
        plugins_dir,
        diagnostics=diagnostics,
    )
    if diagnostics:
        return diagnostics
    actual_content_hash = native_dynamic_content_hash(actual_file_manifest)
    if payload_content_hash != actual_content_hash:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload content_hash "
            f"{payload_content_hash} does not match current bundle plugins directory "
            f"{plugins_dir} content_hash {actual_content_hash}"
        )
    if payload_file_manifest != actual_file_manifest:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_manifest does not match current bundle plugins directory"
        )
    if payload_file_count != len(actual_file_manifest):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_count does not match current bundle plugins directory"
        )
    if payload_package_count != len(payload_packages):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload package_count does not match materialized_packages"
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


def platform_bundle_native_plugins_loader_manifest_diagnostics(
    payload: dict[str, Any],
    plugins_dir: Path,
) -> list[str]:
    loader_manifest = payload.get("loader_manifest")
    if not isinstance(loader_manifest, str):
        return []
    diagnostics: list[str] = []
    if not loader_manifest.strip():
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            "must be a non-empty string"
        ]
    loader_manifest_path = resolve_user_path_or_diagnostic(
        loader_manifest,
        diagnostics,
        "PlatformBundle report native_plugins_payload loader_manifest",
    )
    expected_manifest_path = resolve_user_path_or_diagnostic(
        plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST,
        diagnostics,
        "PlatformBundle current bundle loader manifest",
    )
    if loader_manifest_path is None or expected_manifest_path is None:
        return diagnostics
    if loader_manifest_path != expected_manifest_path:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} does not match current bundle loader manifest "
            f"{expected_manifest_path}"
        )
        return diagnostics
    if not loader_manifest_path.exists():
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} does not exist"
        )
    elif not loader_manifest_path.is_file():
        diagnostics.append(
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"{loader_manifest_path} is not a file"
        )
    return diagnostics


def platform_bundle_native_plugins_loader_manifest_package_diagnostics(
    payload: dict[str, Any],
    packages: list[dict[str, object]],
    *,
    stage_backed_payload: bool = False,
) -> list[str]:
    loader_manifest = payload.get("loader_manifest")
    if not isinstance(loader_manifest, str):
        return []
    if not loader_manifest.strip():
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            "must be a non-empty string"
        ]
    diagnostics: list[str] = []
    loader_manifest_path = resolve_user_path_or_diagnostic(
        loader_manifest,
        diagnostics,
        "PlatformBundle report native_plugins_payload loader_manifest",
    )
    if loader_manifest_path is None:
        return diagnostics
    plugins, plugin_diagnostics = native_dynamic_loader_manifest_plugins_or_diagnostics(
        loader_manifest_path,
        label="PlatformBundle report native_plugins_payload loader_manifest",
    )
    if plugin_diagnostics:
        return plugin_diagnostics
    assert plugins is not None
    plugin_ids = [str(plugin["id"]) for plugin in plugins]
    package_ids = [str(package["package_id"]) for package in packages]
    if plugin_ids != package_ids:
        return [
            "PlatformBundle report native_plugins_payload loader_manifest "
            f"plugin ids {plugin_ids} do not match materialized package ids "
            f"{package_ids}"
        ]
    return native_dynamic_loader_manifest_row_field_diagnostics(
        plugins,
        platform_bundle_native_plugins_loader_manifest_expected_plugins_by_id(
            packages,
            loader_manifest_path.parent,
        ),
        label="PlatformBundle report native_plugins_payload loader_manifest",
        expected_label="materialized package",
        require_fields=stage_backed_payload,
    )


def platform_bundle_native_plugins_loader_manifest_expected_plugins_by_id(
    packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, dict[str, Any]]:
    expected_plugins: dict[str, dict[str, Any]] = {}
    try:
        plugins_root = plugins_dir.resolve()
    except OSError:
        return expected_plugins

    for package in packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(plugins_root)
        except (OSError, ValueError):
            continue
        path = f"plugins/{relative_destination.as_posix().rstrip('/')}"
        expected_plugin = {
            "path": path,
            "manifest": f"{path}/plugin.toml",
            "abi": {
                "abi_version": 3,
                **NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
            },
        }
        package_report = package.get("package_report")
        if isinstance(package_report, str):
            package_report_path = Path(package_report).expanduser()
            try:
                relative_package_report = package_report_path.resolve().relative_to(
                    plugins_root
                )
            except (OSError, ValueError):
                pass
            else:
                expected_plugin["package_report"] = (
                    f"plugins/{relative_package_report.as_posix()}"
                )
        expected_plugins[package_id] = expected_plugin
    return expected_plugins


def current_output_native_dynamic_report_path(
    plugins_dir: Path,
    diagnostics: list[str] | None = None,
) -> Path | None:
    try:
        return plugins_dir.resolve().parents[2] / "stages" / "native_dynamic" / "report.json"
    except IndexError:
        return None
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(
                f"PlatformBundle report native_plugins {plugins_dir} "
                f"could not be resolved: {error}"
            )
        return None

def native_dynamic_stage_report_path(
    stage_reports: list[dict[str, Any]],
    diagnostics: list[str] | None = None,
) -> Path | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "native_dynamic":
            continue
        if stage_report.get("fatal") is True:
            return None
        report_path = stage_report.get("path")
        if not isinstance(report_path, str) or not report_path:
            return None
        if diagnostics is None:
            return resolve_user_path(report_path)
        return resolve_user_path_or_diagnostic(
            report_path,
            diagnostics,
            "NativeDynamic stage report path",
        )
    return None


def platform_bundle_native_plugins_package_path_diagnostics(
    packages: list[dict[str, object]],
    plugins_dir: Path,
    *,
    stage_backed_payload: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    plugins_root = resolve_user_path_or_diagnostic(
        plugins_dir,
        diagnostics,
        "PlatformBundle report native_plugins",
    )
    if plugins_root is None:
        return diagnostics
    for index, package in enumerate(packages):
        destination = str(package["destination"])
        destination_path = resolve_user_path_or_diagnostic(
            destination,
            diagnostics,
            "PlatformBundle report native_plugins_payload "
            f"materialized_packages[{index}] destination",
        )
        if destination_path is None:
            continue
        try:
            destination_path.relative_to(plugins_root)
        except ValueError:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] destination {destination} "
                f"is outside native_plugins {plugins_root}"
            )
            continue
        package_report = package.get("package_report")
        if package_report is None:
            if stage_backed_payload:
                diagnostics.append(
                    "PlatformBundle report native_plugins_payload "
                    f"materialized_packages[{index}] package_report "
                    "is required for stage-backed payloads"
                )
            continue
        package_report_path = resolve_user_path_or_diagnostic(
            str(package_report),
            diagnostics,
            "PlatformBundle report native_plugins_payload "
            f"materialized_packages[{index}] package_report",
        )
        if package_report_path is None:
            continue
        try:
            package_report_path.relative_to(destination_path)
        except ValueError:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report} "
                f"is outside package destination {destination_path}"
            )
            continue
        if not package_report_path.exists():
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report_path} "
                "does not exist"
            )
            continue
        if not package_report_path.is_file():
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"materialized_packages[{index}] package_report {package_report_path} "
                "is not a file"
            )
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_content_diagnostics(
                index,
                package,
                plugins_root,
                destination_path,
                package_report_path,
            )
        )
    return diagnostics


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

def pipeline_native_plugins_payload(
    stage_reports: list[dict[str, Any]],
) -> dict[str, Any] | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            return None
        report = stage_report.get("report")
        if not isinstance(report, dict):
            return None
        native_plugins_payload = report.get("native_plugins_payload")
        if isinstance(native_plugins_payload, dict):
            return native_plugins_payload
        return None
    return None
