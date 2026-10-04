"""Pack delta manifest schema and semantic diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_pack_delta_asset_set_semantics import (
    asset_path_list_is_schema_clean,
    delta_changed_assets_are_schema_clean,
    delta_chunks_are_schema_clean,
)
from .pipeline_report_pack_manifest_schema import (
    pack_document_manifest_schema_diagnostics,
    pack_document_manifest_is_schema_clean,
)
from .pipeline_report_pack_manifest_schema_helpers import (
    PACK_FORMAT_VERSION,
    pack_asset_chunk_size_diagnostics,
    pack_asset_entries_schema_diagnostics,
    pack_chunk_entries_schema_diagnostics,
    pack_chunk_offset_diagnostics,
)
from .pipeline_report_pack_manifest_path_hash_schema_helpers import (
    is_safe_asset_package_path,
    normalized_asset_package_path,
    pack_asset_path_schema_diagnostics,
    pack_asset_path_uniqueness_diagnostics,
    pack_chunk_hash_order_diagnostics,
    pack_chunk_hash_uniqueness_diagnostics,
)
from .pipeline_report_schema_string_array import string_array_no_blank_entries_schema_diagnostics

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
