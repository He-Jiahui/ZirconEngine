"""CookAssets final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .file_digest import file_sha256
from .pipeline_report_cook_assets_manifest_io import (
    cook_assets_is_non_empty_trimmed_string,
    cook_assets_manifest_json,
    cook_assets_report_manifest_path,
    resolve_cook_assets_path_or_diagnostic,
)
from .pipeline_report_cook_assets_manifest_shape import (
    cook_assets_manifest_assets_are_schema_clean,
    cook_assets_manifest_shape_field_diagnostics,
    cook_assets_manifest_roots_are_schema_clean,
)

COOKED_ASSET_MANIFEST_NAME = "assets.json"


def cook_assets_manifest_hash_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest = report.get("cooked_asset_manifest")
        expected_sha256 = report.get("cooked_asset_manifest_sha256")
        if not cook_assets_is_non_empty_trimmed_string(manifest):
            continue
        if not cook_assets_is_non_empty_trimmed_string(expected_sha256):
            continue
        manifest_path = resolve_cook_assets_path_or_diagnostic(
            manifest,
            diagnostics,
            "cook_assets report cooked_asset_manifest",
        )
        if manifest_path is None:
            continue
        try:
            actual_sha256 = file_sha256(manifest_path)
        except OSError as error:
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest {manifest_path} "
                f"could not be read: {error}"
            )
            continue
        if actual_sha256 != expected_sha256:
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest {manifest_path} "
                f"sha256 {actual_sha256} does not match report "
                f"cooked_asset_manifest_sha256 {expected_sha256}"
            )
    return diagnostics


def cook_assets_manifest_count_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        manifest = cook_assets_manifest_json(manifest_path, diagnostics)
        if not isinstance(manifest, dict):
            continue
        diagnostics.extend(
            cook_assets_manifest_count_field_diagnostics(
                report,
                manifest,
                "asset_count",
                "assets",
            )
        )
        diagnostics.extend(
            cook_assets_manifest_count_field_diagnostics(
                report,
                manifest,
                "root_count",
                "roots",
            )
        )
    return diagnostics


def cook_assets_manifest_stage_location_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        report_path_value = stage_report.get("path")
        if not isinstance(report_path_value, str) or not report_path_value:
            continue
        report_path = resolve_cook_assets_path_or_diagnostic(
            report_path_value,
            diagnostics,
            "cook_assets stage report path",
        )
        if report_path is None:
            continue
        expected_manifest_path = (
            report_path.parent / COOKED_ASSET_MANIFEST_NAME
        ).resolve()
        if manifest_path == expected_manifest_path:
            continue
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            "does not match current CookAssets stage manifest "
            f"{expected_manifest_path}"
        )
    return diagnostics


def cook_assets_manifest_shape_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        manifest = cook_assets_manifest_json(manifest_path, diagnostics)
        if not isinstance(manifest, dict):
            continue
        diagnostics.extend(cook_assets_manifest_shape_field_diagnostics(manifest))
    return diagnostics


def cook_assets_manifest_determinism_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        manifest = cook_assets_manifest_json(manifest_path, diagnostics)
        if not isinstance(manifest, dict):
            continue
        diagnostics.extend(
            cook_assets_manifest_determinism_field_diagnostics(manifest)
        )
    return diagnostics


def cook_assets_manifest_source_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        manifest = cook_assets_manifest_json(manifest_path, diagnostics)
        if not isinstance(manifest, dict):
            continue
        diagnostics.extend(cook_assets_manifest_source_field_diagnostics(manifest))
    return diagnostics


def cook_assets_manifest_source_field_diagnostics(
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return diagnostics
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            continue
        source = asset.get("source")
        if not isinstance(source, str) or not source:
            continue
        if not cook_assets_is_non_empty_trimmed_string(source):
            continue
        source_path = Path(source)
        if not source_path.is_absolute():
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].source must be an absolute path"
            )
            continue
        resolved_source = resolve_cook_assets_path_or_diagnostic(
            source_path,
            diagnostics,
            (
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].source"
            ),
        )
        if resolved_source is None:
            continue
        if resolved_source.is_file():
            continue
        if resolved_source.exists():
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].source is not a file: {resolved_source}"
            )
            continue
        diagnostics.append(
            "cook_assets report cooked_asset_manifest "
            f"assets[{index}].source does not exist: {resolved_source}"
        )
    return diagnostics


def cook_assets_manifest_determinism_field_diagnostics(
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    roots = manifest.get("roots", [])
    if cook_assets_manifest_roots_are_schema_clean(manifest):
        if roots != sorted(set(roots)):
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                "roots must be sorted and unique"
            )
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return diagnostics
    if not cook_assets_manifest_assets_are_schema_clean(manifest):
        return diagnostics
    asset_paths: list[str] = []
    for index, asset in enumerate(assets):
        path = asset.get("path")
        asset_paths.append(path)
        dependencies = asset.get("dependencies", [])
        if dependencies != sorted(set(dependencies)):
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].dependencies must be sorted and unique"
            )
        labels = asset.get("labels", [])
        if labels != sorted(set(labels)):
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].labels must be sorted and unique"
            )
    if asset_paths != sorted(asset_paths):
        diagnostics.append(
            "cook_assets report cooked_asset_manifest assets must be sorted by path"
        )
    return diagnostics


def cook_assets_manifest_asset_filter_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest_path = cook_assets_report_manifest_path(report, diagnostics)
        if manifest_path is None:
            continue
        manifest = cook_assets_manifest_json(manifest_path, diagnostics)
        if not isinstance(manifest, dict):
            continue
        report_asset_filter = report.get("asset_filter")
        manifest_asset_filter = manifest.get("asset_filter")
        if report_asset_filter is not None and not isinstance(
            report_asset_filter,
            str,
        ):
            continue
        if isinstance(report_asset_filter, str) and not cook_assets_is_non_empty_trimmed_string(
            report_asset_filter
        ):
            continue
        if manifest_asset_filter is not None and not isinstance(
            manifest_asset_filter,
            str,
        ):
            continue
        if isinstance(manifest_asset_filter, str) and not cook_assets_is_non_empty_trimmed_string(
            manifest_asset_filter
        ):
            continue
        if report_asset_filter != manifest_asset_filter:
            diagnostics.append(
                "cook_assets report asset_filter "
                f"{report_asset_filter} does not match "
                "cooked_asset_manifest asset_filter "
                f"{manifest_asset_filter}"
            )
    return diagnostics


def cook_assets_manifest_count_field_diagnostics(
    report: dict[str, Any],
    manifest: dict[str, Any],
    report_field: str,
    manifest_field: str,
) -> list[str]:
    expected_count = report.get(report_field)
    manifest_value = manifest.get(manifest_field)
    if not isinstance(expected_count, int) or isinstance(expected_count, bool):
        return []
    if manifest_field == "roots" and not cook_assets_manifest_roots_are_schema_clean(
        manifest
    ):
        return []
    if manifest_field == "assets" and not cook_assets_manifest_assets_are_schema_clean(
        manifest
    ):
        return []
    if not isinstance(manifest_value, list):
        return [
            f"cook_assets report cooked_asset_manifest {manifest_field} must be an array"
        ]
    actual_count = len(manifest_value)
    if expected_count != actual_count:
        return [
            f"cook_assets report {report_field} {expected_count} does not match "
            f"cooked_asset_manifest {manifest_field} length {actual_count}"
        ]
    return []
