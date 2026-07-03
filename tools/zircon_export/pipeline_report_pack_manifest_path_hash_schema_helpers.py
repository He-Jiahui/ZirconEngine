"""Pack manifest path and hash schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path


def pack_chunk_hash_uniqueness_diagnostics(
    label: str,
    chunks: list[Any],
) -> list[str]:
    chunk_hashes: set[tuple[int, ...]] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict):
            return []
        chunk_hash = chunk.get("hash")
        if not is_byte_hash(chunk_hash):
            return []
        chunk_hash_key = tuple(chunk_hash)
        if chunk_hash_key in chunk_hashes:
            return [f"{label} contains duplicate chunk hash"]
        chunk_hashes.add(chunk_hash_key)
    return []


def pack_chunk_hash_order_diagnostics(
    label: str,
    chunks: list[Any],
) -> list[str]:
    chunk_hashes: list[tuple[int, ...]] = []
    for chunk in chunks:
        if not isinstance(chunk, dict):
            return []
        chunk_hash = chunk.get("hash")
        if not is_byte_hash(chunk_hash):
            return []
        chunk_hashes.append(tuple(chunk_hash))
    if chunk_hashes != sorted(chunk_hashes):
        return [f"{label} must be sorted by chunk hash"]
    return []


def pack_asset_path_schema_diagnostics(label: str, value: str) -> list[str]:
    if value.strip() != value:
        return [f"{label} must be a non-empty trimmed string"]
    if not is_safe_asset_package_path(value):
        return [f"{label} must be a safe relative asset path"]
    if value != normalized_asset_package_path(value):
        return [f"{label} must use a normalized relative asset path"]
    return []


def pack_asset_path_is_schema_clean(value: str) -> bool:
    return (
        bool(value.strip())
        and value.strip() == value
        and is_safe_asset_package_path(value)
        and value == normalized_asset_package_path(value)
    )


def pack_asset_path_uniqueness_diagnostics(
    label: str,
    assets: list[Any],
) -> list[str]:
    seen_paths: set[str] = set()
    for asset in assets:
        if not isinstance(asset, dict):
            return []
        path = asset.get("path")
        if not isinstance(path, str) or not pack_asset_path_is_schema_clean(path):
            return []
        normalized_path = normalized_asset_package_path(path)
        if normalized_path in seen_paths:
            return [f"{label} path {normalized_path} is declared more than once"]
        seen_paths.add(normalized_path)
    return []


def pack_asset_path_order_diagnostics(
    label: str,
    assets: list[Any],
) -> list[str]:
    paths: list[str] = []
    for asset in assets:
        if not isinstance(asset, dict):
            return []
        path = asset.get("path")
        if not isinstance(path, str) or not pack_asset_path_is_schema_clean(path):
            return []
        paths.append(path)
    if paths != sorted(paths):
        return [f"{label} must be sorted by asset path"]
    return []


def is_safe_asset_package_path(value: str) -> bool:
    normalized = normalize_relative_path(value)
    return bool(normalized) and is_safe_relative_path(normalized)


def normalized_asset_package_path(value: str) -> str:
    return normalize_relative_path(value)


def is_byte_hash(value: Any) -> bool:
    return (
        isinstance(value, list)
        and len(value) == 32
        and all(
            isinstance(item, int)
            and not isinstance(item, bool)
            and item >= 0
            and item <= 255
            for item in value
        )
    )


def validate_byte_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list) or len(value) != 32:
        return [f"{label} must be a 32-byte integer array"]
    if any(
        not isinstance(item, int) or isinstance(item, bool) or item < 0 or item > 255
        for item in value
    ):
        return [f"{label} must be a 32-byte integer array"]
    return []
