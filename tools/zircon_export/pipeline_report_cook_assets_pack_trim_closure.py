"""CookAssets Pack trim-closure diagnostics for final pipeline reports."""

from __future__ import annotations

from typing import Any

from .pipeline_report_cook_assets_manifest_io import (
    cook_assets_manifest_json,
    cook_assets_manifest_path,
)
from .pipeline_report_cook_assets_source_bytes import (
    cook_assets_pack_source_byte_diagnostics,
)
from .pipeline_report_cook_assets_trim_evidence import (
    cook_assets_manifest_trim_evidence,
)


def cook_assets_pack_trim_closure_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    cook_assets_manifest = cook_assets_manifest_path(stage_reports, diagnostics)
    if cook_assets_manifest is None:
        return diagnostics
    manifest = cook_assets_manifest_json(cook_assets_manifest, diagnostics)
    if not isinstance(manifest, dict):
        return diagnostics
    expected_trim_evidence = cook_assets_manifest_trim_evidence(manifest)
    if expected_trim_evidence is None:
        return diagnostics
    manifest_assets_by_path = cook_assets_manifest_assets_by_path(manifest)
    expected_included_assets = expected_trim_evidence["included_assets"]
    expected_trimmed_assets = expected_trim_evidence["trimmed_assets"]
    expected_missing_dependencies = expected_trim_evidence["missing_dependencies"]
    expected_duplicate_assets = expected_trim_evidence["duplicate_assets"]
    expected_diagnostics = expected_trim_evidence["diagnostics"]

    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        trim_report = report.get("trim_report")
        if not isinstance(trim_report, dict):
            continue
        included_assets = trim_report.get("included_assets")
        if not isinstance(included_assets, list):
            continue
        if any(not isinstance(asset, str) for asset in included_assets):
            continue
        diagnostics.extend(
            cook_assets_pack_included_source_diagnostics(
                included_assets,
                manifest_assets_by_path,
            )
        )
        pack_manifest = report.get("manifest")
        if isinstance(pack_manifest, dict):
            diagnostics.extend(
                cook_assets_pack_source_byte_diagnostics(
                    included_assets,
                    manifest_assets_by_path,
                    pack_manifest,
                )
            )
        if sorted(included_assets) != expected_included_assets:
            diagnostics.append(
                "pack report trim_report.included_assets does not match "
                "CookAssets dependency closure"
            )
        trimmed_assets = trim_report.get("trimmed_assets")
        if not isinstance(trimmed_assets, list):
            continue
        actual_trimmed_assets = normalized_pack_trimmed_assets(trimmed_assets)
        if actual_trimmed_assets is None:
            continue
        if actual_trimmed_assets != expected_trimmed_assets:
            diagnostics.append(
                "pack report trim_report.trimmed_assets does not match "
                "CookAssets dependency closure"
            )
        missing_dependencies = trim_report.get("missing_dependencies")
        if not isinstance(missing_dependencies, list):
            continue
        actual_missing_dependencies = normalized_pack_missing_dependencies(
            missing_dependencies
        )
        if actual_missing_dependencies is None:
            continue
        if actual_missing_dependencies != expected_missing_dependencies:
            diagnostics.append(
                "pack report trim_report.missing_dependencies does not match "
                "CookAssets dependency closure"
            )
        duplicate_assets = trim_report.get("duplicate_assets")
        if not isinstance(duplicate_assets, list):
            continue
        if any(not isinstance(asset, str) for asset in duplicate_assets):
            continue
        if sorted(set(duplicate_assets)) != expected_duplicate_assets:
            diagnostics.append(
                "pack report trim_report.duplicate_assets does not match "
                "CookAssets dependency closure"
            )
        trim_diagnostics = trim_report.get("diagnostics")
        if not isinstance(trim_diagnostics, list):
            continue
        if any(not isinstance(diagnostic, str) for diagnostic in trim_diagnostics):
            continue
        if sorted(trim_diagnostics) != expected_diagnostics:
            diagnostics.append(
                "pack report trim_report.diagnostics does not match "
                "CookAssets dependency closure"
            )
    return diagnostics


def cook_assets_manifest_assets_by_path(
    manifest: dict[str, Any],
) -> dict[str, dict[str, Any]]:
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return {}
    assets_by_path: dict[str, dict[str, Any]] = {}
    for asset in assets:
        if not isinstance(asset, dict):
            continue
        path = asset.get("path")
        if not isinstance(path, str) or not path:
            continue
        if path not in assets_by_path:
            assets_by_path[path] = asset
    return assets_by_path


def cook_assets_pack_included_source_diagnostics(
    included_assets: list[str],
    manifest_assets_by_path: dict[str, dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for path in included_assets:
        asset = manifest_assets_by_path.get(path)
        if asset is None:
            continue
        source = asset.get("source")
        if isinstance(source, str) and source:
            continue
        diagnostics.append(
            "pack report trim_report.included_assets contains "
            f"{path} but CookAssets manifest asset is missing source"
        )
    return diagnostics


def normalized_pack_trimmed_assets(
    trimmed_assets: list[Any],
) -> list[dict[str, Any]] | None:
    normalized: list[dict[str, Any]] = []
    for trimmed_asset in trimmed_assets:
        if not isinstance(trimmed_asset, dict):
            return None
        path = trimmed_asset.get("path")
        reason = trimmed_asset.get("reason")
        if not isinstance(path, str) or not path:
            return None
        normalized_reason = normalized_pack_trim_reason(reason)
        if normalized_reason is None:
            return None
        normalized.append({"path": path, "reason": normalized_reason})
    return sorted(normalized, key=lambda asset: str(asset["path"]))


def normalized_pack_missing_dependencies(
    missing_dependencies: list[Any],
) -> list[dict[str, str]] | None:
    normalized: list[dict[str, str]] = []
    for missing_dependency in missing_dependencies:
        if not isinstance(missing_dependency, dict):
            return None
        owner = missing_dependency.get("owner")
        dependency = missing_dependency.get("dependency")
        if not isinstance(owner, str) or not isinstance(dependency, str):
            return None
        normalized.append(
            {
                "owner": owner,
                "dependency": dependency,
            }
        )
    return sorted(
        normalized,
        key=lambda dependency: (
            dependency["owner"],
            dependency["dependency"],
        ),
    )


def normalized_pack_trim_reason(value: object) -> str | dict[str, str] | None:
    if isinstance(value, str):
        return value
    if not isinstance(value, dict) or len(value) != 1:
        return None
    if "AssetFilterMismatch" in value and isinstance(
        value.get("AssetFilterMismatch"),
        str,
    ):
        return {"AssetFilterMismatch": value["AssetFilterMismatch"]}
    if "UnreferencedAndAssetFilterMismatch" in value and isinstance(
        value.get("UnreferencedAndAssetFilterMismatch"),
        str,
    ):
        return {
            "UnreferencedAndAssetFilterMismatch": value[
                "UnreferencedAndAssetFilterMismatch"
            ],
        }
    return None
