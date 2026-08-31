"""Versioned display-output image oracle for Zircon PBR evidence."""

from __future__ import annotations

import hashlib
import json
import math
import struct
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping


_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
_DISPLAY_VISUAL_ORACLE_SCHEMA = "zircon_pbr_display_visual_oracle_v1"
_CURRENT_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v17"
_CURRENT_MATERIAL_PIPELINE_PROVENANCE = {
    "metal-mirror": {
        "required_material_base_pipeline_kind": "environment-only-pbr-base",
        "required_material_base_pipeline_ready_at_capture": "true",
        "environment_only_base_prewarm_requested": "true",
    },
    "dielectric-ior": {
        "required_material_base_pipeline_kind": "generic-forward-pbr-ior",
        "required_material_base_pipeline_ready_at_capture": "true",
        "environment_only_base_prewarm_requested": "false",
    },
}
_MAX_ENCODED_PNG_BYTES = 64 * 1024 * 1024
_MAX_ORACLE_BYTES = 256 * 1024
_MAX_VIEWPORT_DIMENSION = 16_384
_MAX_PIXEL_BYTES = 256 * 1024 * 1024
_MAX_PNG_CHUNKS = 4_096
_MAX_SEMANTIC_REGIONS = 64


@dataclass(frozen=True)
class DecodedRgbaPng:
    width: int
    height: int
    rgba: bytes


@dataclass(frozen=True)
class DisplayVisualOracleResult:
    oracle_path: Path
    oracle_sha256: str
    reference_png_path: Path
    reference_png_sha256: str
    compared_pixel_count: int
    mean_abs_error: float
    p99_abs_error: int
    exceeding_pixel_fraction: float
    semantic_region_mean_abs_errors: Mapping[str, float]


def validate_display_visual_oracle(
    candidate_png_path: str | Path,
    *,
    metadata: Mapping[str, str],
    oracle_path: str | Path,
    _candidate_image: DecodedRgbaPng | None = None,
) -> DisplayVisualOracleResult:
    """Compare a ready-frame display image with one provenance-bound oracle."""

    candidate_png_path = Path(candidate_png_path)
    oracle_path = Path(oracle_path)
    encoded_oracle = _read_bounded_file(
        oracle_path,
        maximum_bytes=_MAX_ORACLE_BYTES,
        label="display visual oracle",
    )
    oracle = _parse_oracle(encoded_oracle, oracle_path)
    _validate_current_material_provenance(
        oracle["expected_metadata"], metadata, oracle_path
    )
    _validate_expected_metadata(oracle["expected_metadata"], metadata, oracle_path)
    reference_png_path = _resolve_reference_png(oracle["reference_png"], oracle_path)
    actual_reference_hash = _sha256_file(reference_png_path)
    if actual_reference_hash != oracle["reference_png_sha256"]:
        raise RuntimeError(
            "display visual oracle reference PNG hash does not match manifest: "
            f"expected={oracle['reference_png_sha256']} actual={actual_reference_hash} "
            f"path={reference_png_path}"
        )

    candidate = _candidate_image or decode_rgba_png(candidate_png_path)
    reference = decode_rgba_png(reference_png_path)
    if (candidate.width, candidate.height) != (reference.width, reference.height):
        raise RuntimeError(
            "display visual oracle reference dimensions do not match candidate: "
            f"reference={reference.width}x{reference.height} "
            f"candidate={candidate.width}x{candidate.height} path={candidate_png_path}"
        )

    comparison = oracle["comparison"]
    metrics = _compare_rgb(candidate.rgba, reference.rgba, comparison)
    if metrics.mean_abs_error > comparison["max_mean_abs_error"]:
        raise RuntimeError(
            "display visual oracle mean absolute error exceeds threshold: "
            f"actual={metrics.mean_abs_error:.6f} "
            f"maximum={comparison['max_mean_abs_error']:.6f} path={candidate_png_path}"
        )
    if metrics.p99_abs_error > comparison["max_p99_abs_error"]:
        raise RuntimeError(
            "display visual oracle p99 absolute error exceeds threshold: "
            f"actual={metrics.p99_abs_error} maximum={comparison['max_p99_abs_error']} "
            f"path={candidate_png_path}"
        )
    if metrics.exceeding_pixel_fraction > comparison["max_exceeding_pixel_fraction"]:
        raise RuntimeError(
            "display visual oracle high-error pixel fraction exceeds threshold: "
            f"actual={metrics.exceeding_pixel_fraction:.6f} "
            f"maximum={comparison['max_exceeding_pixel_fraction']:.6f} path={candidate_png_path}"
        )

    semantic_region_errors = _validate_semantic_regions(
        candidate,
        reference,
        oracle["semantic_regions"],
        oracle_path,
    )
    return DisplayVisualOracleResult(
        oracle_path=oracle_path,
        oracle_sha256=hashlib.sha256(encoded_oracle).hexdigest(),
        reference_png_path=reference_png_path,
        reference_png_sha256=actual_reference_hash,
        compared_pixel_count=metrics.compared_pixel_count,
        mean_abs_error=metrics.mean_abs_error,
        p99_abs_error=metrics.p99_abs_error,
        exceeding_pixel_fraction=metrics.exceeding_pixel_fraction,
        semantic_region_mean_abs_errors=semantic_region_errors,
    )


@dataclass(frozen=True)
class _ComparisonMetrics:
    compared_pixel_count: int
    mean_abs_error: float
    p99_abs_error: int
    exceeding_pixel_fraction: float


def _parse_oracle(encoded: bytes, oracle_path: Path) -> dict[str, object]:
    try:
        decoded = json.loads(encoded.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f"display visual oracle is not valid UTF-8 JSON: {oracle_path}") from error
    if not isinstance(decoded, dict):
        raise RuntimeError(f"display visual oracle must be a JSON object: {oracle_path}")
    expected_keys = {
        "schema",
        "reference_png",
        "reference_png_sha256",
        "expected_metadata",
        "comparison",
        "semantic_regions",
    }
    unknown_keys = set(decoded).difference(expected_keys)
    missing_keys = expected_keys.difference(decoded)
    if missing_keys or unknown_keys:
        raise RuntimeError(
            "display visual oracle has an invalid field set: "
            f"missing={','.join(sorted(missing_keys)) or 'none'} "
            f"unknown={','.join(sorted(unknown_keys)) or 'none'} path={oracle_path}"
        )
    if decoded["schema"] != _DISPLAY_VISUAL_ORACLE_SCHEMA:
        raise RuntimeError(
            "display visual oracle schema is unsupported: "
            f"actual={decoded['schema']!r} path={oracle_path}"
        )
    _require_relative_path(decoded["reference_png"], "reference_png", oracle_path)
    _require_sha256(decoded["reference_png_sha256"], "reference_png_sha256", oracle_path)
    _validate_oracle_metadata(decoded["expected_metadata"], oracle_path)
    _validate_comparison(decoded["comparison"], oracle_path)
    _validate_semantic_region_shape(decoded["semantic_regions"], oracle_path)
    return decoded


def _require_relative_path(value: object, field: str, oracle_path: Path) -> None:
    if not isinstance(value, str) or not value:
        raise RuntimeError(f"display visual oracle {field} must be a non-empty string: {oracle_path}")
    path = Path(value)
    if path.is_absolute() or any(part == ".." for part in path.parts):
        raise RuntimeError(
            f"display visual oracle {field} must stay below the oracle directory: {oracle_path}"
        )


def _require_sha256(value: object, field: str, oracle_path: Path) -> None:
    if not isinstance(value, str) or len(value) != 64 or any(
        character not in "0123456789abcdef" for character in value
    ):
        raise RuntimeError(f"display visual oracle {field} must be a lowercase SHA-256: {oracle_path}")


def _validate_oracle_metadata(value: object, oracle_path: Path) -> None:
    if not isinstance(value, dict) or not value:
        raise RuntimeError(
            f"display visual oracle expected_metadata must be a non-empty object: {oracle_path}"
        )
    for key, expected in value.items():
        if not isinstance(key, str) or not key or not isinstance(expected, str) or not expected:
            raise RuntimeError(
                "display visual oracle expected_metadata must contain non-empty string pairs: "
                f"path={oracle_path}"
            )


def _validate_current_material_provenance(
    expected_metadata: object,
    candidate_metadata: Mapping[str, str],
    oracle_path: Path,
) -> None:
    if candidate_metadata.get("schema") != _CURRENT_READY_FRAME_EVIDENCE_SCHEMA:
        return
    assert isinstance(expected_metadata, dict)
    material_fixture = expected_metadata.get("material_fixture")
    expected_pipeline = _CURRENT_MATERIAL_PIPELINE_PROVENANCE.get(material_fixture)
    if expected_pipeline is None:
        raise RuntimeError(
            "display visual oracle must bind current material provenance: "
            f"field=material_fixture path={oracle_path}"
        )
    for field, expected_value in expected_pipeline.items():
        if expected_metadata.get(field) != expected_value:
            raise RuntimeError(
                "display visual oracle must bind current material provenance: "
                f"field={field} expected={expected_value!r} path={oracle_path}"
            )


def _validate_comparison(value: object, oracle_path: Path) -> None:
    if not isinstance(value, dict):
        raise RuntimeError(f"display visual oracle comparison must be an object: {oracle_path}")
    required_fields = {
        "max_mean_abs_error",
        "max_p99_abs_error",
        "exceeding_abs_error",
        "max_exceeding_pixel_fraction",
    }
    if set(value) != required_fields:
        raise RuntimeError(f"display visual oracle comparison has an invalid field set: {oracle_path}")
    for field in ("max_mean_abs_error", "max_p99_abs_error", "exceeding_abs_error"):
        number = _require_finite_number(value[field], field, oracle_path)
        if number < 0.0 or number > 255.0:
            raise RuntimeError(
                f"display visual oracle {field} must be in the RGBA8 range: {oracle_path}"
            )
    fraction = _require_finite_number(
        value["max_exceeding_pixel_fraction"],
        "max_exceeding_pixel_fraction",
        oracle_path,
    )
    if fraction < 0.0 or fraction > 1.0:
        raise RuntimeError(
            "display visual oracle max_exceeding_pixel_fraction must be in [0, 1]: "
            f"{oracle_path}"
        )


def _validate_semantic_region_shape(value: object, oracle_path: Path) -> None:
    if not isinstance(value, list) or len(value) > _MAX_SEMANTIC_REGIONS:
        raise RuntimeError(
            "display visual oracle semantic_regions must be a bounded array: "
            f"path={oracle_path}"
        )
    region_ids: set[str] = set()
    expected_fields = {"id", "x", "y", "width", "height", "max_mean_abs_error"}
    for region in value:
        if not isinstance(region, dict) or set(region) != expected_fields:
            raise RuntimeError(
                "display visual oracle semantic region has an invalid field set: "
                f"path={oracle_path}"
            )
        identifier = region["id"]
        if not isinstance(identifier, str) or not identifier or identifier in region_ids:
            raise RuntimeError(
                "display visual oracle semantic region IDs must be non-empty and unique: "
                f"path={oracle_path}"
            )
        region_ids.add(identifier)
        for field in ("x", "y", "width", "height"):
            integer = region[field]
            if not isinstance(integer, int) or isinstance(integer, bool) or integer < 0:
                raise RuntimeError(
                    f"display visual oracle semantic region {field} must be a non-negative integer: "
                    f"path={oracle_path}"
                )
        if region["width"] == 0 or region["height"] == 0:
            raise RuntimeError(
                "display visual oracle semantic region dimensions must be non-zero: "
                f"path={oracle_path}"
            )
        threshold = _require_finite_number(region["max_mean_abs_error"], "max_mean_abs_error", oracle_path)
        if threshold < 0.0 or threshold > 255.0:
            raise RuntimeError(
                "display visual oracle semantic region max_mean_abs_error must be in the RGBA8 range: "
                f"path={oracle_path}"
            )


def _require_finite_number(value: object, field: str, oracle_path: Path) -> float:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not math.isfinite(value):
        raise RuntimeError(f"display visual oracle {field} must be a finite number: {oracle_path}")
    return float(value)


def _validate_expected_metadata(
    expected_metadata: object,
    actual_metadata: Mapping[str, str],
    oracle_path: Path,
) -> None:
    assert isinstance(expected_metadata, dict)
    for field, expected in expected_metadata.items():
        actual = actual_metadata.get(field)
        if actual != expected:
            raise RuntimeError(
                "display visual oracle provenance does not match manifest: "
                f"field={field} expected={expected!r} actual={actual!r} path={oracle_path}"
            )


def _resolve_reference_png(reference_png: object, oracle_path: Path) -> Path:
    assert isinstance(reference_png, str)
    root = oracle_path.parent.resolve()
    candidate = (root / reference_png).resolve()
    try:
        candidate.relative_to(root)
    except ValueError as error:
        raise RuntimeError(
            "display visual oracle reference PNG escapes the oracle directory: "
            f"path={oracle_path}"
        ) from error
    return candidate


def _compare_rgb(candidate_rgba: bytes, reference_rgba: bytes, comparison: object) -> _ComparisonMetrics:
    assert isinstance(comparison, dict)
    if len(candidate_rgba) != len(reference_rgba) or len(candidate_rgba) % 4:
        raise RuntimeError("display visual oracle decoded RGBA buffers have incompatible lengths")
    histogram = [0] * 256
    total_abs_error = 0
    exceeding_pixels = 0
    pixel_count = len(candidate_rgba) // 4
    threshold = float(comparison["exceeding_abs_error"])
    for offset in range(0, len(candidate_rgba), 4):
        pixel_max_error = 0
        for channel in range(3):
            error = abs(candidate_rgba[offset + channel] - reference_rgba[offset + channel])
            total_abs_error += error
            histogram[error] += 1
            pixel_max_error = max(pixel_max_error, error)
        if pixel_max_error > threshold:
            exceeding_pixels += 1
    component_count = pixel_count * 3
    p99_rank = max(1, math.ceil(component_count * 0.99))
    cumulative = 0
    p99_abs_error = 0
    for error, count in enumerate(histogram):
        cumulative += count
        if cumulative >= p99_rank:
            p99_abs_error = error
            break
    return _ComparisonMetrics(
        compared_pixel_count=pixel_count,
        mean_abs_error=total_abs_error / component_count,
        p99_abs_error=p99_abs_error,
        exceeding_pixel_fraction=exceeding_pixels / pixel_count,
    )


def _validate_semantic_regions(
    candidate: DecodedRgbaPng,
    reference: DecodedRgbaPng,
    semantic_regions: object,
    oracle_path: Path,
) -> dict[str, float]:
    assert isinstance(semantic_regions, list)
    errors: dict[str, float] = {}
    for region in semantic_regions:
        assert isinstance(region, dict)
        x = int(region["x"])
        y = int(region["y"])
        width = int(region["width"])
        height = int(region["height"])
        if x + width > candidate.width or y + height > candidate.height:
            raise RuntimeError(
                "display visual oracle semantic region exceeds the reference dimensions: "
                f"id={region['id']} reference={candidate.width}x{candidate.height} path={oracle_path}"
            )
        total_abs_error = 0
        component_count = width * height * 3
        for row in range(y, y + height):
            row_offset = row * candidate.width * 4
            for column in range(x, x + width):
                offset = row_offset + column * 4
                total_abs_error += sum(
                    abs(candidate.rgba[offset + channel] - reference.rgba[offset + channel])
                    for channel in range(3)
                )
        mean_abs_error = total_abs_error / component_count
        identifier = str(region["id"])
        maximum = float(region["max_mean_abs_error"])
        if mean_abs_error > maximum:
            raise RuntimeError(
                "display visual oracle semantic region mean absolute error exceeds threshold: "
                f"id={identifier} actual={mean_abs_error:.6f} maximum={maximum:.6f} "
                f"path={oracle_path}"
            )
        errors[identifier] = mean_abs_error
    return errors


def decode_rgba_png(path: str | Path) -> DecodedRgbaPng:
    """Decode a bounded, non-interlaced RGBA8 PNG without image-library dependencies."""

    path = Path(path)
    encoded = _read_bounded_file(path, maximum_bytes=_MAX_ENCODED_PNG_BYTES, label="PNG")
    if not encoded.startswith(_PNG_SIGNATURE):
        raise RuntimeError(f"ready-frame evidence is not a PNG: {path}")
    chunks = _png_chunks(encoded, path)
    if not chunks or chunks[0][0] != b"IHDR":
        raise RuntimeError(f"ready-frame PNG is missing IHDR: {path}")
    ihdr = encoded[chunks[0][1] : chunks[0][2]]
    if len(ihdr) != 13:
        raise RuntimeError(f"ready-frame PNG has an invalid IHDR: {path}")
    width, height, bit_depth, color_type, compression, filtering, interlace = struct.unpack(
        ">IIBBBBB", ihdr
    )
    if (
        not width
        or not height
        or width > _MAX_VIEWPORT_DIMENSION
        or height > _MAX_VIEWPORT_DIMENSION
        or bit_depth != 8
        or color_type != 6
        or compression != 0
        or filtering != 0
        or interlace != 0
    ):
        raise RuntimeError(f"ready-frame PNG must be a bounded non-interlaced RGBA8 image: path={path}")
    if chunks[-1][0] != b"IEND" or chunks[-1][1] != chunks[-1][2]:
        raise RuntimeError(f"ready-frame PNG is missing a terminal IEND: {path}")
    row_bytes = width * 4
    expected_bytes = (row_bytes + 1) * height
    if expected_bytes > _MAX_PIXEL_BYTES:
        raise RuntimeError(f"ready-frame PNG exceeds the evidence pixel budget: {path}")
    raw = _decompress_idat_chunks(encoded, chunks, expected_bytes, path)
    return DecodedRgbaPng(width=width, height=height, rgba=_unfilter_rgba(raw, width, height, path))


def rgba_statistics(
    image: DecodedRgbaPng,
    *,
    max_reported_distinct_colors: int = 4_096,
) -> tuple[int, int]:
    """Return distinct non-transparent colors (capped) and non-black pixel count."""

    colors: set[tuple[int, int, int, int]] = set()
    non_black_pixels = 0
    for offset in range(0, len(image.rgba), 4):
        pixel = tuple(image.rgba[offset : offset + 4])
        if pixel[3] == 0:
            continue
        if len(colors) <= max_reported_distinct_colors:
            colors.add(pixel)
        if pixel[0] or pixel[1] or pixel[2]:
            non_black_pixels += 1
    return len(colors), non_black_pixels


def _read_bounded_file(path: Path, *, maximum_bytes: int, label: str) -> bytes:
    try:
        with path.open("rb") as source:
            encoded = source.read(maximum_bytes + 1)
    except OSError as error:
        raise RuntimeError(f"{label} is unavailable: {path}") from error
    if len(encoded) > maximum_bytes:
        raise RuntimeError(f"{label} exceeds the encoded evidence budget: {path}")
    return encoded


def _sha256_file(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise RuntimeError(f"display visual oracle reference PNG is unavailable: {path}") from error


def _png_chunks(encoded: bytes, path: Path) -> list[tuple[bytes, int, int]]:
    chunks: list[tuple[bytes, int, int]] = []
    offset = len(_PNG_SIGNATURE)
    while offset < len(encoded):
        if len(encoded) - offset < 12:
            raise RuntimeError(f"ready-frame PNG is truncated: {path}")
        length = struct.unpack(">I", encoded[offset : offset + 4])[0]
        kind = encoded[offset + 4 : offset + 8]
        payload_start = offset + 8
        payload_end = payload_start + length
        crc_end = payload_end + 4
        if crc_end > len(encoded):
            raise RuntimeError(f"ready-frame PNG chunk is truncated: {path}")
        expected_crc = struct.unpack(">I", encoded[payload_end:crc_end])[0]
        actual_crc = zlib.crc32(kind)
        actual_crc = zlib.crc32(memoryview(encoded)[payload_start:payload_end], actual_crc)
        if actual_crc & 0xFFFFFFFF != expected_crc:
            raise RuntimeError(f"ready-frame PNG chunk checksum is invalid: {path}")
        if len(chunks) >= _MAX_PNG_CHUNKS:
            raise RuntimeError(f"ready-frame PNG exceeds the chunk budget: {path}")
        chunks.append((kind, payload_start, payload_end))
        offset = crc_end
        if kind == b"IEND":
            if offset != len(encoded):
                raise RuntimeError(f"ready-frame PNG has trailing data: {path}")
            break
    return chunks


def _decompress_idat_chunks(
    encoded: bytes,
    chunks: list[tuple[bytes, int, int]],
    expected_bytes: int,
    path: Path,
) -> bytes:
    decompressor = zlib.decompressobj()
    raw = bytearray()
    idat_count = 0
    for kind, payload_start, payload_end in chunks:
        if kind != b"IDAT":
            continue
        idat_count += 1
        decoded = decompressor.decompress(
            memoryview(encoded)[payload_start:payload_end], expected_bytes - len(raw) + 1
        )
        raw.extend(decoded)
        if len(raw) > expected_bytes or decompressor.unconsumed_tail:
            raise RuntimeError(f"ready-frame PNG image data has an invalid size: {path}")
    if not idat_count or len(raw) != expected_bytes or not decompressor.eof or decompressor.unused_data:
        raise RuntimeError(f"ready-frame PNG image data has an invalid size: {path}")
    return bytes(raw)


def _unfilter_rgba(raw: bytes, width: int, height: int, path: Path) -> bytes:
    row_bytes = width * 4
    previous = bytearray(row_bytes)
    pixels = bytearray(width * height * 4)
    source_offset = 0
    destination_offset = 0
    for _row_index in range(height):
        filter_type = raw[source_offset]
        source_offset += 1
        row = _unfilter_rgba_row(
            filter_type,
            raw[source_offset : source_offset + row_bytes],
            previous,
            path,
        )
        source_offset += row_bytes
        pixels[destination_offset : destination_offset + row_bytes] = row
        destination_offset += row_bytes
        previous = row
    return bytes(pixels)


def _unfilter_rgba_row(filter_type: int, encoded_row: bytes, previous: bytearray, path: Path) -> bytearray:
    row = bytearray(encoded_row)
    if filter_type == 0:
        return row
    if filter_type not in (1, 2, 3, 4):
        raise RuntimeError(f"ready-frame PNG uses an unsupported scanline filter: filter={filter_type} path={path}")
    for index, value in enumerate(row):
        left = row[index - 4] if index >= 4 else 0
        up = previous[index]
        if filter_type == 1:
            row[index] = (value + left) & 0xFF
        elif filter_type == 2:
            row[index] = (value + up) & 0xFF
        elif filter_type == 3:
            row[index] = (value + ((left + up) // 2)) & 0xFF
        else:
            up_left = previous[index - 4] if index >= 4 else 0
            row[index] = (value + _paeth_predictor(left, up, up_left)) & 0xFF
    return row


def _paeth_predictor(left: int, up: int, up_left: int) -> int:
    estimate = left + up - up_left
    left_distance = abs(estimate - left)
    up_distance = abs(estimate - up)
    up_left_distance = abs(estimate - up_left)
    if left_distance <= up_distance and left_distance <= up_left_distance:
        return left
    if up_distance <= up_left_distance:
        return up
    return up_left
