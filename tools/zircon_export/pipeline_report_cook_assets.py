"""CookAssets final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .pipeline_report_cook_assets_manifest_shape import (
    cook_assets_manifest_assets_are_schema_clean,
    cook_assets_manifest_shape_field_diagnostics,
    cook_assets_manifest_roots_are_schema_clean,
    safe_normalized_manifest_path,
)
from .pipeline_report_cook_assets_source_bytes import (
    cook_assets_pack_source_byte_diagnostics,
)

COOKED_ASSET_MANIFEST_NAME = "assets.json"


def resolve_cook_assets_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return Path(path).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


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
            actual_sha256 = hashlib.sha256(manifest_path.read_bytes()).hexdigest()
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


def cook_assets_pack_manifest_handoff_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    cook_assets_manifest = cook_assets_manifest_path(stage_reports, diagnostics)
    if cook_assets_manifest is None:
        return diagnostics
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        pack_manifest = report.get("asset_manifest")
        if not isinstance(pack_manifest, str) or not pack_manifest:
            continue
        pack_manifest_path = resolve_cook_assets_path_or_diagnostic(
            pack_manifest,
            diagnostics,
            "pack report asset_manifest",
        )
        if pack_manifest_path is None:
            continue
        if pack_manifest_path != cook_assets_manifest:
            diagnostics.append(
                f"pack report asset_manifest {pack_manifest_path} does not match "
                f"cook_assets report cooked_asset_manifest {cook_assets_manifest}"
            )
    return diagnostics


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


def cook_assets_manifest_trim_evidence(
    manifest: dict[str, Any],
) -> dict[str, list[Any]] | None:
    roots = manifest.get("roots")
    assets = manifest.get("assets")
    asset_filter = manifest.get("asset_filter")
    if not isinstance(roots, list) or any(not isinstance(root, str) for root in roots):
        return None
    if any(not cook_assets_is_non_empty_trimmed_string(root) for root in roots):
        return None
    if any(not cook_assets_manifest_asset_path_is_schema_clean(root) for root in roots):
        return None
    if not isinstance(assets, list):
        return None
    if asset_filter is not None and not isinstance(asset_filter, str):
        return None
    if isinstance(asset_filter, str) and not cook_assets_is_non_empty_trimmed_string(
        asset_filter
    ):
        return None

    asset_map: dict[str, dict[str, Any]] = {}
    duplicate_assets: list[str] = []
    for asset in assets:
        if not isinstance(asset, dict):
            return None
        path = asset.get("path")
        dependencies = asset.get("dependencies", [])
        labels = asset.get("labels", [])
        source = asset.get("source")
        if not cook_assets_is_non_empty_trimmed_string(path):
            return None
        if not cook_assets_manifest_asset_path_is_schema_clean(path):
            return None
        if path in asset_map:
            duplicate_assets.append(path)
            continue
        if not isinstance(dependencies, list) or any(
            not isinstance(dependency, str) for dependency in dependencies
        ):
            return None
        if any(
            not cook_assets_is_non_empty_trimmed_string(dependency)
            for dependency in dependencies
        ):
            return None
        if any(
            not cook_assets_manifest_asset_path_is_schema_clean(dependency)
            for dependency in dependencies
        ):
            return None
        if not isinstance(labels, list) or any(
            not isinstance(label, str) for label in labels
        ):
            return None
        if any(not cook_assets_is_non_empty_trimmed_string(label) for label in labels):
            return None
        if source is not None and not cook_assets_is_non_empty_trimmed_string(source):
            return None
        asset_map[path] = asset

    reachable_assets, missing_dependencies = cook_assets_manifest_reachable_assets(
        roots,
        asset_map,
    )
    included_assets: list[str] = []
    trimmed_assets: list[dict[str, Any]] = []
    diagnostics = [
        f"asset {path} is duplicated in trim input"
        for path in sorted(set(duplicate_assets))
    ]
    diagnostics.extend(
        f"root asset {dependency['dependency']} is missing"
        for dependency in missing_dependencies
        if dependency["owner"] == "<root>"
    )
    diagnostics.extend(
        f"asset {dependency['owner']} references missing dependency "
        f"{dependency['dependency']}"
        for dependency in missing_dependencies
        if dependency["owner"] != "<root>"
    )
    for path, asset in sorted(asset_map.items()):
        is_reachable = path in reachable_assets
        matches_filter = cook_assets_manifest_asset_matches_filter(asset, asset_filter)
        if is_reachable and matches_filter:
            included_assets.append(path)
            continue
        reason = cook_assets_manifest_trim_reason(
            is_reachable,
            matches_filter,
            asset_filter,
        )
        diagnostics.append(
            f"trimmed asset {path}: {cook_assets_manifest_trim_reason_label(reason)}"
        )
        trimmed_assets.append(
            {
                "path": path,
                "reason": reason,
            }
        )
    return {
        "diagnostics": sorted(diagnostics),
        "duplicate_assets": sorted(set(duplicate_assets)),
        "included_assets": included_assets,
        "missing_dependencies": missing_dependencies,
        "trimmed_assets": trimmed_assets,
    }


def cook_assets_manifest_asset_path_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and safe_normalized_manifest_path(value) == value


def cook_assets_manifest_reachable_assets(
    roots: list[Any],
    asset_map: dict[str, dict[str, Any]],
) -> tuple[set[str], list[dict[str, str]]]:
    reachable_assets: set[str] = set()
    missing_dependencies: list[dict[str, str]] = []
    queue: list[str] = []
    for root in roots:
        if root in asset_map and root not in reachable_assets:
            reachable_assets.add(root)
            queue.append(root)
            continue
        if root not in asset_map:
            missing_dependencies.append(
                {
                    "owner": "<root>",
                    "dependency": root,
                }
            )

    index = 0
    while index < len(queue):
        path = queue[index]
        index += 1
        asset = asset_map.get(path)
        if asset is None:
            continue
        dependencies = asset.get("dependencies", [])
        if not isinstance(dependencies, list):
            continue
        for dependency in dependencies:
            if dependency in asset_map and dependency not in reachable_assets:
                reachable_assets.add(dependency)
                queue.append(dependency)
                continue
            if dependency not in asset_map:
                missing_dependencies.append(
                    {
                        "owner": path,
                        "dependency": dependency,
                    }
                )
    return reachable_assets, sorted(
        missing_dependencies,
        key=lambda dependency: (
            dependency["owner"],
            dependency["dependency"],
        ),
    )


def cook_assets_manifest_asset_matches_filter(
    asset: dict[str, Any],
    asset_filter: object,
) -> bool:
    if asset_filter is None:
        return True
    labels = asset.get("labels", [])
    return isinstance(labels, list) and asset_filter in labels


def cook_assets_manifest_trim_reason(
    is_reachable: bool,
    matches_filter: bool,
    asset_filter: object,
) -> str | dict[str, str]:
    if (
        not is_reachable
        and not matches_filter
        and isinstance(asset_filter, str)
    ):
        return {"UnreferencedAndAssetFilterMismatch": asset_filter}
    if is_reachable and not matches_filter and isinstance(asset_filter, str):
        return {"AssetFilterMismatch": asset_filter}
    return "Unreferenced"


def cook_assets_manifest_trim_reason_label(reason: str | dict[str, str]) -> str:
    if reason == "Unreferenced":
        return "unreferenced"
    if isinstance(reason, dict) and "AssetFilterMismatch" in reason:
        return f"asset_filter {reason['AssetFilterMismatch']} did not match"
    if isinstance(reason, dict) and "UnreferencedAndAssetFilterMismatch" in reason:
        return (
            "unreferenced; asset_filter "
            f"{reason['UnreferencedAndAssetFilterMismatch']} did not match"
        )
    return str(reason)


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


def cook_assets_manifest_path(
    stage_reports: list[dict[str, Any]],
    diagnostics: list[str],
) -> Path | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest = report.get("cooked_asset_manifest")
        if not cook_assets_is_non_empty_trimmed_string(manifest):
            return None
        return resolve_cook_assets_path_or_diagnostic(
            manifest,
            diagnostics,
            "cook_assets report cooked_asset_manifest",
        )
    return None


def cook_assets_report_manifest_path(
    report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    manifest = report.get("cooked_asset_manifest")
    if not cook_assets_is_non_empty_trimmed_string(manifest):
        return None
    return resolve_cook_assets_path_or_diagnostic(
        manifest,
        diagnostics,
        "cook_assets report cooked_asset_manifest",
    )


def cook_assets_manifest_json(
    manifest_path: Path,
    diagnostics: list[str],
) -> object:
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            f"could not be read: {error}"
        )
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            f"is not valid JSON: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            "must be a JSON object"
        )
    return manifest


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


def cook_assets_is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
