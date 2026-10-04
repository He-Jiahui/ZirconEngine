"""Pack delta base/target asset projection helpers."""

from __future__ import annotations

from typing import Any

from .pipeline_report_pack_manifest_schema import pack_document_manifest_is_schema_clean
from .pipeline_report_pack_manifest_schema_helpers import pack_chunk_entry_is_schema_clean
from .pipeline_report_pack_manifest_path_hash_schema_helpers import is_byte_hash


DeltaAssetProjection = tuple[
    list[dict[str, Any]],
    list[dict[str, Any]],
    set[tuple[int, ...]],
]


def delta_removed_asset_paths_from_assets(
    base_assets: list[dict[str, Any]],
    target_assets: list[dict[str, Any]],
) -> list[str]:
    target_paths = {asset["path"] for asset in target_assets}
    return sorted(
        asset["path"] for asset in base_assets if asset["path"] not in target_paths
    )


def delta_changed_and_reused_asset_paths_from_projection(
    target_assets: list[dict[str, Any]],
    base_hashes: set[tuple[int, ...]],
) -> tuple[list[str], list[str], list[dict[str, Any]]]:
    changed_assets: list[str] = []
    changed_entries: list[dict[str, Any]] = []
    reused_assets: list[str] = []
    for asset in sorted(target_assets, key=lambda entry: entry["path"]):
        if tuple(asset["chunk_hash"]) in base_hashes:
            reused_assets.append(asset["path"])
        else:
            changed_assets.append(asset["path"])
            changed_entries.append(asset)
    return changed_assets, reused_assets, changed_entries


def delta_changed_asset_chunk_hashes_match_entries(
    delta_manifest: dict[str, Any],
    parsed_changed_entries: list[dict[str, Any]],
) -> bool:
    chunks = delta_manifest.get("chunks")
    if not isinstance(chunks, list):
        return True
    if not all(
        isinstance(chunk, dict) and pack_chunk_entry_is_schema_clean(chunk)
        for chunk in chunks
    ):
        return True
    chunk_hashes: set[tuple[int, ...]] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict) or not is_byte_hash(chunk.get("hash")):
            return True
        chunk_hashes.add(tuple(chunk["hash"]))
    changed_hashes = {tuple(asset["chunk_hash"]) for asset in parsed_changed_entries}
    return chunk_hashes == changed_hashes


def delta_manifest_asset_projection(
    delta_manifest: dict[str, Any],
) -> DeltaAssetProjection | None:
    base = delta_manifest.get("base")
    target = delta_manifest.get("target")
    if not isinstance(base, dict) or not isinstance(target, dict):
        return None
    if not pack_document_manifest_is_schema_clean(base):
        return None
    if not pack_document_manifest_is_schema_clean(target):
        return None
    base_assets = base.get("assets")
    target_assets = target.get("assets")
    base_pack = base.get("pack")
    if (
        not isinstance(base_assets, list)
        or not isinstance(target_assets, list)
        or not isinstance(base_pack, dict)
    ):
        return None
    base_chunks = base_pack.get("chunks")
    if not isinstance(base_chunks, list):
        return None
    return (
        base_assets,
        target_assets,
        {tuple(chunk["hash"]) for chunk in base_chunks},
    )
