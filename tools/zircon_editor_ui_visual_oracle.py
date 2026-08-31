#!/usr/bin/env python3
"""Validate native Editor capture evidence and inspect rounded-control pixels."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import json
import math
import re
import struct
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


DEFAULT_EXPECTED_EXTENTS = {(640, 520), (900, 620), (1672, 941)}
ROUNDED_CONTROL_ID = re.compile(
    r"(button|chip|tab|field|search|dropdown|toggle)", re.IGNORECASE
)
PROFILE_CONTROL_COLLECTIONS = (
    "template_controls",
    "viewport_toolbar_controls",
    "activity_rail_buttons",
    "document_tabs",
    "drawer_tabs",
    "host_page_tabs",
)
VECTOR_ICON_COLLECTIONS = ("activity_rail_buttons",)
PROFILE_LAYOUT_FRAMES = (
    "center_band",
    "document_region",
    "left_region",
    "right_region",
    "bottom_region",
    "status_bar",
)
MIN_ROUNDED_SHAPE_RADIUS = 4.0
REQUIRED_LAYOUT_FRAMES = {"center_band", "document_region", "status_bar"}
FRAME_BOUNDS_EPSILON = 0.5
NEAR_DUPLICATE_OVERLAP_RATIO = 0.98
NEAR_DUPLICATE_SIZE_RATIO = 0.95
MIN_CONTROL_EXTENT = 16
MAX_CROPS_PER_CAPTURE = 24


class VisualOracleError(RuntimeError):
    pass


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def _png_chunk(kind: bytes, payload: bytes) -> bytes:
    checksum = binascii.crc32(kind + payload) & 0xFFFF_FFFF
    return struct.pack(">I", len(payload)) + kind + payload + struct.pack(">I", checksum)


def _paeth(left: int, above: int, upper_left: int) -> int:
    estimate = left + above - upper_left
    left_distance = abs(estimate - left)
    above_distance = abs(estimate - above)
    upper_left_distance = abs(estimate - upper_left)
    if left_distance <= above_distance and left_distance <= upper_left_distance:
        return left
    if above_distance <= upper_left_distance:
        return above
    return upper_left


@dataclass(frozen=True)
class _RgbImage:
    width: int
    height: int
    pixels: bytes

    @classmethod
    def load_png(cls, path: Path) -> "_RgbImage":
        try:
            source = path.read_bytes()
        except OSError as error:
            raise VisualOracleError(f"could not read screenshot {path}: {error}") from error
        if not source.startswith(PNG_SIGNATURE):
            raise VisualOracleError(f"screenshot is not a PNG file: {path}")

        offset = len(PNG_SIGNATURE)
        header: tuple[int, int, int, int, int, int, int] | None = None
        compressed = bytearray()
        saw_end = False
        while offset < len(source):
            if offset + 12 > len(source):
                raise VisualOracleError(f"truncated PNG chunk in {path}")
            length = struct.unpack_from(">I", source, offset)[0]
            kind = source[offset + 4 : offset + 8]
            payload_start = offset + 8
            payload_end = payload_start + length
            checksum_end = payload_end + 4
            if checksum_end > len(source):
                raise VisualOracleError(f"truncated PNG payload in {path}")
            payload = source[payload_start:payload_end]
            expected_checksum = struct.unpack_from(">I", source, payload_end)[0]
            actual_checksum = binascii.crc32(kind + payload) & 0xFFFF_FFFF
            if actual_checksum != expected_checksum:
                raise VisualOracleError(f"PNG chunk CRC mismatch in {path}")
            if kind == b"IHDR":
                if header is not None or len(payload) != 13:
                    raise VisualOracleError(f"invalid PNG header in {path}")
                header = struct.unpack(">IIBBBBB", payload)
            elif kind == b"IDAT":
                compressed.extend(payload)
            elif kind == b"IEND":
                saw_end = True
                break
            offset = checksum_end

        if header is None or not compressed or not saw_end:
            raise VisualOracleError(f"PNG is missing required chunks: {path}")
        width, height, bit_depth, color_type, compression, filtering, interlace = header
        if width <= 0 or height <= 0:
            raise VisualOracleError(f"PNG dimensions must be positive: {path}")
        if (
            bit_depth != 8
            or color_type not in (2, 6)
            or compression != 0
            or filtering != 0
            or interlace != 0
        ):
            raise VisualOracleError(
                "visual oracle requires a non-interlaced 8-bit RGB/RGBA PNG: "
                f"{path} (depth={bit_depth}, color_type={color_type}, "
                f"compression={compression}, filter={filtering}, interlace={interlace})"
            )

        channels = 3 if color_type == 2 else 4
        row_bytes = width * channels
        try:
            filtered_rows = zlib.decompress(bytes(compressed))
        except zlib.error as error:
            raise VisualOracleError(f"could not decompress PNG {path}: {error}") from error
        expected_length = height * (row_bytes + 1)
        if len(filtered_rows) != expected_length:
            raise VisualOracleError(
                f"PNG scanline bytes {len(filtered_rows)} do not match {expected_length}: {path}"
            )

        previous = bytearray(row_bytes)
        rgb = bytearray(width * height * 3)
        source_offset = 0
        rgb_offset = 0
        for _ in range(height):
            filter_kind = filtered_rows[source_offset]
            source_offset += 1
            encoded = filtered_rows[source_offset : source_offset + row_bytes]
            source_offset += row_bytes
            decoded = bytearray(row_bytes)
            for index, raw in enumerate(encoded):
                left = decoded[index - channels] if index >= channels else 0
                above = previous[index]
                upper_left = previous[index - channels] if index >= channels else 0
                if filter_kind == 0:
                    predictor = 0
                elif filter_kind == 1:
                    predictor = left
                elif filter_kind == 2:
                    predictor = above
                elif filter_kind == 3:
                    predictor = (left + above) // 2
                elif filter_kind == 4:
                    predictor = _paeth(left, above, upper_left)
                else:
                    raise VisualOracleError(f"unsupported PNG row filter {filter_kind}: {path}")
                decoded[index] = (raw + predictor) & 0xFF
            for x in range(width):
                pixel_offset = x * channels
                rgb[rgb_offset : rgb_offset + 3] = decoded[
                    pixel_offset : pixel_offset + 3
                ]
                rgb_offset += 3
            previous = decoded
        return cls(width=width, height=height, pixels=bytes(rgb))

    def getpixel(self, point: tuple[int, int]) -> tuple[int, int, int]:
        x, y = point
        offset = (y * self.width + x) * 3
        return self.pixels[offset], self.pixels[offset + 1], self.pixels[offset + 2]

    def crop(self, bounds: tuple[int, int, int, int]) -> "_RgbImage":
        left, top, right, bottom = bounds
        width = right - left
        height = bottom - top
        pixels = bytearray(width * height * 3)
        destination = 0
        for y in range(top, bottom):
            source = (y * self.width + left) * 3
            length = width * 3
            pixels[destination : destination + length] = self.pixels[source : source + length]
            destination += length
        return _RgbImage(width=width, height=height, pixels=bytes(pixels))

    def resize_nearest(self, width: int, height: int) -> "_RgbImage":
        pixels = bytearray(width * height * 3)
        destination = 0
        for y in range(height):
            source_y = min(self.height - 1, y * self.height // height)
            for x in range(width):
                source_x = min(self.width - 1, x * self.width // width)
                source = (source_y * self.width + source_x) * 3
                pixels[destination : destination + 3] = self.pixels[source : source + 3]
                destination += 3
        return _RgbImage(width=width, height=height, pixels=bytes(pixels))

    def save_png(self, path: Path) -> None:
        rows = bytearray()
        row_bytes = self.width * 3
        for y in range(self.height):
            rows.append(0)
            start = y * row_bytes
            rows.extend(self.pixels[start : start + row_bytes])
        header = struct.pack(">IIBBBBB", self.width, self.height, 8, 2, 0, 0, 0)
        path.write_bytes(
            PNG_SIGNATURE
            + _png_chunk(b"IHDR", header)
            + _png_chunk(b"IDAT", zlib.compress(bytes(rows), level=9))
            + _png_chunk(b"IEND", b"")
        )


def _load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8-sig"))
    except (OSError, json.JSONDecodeError) as error:
        raise VisualOracleError(f"could not read JSON evidence {path}: {error}") from error
    if not isinstance(value, dict):
        raise VisualOracleError(f"JSON evidence must be an object: {path}")
    return value


def _evidence_path(raw: Any, base: Path, label: str) -> Path:
    if not isinstance(raw, str) or not raw.strip():
        raise VisualOracleError(f"{label} must be a non-empty path")
    path = Path(raw)
    if not path.is_absolute():
        path = base / path
    return path.resolve()


def _integer(value: Any, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise VisualOracleError(f"{label} must be numeric")
    rounded = round(float(value))
    if abs(float(value) - rounded) > 1.0e-4:
        raise VisualOracleError(f"{label} must resolve to a physical pixel: {value}")
    return int(rounded)


def _number(value: Any, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise VisualOracleError(f"{label} must be numeric")
    resolved = float(value)
    if not math.isfinite(resolved):
        raise VisualOracleError(f"{label} must be finite")
    return resolved


def _sha256_hex(value: Any, label: str) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9A-Fa-f]{64}", value) is None:
        raise VisualOracleError(f"{label} must be a 64-character SHA-256")
    return value.lower()


def _verify_file_sha256(path: Path, expected: Any, label: str) -> str:
    expected_sha256 = _sha256_hex(expected, f"{label}.sha256")
    if not path.is_file():
        raise VisualOracleError(f"{label} file does not exist: {path}")
    actual_sha256 = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual_sha256 != expected_sha256:
        raise VisualOracleError(f"{label} fingerprint changed after capture: {path}")
    return actual_sha256


def _validate_capture_provenance(
    manifest: dict[str, Any], manifest_directory: Path
) -> dict[str, Any]:
    if manifest.get("schema_version") != 2:
        raise VisualOracleError("capture manifest must use schema_version=2")

    repository = manifest.get("repository")
    if not isinstance(repository, dict):
        raise VisualOracleError("capture manifest source provenance is missing")
    repository_root = _evidence_path(
        repository.get("root"), manifest_directory, "repository.root"
    )
    if not repository_root.is_dir():
        raise VisualOracleError(
            f"capture manifest source provenance root does not exist: {repository_root}"
        )
    source_sha256 = _sha256_hex(
        repository.get("source_sha256"), "repository.source_sha256"
    )
    git = repository.get("git")
    if not isinstance(git, dict):
        raise VisualOracleError("capture manifest source provenance Git metadata is missing")
    revision = git.get("revision")
    if not isinstance(revision, str) or re.fullmatch(
        r"[0-9A-Fa-f]{40}|[0-9A-Fa-f]{64}", revision
    ) is None:
        raise VisualOracleError("repository.git.revision must be a Git object ID")

    source_files = repository.get("critical_source_files")
    if not isinstance(source_files, list) or not source_files:
        raise VisualOracleError(
            "capture manifest source provenance requires critical source files"
        )
    canonical = bytearray(f"revision={revision.lower()}\n".encode("utf-8"))
    source_asset_files: dict[str, tuple[str, int]] = {}
    previous_relative_path: str | None = None
    for index, source_file in enumerate(source_files):
        if not isinstance(source_file, dict):
            raise VisualOracleError(
                f"repository.critical_source_files[{index}] must be an object"
            )
        relative_path = source_file.get("relative_path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            raise VisualOracleError(
                f"repository.critical_source_files[{index}].relative_path must be non-empty"
            )
        if previous_relative_path is not None and relative_path <= previous_relative_path:
            raise VisualOracleError(
                "capture manifest source provenance paths must be unique and sorted"
            )
        previous_relative_path = relative_path
        expected_source_file_sha256 = _sha256_hex(
            source_file.get("sha256"),
            f"repository.critical_source_files[{index}].sha256",
        )
        byte_length = source_file.get("byte_length")
        if (
            isinstance(byte_length, bool)
            or not isinstance(byte_length, int)
            or byte_length < 0
        ):
            raise VisualOracleError(
                f"repository.critical_source_files[{index}].byte_length must be non-negative"
            )
        relative_source_path = Path(relative_path)
        if relative_source_path.is_absolute():
            raise VisualOracleError(
                f"capture manifest source provenance path must be relative: {relative_path}"
            )
        source_path = (repository_root / relative_source_path).resolve()
        try:
            source_path.relative_to(repository_root)
        except ValueError as error:
            raise VisualOracleError(
                f"capture manifest source provenance path escaped repository: {relative_path}"
            ) from error
        if not source_path.is_file():
            raise VisualOracleError(
                f"capture manifest source provenance file does not exist: {source_path}"
            )
        source_bytes = source_path.read_bytes()
        actual_source_file_sha256 = hashlib.sha256(source_bytes).hexdigest()
        if (
            actual_source_file_sha256 != expected_source_file_sha256
            or len(source_bytes) != byte_length
        ):
            raise VisualOracleError(
                f"source fingerprint changed after capture: {relative_path}"
            )
        canonical.extend(relative_path.encode("utf-8"))
        canonical.extend(b"\0")
        canonical.extend(actual_source_file_sha256.encode("ascii"))
        canonical.extend(b"\0")
        canonical.extend(str(len(source_bytes)).encode("ascii"))
        canonical.extend(b"\n")
        for asset_prefix in ("zircon_editor/assets/", "zircon_runtime/assets/"):
            if relative_path.startswith(asset_prefix):
                bundle_relative_path = relative_path[len(asset_prefix) :]
                if bundle_relative_path in source_asset_files:
                    raise VisualOracleError(
                        "Runtime and Editor asset paths collide in the product bundle: "
                        f"{bundle_relative_path}"
                    )
                source_asset_files[bundle_relative_path] = (
                    actual_source_file_sha256,
                    len(source_bytes),
                )
                break

    if hashlib.sha256(canonical).hexdigest() != source_sha256:
        raise VisualOracleError(
            "capture manifest aggregate source fingerprint does not match source files"
        )

    binaries = manifest.get("binaries")
    if not isinstance(binaries, dict):
        raise VisualOracleError("capture manifest binary provenance is missing")
    binary_report: dict[str, Any] = {}
    for name in ("editor", "runtime"):
        binary = binaries.get(name)
        if not isinstance(binary, dict):
            raise VisualOracleError(f"capture manifest {name} binary provenance is missing")
        path = binary.get("path")
        if not isinstance(path, str) or not path.strip():
            raise VisualOracleError(f"binaries.{name}.path must be non-empty")
        expected = _sha256_hex(
            binary.get("expected_sha256"), f"binaries.{name}.expected_sha256"
        )
        actual = _sha256_hex(
            binary.get("actual_sha256"), f"binaries.{name}.actual_sha256"
        )
        if expected != actual:
            raise VisualOracleError(
                f"{name} binary does not match the managed build receipt"
            )
        binary_path = _evidence_path(path, manifest_directory, f"binaries.{name}.path")
        if not binary_path.is_file():
            raise VisualOracleError(f"{name} binary fingerprint file is missing: {binary_path}")
        on_disk = hashlib.sha256(binary_path.read_bytes()).hexdigest()
        if on_disk != actual:
            raise VisualOracleError(
                f"{name} binary fingerprint changed after capture: {binary_path}"
            )
        binary_report[name] = {"path": str(binary_path), "sha256": on_disk}

    assets = manifest.get("assets")
    if not isinstance(assets, dict):
        raise VisualOracleError("capture manifest bundle asset provenance is missing")
    asset_root = _evidence_path(assets.get("root"), manifest_directory, "assets.root")
    if not asset_root.is_dir():
        raise VisualOracleError(f"product bundle asset root does not exist: {asset_root}")
    reported_asset_count = assets.get("bundle_asset_file_count")
    if (
        isinstance(reported_asset_count, bool)
        or not isinstance(reported_asset_count, int)
        or reported_asset_count < 1
    ):
        raise VisualOracleError("assets.bundle_asset_file_count must be positive")
    reported_asset_sha256 = _sha256_hex(
        assets.get("bundle_asset_sha256"), "assets.bundle_asset_sha256"
    )
    actual_asset_paths = {
        path.relative_to(asset_root).as_posix(): path
        for path in asset_root.rglob("*")
        if path.is_file()
    }
    if set(actual_asset_paths) != set(source_asset_files):
        raise VisualOracleError(
            "product bundle asset set differs from current source: "
            f"expected={len(source_asset_files)} actual={len(actual_asset_paths)}"
        )
    asset_canonical = bytearray()
    for relative_path in sorted(source_asset_files):
        expected_sha256, expected_length = source_asset_files[relative_path]
        bundle_bytes = actual_asset_paths[relative_path].read_bytes()
        actual_sha256 = hashlib.sha256(bundle_bytes).hexdigest()
        if actual_sha256 != expected_sha256 or len(bundle_bytes) != expected_length:
            raise VisualOracleError(
                f"product bundle asset differs from current source: {relative_path}"
            )
        asset_canonical.extend(relative_path.encode("utf-8"))
        asset_canonical.extend(b"\0")
        asset_canonical.extend(actual_sha256.encode("ascii"))
        asset_canonical.extend(b"\0")
        asset_canonical.extend(str(len(bundle_bytes)).encode("ascii"))
        asset_canonical.extend(b"\n")
    actual_asset_sha256 = hashlib.sha256(asset_canonical).hexdigest()
    if (
        reported_asset_count != len(source_asset_files)
        or reported_asset_sha256 != actual_asset_sha256
    ):
        raise VisualOracleError(
            "product bundle asset aggregate fingerprint does not match manifest"
        )

    return {
        "root": str(repository_root),
        "source_sha256": source_sha256,
        "revision": revision.lower(),
        "critical_source_file_count": len(source_files),
        "binaries": binary_report,
        "assets": {
            "root": str(asset_root),
            "sha256": actual_asset_sha256,
            "file_count": len(source_asset_files),
        },
    }


def _linear_channel(channel: int) -> float:
    encoded = channel / 255.0
    if encoded <= 0.04045:
        return encoded / 12.92
    return ((encoded + 0.055) / 1.055) ** 2.4


def _linear_rgb(pixel: tuple[int, ...]) -> tuple[float, float, float]:
    return tuple(_linear_channel(channel) for channel in pixel[:3])  # type: ignore[return-value]


def _subtract(
    left: tuple[float, float, float], right: tuple[float, float, float]
) -> tuple[float, float, float]:
    return tuple(a - b for a, b in zip(left, right))  # type: ignore[return-value]


def _dot(left: tuple[float, float, float], right: tuple[float, float, float]) -> float:
    return sum(a * b for a, b in zip(left, right))


def _corner_pixel(
    image: _RgbImage,
    bounds: tuple[int, int, int, int],
    corner: str,
    u: int,
    v: int,
) -> tuple[int, ...]:
    left, top, right, bottom = bounds
    if corner == "top_left":
        point = (left + u, top + v)
    elif corner == "top_right":
        point = (right - 1 - u, top + v)
    elif corner == "bottom_left":
        point = (left + u, bottom - 1 - v)
    else:
        point = (right - 1 - u, bottom - 1 - v)
    return image.getpixel(point)


def _analyze_corner(
    image: _RgbImage,
    bounds: tuple[int, int, int, int],
    corner: str,
    radius: int,
) -> dict[str, Any]:
    outside = _linear_rgb(_corner_pixel(image, bounds, corner, 0, 0))
    inside = _linear_rgb(_corner_pixel(image, bounds, corner, radius, radius))
    direction = _subtract(inside, outside)
    separation_squared = _dot(direction, direction)
    separation = math.sqrt(separation_squared)
    if separation < 0.025:
        return {
            "corner": corner,
            "analyzable": False,
            "endpoint_linear_distance": round(separation, 6),
            "fractional_pixel_count": 0,
            "coverage_bin_count": 0,
            "fractional_row_count": 0,
            "fractional_column_count": 0,
            "antialiased": False,
        }

    fractional = 0
    bins: set[int] = set()
    fractional_rows: set[int] = set()
    fractional_columns: set[int] = set()
    sample_limit = min(radius + 2, (min(bounds[2] - bounds[0], bounds[3] - bounds[1]) + 1) // 2)
    for v in range(sample_limit):
        for u in range(sample_limit):
            sample = _linear_rgb(_corner_pixel(image, bounds, corner, u, v))
            offset = _subtract(sample, outside)
            coverage = _dot(offset, direction) / separation_squared
            projected = tuple(
                outside[index] + coverage * direction[index] for index in range(3)
            )
            residual = math.sqrt(_dot(_subtract(sample, projected), _subtract(sample, projected)))
            if 0.03 < coverage < 0.97 and residual <= 0.05:
                fractional += 1
                bins.add(max(1, min(15, round(coverage * 16))))
                fractional_rows.add(v)
                fractional_columns.add(u)

    return {
        "corner": corner,
        "analyzable": True,
        "endpoint_linear_distance": round(separation, 6),
        "fractional_pixel_count": fractional,
        "coverage_bin_count": len(bins),
        "fractional_row_count": len(fractional_rows),
        "fractional_column_count": len(fractional_columns),
        "antialiased": fractional > 0 and bool(bins),
    }


def _control_bounds(control: dict[str, Any], image: _RgbImage) -> tuple[int, int, int, int] | None:
    frame = control.get("frame")
    if not isinstance(frame, dict):
        return None
    label = str(control.get("id", "<control>"))
    x = _number(frame.get("x"), f"{label}.frame.x")
    y = _number(frame.get("y"), f"{label}.frame.y")
    width = _number(frame.get("width"), f"{label}.frame.width")
    height = _number(frame.get("height"), f"{label}.frame.height")
    left = math.floor(x)
    top = math.floor(y)
    right = math.ceil(x + width)
    bottom = math.ceil(y + height)
    if width < MIN_CONTROL_EXTENT or height < MIN_CONTROL_EXTENT:
        return None
    if left < 0 or top < 0 or right > image.width or bottom > image.height:
        return None
    return left, top, right, bottom


def _shape_radius_for_control(
    control: dict[str, Any],
    shapes: list[dict[str, Any]],
) -> float | None:
    frame = control.get("frame")
    if not isinstance(frame, dict):
        return None
    x = _number(frame.get("x"), "control.frame.x")
    y = _number(frame.get("y"), "control.frame.y")
    width = _number(frame.get("width"), "control.frame.width")
    height = _number(frame.get("height"), "control.frame.height")
    candidates: list[float] = []
    for shape in shapes:
        shape_frame = shape.get("frame")
        if not isinstance(shape_frame, dict):
            continue
        if all(
            abs(float(shape_frame.get(key, math.nan)) - expected) <= FRAME_BOUNDS_EPSILON
            for key, expected in (
                ("x", x),
                ("y", y),
                ("width", width),
                ("height", height),
            )
        ):
            candidates.append(float(shape["corner_radius"]))
    if not candidates:
        return None
    return max(candidates)


def _profile_controls(profile: dict[str, Any]) -> list[dict[str, Any]]:
    controls: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()
    for collection_name in PROFILE_CONTROL_COLLECTIONS:
        collection = profile.get(collection_name, [])
        if not isinstance(collection, list):
            raise VisualOracleError(f"profile {collection_name} must be an array")
        for raw in collection:
            if not isinstance(raw, dict):
                continue
            control_id = raw.get("id")
            surface = raw.get("surface", "")
            if not isinstance(control_id, str) or not ROUNDED_CONTROL_ID.search(control_id):
                continue
            identity = (control_id, str(surface))
            if identity in seen:
                continue
            seen.add(identity)
            controls.append(raw)
    return controls


def _profile_vector_icon_controls(profile: dict[str, Any]) -> list[dict[str, Any]]:
    controls: list[dict[str, Any]] = []
    seen: set[tuple[str, str]] = set()
    for collection_name in VECTOR_ICON_COLLECTIONS:
        collection = profile.get(collection_name, [])
        if not isinstance(collection, list):
            raise VisualOracleError(f"profile {collection_name} must be an array")
        for raw in collection:
            if not isinstance(raw, dict):
                continue
            control_id = raw.get("id")
            surface = raw.get("surface", "")
            if not isinstance(control_id, str) or not control_id:
                continue
            identity = (control_id, str(surface))
            if identity in seen:
                continue
            seen.add(identity)
            controls.append(raw)
    return controls


def _profile_text_runs(profile: dict[str, Any]) -> list[dict[str, Any]]:
    runs = profile.get("text_runs")
    if not isinstance(runs, list):
        raise VisualOracleError("profile text_runs must be an array")
    for index, run in enumerate(runs):
        if not isinstance(run, dict):
            raise VisualOracleError(f"text_runs[{index}] must be an object")
    return runs


def _profile_rounded_shapes(profile: dict[str, Any]) -> list[dict[str, Any]]:
    shapes = profile.get("rounded_shapes")
    if not isinstance(shapes, list):
        raise VisualOracleError("profile rounded_shapes must be an array")
    if not shapes:
        raise VisualOracleError("profile rounded_shapes must contain at least one shape")
    for index, shape in enumerate(shapes):
        if not isinstance(shape, dict):
            raise VisualOracleError(f"rounded_shapes[{index}] must be an object")
        _profile_frame_geometry(shape.get("frame"), f"rounded_shapes[{index}].frame")
        command_index = shape.get("command_index")
        if (
            isinstance(command_index, bool)
            or not isinstance(command_index, int)
            or command_index < 0
        ):
            raise VisualOracleError(
                f"rounded_shapes[{index}].command_index must be non-negative"
            )
        radius = _number(shape.get("corner_radius"), f"rounded_shapes[{index}].corner_radius")
        border_width = _number(shape.get("border_width"), f"rounded_shapes[{index}].border_width")
        if border_width < 0.0 or border_width > radius:
            raise VisualOracleError(
                f"rounded_shapes[{index}].border_width must be within the corner radius"
            )
    return shapes


def _text_run_bounds(
    run: dict[str, Any], image: _RgbImage, index: int
) -> tuple[int, int, int, int] | None:
    label = f"text_runs[{index}]"
    frame = _profile_frame_geometry(run.get("frame"), f"{label}.frame")
    x, y, width, height = frame
    if width <= 0.0 or height <= 0.0:
        return None
    clip_raw = run.get("clip")
    if clip_raw is not None:
        clip = _profile_frame_geometry(clip_raw, f"{label}.clip")
        clip_x, clip_y, clip_width, clip_height = clip
        right = min(x + width, clip_x + clip_width)
        bottom = min(y + height, clip_y + clip_height)
        x = max(x, clip_x)
        y = max(y, clip_y)
        width = right - x
        height = bottom - y
        if width <= 0.0 or height <= 0.0:
            return None
    text_length = run.get("text_length")
    if (
        isinstance(text_length, bool)
        or not isinstance(text_length, int)
        or text_length <= 0
    ):
        raise VisualOracleError(f"{label}.text_length must be positive")
    if _number(run.get("font_size"), f"{label}.font_size") <= 0.0:
        raise VisualOracleError(f"{label}.font_size must be positive")
    if _number(run.get("line_height"), f"{label}.line_height") <= 0.0:
        raise VisualOracleError(f"{label}.line_height must be positive")
    command_index = run.get("command_index")
    if (
        isinstance(command_index, bool)
        or not isinstance(command_index, int)
        or command_index < 0
    ):
        raise VisualOracleError(f"{label}.command_index must be non-negative")
    color = run.get("color")
    if (
        not isinstance(color, list)
        or len(color) != 4
        or any(
            isinstance(channel, bool)
            or not isinstance(channel, int)
            or channel < 0
            or channel > 255
            for channel in color
        )
    ):
        raise VisualOracleError(f"{label}.color must contain four byte channels")
    left = max(0, math.floor(x))
    top = max(0, math.floor(y))
    right = min(image.width, math.ceil(x + width))
    bottom = min(image.height, math.ceil(y + height))
    if right <= left or bottom <= top:
        return None
    return left, top, right, bottom


def _analyze_text_run(
    image: _RgbImage,
    bounds: tuple[int, int, int, int],
    color: list[int],
) -> dict[str, Any]:
    left, top, right, bottom = bounds
    pixels = [
        image.getpixel((x, y))
        for y in range(top, bottom)
        for x in range(left, right)
    ]
    frequencies: dict[tuple[int, int, int], int] = {}
    for pixel in pixels:
        frequencies[pixel] = frequencies.get(pixel, 0) + 1
    background = _linear_rgb(max(frequencies, key=frequencies.get))
    declared_foreground = _linear_rgb(tuple(color[:3]))
    alpha = color[3] / 255.0
    foreground = tuple(
        background[channel]
        + alpha * (declared_foreground[channel] - background[channel])
        for channel in range(3)
    )
    direction = _subtract(foreground, background)
    separation_squared = _dot(direction, direction)
    separation = math.sqrt(separation_squared)
    if separation < 0.04:
        return {
            "analyzable": False,
            "endpoint_linear_distance": round(separation, 6),
            "ink_pixel_count": 0,
            "fractional_pixel_count": 0,
            "coverage_bin_count": 0,
            "fractional_row_count": 0,
            "fractional_column_count": 0,
            "antialiased": False,
        }

    ink_pixels = 0
    fractional_pixels = 0
    bins: set[int] = set()
    fractional_rows: set[int] = set()
    fractional_columns: set[int] = set()
    for y in range(top, bottom):
        for x in range(left, right):
            sample = _linear_rgb(image.getpixel((x, y)))
            offset = _subtract(sample, background)
            coverage = _dot(offset, direction) / separation_squared
            projected = tuple(
                background[channel] + coverage * direction[channel]
                for channel in range(3)
            )
            residual_vector = _subtract(sample, projected)
            residual = math.sqrt(_dot(residual_vector, residual_vector))
            if coverage > 0.04 and residual <= 0.07:
                ink_pixels += 1
            if 0.04 < coverage < 0.96 and residual <= 0.07:
                fractional_pixels += 1
                bins.add(max(1, min(15, round(coverage * 16))))
                fractional_rows.add(y - top)
                fractional_columns.add(x - left)

    analyzable = ink_pixels >= 3
    antialiased = (
        analyzable
        and fractional_pixels >= 2
        and len(bins) >= 2
        and len(fractional_rows) >= 2
        and len(fractional_columns) >= 2
    )
    return {
        "analyzable": analyzable,
        "endpoint_linear_distance": round(separation, 6),
        "ink_pixel_count": ink_pixels,
        "fractional_pixel_count": fractional_pixels,
        "coverage_bin_count": len(bins),
        "fractional_row_count": len(fractional_rows),
        "fractional_column_count": len(fractional_columns),
        "antialiased": antialiased,
    }


def _profile_frame_geometry(
    raw: Any, label: str
) -> tuple[float, float, float, float]:
    if not isinstance(raw, dict):
        raise VisualOracleError(f"{label} must be an object")
    frame = (
        _number(raw.get("x"), f"{label}.x"),
        _number(raw.get("y"), f"{label}.y"),
        _number(raw.get("width"), f"{label}.width"),
        _number(raw.get("height"), f"{label}.height"),
    )
    if frame[2] < 0.0 or frame[3] < 0.0:
        raise VisualOracleError(f"{label} has a negative extent")
    return frame


def _frame_exceeds_image(
    frame: tuple[float, float, float, float], image: _RgbImage
) -> bool:
    x, y, width, height = frame
    return (
        x < -FRAME_BOUNDS_EPSILON
        or y < -FRAME_BOUNDS_EPSILON
        or x + width > image.width + FRAME_BOUNDS_EPSILON
        or y + height > image.height + FRAME_BOUNDS_EPSILON
    )


def _validate_profile_layout(
    profile: dict[str, Any], image: _RgbImage
) -> dict[str, dict[str, float]]:
    layout = profile.get("layout")
    if not isinstance(layout, dict):
        raise VisualOracleError("profile layout must be an object")
    report: dict[str, dict[str, float]] = {}
    for name in PROFILE_LAYOUT_FRAMES:
        frame = _profile_frame_geometry(layout.get(name), f"layout.{name}")
        x, y, width, height = frame
        if name in REQUIRED_LAYOUT_FRAMES and (width <= 0.0 or height <= 0.0):
            raise VisualOracleError(f"required layout frame is empty: {name}")
        if _frame_exceeds_image(frame, image):
            raise VisualOracleError(
                "layout frame exceeds screenshot bounds: "
                f"{name}={frame} surface={image.width}x{image.height}"
            )
        report[name] = {"x": x, "y": y, "width": width, "height": height}
    return report


def _intersection_area(
    left: tuple[float, float, float, float],
    right: tuple[float, float, float, float],
) -> float:
    left_x, left_y, left_width, left_height = left
    right_x, right_y, right_width, right_height = right
    width = max(
        0.0,
        min(left_x + left_width, right_x + right_width) - max(left_x, right_x),
    )
    height = max(
        0.0,
        min(left_y + left_height, right_y + right_height) - max(left_y, right_y),
    )
    return width * height


def _validate_clickable_frames(profile: dict[str, Any], image: _RgbImage) -> int:
    raw_frames = profile.get("clickable_frames")
    if not isinstance(raw_frames, list):
        raise VisualOracleError("profile clickable_frames must be an array")
    frames: list[tuple[str, str, tuple[float, float, float, float]]] = []
    for index, raw in enumerate(raw_frames):
        if not isinstance(raw, dict):
            raise VisualOracleError(f"clickable_frames[{index}] must be an object")
        control_id = raw.get("id")
        if not isinstance(control_id, str) or not control_id:
            raise VisualOracleError(f"clickable_frames[{index}].id must be non-empty")
        frame = _profile_frame_geometry(
            raw.get("frame"), f"clickable_frames[{index}].frame"
        )
        if frame[2] <= 0.0 or frame[3] <= 0.0:
            continue
        if _frame_exceeds_image(frame, image):
            raise VisualOracleError(
                "clickable frame exceeds screenshot bounds: "
                f"{control_id}={frame} surface={image.width}x{image.height}"
            )
        frames.append((control_id, str(raw.get("surface", "")), frame))

    for index, (left_id, left_surface, left) in enumerate(frames):
        left_area = left[2] * left[3]
        for right_id, right_surface, right in frames[index + 1 :]:
            if left_surface != right_surface or left_id == right_id:
                continue
            width_ratio = min(left[2], right[2]) / max(left[2], right[2])
            height_ratio = min(left[3], right[3]) / max(left[3], right[3])
            if (
                width_ratio < NEAR_DUPLICATE_SIZE_RATIO
                or height_ratio < NEAR_DUPLICATE_SIZE_RATIO
            ):
                continue
            right_area = right[2] * right[3]
            overlap_ratio = _intersection_area(left, right) / min(left_area, right_area)
            if overlap_ratio >= NEAR_DUPLICATE_OVERLAP_RATIO:
                raise VisualOracleError(
                    "near-duplicate clickable frames overlap: "
                    f"surface={left_surface} controls={left_id},{right_id} "
                    f"overlap_ratio={overlap_ratio:.4f}"
                )
    return len(frames)


def _vector_icon_bounds(
    control: dict[str, Any], image: _RgbImage
) -> tuple[int, int, int, int] | None:
    bounds = _control_bounds(control, image)
    if bounds is None:
        return None
    left, top, right, bottom = bounds
    inset = max(4, round(min(right - left, bottom - top) * 0.2))
    if right - left <= inset * 2 or bottom - top <= inset * 2:
        return None
    return left + inset, top + inset, right - inset, bottom - inset


def _analyze_vector_icon(
    image: _RgbImage, bounds: tuple[int, int, int, int]
) -> dict[str, Any]:
    left, top, right, bottom = bounds
    raw_pixels = [
        image.getpixel((x, y))
        for y in range(top, bottom)
        for x in range(left, right)
    ]
    frequencies: dict[tuple[int, int, int], int] = {}
    for pixel in raw_pixels:
        frequencies[pixel] = frequencies.get(pixel, 0) + 1
    background_raw = max(frequencies, key=frequencies.get)
    background = _linear_rgb(background_raw)
    linear_pixels = [_linear_rgb(pixel) for pixel in raw_pixels]
    foreground = max(
        linear_pixels,
        key=lambda pixel: _dot(_subtract(pixel, background), _subtract(pixel, background)),
    )
    direction = _subtract(foreground, background)
    separation_squared = _dot(direction, direction)
    separation = math.sqrt(separation_squared)
    if separation < 0.05:
        return {
            "analyzable": False,
            "endpoint_linear_distance": round(separation, 6),
            "fractional_pixel_count": 0,
            "coverage_bin_count": 0,
            "foreground_pixel_count": 0,
            "antialiased": False,
        }

    fractional = 0
    foreground_pixels = 0
    bins: set[int] = set()
    for sample in linear_pixels:
        offset = _subtract(sample, background)
        coverage = _dot(offset, direction) / separation_squared
        projected = tuple(
            background[index] + coverage * direction[index] for index in range(3)
        )
        residual_vector = _subtract(sample, projected)
        residual = math.sqrt(_dot(residual_vector, residual_vector))
        if coverage > 0.03 and residual <= 0.05:
            foreground_pixels += 1
        if 0.03 < coverage < 0.97 and residual <= 0.05:
            fractional += 1
            bins.add(max(1, min(15, round(coverage * 16))))

    return {
        "analyzable": foreground_pixels >= 3,
        "endpoint_linear_distance": round(separation, 6),
        "fractional_pixel_count": fractional,
        "coverage_bin_count": len(bins),
        "foreground_pixel_count": foreground_pixels,
        "antialiased": foreground_pixels >= 3 and fractional > 0 and bool(bins),
    }


def _validate_antialias_population(
    *, label: str, candidate_count: int, antialiased_count: int
) -> None:
    """Require AA on at least half of visible candidates, with one minimum."""
    if candidate_count <= 0 or antialiased_count <= 0 or antialiased_count * 2 < candidate_count:
        raise VisualOracleError(
            f"{label} antialias coverage is too sparse: "
            f"candidates={candidate_count} antialiased={antialiased_count}"
        )


def _write_control_crop(
    image: _RgbImage,
    bounds: tuple[int, int, int, int],
    output_path: Path,
) -> None:
    padding = 2
    left, top, right, bottom = bounds
    crop_bounds = (
        max(0, left - padding),
        max(0, top - padding),
        min(image.width, right + padding),
        min(image.height, bottom + padding),
    )
    crop = image.crop(crop_bounds)
    crop.resize_nearest(crop.width * 8, crop.height * 8).save_png(output_path)


def _safe_file_name(value: str) -> str:
    cleaned = re.sub(r"[^A-Za-z0-9._-]+", "_", value).strip("._")
    return cleaned or "control"


def _analyze_capture(
    capture: dict[str, Any], manifest_directory: Path, output_directory: Path
) -> dict[str, Any]:
    if capture.get("presenter_backend") != "gpu":
        raise VisualOracleError("visual acceptance requires presenter_backend=gpu")

    screenshot_data = capture.get("screenshot")
    if not isinstance(screenshot_data, dict):
        raise VisualOracleError("capture screenshot evidence must be an object")
    screenshot_path = _evidence_path(
        screenshot_data.get("path"), manifest_directory, "screenshot.path"
    )
    profile_path = _evidence_path(
        capture.get("profile_geometry_path"), manifest_directory, "profile_geometry_path"
    )
    screenshot_sha256 = _verify_file_sha256(
        screenshot_path, screenshot_data.get("sha256"), "screenshot"
    )
    profile_geometry_sha256 = _verify_file_sha256(
        profile_path,
        capture.get("profile_geometry_sha256"),
        "profile geometry",
    )
    profile = _load_json(profile_path)
    if profile.get("schema_version") != 4:
        raise VisualOracleError("profile geometry must use schema_version=4")
    if profile.get("presenter_backend") != "gpu":
        raise VisualOracleError("profile geometry requires presenter_backend=gpu")

    image = _RgbImage.load_png(screenshot_path)
    expected_width = _integer(screenshot_data.get("width"), "screenshot.width")
    expected_height = _integer(screenshot_data.get("height"), "screenshot.height")
    image_size = (image.width, image.height)
    if image_size != (expected_width, expected_height):
        raise VisualOracleError(
            f"screenshot dimensions {image_size} do not match manifest "
            f"{expected_width}x{expected_height}"
        )
    profile_size = profile.get("window_client_size")
    if not isinstance(profile_size, dict):
        raise VisualOracleError("profile window_client_size must be an object")
    profile_extent = (
        _integer(profile_size.get("width"), "window_client_size.width"),
        _integer(profile_size.get("height"), "window_client_size.height"),
    )
    if profile_extent != image_size:
        raise VisualOracleError(
            f"profile surface {profile_extent} does not match screenshot {image_size}"
        )
    if (
        _integer(capture.get("profile_surface_width"), "profile_surface_width"),
        _integer(capture.get("profile_surface_height"), "profile_surface_height"),
    ) != image_size:
        raise VisualOracleError("capture profile surface does not match screenshot pixels")

    dpi = _integer(capture.get("window_dpi"), "window_dpi")
    if dpi <= 0:
        raise VisualOracleError("window_dpi must be positive")
    scale_factor = capture.get("window_scale_factor")
    if not isinstance(scale_factor, (int, float)) or isinstance(scale_factor, bool):
        raise VisualOracleError("window_scale_factor must be numeric")
    if abs(float(scale_factor) - dpi / 96.0) > 1.0e-4:
        raise VisualOracleError("window scale factor does not match GetDpiForWindow")

    layout_report = _validate_profile_layout(profile, image)
    clickable_frame_count = _validate_clickable_frames(profile, image)
    rounded_shapes = _profile_rounded_shapes(profile)

    crop_directory = output_directory / "crops"
    crop_directory.mkdir(parents=True, exist_ok=True)
    control_reports = []
    for index, control in enumerate(_profile_controls(profile)):
        bounds = _control_bounds(control, image)
        if bounds is None:
            continue
        radius_value = _shape_radius_for_control(control, rounded_shapes)
        if radius_value is None:
            continue
        if radius_value < MIN_ROUNDED_SHAPE_RADIUS:
            raise VisualOracleError(
                f"rounded_shapes corner_radius must be at least "
                f"{MIN_ROUNDED_SHAPE_RADIUS:g} physical pixels for profiled controls"
            )
        radius = min(
            max(1, round(radius_value)),
            max(1, (min(bounds[2] - bounds[0], bounds[3] - bounds[1]) - 2) // 2),
        )
        corners = [
            _analyze_corner(image, bounds, corner, radius)
            for corner in ("top_left", "top_right", "bottom_left", "bottom_right")
        ]
        control_id = str(control.get("id", f"control-{index}"))
        if index < MAX_CROPS_PER_CAPTURE:
            crop_name = (
                f"{image.width}x{image.height}-{index:02}-"
                f"{_safe_file_name(control_id)}-8x.png"
            )
            _write_control_crop(image, bounds, crop_directory / crop_name)
        control_reports.append(
            {
                "id": control_id,
                "surface": str(control.get("surface", "")),
                "bounds": {
                    "left": bounds[0],
                    "top": bounds[1],
                    "right": bounds[2],
                    "bottom": bounds[3],
                },
                "assessed_radius_pixels": radius,
                "corners": corners,
            }
        )

    vector_icon_reports = []
    for index, control in enumerate(_profile_vector_icon_controls(profile)):
        bounds = _vector_icon_bounds(control, image)
        if bounds is None:
            continue
        analysis = _analyze_vector_icon(image, bounds)
        control_id = str(control.get("id", f"vector-icon-{index}"))
        if index < MAX_CROPS_PER_CAPTURE:
            crop_name = (
                f"{image.width}x{image.height}-{index:02}-"
                f"{_safe_file_name(control_id)}-vector-icon-8x.png"
            )
            _write_control_crop(image, bounds, crop_directory / crop_name)
        vector_icon_reports.append(
            {
                "id": control_id,
                "surface": str(control.get("surface", "")),
                "bounds": {
                    "left": bounds[0],
                    "top": bounds[1],
                    "right": bounds[2],
                    "bottom": bounds[3],
                },
                **analysis,
            }
        )

    text_run_reports = []
    for index, run in enumerate(_profile_text_runs(profile)):
        bounds = _text_run_bounds(run, image, index)
        if bounds is None:
            continue
        analysis = _analyze_text_run(image, bounds, run["color"])
        if index < MAX_CROPS_PER_CAPTURE:
            crop_name = f"{image.width}x{image.height}-{index:02}-text-run-8x.png"
            _write_control_crop(image, bounds, crop_directory / crop_name)
        text_run_reports.append(
            {
                "command_index": run.get("command_index"),
                "bounds": {
                    "left": bounds[0],
                    "top": bounds[1],
                    "right": bounds[2],
                    "bottom": bounds[3],
                },
                "text_length": run["text_length"],
                **analysis,
            }
        )

    analyzable = [
        corner
        for control in control_reports
        for corner in control["corners"]
        if corner["analyzable"]
    ]
    antialiased = [corner for corner in analyzable if corner["antialiased"]]
    fractional_pixels = sum(corner["fractional_pixel_count"] for corner in analyzable)
    if len(analyzable) < 2 or len(antialiased) < 2 or fractional_pixels <= 0:
        raise VisualOracleError(
            "rounded controls do not provide sufficient fractional corner coverage: "
            f"analyzable={len(analyzable)} antialiased={len(antialiased)} "
            f"fractional_pixels={fractional_pixels}"
        )
    continuous_controls = [
        control
        for control in control_reports
        if len(control["corners"]) == 4
        and all(
            corner["analyzable"]
            and corner["antialiased"]
            and corner["coverage_bin_count"] >= 2
            and corner["fractional_row_count"] >= 2
            and corner["fractional_column_count"] >= 2
            for corner in control["corners"]
        )
    ]
    rounded_candidates = [
        control
        for control in control_reports
        if sum(corner["analyzable"] for corner in control["corners"]) >= 2
    ]
    if not continuous_controls:
        raise VisualOracleError(
            "rounded controls do not provide continuous four-corner coverage"
        )
    curved_controls = [
        control
        for control in continuous_controls
        if all(
            corner["fractional_row_count"]
            >= max(4, math.ceil(control["assessed_radius_pixels"] * 0.6))
            and corner["fractional_column_count"]
            >= max(4, math.ceil(control["assessed_radius_pixels"] * 0.6))
            for corner in control["corners"]
        )
    ]
    if not curved_controls:
        raise VisualOracleError(
            "rounded controls do not expose the expected curved-corner span"
        )
    _validate_antialias_population(
        label="rounded-control",
        candidate_count=len(rounded_candidates),
        antialiased_count=len(curved_controls),
    )

    analyzable_vector_icons = [
        icon for icon in vector_icon_reports if icon["analyzable"]
    ]
    antialiased_vector_icons = [
        icon for icon in analyzable_vector_icons if icon["antialiased"]
    ]
    vector_icon_fractional_pixels = sum(
        icon["fractional_pixel_count"] for icon in analyzable_vector_icons
    )
    if (
        not analyzable_vector_icons
        or not antialiased_vector_icons
        or vector_icon_fractional_pixels <= 0
    ):
        raise VisualOracleError(
            "vector icons do not provide fractional edge coverage: "
            f"candidates={len(vector_icon_reports)} "
            f"analyzable={len(analyzable_vector_icons)} "
            f"antialiased={len(antialiased_vector_icons)} "
            f"fractional_pixels={vector_icon_fractional_pixels}"
        )
    _validate_antialias_population(
        label="vector-icon",
        candidate_count=len(analyzable_vector_icons),
        antialiased_count=len(antialiased_vector_icons),
    )

    analyzable_text_runs = [
        run for run in text_run_reports if run["analyzable"]
    ]
    antialiased_text_runs = [
        run for run in analyzable_text_runs if run["antialiased"]
    ]
    text_fractional_pixels = sum(
        run["fractional_pixel_count"] for run in analyzable_text_runs
    )
    if (
        not analyzable_text_runs
        or not antialiased_text_runs
        or text_fractional_pixels <= 0
    ):
        raise VisualOracleError(
            "text runs do not provide fractional glyph edge coverage: "
            f"candidates={len(text_run_reports)} "
            f"analyzable={len(analyzable_text_runs)} "
            f"antialiased={len(antialiased_text_runs)} "
            f"fractional_pixels={text_fractional_pixels}"
        )
    _validate_antialias_population(
        label="text-run",
        candidate_count=len(analyzable_text_runs),
        antialiased_count=len(antialiased_text_runs),
    )

    return {
        "screenshot_path": str(screenshot_path),
        "screenshot_sha256": screenshot_sha256,
        "profile_geometry_path": str(profile_path),
        "profile_geometry_sha256": profile_geometry_sha256,
        "extent": {"width": image.width, "height": image.height},
        "window_dpi": dpi,
        "window_scale_factor": float(scale_factor),
        "layout": layout_report,
        "rounded_shape_count": len(rounded_shapes),
        "rounded_shape_min_radius": min(
            (float(shape["corner_radius"]) for shape in rounded_shapes),
            default=0.0,
        ),
        "clickable_frame_count": clickable_frame_count,
        "near_duplicate_clickable_overlap_count": 0,
        "candidate_control_count": len(control_reports),
        "analyzable_corner_count": len(analyzable),
        "antialiased_corner_count": len(antialiased),
        "continuous_four_corner_control_count": len(continuous_controls),
        "expected_radius_curve_control_count": len(curved_controls),
        "rounded_control_candidate_count": len(rounded_candidates),
        "rounded_control_antialiased_ratio": (
            len(curved_controls) / len(rounded_candidates)
            if rounded_candidates
            else 0.0
        ),
        "fractional_pixel_count": fractional_pixels,
        "candidate_vector_icon_count": len(vector_icon_reports),
        "analyzable_vector_icon_count": len(analyzable_vector_icons),
        "antialiased_vector_icon_count": len(antialiased_vector_icons),
        "vector_icon_antialiased_ratio": (
            len(antialiased_vector_icons) / len(analyzable_vector_icons)
            if analyzable_vector_icons
            else 0.0
        ),
        "vector_icon_fractional_pixel_count": vector_icon_fractional_pixels,
        "candidate_text_run_count": len(text_run_reports),
        "analyzable_text_run_count": len(analyzable_text_runs),
        "antialiased_text_run_count": len(antialiased_text_runs),
        "text_run_antialiased_ratio": (
            len(antialiased_text_runs) / len(analyzable_text_runs)
            if analyzable_text_runs
            else 0.0
        ),
        "text_fractional_pixel_count": text_fractional_pixels,
        "controls": control_reports,
        "vector_icons": vector_icon_reports,
        "text_runs": text_run_reports,
    }


def validate_capture_manifest(
    manifest_path: Path,
    *,
    expected_extents: set[tuple[int, int]] = DEFAULT_EXPECTED_EXTENTS,
    output_directory: Path,
) -> dict[str, Any]:
    manifest_path = manifest_path.resolve()
    manifest = _load_json(manifest_path)
    provenance = _validate_capture_provenance(manifest, manifest_path.parent)
    captures = manifest.get("captures")
    if not isinstance(captures, list) or not captures:
        raise VisualOracleError("capture manifest must contain a non-empty captures array")
    actual_extents: set[tuple[int, int]] = set()
    for capture in captures:
        if not isinstance(capture, dict):
            raise VisualOracleError("each capture manifest entry must be an object")
        screenshot = capture.get("screenshot")
        if not isinstance(screenshot, dict):
            raise VisualOracleError("capture screenshot evidence must be an object")
        actual_extents.add(
            (
                _integer(screenshot.get("width"), "screenshot.width"),
                _integer(screenshot.get("height"), "screenshot.height"),
            )
        )
    if actual_extents != expected_extents:
        raise VisualOracleError(
            f"capture extents {sorted(actual_extents)} do not match required "
            f"{sorted(expected_extents)}"
        )
    if len(captures) != len(expected_extents):
        raise VisualOracleError(
            "visual acceptance requires exactly one capture process per physical extent"
        )

    output_directory = output_directory.resolve()
    output_directory.mkdir(parents=True, exist_ok=True)
    capture_reports = [
        _analyze_capture(capture, manifest_path.parent, output_directory)
        for capture in captures
    ]
    report = {
        "schema_version": 2,
        "passed": True,
        "capture_manifest_path": str(manifest_path),
        "provenance": provenance,
        "expected_extents": [
            {"width": width, "height": height}
            for width, height in sorted(expected_extents)
        ],
        "captures": capture_reports,
    }
    report_path = output_directory / "editor_ui_visual_oracle.json"
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def _parse_extent(raw: str) -> tuple[int, int]:
    match = re.fullmatch(r"([1-9][0-9]*)x([1-9][0-9]*)", raw)
    if match is None:
        raise argparse.ArgumentTypeError("extent must use WIDTHxHEIGHT")
    return int(match.group(1)), int(match.group(2))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture-manifest", required=True, type=Path)
    parser.add_argument("--output-directory", required=True, type=Path)
    parser.add_argument(
        "--expected-extent",
        action="append",
        type=_parse_extent,
        help="required physical extent; defaults to the three Editor acceptance sizes",
    )
    arguments = parser.parse_args()
    expected_extents = (
        set(arguments.expected_extent)
        if arguments.expected_extent
        else DEFAULT_EXPECTED_EXTENTS
    )
    try:
        report = validate_capture_manifest(
            arguments.capture_manifest,
            expected_extents=expected_extents,
            output_directory=arguments.output_directory,
        )
    except VisualOracleError as error:
        parser.error(str(error))
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
