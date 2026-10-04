"""CookAssets manifest trim evidence reconstruction for final reports."""

from __future__ import annotations

from typing import Any

from .pipeline_report_cook_assets_manifest_io import (
    cook_assets_is_non_empty_trimmed_string,
)
from .pipeline_report_cook_assets_manifest_shape import safe_normalized_manifest_path


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
    ordered_duplicate_assets = sorted(set(duplicate_assets))
    diagnostics = [
        f"asset {path} is duplicated in trim input"
        for path in ordered_duplicate_assets
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
        "duplicate_assets": ordered_duplicate_assets,
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
