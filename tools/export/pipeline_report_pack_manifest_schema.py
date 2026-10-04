"""Pack document manifest schema and semantic diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_manifest_schema_helpers import (
    pack_asset_chunk_reference_diagnostics,
    pack_asset_chunk_size_diagnostics,
    pack_asset_entries_schema_diagnostics,
    pack_asset_entry_is_schema_clean,
    pack_chunk_entry_is_schema_clean,
    pack_manifest_is_schema_clean,
    pack_manifest_schema_diagnostics,
)
from .pipeline_report_pack_manifest_path_hash_schema_helpers import (
    pack_asset_path_is_schema_clean,
    pack_asset_path_order_diagnostics,
    pack_asset_path_uniqueness_diagnostics,
)

PACK_DOCUMENT_MANIFEST_FIELDS = ("assets", "pack")
PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_FIELDS = ("pack",)
PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS = ("assets",)

SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_report_manifest_count_diagnostics(
    report: dict[str, Any],
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    assets = manifest.get("assets")
    asset_count = report.get("asset_count")
    if (
        isinstance(assets, list)
        and all(pack_asset_entry_is_schema_clean(asset) for asset in assets)
        and isinstance(asset_count, int)
        and not isinstance(asset_count, bool)
        and asset_count != len(assets)
    ):
        diagnostics.append(
            f"pack report asset_count {asset_count} does not match "
            f"manifest.assets length {len(assets)}"
        )
    pack = manifest.get("pack")
    if not isinstance(pack, dict):
        return diagnostics
    chunks = pack.get("chunks")
    chunk_count = report.get("chunk_count")
    if (
        isinstance(chunks, list)
        and all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks)
        and isinstance(chunk_count, int)
        and not isinstance(chunk_count, bool)
        and chunk_count != len(chunks)
    ):
        diagnostics.append(
            f"pack report chunk_count {chunk_count} does not match "
            f"manifest.pack.chunks length {len(chunks)}"
        )
    return diagnostics

def pack_report_deduplicated_assets_diagnostics(
    report: dict[str, Any],
    manifest: dict[str, Any],
) -> list[str]:
    deduplicated_assets = report.get("deduplicated_assets")
    if (
        not isinstance(deduplicated_assets, list)
        or any(not isinstance(path, str) for path in deduplicated_assets)
        or any(
            not pack_asset_path_is_schema_clean(path)
            for path in deduplicated_assets
            if isinstance(path, str)
        )
    ):
        return []
    expected = manifest_deduplicated_asset_paths(manifest)
    if expected is None or sorted(deduplicated_assets) == expected:
        return []
    return [
        "pack report deduplicated_assets does not match "
        "manifest duplicate chunk paths"
    ]


def manifest_deduplicated_asset_paths(manifest: dict[str, Any]) -> list[str] | None:
    if not pack_document_manifest_is_schema_clean(manifest):
        return None
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return None
    seen_hashes: set[tuple[int, ...]] = set()
    deduplicated_paths: list[str] = []
    for asset in sorted(assets, key=manifest_asset_sort_key):
        if not pack_asset_entry_is_schema_clean(asset):
            return None
        path = asset["path"]
        chunk_hash = asset["chunk_hash"]
        hash_key = tuple(chunk_hash)
        if hash_key in seen_hashes:
            deduplicated_paths.append(path)
            continue
        seen_hashes.add(hash_key)
    return deduplicated_paths


def manifest_asset_sort_key(asset: Any) -> str:
    if isinstance(asset, dict) and isinstance(asset.get("path"), str):
        return asset["path"]
    return ""


def pack_document_manifest_is_schema_clean(manifest: dict[str, Any]) -> bool:
    if any(field not in PACK_DOCUMENT_MANIFEST_FIELDS for field in manifest):
        return False
    pack = manifest.get("pack")
    assets = manifest.get("assets")
    return (
        isinstance(pack, dict)
        and pack_manifest_is_schema_clean(pack)
        and isinstance(assets, list)
        and all(pack_asset_entry_is_schema_clean(asset) for asset in assets)
    )


def pack_document_manifest_schema_diagnostics(
    label: str,
    manifest: dict[str, Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(manifest)
        if field not in PACK_DOCUMENT_MANIFEST_FIELDS
    )
    pack = manifest.get("pack")
    chunks = pack.get("chunks") if isinstance(pack, dict) else None
    chunks_are_schema_clean = (
        all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks)
        if isinstance(chunks, list)
        else None
    )
    for field in PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_FIELDS:
        diagnostics.extend(
            validate_object_schema_diagnostics(f"{label}.{field}", manifest.get(field))
        )
    if isinstance(pack, dict):
        diagnostics.extend(
            pack_manifest_schema_diagnostics(
                f"{label}.pack",
                pack,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
                chunks_are_schema_clean=chunks_are_schema_clean,
            )
        )
    assets = manifest.get("assets")
    assets_are_schema_clean = (
        all(pack_asset_entry_is_schema_clean(asset) for asset in assets)
        if isinstance(assets, list)
        else None
    )
    for field in PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.{field}",
                manifest.get(field),
            )
        )
    if isinstance(assets, list):
        diagnostics.extend(
            pack_asset_entries_schema_diagnostics(
                f"{label}.assets",
                assets,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
            )
        )
        diagnostics.extend(
            pack_asset_path_uniqueness_diagnostics(f"{label}.assets", assets)
        )
        diagnostics.extend(pack_asset_path_order_diagnostics(f"{label}.assets", assets))
        if isinstance(pack, dict):
            diagnostics.extend(
                pack_asset_chunk_reference_diagnostics(
                    label,
                    pack,
                    assets,
                    chunks_are_schema_clean=chunks_are_schema_clean,
                    assets_are_schema_clean=assets_are_schema_clean,
                )
            )
            if isinstance(chunks, list):
                diagnostics.extend(
                    pack_asset_chunk_size_diagnostics(
                        f"{label}.assets",
                        f"{label}.pack.chunks",
                        chunks,
                        assets,
                        chunks_are_schema_clean=chunks_are_schema_clean,
                        assets_are_schema_clean=assets_are_schema_clean,
                    )
                )
    return diagnostics
