"""Pack stage report schema diagnostics."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable

from .pipeline_report_pack_delta_schema import (
    pack_delta_manifest_schema_diagnostics,
    pack_delta_manifest_is_schema_clean,
    pack_report_delta_asset_set_diagnostics,
    pack_report_delta_manifest_count_diagnostics,
    pack_report_delta_publication_diagnostics,
    pack_report_delta_target_manifest_diagnostics,
)
from .pipeline_report_pack_manifest_schema import (
    PACK_FORMAT_VERSION,
    non_negative_integer_diagnostics,
    pack_chunk_entry_is_schema_clean,
    pack_document_manifest_schema_diagnostics,
    pack_document_manifest_is_schema_clean,
    pack_report_deduplicated_assets_diagnostics,
    pack_report_manifest_count_diagnostics,
)
from .pipeline_report_pack_trim_schema import (
    pack_asset_path_array_schema_diagnostics,
    pack_report_trim_manifest_consistency_diagnostics,
    pack_trim_report_is_schema_clean,
    pack_trim_report_non_fatal_preflight_diagnostics,
    pack_trim_report_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
)

PACK_REPORT_FIELDS = (
    "asset_count",
    "asset_manifest",
    "chunk_count",
    "deduplicated_assets",
    "delta_apply_verified",
    "delta_asset_count",
    "delta_chunk_count",
    "delta_manifest",
    "delta_pack",
    "delta_removed_assets",
    "delta_reused_assets",
    "deterministic_double_run",
    "diagnostics",
    "fatal",
    "manifest",
    "pack",
    "previous_pack",
    "profile",
    "stage",
    "stage_output",
    "trim_report",
)
PACK_REPORT_STRING_FIELDS = (
    "asset_manifest",
    "delta_pack",
    "pack",
    "previous_pack",
    "stage_output",
)
PACK_REPORT_NO_BLANK_STRING_FIELDS = (
    "delta_pack",
    "previous_pack",
)
PACK_REPORT_TRIMMED_STRING_FIELDS = PACK_REPORT_STRING_FIELDS
PACK_REPORT_INTEGER_FIELDS = (
    "asset_count",
    "chunk_count",
    "delta_asset_count",
    "delta_chunk_count",
)
PACK_REPORT_NON_NEGATIVE_INTEGER_FIELDS = PACK_REPORT_INTEGER_FIELDS
PACK_REPORT_STRING_ARRAY_FIELDS = (
    "deduplicated_assets",
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_NO_BLANK_STRING_ARRAY_FIELDS = (
    "deduplicated_assets",
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_ASSET_PATH_ARRAY_FIELDS = (
    "deduplicated_assets",
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_BOOL_FIELDS = (
    "delta_apply_verified",
    "deterministic_double_run",
)
PACK_REPORT_OBJECT_FIELDS = (
    "delta_manifest",
    "manifest",
    "trim_report",
)
PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "asset_manifest",
    "pack",
    "stage_output",
)
PACK_REPORT_REQUIRED_NON_FATAL_FILE_FIELDS = (
    "asset_manifest",
    "pack",
)
PACK_REPORT_OPTIONAL_NON_FATAL_FILE_FIELDS = (
    "delta_pack",
    "previous_pack",
)
PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "asset_count",
    "chunk_count",
)
PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("deduplicated_assets",)
PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = ("deterministic_double_run",)
PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    "manifest",
    "trim_report",
)
PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS = (
    "delta_asset_count",
    "delta_chunk_count",
)
PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS = ("delta_pack", "previous_pack")
PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS = (
    "delta_removed_assets",
    "delta_reused_assets",
)
PACK_REPORT_REQUIRED_DELTA_TRUE_BOOL_FIELDS = ("delta_apply_verified",)
PACK_BINARY_HEADER_SIZE = 24
PACK_BINARY_MAGIC = b"ZRPK"
PACK_DELTA_BINARY_MAGIC = b"ZRPD"
ZRPACK_HASH_SEEDS = (
    0xCBF2_9CE4_8422_2325,
    0x9AE1_6A3B_2F90_404F,
    0x6EED_0E9D_A4D9_4A4F,
    0xACE5_929A_D4D9_8F13,
)
SchemaDiagnostic = Callable[[str, Any], list[str]]


def pack_report_schema_diagnostics(
    report: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    for field in PACK_REPORT_STRING_FIELDS:
        if field in report and report.get(field) is not None:
            value = report.get(field)
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"pack report {field}",
                    value,
                )
            )
            if (
                field in PACK_REPORT_NO_BLANK_STRING_FIELDS
                and isinstance(value, str)
                and not value.strip()
            ):
                diagnostics.append(f"pack report {field} must be a non-empty string")
            if (
                field in PACK_REPORT_TRIMMED_STRING_FIELDS
                and isinstance(value, str)
                and value.strip()
                and value.strip() != value
            ):
                diagnostics.append(
                    f"pack report {field} must be a non-empty trimmed string"
                )
    for field in PACK_REPORT_INTEGER_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
            if field in PACK_REPORT_NON_NEGATIVE_INTEGER_FIELDS:
                diagnostics.extend(
                    non_negative_integer_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
    for field in PACK_REPORT_STRING_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            label = f"pack report {field}"
            diagnostics.extend(
                pack_string_array_entry_type_schema_diagnostics(
                    label,
                    report.get(field),
                )
            )
            if field in PACK_REPORT_NO_BLANK_STRING_ARRAY_FIELDS:
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        label,
                        report.get(field),
                    )
                )
            if field in PACK_REPORT_ASSET_PATH_ARRAY_FIELDS:
                diagnostics.extend(
                    pack_asset_path_array_schema_diagnostics(
                        label,
                        report.get(field),
                    )
                )
    for field in PACK_REPORT_BOOL_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    for field in PACK_REPORT_OBJECT_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"pack report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in PACK_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
            elif (
                field not in PACK_REPORT_NO_BLANK_STRING_FIELDS
                and isinstance(report.get(field), str)
                and not report.get(field).strip()
            ):
                diagnostics.append(f"pack report {field} must be a non-empty string")
        for field in PACK_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        diagnostics.extend(pack_report_file_evidence_diagnostics(report))
        diagnostics.extend(pack_report_binary_manifest_evidence_diagnostics(report))
    manifest = report.get("manifest")
    if isinstance(manifest, dict):
        diagnostics.extend(
            pack_document_manifest_schema_diagnostics(
                "pack report manifest",
                manifest,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(pack_report_manifest_count_diagnostics(report, manifest))
        diagnostics.extend(pack_report_deduplicated_assets_diagnostics(report, manifest))
    diagnostics.extend(pack_report_delta_publication_diagnostics(report))
    delta_manifest = report.get("delta_manifest")
    delta_pack = report.get("delta_pack")
    if (
        report.get("fatal") is False
        and isinstance(delta_pack, str)
        and delta_pack.strip()
        and delta_pack.strip() == delta_pack
        and isinstance(delta_manifest, dict)
    ):
        for field in PACK_REPORT_REQUIRED_DELTA_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
            elif isinstance(report.get(field), str) and not report.get(field).strip():
                diagnostics.append(f"pack report {field} must be a non-empty string")
        for field in PACK_REPORT_REQUIRED_DELTA_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_DELTA_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"pack report {field}",
                        report.get(field),
                    )
                )
        for field in PACK_REPORT_REQUIRED_DELTA_TRUE_BOOL_FIELDS:
            value = report.get(field)
            if field not in report or value is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"pack report {field}",
                        value,
                    )
                )
            elif isinstance(value, bool) and value is not True:
                diagnostics.append(
                    f"pack report {field} must be true when delta_pack is published"
                )
    if isinstance(delta_manifest, dict):
        diagnostics.extend(
            pack_delta_manifest_schema_diagnostics(
                "pack report delta_manifest",
                delta_manifest,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    pack_delta_asset_list_schema_diagnostics
                ),
                validate_object_schema_diagnostics=validate_object_schema_diagnostics,
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        if pack_delta_manifest_is_schema_clean(delta_manifest):
            diagnostics.extend(
                pack_report_delta_manifest_count_diagnostics(report, delta_manifest)
            )
            if isinstance(manifest, dict):
                diagnostics.extend(
                    pack_report_delta_target_manifest_diagnostics(
                        manifest,
                        delta_manifest,
                    )
                )
            diagnostics.extend(
                pack_report_delta_asset_set_diagnostics(report, delta_manifest)
            )
    trim_report = report.get("trim_report")
    if isinstance(trim_report, dict):
        trim_report_schema_clean = pack_trim_report_is_schema_clean(trim_report)
        diagnostics.extend(
            pack_trim_report_schema_diagnostics(
                "pack report trim_report",
                trim_report,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
            )
        )
        if report.get("fatal") is False and trim_report_schema_clean:
            diagnostics.extend(
                pack_trim_report_non_fatal_preflight_diagnostics(
                    "pack report trim_report",
                    trim_report,
                )
            )
        if (
            isinstance(manifest, dict)
            and trim_report_schema_clean
            and pack_document_manifest_is_schema_clean(manifest)
        ):
            diagnostics.extend(
                pack_report_trim_manifest_consistency_diagnostics(
                    trim_report,
                    manifest,
                )
            )
    return diagnostics


def pack_delta_asset_list_schema_diagnostics(label: str, value: Any) -> list[str]:
    return pack_string_array_entry_type_schema_diagnostics(label, value)


def pack_string_array_entry_type_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, item in enumerate(value)
        if not isinstance(item, str)
    ]


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
        if zrpack_content_hash(bytes_value[offset:end]) != chunk_hash:
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


def zrpack_content_hash(bytes_value: bytes) -> list[int]:
    hash_bytes = bytearray()
    for seed in ZRPACK_HASH_SEEDS:
        hash_bytes.extend(fnv1a64(bytes_value, seed).to_bytes(8, "little"))
    return list(hash_bytes)


def fnv1a64(bytes_value: bytes, seed: int) -> int:
    value = seed
    for byte in bytes_value:
        value ^= byte
        value = (value * 0x100_0000_01B3) & 0xFFFF_FFFF_FFFF_FFFF
    return value


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
