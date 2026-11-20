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
        _require_uint(metadata, "decoded_bytes", 0, _MAX_DECODED_BYTES, manifest)
        _require_uint(metadata, "created_unix_seconds", 0, 2**64 - 1, manifest)
        _validate_published_payload(stem.with_suffix(".wgsl.zst"), manifest, compressed_bytes)


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
        metadata = json.loads(data.decode("utf-8"))
    except (UnicodeError, ValueError, RecursionError) as error:
        raise RuntimeError(f"shader prewarm cache has invalid v3 manifest ({path}: {error})") from error
    if not isinstance(metadata, Mapping):
        raise RuntimeError(f"shader prewarm cache has invalid v3 manifest ({path})")
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


def _validate_published_payload(path: Path, manifest: Path, expected_bytes: int) -> None:
    try:
        status = path.stat()
        if not stat.S_ISREG(status.st_mode):
            raise RuntimeError(f"shader prewarm cache manifest payload is not a regular file ({path})")
        if status.st_size != expected_bytes:
            raise RuntimeError(f"shader prewarm cache manifest compressed_bytes mismatch ({manifest}: expected {expected_bytes}, found {status.st_size})")
        with path.open("rb") as stream:
            if stream.read(len(_ZSTD_MAGIC)) != _ZSTD_MAGIC:
                raise RuntimeError(f"shader prewarm cache payload is not a Zstd frame ({path})")
    except FileNotFoundError as error:
        raise RuntimeError(f"shader prewarm cache manifest has no immutable WGSL payload ({manifest})") from error
    except OSError as error:
        raise RuntimeError(f"shader prewarm cache payload unavailable ({path}: {error})") from error
