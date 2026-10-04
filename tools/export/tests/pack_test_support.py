from __future__ import annotations


def pack_manifest(*, hash_value: int = 1) -> dict[str, object]:
    return {
        "pack": pack_plan(hash_value=hash_value),
        "assets": [asset_entry(hash_value=hash_value)],
    }


def pack_plan(*, hash_value: int = 1) -> dict[str, object]:
    return {
        "version": 1,
        "chunks": [chunk_entry(hash_value=hash_value)],
        "total_size": 8,
    }


def chunk_entry(*, hash_value: int = 1) -> dict[str, object]:
    return {
        "hash": [hash_value] * 32,
        "offset": 24,
        "size": 8,
    }


def asset_entry(
    *,
    hash_value: int = 1,
    path: str = "scenes/main.zscene",
) -> dict[str, object]:
    return {
        "path": path,
        "chunk_hash": [hash_value] * 32,
        "size": 8,
    }


def manifest_for_assets(
    assets: list[dict[str, object]],
    *,
    hash_values: list[int],
) -> dict[str, object]:
    return {
        "pack": {
            "version": 1,
            "chunks": [
                {
                    **chunk_entry(hash_value=value),
                    "offset": 24 + index * 8,
                }
                for index, value in enumerate(hash_values)
            ],
            "total_size": 8 * len(hash_values),
        },
        "assets": assets,
    }


def delta_manifest() -> dict[str, object]:
    return {
        "format_version": 1,
        "base": manifest_for_assets(
            [
                asset_entry(hash_value=1, path="scenes/main.zscene"),
                asset_entry(hash_value=3, path="textures/old.png"),
            ],
            hash_values=[1, 3],
        ),
        "target": manifest_for_assets(
            [
                asset_entry(hash_value=2, path="scenes/main.zscene"),
                asset_entry(hash_value=1, path="textures/reused.png"),
            ],
            hash_values=[1, 2],
        ),
        "chunks": [chunk_entry(hash_value=2)],
        "changed_assets": [asset_entry(hash_value=2)],
        "removed_assets": ["textures/old.png"],
    }


def empty_delta_manifest() -> dict[str, object]:
    return {
        "format_version": 1,
        "base": empty_pack_document_manifest(),
        "target": empty_pack_document_manifest(),
        "chunks": [],
        "changed_assets": [],
        "removed_assets": [],
    }


def empty_pack_document_manifest() -> dict[str, object]:
    return {
        "pack": {
            "version": 1,
            "chunks": [],
            "total_size": 0,
        },
        "assets": [],
    }
