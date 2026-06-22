"""Pack delta manifest schema and semantic diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_manifest_schema import (
    PACK_FORMAT_VERSION,
    is_byte_hash,
    pack_asset_entry_is_schema_clean,
    pack_asset_path_is_schema_clean,
    is_safe_asset_package_path,
    normalized_asset_package_path,
    pack_asset_chunk_size_diagnostics,
    pack_asset_entries_schema_diagnostics,
    pack_asset_path_schema_diagnostics,
    pack_asset_path_uniqueness_diagnostics,
    pack_chunk_entry_is_schema_clean,
    pack_chunk_entries_schema_diagnostics,
    pack_chunk_hash_order_diagnostics,
    pack_chunk_hash_uniqueness_diagnostics,
    pack_chunk_offset_diagnostics,
    pack_document_manifest_schema_diagnostics,
    pack_document_manifest_is_schema_clean,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
)

PACK_DELTA_MANIFEST_FIELDS = (
    "base",
    "changed_assets",
    "chunks",
    "format_version",
    "removed_assets",
    "target",
)
PACK_DELTA_MANIFEST_INTEGER_FIELDS = ("format_version",)
PACK_DELTA_MANIFEST_REQUIRED_INTEGER_FIELDS = ("format_version",)
PACK_DELTA_MANIFEST_OBJECT_FIELDS = ("base", "target")
PACK_DELTA_MANIFEST_REQUIRED_OBJECT_FIELDS = ("base", "target")
PACK_DELTA_MANIFEST_OBJECT_ARRAY_FIELDS = ("changed_assets", "chunks")
PACK_DELTA_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS = ("changed_assets", "chunks")
PACK_DELTA_MANIFEST_STRING_ARRAY_FIELDS = ("removed_assets",)
PACK_DELTA_MANIFEST_REQUIRED_STRING_ARRAY_FIELDS = ("removed_assets",)
PACK_DELTA_MANIFEST_NO_BLANK_STRING_ARRAY_FIELDS = ("removed_assets",)

SchemaDiagnostic = Callable[[str, Any], list[str]]
PackChunkFingerprint = tuple[tuple[int, ...], int, int]
PackPlanFingerprint = tuple[int, tuple[PackChunkFingerprint, ...], int]
PackAssetFingerprint = tuple[str, tuple[int, ...], int]
PackDocumentFingerprint = tuple[
    PackPlanFingerprint,
    tuple[PackAssetFingerprint, ...],
]


def pack_report_delta_publication_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    delta_pack = report.get("delta_pack")
    delta_manifest = report.get("delta_manifest")
    previous_pack = report.get("previous_pack")
    delta_pack_present = report_path_is_present(delta_pack)
    previous_pack_present = report_path_is_present(previous_pack)
    if delta_pack_present:
        if delta_manifest is None:
            diagnostics.append(
                "pack report delta_pack is present but delta_manifest is missing"
            )
    elif isinstance(delta_manifest, dict):
        diagnostics.append(
            "pack report delta_manifest is present but delta_pack is missing"
        )
    if previous_pack_present and not delta_pack_present:
        diagnostics.append(
            "pack report previous_pack is present but delta_pack is missing"
        )
    return diagnostics


def report_path_is_present(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def pack_report_delta_manifest_count_diagnostics(
    report: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    changed_assets = delta_manifest.get("changed_assets")
    delta_asset_count = report.get("delta_asset_count")
    if (
        isinstance(changed_assets, list)
        and delta_changed_assets_are_schema_clean(changed_assets)
        and isinstance(delta_asset_count, int)
        and not isinstance(delta_asset_count, bool)
        and delta_asset_count != len(changed_assets)
    ):
        diagnostics.append(
            f"pack report delta_asset_count {delta_asset_count} does not match "
            f"delta_manifest.changed_assets length {len(changed_assets)}"
        )
    chunks = delta_manifest.get("chunks")
    delta_chunk_count = report.get("delta_chunk_count")
    if (
        isinstance(chunks, list)
        and delta_chunks_are_schema_clean(chunks)
        and isinstance(delta_chunk_count, int)
        and not isinstance(delta_chunk_count, bool)
        and delta_chunk_count != len(chunks)
    ):
        diagnostics.append(
            f"pack report delta_chunk_count {delta_chunk_count} does not match "
            f"delta_manifest.chunks length {len(chunks)}"
        )
    return diagnostics


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


def pack_delta_manifest_is_schema_clean(delta_manifest: dict[str, Any]) -> bool:
    if any(field not in PACK_DELTA_MANIFEST_FIELDS for field in delta_manifest):
        return False
    format_version = delta_manifest.get("format_version")
    base = delta_manifest.get("base")
    target = delta_manifest.get("target")
    changed_assets = delta_manifest.get("changed_assets")
    chunks = delta_manifest.get("chunks")
    removed_assets = delta_manifest.get("removed_assets")
    return (
        isinstance(format_version, int)
        and not isinstance(format_version, bool)
        and format_version == PACK_FORMAT_VERSION
        and isinstance(base, dict)
        and pack_document_manifest_is_schema_clean(base)
        and isinstance(target, dict)
        and pack_document_manifest_is_schema_clean(target)
        and isinstance(changed_assets, list)
        and delta_changed_assets_are_schema_clean(changed_assets)
        and isinstance(chunks, list)
        and delta_chunks_are_schema_clean(chunks)
        and isinstance(removed_assets, list)
        and asset_path_list_is_schema_clean(removed_assets)
    )


def pack_report_delta_target_manifest_diagnostics(
    manifest: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    target = delta_manifest.get("target")
    if not isinstance(target, dict):
        return []
    parsed_manifest = pack_document_manifest_fingerprint(manifest)
    parsed_target = pack_document_manifest_fingerprint(target)
    if parsed_manifest is None or parsed_target is None:
        return []
    if parsed_manifest == parsed_target:
        return []
    return ["pack report delta_manifest.target does not match manifest"]


def pack_report_delta_asset_set_diagnostics(
    report: dict[str, Any],
    delta_manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    expected_removed_assets = delta_removed_asset_paths(delta_manifest)
    if expected_removed_assets is None:
        return diagnostics
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

    expected_delta_asset_sets = delta_changed_and_reused_asset_paths(delta_manifest)
    if expected_delta_asset_sets is None:
        return diagnostics
    expected_changed_assets, expected_reused_assets, expected_changed_entries = (
        expected_delta_asset_sets
    )
    changed_assets = delta_manifest.get("changed_assets")
    if isinstance(changed_assets, list) and delta_changed_assets_are_schema_clean(
        changed_assets
    ):
        changed_asset_paths = manifest_asset_paths(changed_assets)
        if (
            changed_asset_paths is not None
            and changed_asset_paths != expected_changed_assets
        ):
            diagnostics.append(
                "pack report delta_manifest.changed_assets does not match "
                "target assets missing from base chunks"
            )
        elif not delta_changed_asset_entries_match(
            changed_assets,
            expected_changed_entries,
        ):
            diagnostics.append(
                "pack report delta_manifest.changed_assets does not match "
                "target manifest asset entries"
            )
        if not delta_changed_asset_chunk_hashes_match(delta_manifest, changed_assets):
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


def delta_removed_asset_paths(delta_manifest: dict[str, Any]) -> list[str] | None:
    base_assets = delta_manifest_assets(delta_manifest, "base")
    target_assets = delta_manifest_assets(delta_manifest, "target")
    if base_assets is None or target_assets is None:
        return None
    target_paths = {asset["path"] for asset in target_assets}
    return sorted(asset["path"] for asset in base_assets if asset["path"] not in target_paths)


def delta_changed_and_reused_asset_paths(
    delta_manifest: dict[str, Any],
) -> tuple[list[str], list[str], list[dict[str, Any]]] | None:
    base_hashes = delta_manifest_base_chunk_hashes(delta_manifest)
    target_assets = delta_manifest_assets(delta_manifest, "target")
    if base_hashes is None or target_assets is None:
        return None
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


def delta_changed_asset_entries(changed_assets: list[Any]) -> list[dict[str, Any]] | None:
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


def pack_delta_manifest_schema_diagnostics(
    label: str,
    delta_manifest: dict[str, Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(delta_manifest)
        if field not in PACK_DELTA_MANIFEST_FIELDS
    )
    for field in PACK_DELTA_MANIFEST_INTEGER_FIELDS:
        if field in delta_manifest or field in PACK_DELTA_MANIFEST_REQUIRED_INTEGER_FIELDS:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    delta_manifest.get(field),
                )
            )
    diagnostics.extend(pack_delta_format_version_diagnostics(label, delta_manifest))
    for field in PACK_DELTA_MANIFEST_OBJECT_FIELDS:
        value = delta_manifest.get(field)
        if field in delta_manifest or field in PACK_DELTA_MANIFEST_REQUIRED_OBJECT_FIELDS:
            diagnostics.extend(validate_object_schema_diagnostics(f"{label}.{field}", value))
        if isinstance(value, dict):
            diagnostics.extend(
                pack_document_manifest_schema_diagnostics(
                    f"{label}.{field}",
                    value,
                    validate_integer_schema_diagnostics=(
                        validate_integer_schema_diagnostics
                    ),
                    validate_string_schema_diagnostics=(
                        validate_string_schema_diagnostics
                    ),
                    validate_object_schema_diagnostics=(
                        validate_object_schema_diagnostics
                    ),
                    validate_object_array_schema_diagnostics=(
                        validate_object_array_schema_diagnostics
                    ),
                )
            )
    chunks = delta_manifest.get("chunks")
    if "chunks" in delta_manifest or "chunks" in PACK_DELTA_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS:
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
        diagnostics.extend(pack_chunk_offset_diagnostics(f"{label}.chunks", chunks))
    changed_assets = delta_manifest.get("changed_assets")
    if (
        "changed_assets" in delta_manifest
        or "changed_assets" in PACK_DELTA_MANIFEST_REQUIRED_OBJECT_ARRAY_FIELDS
    ):
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.changed_assets",
                changed_assets,
            )
        )
    if isinstance(changed_assets, list):
        diagnostics.extend(
            pack_asset_entries_schema_diagnostics(
                f"{label}.changed_assets",
                changed_assets,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
            )
        )
        diagnostics.extend(
            pack_asset_path_uniqueness_diagnostics(
                f"{label}.changed_assets",
                changed_assets,
            )
        )
        if isinstance(chunks, list):
            diagnostics.extend(
                pack_asset_chunk_size_diagnostics(
                    f"{label}.changed_assets",
                    f"{label}.chunks",
                    chunks,
                    changed_assets,
                )
            )
    for field in PACK_DELTA_MANIFEST_STRING_ARRAY_FIELDS:
        if field in delta_manifest or field in PACK_DELTA_MANIFEST_REQUIRED_STRING_ARRAY_FIELDS:
            field_label = f"{label}.{field}"
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    field_label,
                    delta_manifest.get(field),
                )
            )
            if field in PACK_DELTA_MANIFEST_NO_BLANK_STRING_ARRAY_FIELDS:
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        field_label,
                        delta_manifest.get(field),
                    )
                )
            if field == "removed_assets":
                diagnostics.extend(
                    pack_removed_asset_path_schema_diagnostics(
                        field_label,
                        delta_manifest.get(field),
                    )
                )
    return diagnostics


def pack_delta_format_version_diagnostics(
    label: str,
    delta_manifest: dict[str, Any],
) -> list[str]:
    format_version = delta_manifest.get("format_version")
    if not isinstance(format_version, int) or isinstance(format_version, bool):
        return []
    if format_version != PACK_FORMAT_VERSION:
        return [
            f"{label}.format_version {format_version} is not supported; "
            f"expected {PACK_FORMAT_VERSION}"
        ]
    return []


def pack_removed_asset_path_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    seen_paths: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            continue
        item_label = f"{label}[{index}]"
        path_diagnostics = pack_asset_path_schema_diagnostics(item_label, item)
        diagnostics.extend(path_diagnostics)
        if path_diagnostics:
            continue
        if not is_safe_asset_package_path(item):
            continue
        normalized_path = normalized_asset_package_path(item)
        if normalized_path in seen_paths:
            diagnostics.append(
                f"{label} path {normalized_path} is declared more than once"
            )
        else:
            seen_paths.add(normalized_path)
    return diagnostics
