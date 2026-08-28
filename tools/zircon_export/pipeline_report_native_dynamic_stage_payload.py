"""NativeDynamic stage payload diagnostics for final reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_dynamic_payload import normalized_materialized_packages
from .native_dynamic_payload_directory import (
    materialized_package_loadable_artifacts_match_manifest,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_plugins_file_manifest,
    normalized_file_manifest,
)
from .pipeline_report_native_dynamic_build_execution import (
    native_dynamic_build_execution_artifact_diagnostics,
    native_dynamic_build_execution_plan_diagnostics,
)
from .pipeline_report_native_dynamic_package_exports import (
    native_dynamic_package_export_materialization_diagnostics,
    schema_clean_native_dynamic_package_exports,
    validate_native_dynamic_package_export_ids,
    validate_native_dynamic_package_exports,
)
from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
)
from .pipeline_report_native_dynamic_payload_file_manifest_schema import (
    native_dynamic_file_manifest_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_materialized_packages_schema import (
    native_dynamic_materialized_packages_schema_diagnostics,
)
from .pipeline_report_native_dynamic_report_hash_schema import (
    native_dynamic_content_hash_is_schema_clean,
)
from .pipeline_report_native_dynamic_stage_loader_manifest import (
    native_dynamic_loader_manifest_package_diagnostics,
)
from .pipeline_report_native_dynamic_stage_package_report import (
    native_dynamic_package_report_diagnostics,
)
from .pipeline_report_native_dynamic_stage_payload_operation_audit import (
    materialized_package_relative_artifacts,
    native_dynamic_operation_audit_artifact_diagnostics,
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
    if native_dynamic_materialized_packages_schema_diagnostics(
        "native_dynamic report",
        report,
    ):
        return []
    if native_dynamic_file_manifest_schema_diagnostics(
        "native_dynamic report",
        report,
    ):
        return []
    if not native_dynamic_content_hash_is_schema_clean(content_hash):
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
    if (
        type(package_count) is int
        and package_count >= 0
        and package_count != len(materialized_packages)
    ):
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
    if native_dynamic_string_array_is_schema_clean(selected_packages):
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
    if type(package_count) is int:
        if package_count < 0:
            return diagnostics
        if package_count != len(packages):
            diagnostics.append(
                f"native_dynamic report {field}.package_count {package_count} "
                f"does not match {field}.packages {len(packages)}"
            )
    if table.get("enabled") is False and package_count == 0 and packages == []:
        return diagnostics
    if not native_dynamic_package_ids_are_schema_clean(packages):
        return diagnostics
    package_ids = [str(package["package_id"]) for package in packages]
    if package_ids != materialized_package_ids:
        diagnostics.append(
            f"native_dynamic report {field} package ids {package_ids} do not "
            f"match materialized package ids {materialized_package_ids}"
        )
    return diagnostics


def native_dynamic_string_array_is_schema_clean(value: object) -> bool:
    return (
        isinstance(value, list)
        and all(
            isinstance(entry, str) and entry.strip() and entry.strip() == entry
            for entry in value
        )
        and len(set(value)) == len(value)
    )


def native_dynamic_package_ids_are_schema_clean(
    packages: list[object],
) -> bool:
    return all(
        isinstance(package, dict)
        and isinstance(package.get("package_id"), str)
        and package["package_id"].strip()
        and package["package_id"].strip() == package["package_id"]
        for package in packages
    )
