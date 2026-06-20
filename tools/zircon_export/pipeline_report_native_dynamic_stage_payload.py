"""NativeDynamic stage payload diagnostics for final reports."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
)
from .native_dynamic_payload import (
    materialized_package_loadable_artifacts_match_manifest,
    native_dynamic_content_hash,
    native_dynamic_plugins_file_manifest,
    normalized_file_manifest,
    normalized_materialized_packages,
)
from .pipeline_report_native_dynamic_payload import (
    platform_bundle_native_plugins_package_report_content_diagnostics,
)
from .pipeline_report_native_dynamic_loader_manifest import (
    native_dynamic_loader_manifest_abi_field_type_diagnostics,
    native_dynamic_loader_manifest_plugins_or_diagnostics,
)
from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
)
from .pipeline_report_native_dynamic_package_export_schema import (
    native_dynamic_package_export_schema_diagnostics,
)


def native_dynamic_stage_payload_diagnostics(
    stage_key: str,
    report: dict[str, Any],
    report_path: Path,
    *,
    validate_payload: dict[str, Any] | None = None,
) -> list[str]:
    if stage_key != "native_dynamic" or report.get("fatal") is not False:
        return []

    file_manifest = normalized_file_manifest(report.get("file_manifest"))
    materialized_packages = normalized_materialized_packages(
        report.get("materialized_packages")
    )
    content_hash = report.get("content_hash")
    if (
        file_manifest is None
        or materialized_packages is None
        or not isinstance(content_hash, str)
    ):
        return []

    try:
        stage_dir = report_path.expanduser().parent.resolve()
    except OSError as error:
        return [
            "native_dynamic report stage report directory "
            f"{report_path.parent} could not be resolved: {error}"
        ]
    plugins_dir = stage_dir / "plugins"
    loader_manifest = plugins_dir / "native_plugins.toml"
    manifest_diagnostics: list[str] = []
    current_file_manifest = native_dynamic_plugins_file_manifest(
        stage_dir,
        plugins_dir,
        diagnostics=manifest_diagnostics,
    )
    if manifest_diagnostics:
        return manifest_diagnostics

    diagnostics: list[str] = []
    package_count = report.get("package_count")
    if type(package_count) is int and package_count != len(materialized_packages):
        diagnostics.append(
            "native_dynamic report package_count "
            f"{package_count} does not match materialized_packages "
            f"{len(materialized_packages)}"
        )
    selected_packages = report.get("native_dynamic_packages")
    materialized_package_ids = [
        str(package["package_id"]) for package in materialized_packages
    ]
    materialized_package_artifacts = materialized_package_relative_artifacts(
        materialized_packages,
        plugins_dir,
    )
    validate_packages = validate_native_dynamic_packages(validate_payload)
    if (
        validate_packages is not None
        and materialized_package_ids != validate_packages
    ):
        diagnostics.append(
            "native_dynamic report materialized package ids "
            f"{materialized_package_ids} do not match validate report "
            f"plan_summary.native_dynamic_packages {validate_packages}"
        )
    validate_package_exports = validate_native_dynamic_package_export_ids(
        validate_payload,
    )
    validate_package_export_rows = validate_native_dynamic_package_exports(
        validate_payload,
    )
    report_package_export_rows = schema_clean_native_dynamic_package_exports(
        report.get("package_exports"),
        "native_dynamic report package_exports",
    )
    if (
        validate_package_exports is not None
        and materialized_package_ids != validate_package_exports
    ):
        diagnostics.append(
            "native_dynamic report materialized package ids "
            f"{materialized_package_ids} do not match validate report "
            "plan_summary.native_dynamic_package_exports package ids "
            f"{validate_package_exports}"
        )
    if report_package_export_rows is not None:
        expected_package_export_rows = report_package_export_rows
        expected_package_export_label = "native_dynamic report package_exports"
    else:
        expected_package_export_rows = validate_package_export_rows
        expected_package_export_label = (
            "validate report plan_summary.native_dynamic_package_exports"
            if validate_package_export_rows is not None
            else None
        )
    if report_package_export_rows is not None:
        diagnostics.extend(
            native_dynamic_package_export_materialization_diagnostics(
                report_package_export_rows,
                materialized_packages,
                plugins_dir,
                "native_dynamic report package_exports",
            )
        )
    if validate_package_export_rows is not None:
        diagnostics.extend(
            native_dynamic_package_export_materialization_diagnostics(
                validate_package_export_rows,
                materialized_packages,
                plugins_dir,
                "validate report plan_summary.native_dynamic_package_exports",
            )
        )
    diagnostics.extend(
        native_dynamic_loader_manifest_package_diagnostics(
            loader_manifest,
            materialized_package_ids,
            expected_package_exports=expected_package_export_rows,
            expected_package_exports_label=expected_package_export_label,
        )
    )
    diagnostics.extend(
        native_dynamic_package_report_diagnostics(
            materialized_packages,
            plugins_dir,
            report.get("native_plugin_root"),
        )
    )
    if isinstance(selected_packages, list) and all(
        isinstance(package_id, str) for package_id in selected_packages
    ):
        if selected_packages != materialized_package_ids:
            diagnostics.append(
                "native_dynamic report native_dynamic_packages "
                f"{selected_packages} does not match materialized package ids "
                f"{materialized_package_ids}"
            )
        if validate_packages is not None and selected_packages != validate_packages:
            diagnostics.append(
                "native_dynamic report native_dynamic_packages "
                f"{selected_packages} does not match validate report "
                f"plan_summary.native_dynamic_packages {validate_packages}"
            )
    if report_package_export_rows is not None:
        package_export_ids = [
            str(package_export["package_id"])
            for package_export in report_package_export_rows
        ]
        if package_export_ids != materialized_package_ids:
            diagnostics.append(
                "native_dynamic report package_exports package ids "
                f"{package_export_ids} do not match materialized package ids "
                f"{materialized_package_ids}"
            )
        if (
            validate_package_exports is not None
            and package_export_ids != validate_package_exports
        ):
            diagnostics.append(
                "native_dynamic report package_exports package ids "
                f"{package_export_ids} do not match validate report "
                "plan_summary.native_dynamic_package_exports package ids "
                f"{validate_package_exports}"
            )
    diagnostics.extend(
        native_dynamic_package_table_diagnostics(
            report.get("native_build_plan"),
            "native_build_plan",
            materialized_package_ids,
        )
    )
    diagnostics.extend(
        native_dynamic_package_table_diagnostics(
            report.get("native_build_execution"),
            "native_build_execution",
            materialized_package_ids,
        )
    )
    diagnostics.extend(
        native_dynamic_build_execution_plan_diagnostics(
            report.get("native_build_plan"),
            report.get("native_build_execution"),
        )
    )
    diagnostics.extend(
        native_dynamic_build_execution_artifact_diagnostics(
            report.get("native_build_execution"),
            materialized_packages,
            plugins_dir,
            current_file_manifest,
        )
    )
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        diagnostics.extend(
            native_dynamic_package_table_diagnostics(
                report.get(field),
                field,
                materialized_package_ids,
            )
        )
        diagnostics.extend(
            native_dynamic_operation_audit_artifact_diagnostics(
                report.get(field),
                field,
                materialized_package_artifacts,
            )
        )
    current_content_hash = native_dynamic_content_hash(current_file_manifest)
    if content_hash != current_content_hash:
        diagnostics.append(
            "native_dynamic report content_hash "
            f"{content_hash} does not match current NativeDynamic plugins "
            f"directory {plugins_dir} content_hash {current_content_hash}"
        )
    if file_manifest != current_file_manifest:
        diagnostics.append(
            "native_dynamic report file_manifest does not match current "
            f"NativeDynamic plugins directory {plugins_dir}"
        )
    if not materialized_package_loadable_artifacts_match_manifest(
        materialized_packages,
        current_file_manifest,
        plugins_dir,
        diagnostics,
    ):
        diagnostics.append(
            "native_dynamic report loadable_artifacts are not present in "
            "current NativeDynamic plugins directory"
        )
    return diagnostics


def native_dynamic_package_report_diagnostics(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    native_plugin_root: object,
) -> list[str]:
    diagnostics: list[str] = []
    try:
        plugins_root = plugins_dir.expanduser().resolve()
    except OSError as error:
        return [f"native_dynamic report plugins_dir {plugins_dir} could not be resolved: {error}"]
    native_plugin_root_path: Path | None = None
    if isinstance(native_plugin_root, str) and native_plugin_root:
        try:
            native_plugin_root_path = Path(native_plugin_root).expanduser().resolve()
        except OSError as error:
            diagnostics.append(
                "native_dynamic report native_plugin_root "
                f"{native_plugin_root} could not be resolved: {error}"
            )

    for index, package in enumerate(materialized_packages):
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        source_label = (
            "native_dynamic report "
            f"materialized_packages[{index}] source"
        )
        label = (
            "native_dynamic report "
            f"materialized_packages[{index}] package_report"
        )
        if package.get("source") is None:
            diagnostics.append(
                f"{source_label} is required for NativeDynamic stage materialized packages"
            )
        source = package.get("source")
        if isinstance(source, str) and not source:
            diagnostics.append(
                f"{source_label} must be a non-empty string for "
                "NativeDynamic stage materialized packages"
            )
        if isinstance(source, str) and source:
            try:
                source_path = Path(source).expanduser().resolve()
            except OSError as error:
                diagnostics.append(f"{source_label} {source} could not be resolved: {error}")
            else:
                if native_plugin_root_path is not None:
                    try:
                        source_path.relative_to(native_plugin_root_path)
                    except ValueError:
                        diagnostics.append(
                            f"{source_label} {source_path} is outside "
                            f"native_plugin_root {native_plugin_root_path}"
                        )
                if not source_path.exists():
                    diagnostics.append(f"{source_label} {source_path} does not exist")
                elif not source_path.is_dir():
                    diagnostics.append(f"{source_label} {source_path} is not a directory")
                else:
                    source_manifest = source_path / "plugin.toml"
                    if not source_manifest.exists():
                        diagnostics.append(
                            f"{source_label} manifest {source_manifest} does not exist"
                        )
                    elif not source_manifest.is_file():
                        diagnostics.append(
                            f"{source_label} manifest {source_manifest} is not a file"
                        )
                    else:
                        manifest_id = native_dynamic_source_manifest_id(
                            source_manifest,
                            source_label,
                            diagnostics,
                        )
                        if manifest_id is not None and manifest_id != package_id:
                            diagnostics.append(
                                f"{source_label} manifest id {manifest_id} "
                                f"does not match materialized package {package_id}"
                            )
        try:
            package_dir = destination.resolve()
        except OSError as error:
            diagnostics.append(
                "native_dynamic report "
                f"materialized_packages[{index}] destination {destination} "
                f"could not be resolved: {error}"
            )
            continue
        try:
            package_dir.relative_to(plugins_root)
        except ValueError:
            diagnostics.append(
                "native_dynamic report "
                f"materialized_packages[{index}] destination {destination} "
                f"is outside plugins_dir {plugins_root}"
            )
            continue

        declared_package_report = package.get("package_report")
        expected_package_report = package_dir / NATIVE_DYNAMIC_PACKAGE_REPORT_FILE
        if declared_package_report is None:
            diagnostics.append(
                f"{label} is required for NativeDynamic stage materialized packages"
            )
            continue
        if isinstance(declared_package_report, str) and not declared_package_report:
            diagnostics.append(
                f"{label} must be a non-empty string for "
                "NativeDynamic stage materialized packages"
            )
            continue
        declared_path = Path(str(declared_package_report)).expanduser()
        try:
            declared_path = declared_path.resolve()
        except OSError as error:
            diagnostics.append(
                f"{label} {declared_package_report} could not be resolved: {error}"
            )
            continue
        if declared_path != expected_package_report:
            diagnostics.append(
                f"{label} {declared_path} does not match expected "
                f"{expected_package_report} for package {package_id}"
            )
            continue
        package_report_path = expected_package_report
        if not package_report_path.exists():
            diagnostics.append(f"{label} {package_report_path} does not exist")
            continue
        if not package_report_path.is_file():
            diagnostics.append(f"{label} {package_report_path} is not a file")
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_package_report_content_diagnostics(
                index,
                package,
                plugins_root,
                package_dir,
                package_report_path,
                label=label,
            )
        )
    return diagnostics


def native_dynamic_source_manifest_id(
    source_manifest: Path,
    source_label: str,
    diagnostics: list[str],
) -> str | None:
    try:
        with source_manifest.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"{source_label} manifest {source_manifest} could not be parsed: {error}"
        )
        return None
    except OSError as error:
        diagnostics.append(
            f"{source_label} manifest {source_manifest} could not be read: {error}"
        )
        return None
    manifest_id = manifest.get("id")
    if isinstance(manifest_id, str) and manifest_id:
        return manifest_id
    diagnostics.append(f"{source_label} manifest id must be a non-empty string")
    return None


def native_dynamic_loader_manifest_package_diagnostics(
    loader_manifest: Path,
    materialized_package_ids: list[str],
    *,
    expected_package_exports: list[dict[str, Any]] | None = None,
    expected_package_exports_label: str | None = None,
) -> list[str]:
    plugins, diagnostics = native_dynamic_loader_manifest_plugins_or_diagnostics(
        loader_manifest,
        label="native_dynamic loader_manifest",
    )
    if diagnostics:
        return diagnostics
    assert plugins is not None
    plugin_ids = [str(plugin["id"]) for plugin in plugins]
    if plugin_ids != materialized_package_ids:
        return [
            "native_dynamic loader_manifest plugin ids "
            f"{plugin_ids} do not match materialized package ids "
            f"{materialized_package_ids}"
        ]

    if expected_package_exports is None or expected_package_exports_label is None:
        return []
    return native_dynamic_loader_manifest_package_export_diagnostics(
        plugins,
        expected_package_exports,
        expected_package_exports_label,
    )


def native_dynamic_loader_manifest_package_export_diagnostics(
    loader_plugins: list[dict[str, Any]],
    expected_package_exports: list[dict[str, Any]],
    expected_label: str,
) -> list[str]:
    expected_exports_by_id = {
        str(package_export["package_id"]): package_export
        for package_export in expected_package_exports
        if isinstance(package_export.get("package_id"), str)
    }
    diagnostics: list[str] = []
    for plugin in loader_plugins:
        package_id = str(plugin["id"])
        expected_export = expected_exports_by_id.get(package_id)
        if expected_export is None:
            continue
        for field in ("path", "manifest", "package_report"):
            plugin_value = plugin.get(field)
            expected_value = expected_export.get(field)
            if isinstance(expected_value, str) and field not in plugin:
                diagnostics.append(
                    "native_dynamic loader_manifest plugin "
                    f"{package_id} {field} is required by {expected_label}"
                )
                continue
            if (
                isinstance(plugin_value, str)
                and isinstance(expected_value, str)
                and plugin_value != expected_value
            ):
                diagnostics.append(
                    "native_dynamic loader_manifest plugin "
                    f"{package_id} {field} {plugin_value} does not match "
                    f"{expected_label} {field} {expected_value}"
                )
        plugin_abi = plugin.get("abi")
        expected_abi = expected_export.get("abi")
        if isinstance(expected_abi, dict) and "abi" not in plugin:
            diagnostics.append(
                "native_dynamic loader_manifest plugin "
                f"{package_id} abi is required by {expected_label}"
            )
            continue
        if not isinstance(plugin_abi, dict) or not isinstance(expected_abi, dict):
            continue
        for field in plugin_abi:
            if field not in expected_abi:
                diagnostics.append(
                    "native_dynamic loader_manifest plugin "
                    f"{package_id} abi.{field} is not supported by "
                    f"{expected_label}"
                )
        invalid_abi_fields = native_dynamic_loader_manifest_abi_field_type_diagnostics(
            package_id,
            plugin_abi,
            expected_abi,
            label="native_dynamic loader_manifest",
        )
        diagnostics.extend(invalid_abi_fields.values())
        for field in ("abi_version", *NATIVE_DYNAMIC_ABI_STRING_FIELDS):
            plugin_value = plugin_abi.get(field)
            expected_value = expected_abi.get(field)
            if (
                field in plugin_abi
                and field in expected_abi
                and field not in invalid_abi_fields
                and plugin_value != expected_value
            ):
                diagnostics.append(
                    "native_dynamic loader_manifest plugin "
                    f"{package_id} abi.{field} {plugin_value} does not match "
                    f"{expected_label} abi.{field} {expected_value}"
                )
    return diagnostics


def validate_native_dynamic_packages(
    validate_payload: dict[str, Any] | None,
) -> list[str] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    packages = plan_summary.get("native_dynamic_packages")
    if not isinstance(packages, list) or not all(
        isinstance(package, str) for package in packages
    ):
        return None
    return list(packages)


def validate_native_dynamic_package_export_ids(
    validate_payload: dict[str, Any] | None,
) -> list[str] | None:
    package_exports = validate_native_dynamic_package_exports(validate_payload)
    if package_exports is None:
        return None
    return [
        str(package_export["package_id"]) for package_export in package_exports
    ]


def validate_native_dynamic_package_exports(
    validate_payload: dict[str, Any] | None,
) -> list[dict[str, Any]] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    return schema_clean_native_dynamic_package_exports(
        plan_summary.get("native_dynamic_package_exports"),
        "validate report plan_summary.native_dynamic_package_exports",
    )


def schema_clean_native_dynamic_package_exports(
    package_exports: object,
    label: str,
) -> list[dict[str, Any]] | None:
    normalized_package_exports = normalized_native_dynamic_package_exports(
        package_exports
    )
    if normalized_package_exports is None:
        return None
    if native_dynamic_package_export_schema_diagnostics(
        label,
        normalized_package_exports,
    ):
        return None
    return normalized_package_exports


def normalized_native_dynamic_package_exports(
    package_exports: object,
) -> list[dict[str, Any]] | None:
    if not isinstance(package_exports, list) or not all(
        isinstance(package_export, dict)
        and isinstance(package_export.get("package_id"), str)
        for package_export in package_exports
    ):
        return None
    return [dict(package_export) for package_export in package_exports]


def native_dynamic_package_export_materialization_diagnostics(
    package_exports: list[dict[str, Any]],
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    label: str,
) -> list[str]:
    expected_exports_by_id = materialized_package_exports_by_id(
        materialized_packages,
        plugins_dir,
    )
    diagnostics: list[str] = []
    for package_export in package_exports:
        package_id = str(package_export["package_id"])
        expected_export = expected_exports_by_id.get(package_id)
        if expected_export is None:
            continue
        for field, expected_value in expected_export.items():
            value = package_export.get(field)
            if isinstance(value, str) and value != expected_value:
                diagnostics.append(
                    f"{label} package {package_id} {field} {value} "
                    f"does not match materialized package {field} "
                    f"{expected_value}"
                )
    return diagnostics


def materialized_package_exports_by_id(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, dict[str, str]]:
    package_exports: dict[str, dict[str, str]] = {}
    try:
        plugins_root = plugins_dir.resolve()
    except OSError:
        return package_exports

    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(plugins_root)
        except (OSError, ValueError):
            continue
        directory = relative_destination.as_posix().rstrip("/")
        path = f"plugins/{directory}"
        package_exports[package_id] = {
            "directory": directory,
            "path": path,
            "manifest": f"{path}/plugin.toml",
            "package_report": f"{path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}",
        }
    return package_exports


def materialized_package_relative_artifacts(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, list[str]]:
    package_artifacts: dict[str, list[str]] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(
                plugins_dir.resolve(),
            )
        except (OSError, ValueError):
            continue
        package_prefix = f"plugins/{relative_destination.as_posix().rstrip('/')}/"
        artifacts: list[str] = []
        for artifact in package["loadable_artifacts"]:
            artifact_path = str(artifact)
            if artifact_path.startswith(package_prefix):
                artifacts.append(artifact_path.removeprefix(package_prefix))
        package_artifacts[package_id] = artifacts
    return package_artifacts


def materialized_package_loadable_artifact_paths(
    materialized_packages: list[dict[str, object]],
) -> dict[str, list[str]]:
    package_artifacts: dict[str, list[str]] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        loadable_artifacts = package.get("loadable_artifacts")
        if not isinstance(loadable_artifacts, list):
            continue
        package_artifacts[package_id] = [
            str(artifact)
            for artifact in loadable_artifacts
            if isinstance(artifact, str)
        ]
    return package_artifacts


def materialized_package_native_dir_paths(
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
) -> dict[str, str]:
    package_native_dirs: dict[str, str] = {}
    for package in materialized_packages:
        package_id = str(package["package_id"])
        destination = Path(str(package["destination"])).expanduser()
        try:
            relative_destination = destination.resolve().relative_to(
                plugins_dir.resolve(),
            )
        except (OSError, ValueError):
            continue
        package_native_dirs[
            package_id
        ] = f"plugins/{relative_destination.as_posix().rstrip('/')}/native/"
    return package_native_dirs


def native_dynamic_file_manifest_paths(
    file_manifest: list[dict[str, object]],
) -> set[str]:
    return {
        str(entry["path"])
        for entry in file_manifest
        if isinstance(entry.get("path"), str)
    }


def native_dynamic_file_manifest_contains_path_or_directory(
    bundle_path: str,
    file_manifest_paths: set[str],
) -> bool:
    normalized_path = bundle_path.rstrip("/")
    return normalized_path in file_manifest_paths or any(
        path.startswith(f"{normalized_path}/") for path in file_manifest_paths
    )


def native_dynamic_build_execution_plan_diagnostics(
    build_plan: object,
    build_execution: object,
) -> list[str]:
    if not isinstance(build_plan, dict) or not isinstance(build_execution, dict):
        return []
    plan_packages = native_dynamic_build_package_rows_by_id(
        build_plan.get("packages")
    )
    execution_packages = build_execution.get("packages")
    if not plan_packages or not isinstance(execution_packages, list):
        return []

    diagnostics: list[str] = []
    for execution_package in execution_packages:
        if not isinstance(execution_package, dict):
            continue
        package_id = execution_package.get("package_id")
        if not isinstance(package_id, str) or not package_id.strip():
            continue
        plan_package = plan_packages.get(package_id)
        if plan_package is None:
            continue
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "crate_name",
                plan_package,
                execution_package,
            )
        )
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "command",
                plan_package,
                execution_package,
            )
        )
        diagnostics.extend(
            native_dynamic_build_execution_plan_field_diagnostics(
                package_id,
                "expected_loadable_artifact",
                plan_package,
                execution_package,
            )
        )
    return diagnostics


def native_dynamic_build_package_rows_by_id(
    packages: object,
) -> dict[str, dict[str, object]]:
    if not isinstance(packages, list):
        return {}
    rows: dict[str, dict[str, object]] = {}
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        if isinstance(package_id, str) and package_id.strip():
            rows[package_id] = package
    return rows


def native_dynamic_build_execution_plan_field_diagnostics(
    package_id: str,
    field: str,
    plan_package: dict[str, object],
    execution_package: dict[str, object],
) -> list[str]:
    plan_value = plan_package.get(field)
    execution_value = execution_package.get(field)
    if field == "command":
        if not (
            isinstance(plan_value, list)
            and all(isinstance(part, str) for part in plan_value)
            and isinstance(execution_value, list)
            and all(isinstance(part, str) for part in execution_value)
        ):
            return []
    elif not isinstance(plan_value, str) or not isinstance(execution_value, str):
        return []
    if execution_value == plan_value:
        return []
    return [
        "native_dynamic report native_build_execution package "
        f"{package_id} {field} {execution_value} does not match "
        f"native_build_plan package {field} {plan_value}"
    ]


def native_dynamic_build_execution_artifact_diagnostics(
    table: object,
    materialized_packages: list[dict[str, object]],
    plugins_dir: Path,
    current_file_manifest: list[dict[str, object]],
) -> list[str]:
    if not isinstance(table, dict):
        return []
    packages = table.get("packages")
    if not isinstance(packages, list):
        return []

    expected_artifacts = materialized_package_loadable_artifact_paths(
        materialized_packages
    )
    expected_native_dirs = materialized_package_native_dir_paths(
        materialized_packages,
        plugins_dir,
    )
    current_file_paths = native_dynamic_file_manifest_paths(current_file_manifest)
    diagnostics: list[str] = []
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        copied_artifact = package.get("copied_loadable_artifact")
        if (
            not isinstance(package_id, str)
            or not package_id.strip()
            or not isinstance(copied_artifact, str)
            or not copied_artifact.strip()
        ):
            continue
        package_expected_artifacts = expected_artifacts.get(package_id)
        if package_expected_artifacts is None:
            continue
        copied_artifact_path = native_dynamic_copied_artifact_bundle_path(
            copied_artifact,
            plugins_dir,
        )
        if copied_artifact_path is None:
            continue
        if copied_artifact_path not in package_expected_artifacts:
            diagnostics.append(
                "native_dynamic report native_build_execution package "
                f"{package_id} copied_loadable_artifact {copied_artifact_path} "
                "does not match materialized loadable artifacts "
                f"{package_expected_artifacts}"
            )
        copied_sidecars = package.get("copied_sidecars")
        if not isinstance(copied_sidecars, list):
            continue
        package_native_dir = expected_native_dirs.get(package_id)
        for sidecar_index, copied_sidecar in enumerate(copied_sidecars):
            if not isinstance(copied_sidecar, str) or not copied_sidecar.strip():
                continue
            copied_sidecar_path = native_dynamic_copied_artifact_bundle_path(
                copied_sidecar,
                plugins_dir,
            )
            if copied_sidecar_path is None:
                continue
            if (
                package_native_dir is not None
                and not copied_sidecar_path.startswith(package_native_dir)
            ):
                diagnostics.append(
                    "native_dynamic report native_build_execution package "
                    f"{package_id} copied_sidecars[{sidecar_index}] "
                    f"{copied_sidecar_path} is not inside materialized "
                    f"native package directory {package_native_dir}"
                )
                continue
            if not native_dynamic_file_manifest_contains_path_or_directory(
                copied_sidecar_path,
                current_file_paths,
            ):
                diagnostics.append(
                    "native_dynamic report native_build_execution package "
                    f"{package_id} copied_sidecars[{sidecar_index}] "
                    f"{copied_sidecar_path} is not present in current "
                    "NativeDynamic plugins file_manifest"
                )
    return diagnostics


def native_dynamic_copied_artifact_bundle_path(
    copied_artifact: str,
    plugins_dir: Path,
) -> str | None:
    copied_path = Path(copied_artifact).expanduser()
    if copied_path.is_absolute():
        try:
            relative_path = copied_path.resolve().relative_to(plugins_dir.resolve())
        except (OSError, ValueError):
            return copied_path.as_posix()
        return f"plugins/{relative_path.as_posix()}"
    return copied_path.as_posix()


def native_dynamic_package_table_diagnostics(
    table: object,
    field: str,
    materialized_package_ids: list[str],
) -> list[str]:
    if not isinstance(table, dict):
        return []
    packages = table.get("packages")
    if not isinstance(packages, list) or not all(
        isinstance(package, dict) and isinstance(package.get("package_id"), str)
        for package in packages
    ):
        return []

    diagnostics: list[str] = []
    package_count = table.get("package_count")
    if type(package_count) is int and package_count != len(packages):
        diagnostics.append(
            f"native_dynamic report {field}.package_count {package_count} "
            f"does not match {field}.packages {len(packages)}"
        )
    if table.get("enabled") is False and package_count == 0 and packages == []:
        return diagnostics
    package_ids = [str(package["package_id"]) for package in packages]
    if package_ids != materialized_package_ids:
        diagnostics.append(
            f"native_dynamic report {field} package ids {package_ids} do not "
            f"match materialized package ids {materialized_package_ids}"
        )
    return diagnostics


def native_dynamic_operation_audit_artifact_diagnostics(
    table: object,
    field: str,
    materialized_package_artifacts: dict[str, list[str]],
) -> list[str]:
    if not isinstance(table, dict):
        return []
    packages = table.get("packages")
    if not isinstance(packages, list):
        return []

    diagnostics: list[str] = []
    for package in packages:
        if not isinstance(package, dict):
            continue
        package_id = package.get("package_id")
        artifacts = package.get("artifacts")
        artifact_count = package.get("artifact_count")
        if not isinstance(package_id, str) or not isinstance(artifacts, list):
            continue
        if not all(
            isinstance(artifact, dict)
            and isinstance(artifact.get("package_relative_artifact"), str)
            for artifact in artifacts
        ):
            continue
        for artifact_index, artifact in enumerate(artifacts):
            artifact_path = artifact.get("artifact")
            package_relative_artifact = artifact.get("package_relative_artifact")
            if not (
                isinstance(artifact_path, str)
                and isinstance(package_relative_artifact, str)
                and package_id.strip()
                and package_relative_artifact.strip()
            ):
                continue
            expected_artifact = (
                f"plugins/{package_id}/{package_relative_artifact}"
            )
            if artifact_path != expected_artifact:
                diagnostics.append(
                    f"native_dynamic report {field} package {package_id} "
                    f"artifacts[{artifact_index}].artifact {artifact_path} "
                    "does not match package_relative_artifact "
                    f"{expected_artifact}"
                )
        package_relative_artifacts = [
            str(artifact["package_relative_artifact"]) for artifact in artifacts
        ]
        if type(artifact_count) is int and artifact_count != len(artifacts):
            diagnostics.append(
                f"native_dynamic report {field} package {package_id} "
                f"artifact_count {artifact_count} does not match artifacts "
                f"{len(artifacts)}"
            )
        expected_artifacts = materialized_package_artifacts.get(package_id)
        if expected_artifacts is None:
            continue
        if package_relative_artifacts != expected_artifacts:
            diagnostics.append(
                f"native_dynamic report {field} package {package_id} "
                f"package_relative_artifacts {package_relative_artifacts} do "
                f"not match materialized loadable artifacts {expected_artifacts}"
            )
    return diagnostics
