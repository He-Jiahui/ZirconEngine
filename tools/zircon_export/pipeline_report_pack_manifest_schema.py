"""Pack document manifest schema and semantic diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .export_template import is_safe_relative_path, normalize_relative_path

PACK_DOCUMENT_MANIFEST_FIELDS = ("assets", "pack")
PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_FIELDS = ("pack",)
PACK_DOCUMENT_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS = ("assets",)
PACK_FORMAT_VERSION = 1
PACK_MANIFEST_FIELDS = ("chunks", "total_size", "version")
PACK_MANIFEST_INTEGER_FIELDS = ("total_size", "version")
PACK_MANIFEST_REQUIRED_INTEGER_FIELDS = ("total_size", "version")
PACK_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS = ("chunks",)
PACK_MANIFEST_NON_NEGATIVE_INTEGER_FIELDS = ("total_size",)
PACK_CHUNK_ENTRY_FIELDS = ("hash", "offset", "size")
PACK_CHUNK_ENTRY_INTEGER_FIELDS = ("offset", "size")
PACK_CHUNK_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS = ("hash",)
PACK_CHUNK_ENTRY_REQUIRED_INTEGER_FIELDS = ("offset", "size")
PACK_CHUNK_ENTRY_NON_NEGATIVE_INTEGER_FIELDS = ("offset", "size")
PACK_ASSET_ENTRY_FIELDS = ("chunk_hash", "path", "size")
PACK_ASSET_ENTRY_STRING_FIELDS = ("path",)
PACK_ASSET_ENTRY_INTEGER_FIELDS = ("size",)
PACK_ASSET_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS = ("chunk_hash",)
PACK_ASSET_ENTRY_REQUIRED_STRING_FIELDS = ("path",)
PACK_ASSET_ENTRY_REQUIRED_INTEGER_FIELDS = ("size",)
PACK_ASSET_ENTRY_NON_NEGATIVE_INTEGER_FIELDS = ("size",)

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


def pack_manifest_is_schema_clean(pack: dict[str, Any]) -> bool:
    if any(field not in PACK_MANIFEST_FIELDS for field in pack):
        return False
    version = pack.get("version")
    total_size = pack.get("total_size")
    chunks = pack.get("chunks")
    return (
        isinstance(version, int)
        and not isinstance(version, bool)
        and version == PACK_FORMAT_VERSION
        and isinstance(total_size, int)
        and not isinstance(total_size, bool)
        and total_size >= 0
        and isinstance(chunks, list)
        and all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks)
    )


def pack_chunk_entry_is_schema_clean(chunk: Any) -> bool:
    if not isinstance(chunk, dict):
        return False
    if any(field not in PACK_CHUNK_ENTRY_FIELDS for field in chunk):
        return False
    offset = chunk.get("offset")
    size = chunk.get("size")
    return (
        is_byte_hash(chunk.get("hash"))
        and isinstance(offset, int)
        and not isinstance(offset, bool)
        and offset >= 0
        and isinstance(size, int)
        and not isinstance(size, bool)
        and size >= 0
    )


def pack_asset_entry_is_schema_clean(asset: Any) -> bool:
    if not isinstance(asset, dict):
        return False
    if any(field not in PACK_ASSET_ENTRY_FIELDS for field in asset):
        return False
    path = asset.get("path")
    size = asset.get("size")
    return (
        isinstance(path, str)
        and pack_asset_path_is_schema_clean(path)
        and is_byte_hash(asset.get("chunk_hash"))
        and isinstance(size, int)
        and not isinstance(size, bool)
        and size >= 0
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
            )
        )
    assets = manifest.get("assets")
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
                pack_asset_chunk_reference_diagnostics(label, pack, assets)
            )
            chunks = pack.get("chunks")
            if isinstance(chunks, list):
                diagnostics.extend(
                    pack_asset_chunk_size_diagnostics(
                        f"{label}.assets",
                        f"{label}.pack.chunks",
                        chunks,
                        assets,
                    )
                )
    return diagnostics


def pack_manifest_schema_diagnostics(
    label: str,
    pack: dict[str, Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(pack)
        if field not in PACK_MANIFEST_FIELDS
    )
    for field in PACK_MANIFEST_INTEGER_FIELDS:
        if field in pack or field in PACK_MANIFEST_REQUIRED_INTEGER_FIELDS:
            field_label = f"{label}.{field}"
            field_value = pack.get(field)
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    field_label,
                    field_value,
                )
            )
            if field in PACK_MANIFEST_NON_NEGATIVE_INTEGER_FIELDS:
                diagnostics.extend(non_negative_integer_diagnostics(field_label, field_value))
    diagnostics.extend(pack_version_diagnostics(label, pack))
    chunks = pack.get("chunks")
    if "chunks" in pack or "chunks" in PACK_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(f"{label}.chunks", chunks)
        )
    if isinstance(chunks, list):
        diagnostics.extend(
            pack_chunk_entries_schema_diagnostics(
                f"{label}.chunks",
                chunks,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(
            pack_chunk_hash_uniqueness_diagnostics(f"{label}.chunks", chunks)
        )
        diagnostics.extend(pack_chunk_hash_order_diagnostics(f"{label}.chunks", chunks))
        diagnostics.extend(pack_total_size_diagnostics(label, pack, chunks))
        diagnostics.extend(pack_chunk_offset_diagnostics(f"{label}.chunks", chunks))
    return diagnostics


def pack_chunk_entries_schema_diagnostics(
    label: str,
    chunks: list[Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(PACK_CHUNK_ENTRY_FIELDS)
    for index, chunk in enumerate(chunks):
        if not isinstance(chunk, dict):
            continue
        chunk_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{chunk_label} unknown field {field}"
            for field in sorted(chunk)
            if field not in known_fields
        )
        for field in PACK_CHUNK_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS:
            diagnostics.extend(
                validate_byte_array_schema_diagnostics(
                    f"{chunk_label}.{field}",
                    chunk.get(field),
                )
            )
        for field in PACK_CHUNK_ENTRY_INTEGER_FIELDS:
            if field in chunk or field in PACK_CHUNK_ENTRY_REQUIRED_INTEGER_FIELDS:
                field_label = f"{chunk_label}.{field}"
                field_value = chunk.get(field)
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        field_label,
                        field_value,
                    )
                )
                if field in PACK_CHUNK_ENTRY_NON_NEGATIVE_INTEGER_FIELDS:
                    diagnostics.extend(
                        non_negative_integer_diagnostics(field_label, field_value)
                    )
    return diagnostics


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


def pack_total_size_diagnostics(
    label: str,
    pack: dict[str, Any],
    chunks: list[Any],
) -> list[str]:
    total_size = pack.get("total_size")
    if not isinstance(total_size, int) or isinstance(total_size, bool):
        return []
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return []
    chunk_size_sum = 0
    for chunk in chunks:
        chunk_size = chunk["size"]
        chunk_size_sum += chunk_size
    if total_size != chunk_size_sum:
        return [
            f"{label}.total_size {total_size} does not match "
            f"{label}.chunks size sum {chunk_size_sum}"
        ]
    return []


def pack_version_diagnostics(
    label: str,
    pack: dict[str, Any],
) -> list[str]:
    version = pack.get("version")
    if not isinstance(version, int) or isinstance(version, bool):
        return []
    if version != PACK_FORMAT_VERSION:
        return [
            f"{label}.version {version} is not supported; "
            f"expected {PACK_FORMAT_VERSION}"
        ]
    return []


def pack_chunk_offset_diagnostics(
    label: str,
    chunks: list[Any],
) -> list[str]:
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return []
    expected_offset = 24
    for index, chunk in enumerate(sorted(chunks, key=pack_chunk_offset_sort_key)):
        offset = chunk.get("offset")
        size = chunk.get("size")
        if offset != expected_offset:
            return [
                f"{label}[{index}].offset {offset} does not match "
                f"expected chunk offset {expected_offset}"
            ]
        expected_offset += size
    return []


def pack_chunk_offset_sort_key(chunk: Any) -> int:
    if isinstance(chunk, dict):
        offset = chunk.get("offset")
        if isinstance(offset, int) and not isinstance(offset, bool):
            return offset
    return 0


def pack_asset_chunk_reference_diagnostics(
    label: str,
    pack: dict[str, Any],
    assets: list[Any],
) -> list[str]:
    chunks = pack.get("chunks")
    if not isinstance(chunks, list):
        return []
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return []
    if not all(pack_asset_entry_is_schema_clean(asset) for asset in assets):
        return []
    chunk_hashes: set[tuple[int, ...]] = set()
    for chunk in chunks:
        if not isinstance(chunk, dict):
            return []
        chunk_hash = chunk.get("hash")
        if not is_byte_hash(chunk_hash):
            return []
        chunk_hashes.add(tuple(chunk_hash))
    diagnostics: list[str] = []
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            return []
        chunk_hash = asset.get("chunk_hash")
        if not is_byte_hash(chunk_hash):
            return []
        if tuple(chunk_hash) not in chunk_hashes:
            diagnostics.append(
                f"{label}.assets[{index}].chunk_hash "
                f"is not present in {label}.pack.chunks"
            )
    return diagnostics


def pack_asset_chunk_size_diagnostics(
    asset_label: str,
    chunk_label: str,
    chunks: list[Any],
    assets: list[Any],
) -> list[str]:
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return []
    if not all(pack_asset_entry_is_schema_clean(asset) for asset in assets):
        return []
    chunk_sizes: dict[tuple[int, ...], int] = {}
    for chunk in chunks:
        chunk_hash = chunk.get("hash")
        size = chunk.get("size")
        chunk_hash_key = tuple(chunk_hash)
        if chunk_hash_key in chunk_sizes:
            return []
        chunk_sizes[chunk_hash_key] = size
    diagnostics: list[str] = []
    for index, asset in enumerate(assets):
        chunk_hash = asset.get("chunk_hash")
        asset_size = asset.get("size")
        chunk_size = chunk_sizes.get(tuple(chunk_hash))
        if chunk_size is None:
            continue
        if asset_size != chunk_size:
            diagnostics.append(
                f"{asset_label}[{index}].size {asset_size} does not match "
                f"{chunk_label} size {chunk_size}"
            )
    return diagnostics


def pack_asset_entries_schema_diagnostics(
    label: str,
    assets: list[Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(PACK_ASSET_ENTRY_FIELDS)
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            continue
        asset_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{asset_label} unknown field {field}"
            for field in sorted(asset)
            if field not in known_fields
        )
        for field in PACK_ASSET_ENTRY_STRING_FIELDS:
            if field in asset or field in PACK_ASSET_ENTRY_REQUIRED_STRING_FIELDS:
                field_label = f"{asset_label}.{field}"
                field_value = asset.get(field)
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        field_label,
                        field_value,
                    )
                )
                if isinstance(field_value, str) and not field_value.strip():
                    diagnostics.append(f"{field_label} must be a non-empty string")
                elif isinstance(field_value, str):
                    diagnostics.extend(
                        pack_asset_path_schema_diagnostics(field_label, field_value)
                    )
        for field in PACK_ASSET_ENTRY_REQUIRED_BYTE_ARRAY_FIELDS:
            diagnostics.extend(
                validate_byte_array_schema_diagnostics(
                    f"{asset_label}.{field}",
                    asset.get(field),
                )
            )
        for field in PACK_ASSET_ENTRY_INTEGER_FIELDS:
            if field in asset or field in PACK_ASSET_ENTRY_REQUIRED_INTEGER_FIELDS:
                field_label = f"{asset_label}.{field}"
                field_value = asset.get(field)
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        field_label,
                        field_value,
                    )
                )
                if field in PACK_ASSET_ENTRY_NON_NEGATIVE_INTEGER_FIELDS:
                    diagnostics.extend(
                        non_negative_integer_diagnostics(field_label, field_value)
                    )
    return diagnostics


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


def non_negative_integer_diagnostics(label: str, value: Any) -> list[str]:
    if isinstance(value, int) and not isinstance(value, bool) and value < 0:
        return [f"{label} must be non-negative"]
    return []


def validate_byte_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list) or len(value) != 32:
        return [f"{label} must be a 32-byte integer array"]
    if any(
        not isinstance(item, int) or isinstance(item, bool) or item < 0 or item > 255
        for item in value
    ):
        return [f"{label} must be a 32-byte integer array"]
    return []
