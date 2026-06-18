"""CookAssets source-byte evidence diagnostics for Pack reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

ZRPACK_HASH_SEEDS = (
    0xCBF2_9CE4_8422_2325,
    0x9AE1_6A3B_2F90_404F,
    0x6EED_0E9D_A4D9_4A4F,
    0xACE5_929A_D4D9_8F13,
)
FNV1A64_PRIME = 0x100_0000_01B3
U64_MASK = (1 << 64) - 1


def cook_assets_pack_source_byte_diagnostics(
    included_assets: list[str],
    cook_assets_by_path: dict[str, dict[str, Any]],
    pack_manifest: dict[str, Any],
) -> list[str]:
    assets = pack_manifest.get("assets")
    if not isinstance(assets, list):
        return []
    pack_assets_by_path = pack_manifest_assets_by_path(assets)
    diagnostics: list[str] = []
    for path in included_assets:
        cook_asset = cook_assets_by_path.get(path)
        pack_asset = pack_assets_by_path.get(path)
        if cook_asset is None or pack_asset is None:
            continue
        source = cook_asset.get("source")
        if not isinstance(source, str) or not source:
            continue
        try:
            source_bytes = Path(source).read_bytes()
        except OSError as error:
            diagnostics.append(
                f"CookAssets source {source} for included asset {path} "
                f"could not be read: {error}"
            )
            continue
        diagnostics.extend(
            pack_asset_source_byte_field_diagnostics(
                path,
                pack_asset["index"],
                pack_asset["asset"],
                source_bytes,
            )
        )
    return diagnostics


def pack_manifest_assets_by_path(
    assets: list[Any],
) -> dict[str, dict[str, Any]]:
    assets_by_path: dict[str, dict[str, Any]] = {}
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            continue
        path = asset.get("path")
        if not isinstance(path, str) or not path:
            continue
        if path not in assets_by_path:
            assets_by_path[path] = {
                "asset": asset,
                "index": index,
            }
    return assets_by_path


def pack_asset_source_byte_field_diagnostics(
    path: str,
    index: int,
    pack_asset: dict[str, Any],
    source_bytes: bytes,
) -> list[str]:
    diagnostics: list[str] = []
    size = pack_asset.get("size")
    if isinstance(size, int) and not isinstance(size, bool):
        source_size = len(source_bytes)
        if size != source_size:
            diagnostics.append(
                f"pack report manifest.assets[{index}].size {size} "
                f"does not match CookAssets source byte length {source_size} "
                f"for included asset {path}"
            )
    chunk_hash = pack_asset.get("chunk_hash")
    if is_byte_hash(chunk_hash):
        expected_hash = zrpack_content_hash(source_bytes)
        if chunk_hash != expected_hash:
            diagnostics.append(
                f"pack report manifest.assets[{index}].chunk_hash "
                "does not match CookAssets source content hash for "
                f"included asset {path}"
            )
    return diagnostics


def zrpack_content_hash(source_bytes: bytes) -> list[int]:
    hash_bytes = bytearray()
    for seed in ZRPACK_HASH_SEEDS:
        value = fnv1a64(source_bytes, seed)
        hash_bytes.extend(value.to_bytes(8, byteorder="little"))
    return list(hash_bytes)


def fnv1a64(source_bytes: bytes, seed: int) -> int:
    value = seed
    for byte in source_bytes:
        value ^= byte
        value = (value * FNV1A64_PRIME) & U64_MASK
    return value


def is_byte_hash(value: Any) -> bool:
    return (
        isinstance(value, list)
        and len(value) == 32
        and all(
            isinstance(item, int)
            and not isinstance(item, bool)
            and 0 <= item <= 255
            for item in value
        )
    )
