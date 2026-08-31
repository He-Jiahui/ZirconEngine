"""Pack delta asset-set semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_pack_manifest_schema import (
    pack_document_manifest_is_schema_clean,
)
from .pipeline_report_pack_manifest_schema_helpers import (
    pack_asset_entry_is_schema_clean,
    pack_chunk_entry_is_schema_clean,
)
from .pipeline_report_pack_manifest_path_hash_schema_helpers import (
    is_byte_hash,
    pack_asset_path_is_schema_clean,
)

PackChunkFingerprint = tuple[tuple[int, ...], int, int]
PackPlanFingerprint = tuple[int, tuple[PackChunkFingerprint, ...], int]
PackAssetFingerprint = tuple[str, tuple[int, ...], int]
PackDocumentFingerprint = tuple[
    PackPlanFingerprint,
    tuple[PackAssetFingerprint, ...],
]
DeltaAssetProjection = tuple[
    list[dict[str, Any]],
    list[dict[str, Any]],
    set[tuple[int, ...]],
]


def pack_report_delta_asset_set_diagnostics(
    report: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    projection = delta_manifest_asset_projection(delta_manifest)
    if projection is None:
        return diagnostics
    base_assets, target_assets, base_hashes = projection
    expected_removed_assets = delta_removed_asset_paths_from_assets(
        base_assets,
        target_assets,
    )
    removed_assets = delta_manifest.get("removed_assets")
    if (
        isinstance(removed_assets, list)
        and asset_path_list_is_schema_clean(removed_assets)
        and sorted(removed_assets) != expected_removed_assets
    ):
        diagnostics.append(
            "pack report delta_manifest.removed_assets does not match "
            "base/target asset path difference"
        )
    report_removed_assets = report.get("delta_removed_assets")
    if (
        isinstance(report_removed_assets, list)
        and asset_path_list_is_schema_clean(report_removed_assets)
        and sorted(report_removed_assets) != expected_removed_assets
    ):
        diagnostics.append(
            "pack report delta_removed_assets does not match "
            "delta_manifest.removed_assets"
        )

    expected_changed_assets, expected_reused_assets, expected_changed_entries = (
        delta_changed_and_reused_asset_paths_from_projection(
            target_assets,
            base_hashes,
        )
    )
    changed_assets = delta_manifest.get("changed_assets")
    parsed_changed_entries = (
        delta_changed_asset_entries(changed_assets)
        if isinstance(changed_assets, list)
        else None
    )
    if parsed_changed_entries is not None:
        sorted_changed_entries = sorted(
            parsed_changed_entries,
            key=lambda entry: entry["path"],
        )
        changed_asset_paths = [entry["path"] for entry in sorted_changed_entries]
        if changed_asset_paths != expected_changed_assets:
            diagnostics.append(
                "pack report delta_manifest.changed_assets does not match "
                "target assets missing from base chunks"
            )
        elif sorted_changed_entries != expected_changed_entries:
            diagnostics.append(
                "pack report delta_manifest.changed_assets does not match "
                "target manifest asset entries"
            )
        if not delta_changed_asset_chunk_hashes_match_entries(
            delta_manifest,
            parsed_changed_entries,
        ):
            diagnostics.append(
                "pack report delta_manifest.chunks does not match "
                "changed asset chunk hashes"
            )
    report_reused_assets = report.get("delta_reused_assets")
    if (
        isinstance(report_reused_assets, list)
        and asset_path_list_is_schema_clean(report_reused_assets)
        and sorted(report_reused_assets) != expected_reused_assets
    ):
        diagnostics.append(
            "pack report delta_reused_assets does not match "
            "delta_manifest target assets reused from base chunks"
        )
    return diagnostics


def asset_path_list_is_schema_clean(value: list[Any]) -> bool:
    return all(
        isinstance(asset, str) and pack_asset_path_is_schema_clean(asset)
        for asset in value
    )


def pack_document_manifest_fingerprint(
    manifest: dict[str, Any],
) -> PackDocumentFingerprint | None:
    if not pack_document_manifest_is_schema_clean(manifest):
        return None
    pack = manifest.get("pack")
    assets = manifest.get("assets")
    if not isinstance(pack, dict) or not isinstance(assets, list):
        return None
    version = pack.get("version")
    total_size = pack.get("total_size")
    chunks = pack.get("chunks")
    if (
        not isinstance(version, int)
        or isinstance(version, bool)
        or not isinstance(total_size, int)
        or isinstance(total_size, bool)
        or not isinstance(chunks, list)
    ):
        return None
    parsed_chunks: list[PackChunkFingerprint] = []
    for chunk in chunks:
        if not isinstance(chunk, dict):
            return None
        chunk_hash = chunk.get("hash")
        offset = chunk.get("offset")
        size = chunk.get("size")
        if (
            not is_byte_hash(chunk_hash)
            or not isinstance(offset, int)
            or isinstance(offset, bool)
            or not isinstance(size, int)
            or isinstance(size, bool)
        ):
            return None
        parsed_chunks.append((tuple(chunk_hash), offset, size))
    parsed_assets: list[PackAssetFingerprint] = []
    for asset in assets:
        if not isinstance(asset, dict):
            return None
        path = asset.get("path")
        chunk_hash = asset.get("chunk_hash")
        size = asset.get("size")
        if (
            not isinstance(path, str)
            or not pack_asset_path_is_schema_clean(path)
            or not is_byte_hash(chunk_hash)
            or not isinstance(size, int)
            or isinstance(size, bool)
        ):
            return None
        parsed_assets.append((path, tuple(chunk_hash), size))
    return (
        (version, tuple(sorted(parsed_chunks)), total_size),
        tuple(sorted(parsed_assets)),
    )


def delta_changed_assets_are_schema_clean(changed_assets: list[Any]) -> bool:
    return all(
        isinstance(asset, dict) and pack_asset_entry_is_schema_clean(asset)
        for asset in changed_assets
    )


def delta_chunks_are_schema_clean(chunks: list[Any]) -> bool:
    return all(
        isinstance(chunk, dict) and pack_chunk_entry_is_schema_clean(chunk)
        for chunk in chunks
    )


def delta_removed_asset_paths(delta_manifest: dict[str, Any]) -> list[str] | None:
    projection = delta_manifest_asset_projection(delta_manifest)
    if projection is None:
        return None
    base_assets, target_assets, _ = projection
    return delta_removed_asset_paths_from_assets(base_assets, target_assets)


def delta_removed_asset_paths_from_assets(
    base_assets: list[dict[str, Any]],
    target_assets: list[dict[str, Any]],
) -> list[str]:
    target_paths = {asset["path"] for asset in target_assets}
    return sorted(
        asset["path"] for asset in base_assets if asset["path"] not in target_paths
    )


def delta_changed_and_reused_asset_paths(
    delta_manifest: dict[str, Any],
) -> tuple[list[str], list[str], list[dict[str, Any]]] | None:
    projection = delta_manifest_asset_projection(delta_manifest)
    if projection is None:
        return None
    _, target_assets, base_hashes = projection
    return delta_changed_and_reused_asset_paths_from_projection(
        target_assets,
        base_hashes,
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


def delta_changed_asset_entries_match(
    changed_assets: list[Any],
    expected_changed_entries: list[dict[str, Any]],
) -> bool:
    parsed_changed_entries = delta_changed_asset_entries(changed_assets)
    if parsed_changed_entries is None:
        return True
    return sorted(parsed_changed_entries, key=lambda entry: entry["path"]) == sorted(
        expected_changed_entries,
        key=lambda entry: entry["path"],
    )


def delta_changed_asset_entries(
    changed_assets: list[Any],
) -> list[dict[str, Any]] | None:
    parsed_assets: list[dict[str, Any]] = []
    for asset in changed_assets:
        if not pack_asset_entry_is_schema_clean(asset):
            return None
        parsed_assets.append(asset)
    return parsed_assets


def delta_changed_asset_chunk_hashes_match(
    delta_manifest: dict[str, Any],
    changed_assets: list[Any],
) -> bool:
    parsed_changed_entries = delta_changed_asset_entries(changed_assets)
    if parsed_changed_entries is None:
        return True
    return delta_changed_asset_chunk_hashes_match_entries(
        delta_manifest,
        parsed_changed_entries,
    )


def delta_changed_asset_chunk_hashes_match_entries(
    delta_manifest: dict[str, Any],
    parsed_changed_entries: list[dict[str, Any]],
) -> bool:
    chunks = delta_manifest.get("chunks")
    if not isinstance(chunks, list):
        return True
    if not delta_chunks_are_schema_clean(chunks):
        return True
    chunk_hashes: set[tuple[int, ...]] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict) or not is_byte_hash(chunk.get("hash")):
            return True
        chunk_hashes.add(tuple(chunk["hash"]))
    changed_hashes = {tuple(asset["chunk_hash"]) for asset in parsed_changed_entries}
    return chunk_hashes == changed_hashes


def delta_manifest_base_chunk_hashes(
    delta_manifest: dict[str, Any],
) -> set[tuple[int, ...]] | None:
    base = delta_manifest.get("base")
    if not isinstance(base, dict):
        return None
    if not pack_document_manifest_is_schema_clean(base):
        return None
    pack = base.get("pack")
    if not isinstance(pack, dict):
        return None
    chunks = pack.get("chunks")
    if not isinstance(chunks, list):
        return None
    hashes: set[tuple[int, ...]] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict) or not is_byte_hash(chunk.get("hash")):
            return None
        hashes.add(tuple(chunk["hash"]))
    return hashes


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


def delta_manifest_assets(
    delta_manifest: dict[str, Any],
    field: str,
) -> list[dict[str, Any]] | None:
    manifest = delta_manifest.get(field)
    if not isinstance(manifest, dict):
        return None
    if not pack_document_manifest_is_schema_clean(manifest):
        return None
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return None
    parsed_assets: list[dict[str, Any]] = []
    for asset in assets:
        if not pack_asset_entry_is_schema_clean(asset):
            return None
        parsed_assets.append(asset)
    return parsed_assets


def manifest_asset_paths(assets: list[Any]) -> list[str] | None:
    paths: list[str] = []
    for asset in assets:
        if not isinstance(asset, dict) or not isinstance(asset.get("path"), str):
            return None
        if not pack_asset_path_is_schema_clean(asset["path"]):
            return None
        paths.append(asset["path"])
    return sorted(paths)
