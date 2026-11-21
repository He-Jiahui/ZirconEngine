"""Staged shader cache evidence for the runtime's committed v3 format."""

from __future__ import annotations

import json
import stat
from collections.abc import Mapping
from pathlib import Path

from .zircon_build_shader_prewarm_written_variants import (
    ReportedWrittenVariant,
    validate_cache_hash_shape,
)


# Match ShaderVariantCacheDisk. Runtime also verifies BLAKE3 and decodes WGSL
# before reporting a write. This build gate checks staged layout and evidence.
_SCHEMA_VERSION = 3
_MAX_MANIFEST_BYTES = 64 * 1024
_MAX_COMPRESSED_BYTES = 8 * 1024 * 1024
_MAX_DECODED_BYTES = 32 * 1024 * 1024
_ZSTD_MAGIC = b"\x28\xb5\x2f\xfd"
_MAX_ZSTD_BLOCK_BYTES = 128 * 1024
_MANIFEST_FIELDS = frozenset({
    "schema_version", "hash", "canonical_string", "source_id", "source_hash",
    "template_revision", "naga_version", "wgpu_version", "compressed_bytes",
    "decoded_bytes", "payload_hash", "created_unix_seconds",
})


class _JsonObjectPairs(list[tuple[str, object]]):
    """Distinguish JSON objects from arrays without losing duplicate keys."""


def _reject_non_json_constant(token: str) -> None:
    raise ValueError(f"non-JSON literal {token}")


def validate_v3_shader_cache_artifacts(
    cache_root: Path,
    report: Mapping[str, object],
    variants: tuple[ReportedWrittenVariant, ...],
) -> None:
    # The common report parser already validated the row types and unique hashes.
    source_id_by_hash = {
        row["cache_hash"]: row.get("source_id") for row in report["written_variants"]
    }
    provenance = report.get("source_provenance")
    sources = provenance.get("sources") if isinstance(provenance, Mapping) else None
    for variant in variants:
        cache_hash = variant.cache_hash
        stem = cache_root / f"v{_SCHEMA_VERSION}" / cache_hash[:2] / cache_hash
        manifest = stem.with_suffix(".manifest")
        metadata = _read_committed_manifest(manifest)
        if type(metadata.get("schema_version")) is not int or metadata["schema_version"] != _SCHEMA_VERSION:
            raise RuntimeError(f"shader prewarm cache manifest schema mismatch ({manifest}: expected 3, actual {metadata.get('schema_version')!r})")
        if not variant.matches_metadata(metadata):
            raise RuntimeError("shader prewarm cache reported cache variant mismatch: " + variant.describe_mismatch(metadata))
        source_id = _require_digest(metadata, "source_id", manifest)
        source_hash = _require_digest(metadata, "source_hash", manifest)
        _require_digest(metadata, "payload_hash", manifest)
        if source_id_by_hash[cache_hash] != source_id:
            raise RuntimeError(f"shader prewarm cache manifest source_id mismatch ({manifest})")
        source = sources.get(source_id) if isinstance(sources, Mapping) else None
        if not isinstance(source, Mapping) or variant.source_label is None or any(
            source.get(field) != expected for field, expected in (
                ("source_id", source_id), ("source_hash", source_hash), ("source_label", variant.source_label),
                ("template_revision", variant.template_revision), ("naga_version", variant.naga_version),
                ("wgpu_version", variant.wgpu_version),
            )
        ):
            raise RuntimeError(f"shader prewarm cache manifest source provenance mismatch ({manifest})")
        compressed_bytes = _require_uint(metadata, "compressed_bytes", len(_ZSTD_MAGIC), _MAX_COMPRESSED_BYTES, manifest)
        decoded_bytes = _require_uint(metadata, "decoded_bytes", 0, _MAX_DECODED_BYTES, manifest)
        _require_uint(metadata, "created_unix_seconds", 0, 2**64 - 1, manifest)
        _validate_published_payload(
            stem.with_suffix(".wgsl.zst"), manifest, compressed_bytes, decoded_bytes,
        )


def _read_committed_manifest(path: Path) -> Mapping[str, object]:
    try:
        with path.open("rb") as stream:
            data = stream.read(_MAX_MANIFEST_BYTES + 1)
    except FileNotFoundError as error:
        raise RuntimeError(f"shader prewarm cache missing committed v3 manifest for reported variant ({path})") from error
    except OSError as error:
        raise RuntimeError(f"shader prewarm cache manifest unavailable ({path}: {error})") from error
    if len(data) > _MAX_MANIFEST_BYTES:
        raise RuntimeError(f"shader prewarm cache manifest exceeds byte budget ({path})")
    try:
        pairs = json.loads(
            data.decode("utf-8"),
            object_pairs_hook=_JsonObjectPairs,
            parse_constant=_reject_non_json_constant,
        )
    except (UnicodeError, ValueError, RecursionError) as error:
        raise RuntimeError(f"shader prewarm cache has invalid v3 manifest ({path}: {error})") from error
    if not isinstance(pairs, _JsonObjectPairs):
        raise RuntimeError(f"shader prewarm cache has invalid v3 manifest ({path})")
    # Serde rejects duplicate fields of the runtime struct and ignores unknown fields.
    metadata: dict[str, object] = {}
    for field, value in pairs:
        if field not in _MANIFEST_FIELDS:
            continue
        if field in metadata:
            raise RuntimeError(f"shader prewarm cache has duplicate manifest field {field!r} ({path})")
        metadata[field] = value
    return metadata


def _require_digest(metadata: Mapping[str, object], field: str, path: Path) -> str:
    value = metadata.get(field)
    if not isinstance(value, str):
        raise RuntimeError(f"shader prewarm cache manifest has invalid {field} ({path})")
    validate_cache_hash_shape(value, source=f"manifest={path} field={field}")
    return value


def _require_uint(metadata: Mapping[str, object], field: str, lower: int, upper: int, path: Path) -> int:
    value = metadata.get(field)
    if type(value) is not int or not lower <= value <= upper:
        raise RuntimeError(f"shader prewarm cache manifest has invalid {field} ({path})")
    return value


def _validate_published_payload(
    path: Path, manifest: Path, compressed_bytes: int, decoded_bytes: int,
) -> None:
    def require_compressed_size(actual: int) -> None:
        if actual != compressed_bytes:
            raise RuntimeError(
                f"shader prewarm cache manifest compressed_bytes mismatch "
                f"({manifest}: expected {compressed_bytes}, found {actual})"
            )

    try:
        status = path.stat()
        if not stat.S_ISREG(status.st_mode):
            raise RuntimeError(f"shader prewarm cache manifest payload is not a regular file ({path})")
        require_compressed_size(status.st_size)
        with path.open("rb") as stream:
            data = stream.read(compressed_bytes + 1)
    except FileNotFoundError as error:
        raise RuntimeError(f"shader prewarm cache manifest has no immutable WGSL payload ({manifest})") from error
    except OSError as error:
        raise RuntimeError(f"shader prewarm cache payload unavailable ({path}: {error})") from error

    require_compressed_size(len(data))
    if not data.startswith(_ZSTD_MAGIC):
        raise RuntimeError(f"shader prewarm cache payload is not a Zstd frame ({path})")
    try:
        _check_zstd_frame(data, decoded_bytes)
    except ValueError as error:
        raise RuntimeError(f"shader prewarm cache payload has invalid Zstd frame ({path}: {error})") from error


def _check_zstd_frame(data: bytes, decoded_bytes: int) -> None:
    # RFC 8878 framing only: the producer writes one non-dictionary frame.
    # Streaming frames may omit content size. No block decode or BLAKE3 here.
    offset = len(_ZSTD_MAGIC)
    if offset == len(data):
        raise ValueError("missing frame header")
    descriptor = data[offset]
    if descriptor & 0x08:
        raise ValueError("reserved frame header bit")
    single_segment = bool(descriptor & 0x20)
    checksum_size = 4 if descriptor & 0x04 else 0
    dictionary_size = (0, 1, 2, 4)[descriptor & 3]
    size_flag = descriptor >> 6
    size_width = (1 if single_segment else 0, 2, 4, 8)[size_flag]
    offset += 1 if single_segment else 2  # Descriptor, then optional window descriptor.
    header_end = offset + dictionary_size + size_width
    if header_end > len(data):
        raise ValueError("truncated frame header")
    if any(data[offset:offset + dictionary_size]):
        raise ValueError("frame requires a dictionary")
    if size_width:
        content_size = int.from_bytes(data[header_end - size_width:header_end], "little")
        if size_width == 2:
            content_size += 256
        if content_size != decoded_bytes:
            raise ValueError("frame content size disagrees with manifest decoded_bytes")

    offset = header_end
    blocks_end = len(data) - checksum_size
    while True:
        if offset + 3 > blocks_end:
            raise ValueError("missing block header")
        block = int.from_bytes(data[offset:offset + 3], "little")
        block_type = (block >> 1) & 3
        block_size = block >> 3
        if block_type == 3:
            raise ValueError("reserved block type")
        if block_size > _MAX_ZSTD_BLOCK_BYTES:
            raise ValueError("block exceeds Zstd size limit")
        offset += 3 + (1 if block_type == 1 else block_size)  # RLE stores one byte.
        if offset > blocks_end:
            raise ValueError("truncated block or frame checksum")
        if block & 1:
            if offset != blocks_end:
                raise ValueError("trailing bytes after last block")
            return
