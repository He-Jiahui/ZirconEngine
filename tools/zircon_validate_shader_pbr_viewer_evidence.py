"""Validate one Zircon PBR viewer ready-frame PNG and its provenance sidecar."""

from __future__ import annotations

import argparse
import json
import math
import re
import struct
import sys
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping


_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
_READY_FRAME_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_ready_frame_evidence_v2"
_PROCESS_LOCAL_MESH_PIPELINE_CACHE = "process_local_mesh_pipeline_cache"
_MAX_VIEWPORT_DIMENSION = 16_384
_MAX_ENCODED_PNG_BYTES = 64 * 1024 * 1024
_MAX_PIXEL_BYTES = 256 * 1024 * 1024
_MAX_PNG_CHUNKS = 4_096
_MAX_REPORTED_DISTINCT_COLORS = 4_096
_VIEWPORT_PATTERN = re.compile(r"([1-9][0-9]*)x([1-9][0-9]*)\Z")
_POSITIVE_INTEGER_PATTERN = re.compile(r"[1-9][0-9]*\Z")
_NON_NEGATIVE_INTEGER_PATTERN = re.compile(r"[0-9]+\Z")
_REQUIRED_METADATA_FIELDS = (
    "schema",
    "screenshot",
    "screenshot_presentation",
    "interactive_direct_present_enabled",
    "backend",
    "hdri_path",
    "requested_source_face_size",
    "requested_pmrem_face_size",
    "active_source_cubemap_face_size",
    "active_source_cubemap_mip_count",
    "active_pmrem_face_size",
    "active_pmrem_mip_count",
    "render_profile",
    "environment_only_base_prewarm_cache_hit",
    "environment_only_base_prewarm_cache_scope",
    "environment_only_base_prewarm_shader_source_resolution_ns",
    "environment_only_base_prewarm_pipeline_creation_ns",
    "environment_only_base_prewarm_elapsed_ns",
    "viewport",
    "camera_yaw_degrees",
    "camera_pitch_degrees",
    "ibl_bake_algorithm_version",
    "ibl_staging_status",
    "ibl_staging_elapsed_ns",
    "ibl_total_elapsed_ns",
    "ready_frame_render_elapsed_ns",
    "ready_frame_extract_ns",
    "ready_frame_renderer_call_ns",
    "ready_frame_readback_and_completion_ns",
)


@dataclass(frozen=True)
class ReadyFrameEvidence:
    screenshot_path: Path
    sidecar_path: Path
    viewport: tuple[int, int]
    backend: str
    render_profile: str
    distinct_rgba_colors: int
    non_black_pixel_count: int
    metadata: Mapping[str, str]


def validate_ready_frame_evidence(
    png_path: str | Path,
    *,
    expected_backend: str | None = None,
    require_direct_present: bool = False,
    min_distinct_rgba_colors: int = 2,
    min_non_black_pixels: int = 1,
) -> ReadyFrameEvidence:
    """Validate a ready-frame PNG/sidecar pair without starting the engine."""

    if min_distinct_rgba_colors < 2 or min_non_black_pixels < 1:
        raise ValueError("visual evidence thresholds must be positive and distinguish colors")
    screenshot_path = Path(png_path)
    width, height, distinct_colors, non_black_pixels = _inspect_rgba_png(screenshot_path)
    sidecar_path = screenshot_path.with_name(f"{screenshot_path.name}.txt")
    metadata = _read_metadata(sidecar_path)
    _validate_metadata(
        metadata,
        screenshot_path=screenshot_path,
        width=width,
        height=height,
        expected_backend=expected_backend,
        require_direct_present=require_direct_present,
    )
    if (
        distinct_colors < min_distinct_rgba_colors
        or non_black_pixels < min_non_black_pixels
    ):
        raise RuntimeError(
            "ready-frame PNG is visually insufficient: "
            f"distinct_rgba_colors={distinct_colors} "
            f"non_black_pixels={non_black_pixels} path={screenshot_path}"
        )
    return ReadyFrameEvidence(
        screenshot_path=screenshot_path,
        sidecar_path=sidecar_path,
        viewport=(width, height),
        backend=metadata["backend"],
        render_profile=metadata["render_profile"],
        distinct_rgba_colors=distinct_colors,
        non_black_pixel_count=non_black_pixels,
        metadata=metadata,
    )


def _read_metadata(sidecar_path: Path) -> dict[str, str]:
    try:
        text = sidecar_path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"ready-frame provenance sidecar is unavailable: {sidecar_path}") from error
    metadata: dict[str, str] = {}
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line:
            continue
        key, separator, value = line.partition("=")
        if not separator or not key or key != key.strip():
            raise RuntimeError(
                "ready-frame provenance sidecar contains an invalid field: "
                f"line={line_number} path={sidecar_path}"
            )
        if key in metadata:
            raise RuntimeError(
                "ready-frame provenance sidecar repeats a field: "
                f"field={key} path={sidecar_path}"
            )
        metadata[key] = value
    return metadata


def _validate_metadata(
    metadata: Mapping[str, str],
    *,
    screenshot_path: Path,
    width: int,
    height: int,
    expected_backend: str | None,
    require_direct_present: bool,
) -> None:
    missing = [field for field in _REQUIRED_METADATA_FIELDS if field not in metadata]
    if missing:
        raise RuntimeError(
            "ready-frame provenance sidecar is missing required fields: "
            f"{', '.join(missing)} path={screenshot_path}"
        )
    if metadata["schema"] != _READY_FRAME_EVIDENCE_SCHEMA:
        raise RuntimeError(
            "ready-frame provenance schema is unsupported: "
            f"schema={metadata['schema']} path={screenshot_path}"
        )
    if metadata["screenshot"] != screenshot_path.name:
        raise RuntimeError(
            "ready-frame provenance screenshot name does not match PNG: "
            f"sidecar={metadata['screenshot']} png={screenshot_path.name}"
        )
    if metadata["screenshot_presentation"] != "cpu_readback":
        raise RuntimeError(
            "ready-frame provenance must identify the CPU readback capture path: "
            f"path={screenshot_path}"
        )
    _require_boolean(metadata, "interactive_direct_present_enabled", screenshot_path)
    _require_boolean(metadata, "environment_only_base_prewarm_cache_hit", screenshot_path)
    if require_direct_present and metadata["interactive_direct_present_enabled"] != "true":
        raise RuntimeError(
            "ready-frame provenance requires the interactive direct-present path: "
            f"path={screenshot_path}"
        )
    _require_nonempty(metadata, "backend", screenshot_path)
    _require_nonempty(metadata, "hdri_path", screenshot_path)
    _require_nonempty(metadata, "render_profile", screenshot_path)
    _require_nonempty(metadata, "ibl_staging_status", screenshot_path)
    if expected_backend is not None and metadata["backend"] != expected_backend:
        raise RuntimeError(
            "ready-frame provenance backend does not match expectation: "
            f"expected={expected_backend} actual={metadata['backend']} path={screenshot_path}"
        )
    if metadata["render_profile"] != "environment_only_pbr_preview":
        raise RuntimeError(
            "ready-frame provenance must use the environment-only PBR profile: "
            f"profile={metadata['render_profile']} path={screenshot_path}"
        )
    if (
        metadata["environment_only_base_prewarm_cache_scope"]
        != _PROCESS_LOCAL_MESH_PIPELINE_CACHE
    ):
        raise RuntimeError(
            "ready-frame provenance cache scope is not the process-local MeshPipelineCache: "
            f"path={screenshot_path}"
        )
    _require_face_size(metadata, "requested_source_face_size", screenshot_path)
    _require_face_size(metadata, "requested_pmrem_face_size", screenshot_path)
    for field in (
        "active_source_cubemap_face_size",
        "active_source_cubemap_mip_count",
        "active_pmrem_face_size",
        "active_pmrem_mip_count",
        "ibl_bake_algorithm_version",
    ):
        _require_positive_integer(metadata, field, screenshot_path)
    _require_complete_cubemap_mip_chain(
        metadata,
        face_size_field="active_source_cubemap_face_size",
        mip_count_field="active_source_cubemap_mip_count",
        screenshot_path=screenshot_path,
    )
    _require_complete_cubemap_mip_chain(
        metadata,
        face_size_field="active_pmrem_face_size",
        mip_count_field="active_pmrem_mip_count",
        screenshot_path=screenshot_path,
    )
    for field in (
        "environment_only_base_prewarm_shader_source_resolution_ns",
        "environment_only_base_prewarm_pipeline_creation_ns",
        "environment_only_base_prewarm_elapsed_ns",
        "ibl_staging_elapsed_ns",
        "ibl_total_elapsed_ns",
        "ready_frame_render_elapsed_ns",
        "ready_frame_extract_ns",
        "ready_frame_renderer_call_ns",
        "ready_frame_readback_and_completion_ns",
    ):
        _require_non_negative_integer(metadata, field, screenshot_path)
    _require_duration_hierarchy(
        metadata,
        total_field="environment_only_base_prewarm_elapsed_ns",
        component_fields=(
            "environment_only_base_prewarm_shader_source_resolution_ns",
            "environment_only_base_prewarm_pipeline_creation_ns",
        ),
        screenshot_path=screenshot_path,
    )
    _require_duration_hierarchy(
        metadata,
        total_field="ibl_total_elapsed_ns",
        component_fields=("ibl_staging_elapsed_ns",),
        screenshot_path=screenshot_path,
    )
    _require_duration_hierarchy(
        metadata,
        total_field="ready_frame_render_elapsed_ns",
        component_fields=(
            "ready_frame_extract_ns",
            "ready_frame_renderer_call_ns",
            "ready_frame_readback_and_completion_ns",
        ),
        screenshot_path=screenshot_path,
    )
    for field in ("camera_yaw_degrees", "camera_pitch_degrees"):
        _require_finite_float(metadata, field, screenshot_path)
    viewport_match = _VIEWPORT_PATTERN.fullmatch(metadata["viewport"])
    if viewport_match is None:
        raise RuntimeError(
            "ready-frame provenance viewport is malformed: "
            f"viewport={metadata['viewport']} path={screenshot_path}"
        )
    viewport = (int(viewport_match.group(1)), int(viewport_match.group(2)))
    if viewport != (width, height):
        raise RuntimeError(
            "ready-frame provenance viewport does not match PNG dimensions: "
            f"sidecar={viewport[0]}x{viewport[1]} png={width}x{height} path={screenshot_path}"
        )


def _require_boolean(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    if metadata[field] not in ("true", "false"):
        raise RuntimeError(
            "ready-frame provenance boolean is malformed: "
            f"field={field} path={screenshot_path}"
        )


def _require_nonempty(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    value = metadata[field]
    if not value or value != value.strip():
        raise RuntimeError(
            "ready-frame provenance value is blank or padded: "
            f"field={field} path={screenshot_path}"
        )


def _require_face_size(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    if metadata[field] != "automatic":
        _require_positive_integer(metadata, field, screenshot_path)


def _require_complete_cubemap_mip_chain(
    metadata: Mapping[str, str],
    *,
    face_size_field: str,
    mip_count_field: str,
    screenshot_path: Path,
) -> None:
    face_size = int(metadata[face_size_field])
    mip_count = int(metadata[mip_count_field])
    expected_mip_count = face_size.bit_length()
    if mip_count != expected_mip_count:
        raise RuntimeError(
            "ready-frame provenance cubemap mip layout is inconsistent: "
            f"face_size={face_size} mip_count={mip_count} expected={expected_mip_count} "
            f"source={face_size_field} path={screenshot_path}"
        )


def _require_positive_integer(
    metadata: Mapping[str, str], field: str, screenshot_path: Path
) -> None:
    if _POSITIVE_INTEGER_PATTERN.fullmatch(metadata[field]) is None:
        raise RuntimeError(
            "ready-frame provenance positive integer is malformed: "
            f"field={field} path={screenshot_path}"
        )


def _require_non_negative_integer(
    metadata: Mapping[str, str], field: str, screenshot_path: Path
) -> None:
    if _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(metadata[field]) is None:
        raise RuntimeError(
            "ready-frame provenance duration is malformed: "
            f"field={field} path={screenshot_path}"
        )


def _require_duration_hierarchy(
    metadata: Mapping[str, str],
    *,
    total_field: str,
    component_fields: tuple[str, ...],
    screenshot_path: Path,
) -> None:
    total = int(metadata[total_field])
    component_total = sum(int(metadata[field]) for field in component_fields)
    if total < component_total:
        raise RuntimeError(
            "ready-frame provenance duration hierarchy is inconsistent: "
            f"total_field={total_field} total_ns={total} "
            f"component_fields={','.join(component_fields)} component_total_ns={component_total} "
            f"path={screenshot_path}"
        )


def _require_finite_float(metadata: Mapping[str, str], field: str, screenshot_path: Path) -> None:
    try:
        value = float(metadata[field])
    except ValueError as error:
        raise RuntimeError(
            "ready-frame provenance camera angle is malformed: "
            f"field={field} path={screenshot_path}"
        ) from error
    if not math.isfinite(value):
        raise RuntimeError(
            "ready-frame provenance camera angle must be finite: "
            f"field={field} path={screenshot_path}"
        )


def _inspect_rgba_png(path: Path) -> tuple[int, int, int, int]:
    encoded = _read_bounded_png(path)
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
        raise RuntimeError(
            "ready-frame PNG must be a bounded non-interlaced RGBA8 image: "
            f"path={path}"
        )
    if chunks[-1][0] != b"IEND" or chunks[-1][1] != chunks[-1][2]:
        raise RuntimeError(f"ready-frame PNG is missing a terminal IEND: {path}")
    row_bytes = width * 4
    expected_bytes = (row_bytes + 1) * height
    if expected_bytes > _MAX_PIXEL_BYTES:
        raise RuntimeError(f"ready-frame PNG exceeds the evidence pixel budget: {path}")
    raw = _decompress_idat_chunks(encoded, chunks, expected_bytes, path)
    return _rgba_statistics(raw, width, height, path)


def _read_bounded_png(path: Path) -> bytes:
    try:
        with path.open("rb") as png_file:
            encoded = png_file.read(_MAX_ENCODED_PNG_BYTES + 1)
    except OSError as error:
        raise RuntimeError(f"ready-frame PNG is unavailable: {path}") from error
    if len(encoded) > _MAX_ENCODED_PNG_BYTES:
        raise RuntimeError(f"ready-frame PNG exceeds the encoded evidence budget: {path}")
    return encoded


def _decompress_idat_chunks(
    encoded: bytes,
    chunks: list[tuple[bytes, int, int]],
    expected_bytes: int,
    path: Path,
) -> bytearray:
    decompressor = zlib.decompressobj()
    raw = bytearray()
    idat_count = 0
    for kind, payload_start, payload_end in chunks:
        if kind != b"IDAT":
            continue
        idat_count += 1
        remaining_bytes = expected_bytes - len(raw)
        decoded = decompressor.decompress(
            memoryview(encoded)[payload_start:payload_end], remaining_bytes + 1
        )
        raw.extend(decoded)
        if len(raw) > expected_bytes or decompressor.unconsumed_tail:
            raise RuntimeError(f"ready-frame PNG image data has an invalid size: {path}")
    if (
        not idat_count
        or len(raw) != expected_bytes
        or not decompressor.eof
        or decompressor.unused_data
    ):
        raise RuntimeError(f"ready-frame PNG image data has an invalid size: {path}")
    return raw


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
        actual_crc &= 0xFFFFFFFF
        if actual_crc != expected_crc:
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


def _rgba_statistics(raw: bytes, width: int, height: int, path: Path) -> tuple[int, int, int, int]:
    row_bytes = width * 4
    previous = bytearray(row_bytes)
    colors: set[tuple[int, int, int, int]] = set()
    non_black_pixels = 0
    offset = 0
    for _row_index in range(height):
        filter_type = raw[offset]
        offset += 1
        encoded_row = raw[offset : offset + row_bytes]
        offset += row_bytes
        row = _unfilter_rgba_row(filter_type, encoded_row, previous, path)
        for pixel_offset in range(0, row_bytes, 4):
            pixel = tuple(row[pixel_offset : pixel_offset + 4])
            if pixel[3] == 0:
                continue
            if len(colors) <= _MAX_REPORTED_DISTINCT_COLORS:
                colors.add(pixel)
            if pixel[0] or pixel[1] or pixel[2]:
                non_black_pixels += 1
        previous = row
    return width, height, len(colors), non_black_pixels


def _unfilter_rgba_row(
    filter_type: int, encoded_row: bytes, previous: bytearray, path: Path
) -> bytearray:
    row = bytearray(encoded_row)
    if filter_type == 0:
        return row
    if filter_type not in (1, 2, 3, 4):
        raise RuntimeError(
            "ready-frame PNG uses an unsupported scanline filter: "
            f"filter={filter_type} path={path}"
        )
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


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a Zircon PBR viewer ready-frame PNG and v2 provenance sidecar."
    )
    parser.add_argument("png", type=Path, help="Ready-frame PNG written by zircon_shader_pbr_viewer")
    parser.add_argument("--expected-backend", help="Require the recorded backend, for example Dx12")
    parser.add_argument(
        "--require-direct-present",
        action="store_true",
        help="Require the normal interactive path to have used direct presentation",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        evidence = validate_ready_frame_evidence(
            arguments.png,
            expected_backend=arguments.expected_backend,
            require_direct_present=arguments.require_direct_present,
        )
    except (OSError, RuntimeError, ValueError) as error:
        print(f"PBR viewer evidence validation failed: {error}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "schema": evidence.metadata["schema"],
                "png": str(evidence.screenshot_path),
                "sidecar": str(evidence.sidecar_path),
                "viewport": list(evidence.viewport),
                "backend": evidence.backend,
                "render_profile": evidence.render_profile,
                "distinct_rgba_colors": evidence.distinct_rgba_colors,
                "non_black_pixel_count": evidence.non_black_pixel_count,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
