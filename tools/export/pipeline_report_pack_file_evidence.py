"""Pack stage file and binary evidence diagnostics."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .pipeline_report_pack_delta_schema import pack_delta_manifest_is_schema_clean
from .pipeline_report_pack_manifest_schema import (
    pack_document_manifest_is_schema_clean,
)
from .pipeline_report_pack_manifest_schema_helpers import (
    PACK_FORMAT_VERSION,
    pack_chunk_entry_is_schema_clean,
)
from .zrpack_hash import zrpack_content_hash as _zrpack_content_hash


PACK_REPORT_REQUIRED_NON_FATAL_FILE_FIELDS = (
    "asset_manifest",
    "pack",
)
PACK_REPORT_OPTIONAL_NON_FATAL_FILE_FIELDS = (
    "delta_pack",
    "previous_pack",
)
PACK_BINARY_HEADER_SIZE = 24
PACK_BINARY_MAGIC = b"ZRPK"
PACK_DELTA_BINARY_MAGIC = b"ZRPD"


def pack_report_binary_manifest_evidence_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    pack_manifest = report.get("manifest")
    parsed_pack_manifest, pack_bytes, pack_diagnostics = pack_report_embedded_manifest(
        "pack report pack",
        report.get("pack"),
        PACK_BINARY_MAGIC,
    )
    diagnostics.extend(pack_diagnostics)
    diagnostics.extend(
        pack_report_chunk_payload_hash_diagnostics(
            "pack report pack",
            report.get("pack"),
            pack_bytes,
            parsed_pack_manifest,
            "pack.chunks",
        )
    )
    if (
        parsed_pack_manifest is not None
        and isinstance(pack_manifest, dict)
        and pack_document_manifest_is_schema_clean(pack_manifest)
        and parsed_pack_manifest != pack_manifest
    ):
        diagnostics.append("pack report pack embedded manifest does not match manifest")

    delta_manifest = report.get("delta_manifest")
    if isinstance(report.get("delta_pack"), str) and report["delta_pack"].strip():
        parsed_delta_manifest, delta_bytes, delta_diagnostics = (
            pack_report_embedded_manifest(
                "pack report delta_pack",
                report.get("delta_pack"),
                PACK_DELTA_BINARY_MAGIC,
            )
        )
        diagnostics.extend(delta_diagnostics)
        diagnostics.extend(
            pack_report_chunk_payload_hash_diagnostics(
                "pack report delta_pack",
                report.get("delta_pack"),
                delta_bytes,
                parsed_delta_manifest,
                "chunks",
            )
        )
        if (
            parsed_delta_manifest is not None
            and isinstance(delta_manifest, dict)
            and pack_delta_manifest_is_schema_clean(delta_manifest)
            and parsed_delta_manifest != delta_manifest
        ):
            diagnostics.append(
                "pack report delta_pack embedded manifest does not match delta_manifest"
            )

    if isinstance(report.get("previous_pack"), str) and report["previous_pack"].strip():
        parsed_previous_manifest, previous_bytes, previous_diagnostics = (
            pack_report_embedded_manifest(
                "pack report previous_pack",
                report.get("previous_pack"),
                PACK_BINARY_MAGIC,
            )
        )
        diagnostics.extend(previous_diagnostics)
        diagnostics.extend(
            pack_report_chunk_payload_hash_diagnostics(
                "pack report previous_pack",
                report.get("previous_pack"),
                previous_bytes,
                parsed_previous_manifest,
                "pack.chunks",
            )
        )
        if parsed_previous_manifest is not None and isinstance(delta_manifest, dict):
            base_manifest = delta_manifest.get("base")
            if (
                isinstance(base_manifest, dict)
                and pack_document_manifest_is_schema_clean(base_manifest)
                and parsed_previous_manifest != base_manifest
            ):
                diagnostics.append(
                    "pack report previous_pack embedded manifest does not match "
                    "delta_manifest.base"
                )
    return diagnostics


def pack_report_chunk_payload_hash_diagnostics(
    label: str,
    value: Any,
    bytes_value: bytes | None,
    manifest: dict[str, Any] | None,
    chunk_path: str,
) -> list[str]:
    if bytes_value is None or manifest is None:
        return []
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return []
    chunks = manifest_chunk_rows(manifest, chunk_path)
    if not isinstance(chunks, list):
        return []
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return []
    diagnostics: list[str] = []
    path = Path(value)
    pack_view = memoryview(bytes_value)
    manifest_offset = int.from_bytes(bytes_value[8:16], "little")
    payload_end = manifest_chunk_payload_end(chunks)
    if payload_end is not None and manifest_offset != payload_end:
        diagnostics.append(
            f"{label} {path} manifest offset {manifest_offset} does not match "
            f"{chunk_path} payload end {payload_end}"
        )
    for index, chunk in enumerate(chunks):
        if not isinstance(chunk, dict):
            continue
        chunk_hash = chunk.get("hash")
        offset = chunk.get("offset")
        size = chunk.get("size")
        if (
            not isinstance(chunk_hash, list)
            or len(chunk_hash) != 32
            or not all(
                isinstance(item, int) and not isinstance(item, bool)
                for item in chunk_hash
            )
            or not isinstance(offset, int)
            or isinstance(offset, bool)
            or not isinstance(size, int)
            or isinstance(size, bool)
        ):
            continue
        end = offset + size
        if (
            offset < PACK_BINARY_HEADER_SIZE
            or end > len(bytes_value)
            or end > manifest_offset
            or end < offset
        ):
            diagnostics.append(
                f"{label} {path} {chunk_path}[{index}] payload range is out of bounds"
            )
            continue
        if zrpack_content_hash(pack_view[offset:end]) != chunk_hash:
            diagnostics.append(
                f"{label} {path} {chunk_path}[{index}] payload does not match "
                "manifest hash"
            )
    return diagnostics


def manifest_chunk_payload_end(chunks: list[Any]) -> int | None:
    if not all(pack_chunk_entry_is_schema_clean(chunk) for chunk in chunks):
        return None
    payload_end = PACK_BINARY_HEADER_SIZE
    for chunk in sorted(chunks, key=pack_report_chunk_offset_sort_key):
        if not isinstance(chunk, dict):
            return None
        offset = chunk.get("offset")
        size = chunk.get("size")
        if (
            not isinstance(offset, int)
            or isinstance(offset, bool)
            or not isinstance(size, int)
            or isinstance(size, bool)
        ):
            return None
        if offset != payload_end:
            return None
        payload_end += size
    return payload_end


def pack_report_chunk_offset_sort_key(chunk: Any) -> int:
    if isinstance(chunk, dict):
        offset = chunk.get("offset")
        if isinstance(offset, int) and not isinstance(offset, bool):
            return offset
    return 0


def manifest_chunk_rows(manifest: dict[str, Any], chunk_path: str) -> Any:
    if chunk_path == "chunks":
        return manifest.get("chunks")
    if chunk_path == "pack.chunks":
        pack = manifest.get("pack")
        if isinstance(pack, dict):
            return pack.get("chunks")
    return None


def zrpack_content_hash(bytes_value: bytes | memoryview) -> list[int]:
    return _zrpack_content_hash(bytes_value)


def pack_report_embedded_manifest(
    label: str,
    value: Any,
    magic: bytes,
) -> tuple[dict[str, Any] | None, bytes | None, list[str]]:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None, None, []
    path = Path(value)
    try:
        if not path.is_file() or path.stat().st_size == 0:
            return None, None, []
        bytes_value = path.read_bytes()
    except OSError as error:
        return None, None, [f"{label} {path} could not be read: {error}"]
    if len(bytes_value) < PACK_BINARY_HEADER_SIZE:
        return None, bytes_value, [f"{label} {path} header is too small"]
    if bytes_value[0:4] != magic:
        return None, bytes_value, [f"{label} {path} header magic is invalid"]
    version = int.from_bytes(bytes_value[4:8], "little")
    if version != PACK_FORMAT_VERSION:
        return None, bytes_value, [
            f"{label} {path} format version {version} is unsupported; "
            f"expected {PACK_FORMAT_VERSION}"
        ]
    manifest_offset = int.from_bytes(bytes_value[8:16], "little")
    manifest_size = int.from_bytes(bytes_value[16:24], "little")
    manifest_end = manifest_offset + manifest_size
    if (
        manifest_offset < PACK_BINARY_HEADER_SIZE
        or manifest_end > len(bytes_value)
        or manifest_end < manifest_offset
    ):
        return None, bytes_value, [f"{label} {path} manifest range is out of bounds"]
    if manifest_end != len(bytes_value):
        return None, bytes_value, [
            f"{label} {path} manifest end {manifest_end} does not match "
            f"artifact size {len(bytes_value)}"
        ]
    try:
        manifest = json.loads(
            bytes_value[manifest_offset:manifest_end].decode("utf-8")
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        return None, bytes_value, [
            f"{label} {path} embedded manifest could not be decoded: {error}"
        ]
    if not isinstance(manifest, dict):
        return None, bytes_value, [f"{label} {path} embedded manifest must be an object"]
    return manifest, bytes_value, []


def pack_report_file_evidence_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in PACK_REPORT_REQUIRED_NON_FATAL_FILE_FIELDS:
        diagnostics.extend(
            pack_report_path_file_evidence_diagnostics(
                f"pack report {field}",
                report.get(field),
            )
        )
    for field in PACK_REPORT_OPTIONAL_NON_FATAL_FILE_FIELDS:
        value = report.get(field)
        if isinstance(value, str) and value.strip():
            diagnostics.extend(
                pack_report_path_file_evidence_diagnostics(
                    f"pack report {field}",
                    value,
                )
            )
    return diagnostics


def pack_report_path_file_evidence_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return []
    path = Path(value)
    try:
        if not path.exists():
            return [f"{label} {path} does not exist"]
        if not path.is_file():
            return [f"{label} {path} is not a file"]
        if path.stat().st_size == 0:
            return [f"{label} {path} must not be empty"]
    except OSError as error:
        return [f"{label} {path} could not be inspected: {error}"]
    return []
