"""Offline `.zsdf` build-target owner for ``tools/zircon_build.py``."""

from __future__ import annotations

import dataclasses
import json
import subprocess
import uuid
from pathlib import Path
from typing import Mapping, Sequence

try:
    from .zircon_build_cargo_environment import managed_cargo_environment
except ImportError:  # pragma: no cover - direct script import path.
    from zircon_build_cargo_environment import managed_cargo_environment


FONT_SDF_MANIFEST_VERSION = 1
FONT_SDF_MODES = ("sdf", "msdf", "mtsdf")


class FontSdfBakeManifestError(ValueError):
    """Raised when a font-SDF build manifest is incomplete or ambiguous."""


@dataclasses.dataclass(frozen=True)
class FontSdfBakeSpec:
    font: Path
    cache_root: Path
    asset_guid: str
    face_index: int = 0
    mode: str = "sdf"
    codepoints: tuple[str, ...] = ()
    all_cmap: bool = False
    page_size: int = 1024
    bake_em_px: int = 48
    spread_px_milli: int = 8_000
    variation_hash: str | None = None


def load_font_sdf_manifest(
    manifest_path: Path, repo_root: Path
) -> tuple[FontSdfBakeSpec, ...]:
    try:
        document = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise FontSdfBakeManifestError(
            f"cannot read font-SDF manifest {manifest_path}: {error}"
        ) from error
    if not isinstance(document, Mapping):
        raise FontSdfBakeManifestError("font-SDF manifest root must be an object")
    if document.get("format_version") != FONT_SDF_MANIFEST_VERSION:
        raise FontSdfBakeManifestError(
            "font-SDF manifest format_version must be "
            f"{FONT_SDF_MANIFEST_VERSION}"
        )
    raw_bakes = document.get("bakes")
    if not isinstance(raw_bakes, list) or not raw_bakes:
        raise FontSdfBakeManifestError("font-SDF manifest requires non-empty bakes")
    specs = tuple(_spec_from_record(record, repo_root) for record in raw_bakes)
    for spec in specs:
        validate_font_sdf_spec(spec)
    return specs


def validate_font_sdf_spec(spec: FontSdfBakeSpec) -> None:
    if spec.mode not in FONT_SDF_MODES:
        raise FontSdfBakeManifestError(
            f"font-SDF mode must be one of {', '.join(FONT_SDF_MODES)}"
        )
    try:
        canonical_guid = str(uuid.UUID(spec.asset_guid))
    except ValueError as error:
        raise FontSdfBakeManifestError(
            f"invalid font asset GUID {spec.asset_guid!r}"
        ) from error
    if canonical_guid != spec.asset_guid:
        raise FontSdfBakeManifestError(
            f"font asset GUID must be canonical lowercase: {canonical_guid}"
        )
    if spec.face_index < 0 or spec.face_index > 0xFFFFFFFF:
        raise FontSdfBakeManifestError("face_index must fit u32")
    for label, value in (
        ("page_size", spec.page_size),
        ("bake_em_px", spec.bake_em_px),
        ("spread_px_milli", spec.spread_px_milli),
    ):
        if value <= 0 or value > 0xFFFFFFFF:
            raise FontSdfBakeManifestError(f"{label} must be a positive u32")
    if spec.all_cmap == bool(spec.codepoints):
        raise FontSdfBakeManifestError(
            "select exactly one of all_cmap=true or a non-empty codepoints list"
        )
    if spec.variation_hash is not None:
        _validate_hash(spec.variation_hash, "variation_hash")


def build_font_sdf_command(config, spec: FontSdfBakeSpec) -> list[str]:
    validate_font_sdf_spec(spec)
    command = [
        str(config.cargo),
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_font_sdf_bake",
        "--no-default-features",
        "--features",
        "font-sdf-build-tool",
        "--target-dir",
        str(config.targets_root / "font-sdf"),
    ]
    if config.locked:
        command.append("--locked")
    if config.jobs:
        command.extend(["--jobs", str(config.jobs)])
    command.extend(
        [
            "--",
            "--font",
            str(spec.font),
            "--cache-root",
            str(spec.cache_root),
            "--asset-guid",
            spec.asset_guid,
            "--face-index",
            str(spec.face_index),
            "--mode",
            spec.mode,
            "--page-size",
            str(spec.page_size),
            "--bake-em-px",
            str(spec.bake_em_px),
            "--spread-px-milli",
            str(spec.spread_px_milli),
        ]
    )
    if spec.variation_hash is not None:
        command.extend(["--variation-hash", spec.variation_hash])
    if spec.all_cmap:
        command.append("--all-cmap")
    else:
        for codepoint in spec.codepoints:
            command.extend(["--codepoint", codepoint])
    return command


def bake_font_sdf_manifest(config, manifest_path: Path) -> None:
    specs = load_font_sdf_manifest(manifest_path, config.repo_root)
    for spec in specs:
        command = build_font_sdf_command(config, spec)
        if config.dry_run:
            print("DRY-RUN", _quote_command(command))
            continue
        environment = managed_cargo_environment(
            config.targets_root / "font-sdf", config.targets_root
        )
        print(_quote_command(command))
        subprocess.run(command, cwd=config.repo_root, check=True, env=environment)


def _spec_from_record(record: object, repo_root: Path) -> FontSdfBakeSpec:
    if not isinstance(record, Mapping):
        raise FontSdfBakeManifestError("each font-SDF bake must be an object")
    font = _required_path(record, "font", repo_root)
    cache_root = _required_path(record, "cache_root", repo_root)
    asset_guid = _required_string(record, "asset_guid")
    codepoints = _codepoints(record.get("codepoints", ()))
    all_cmap = record.get("all_cmap", False)
    if not isinstance(all_cmap, bool):
        raise FontSdfBakeManifestError("all_cmap must be a boolean")
    variation_hash = record.get("variation_hash")
    if variation_hash is not None and not isinstance(variation_hash, str):
        raise FontSdfBakeManifestError("variation_hash must be a hexadecimal string")
    return FontSdfBakeSpec(
        font=font,
        cache_root=cache_root,
        asset_guid=asset_guid,
        face_index=_integer(record, "face_index", 0),
        mode=_string(record, "mode", "sdf").lower(),
        codepoints=codepoints,
        all_cmap=all_cmap,
        page_size=_integer(record, "page_size", 1024),
        bake_em_px=_integer(record, "bake_em_px", 48),
        spread_px_milli=_integer(record, "spread_px_milli", 8_000),
        variation_hash=variation_hash,
    )


def _required_path(record: Mapping, field: str, repo_root: Path) -> Path:
    value = _required_string(record, field)
    path = Path(value)
    return path if path.is_absolute() else repo_root / path


def _required_string(record: Mapping, field: str) -> str:
    value = record.get(field)
    if not isinstance(value, str) or not value.strip():
        raise FontSdfBakeManifestError(f"{field} must be a non-empty string")
    if value != value.strip():
        raise FontSdfBakeManifestError(f"{field} must not contain surrounding whitespace")
    return value


def _string(record: Mapping, field: str, default: str) -> str:
    value = record.get(field, default)
    if not isinstance(value, str) or not value:
        raise FontSdfBakeManifestError(f"{field} must be a non-empty string")
    return value


def _integer(record: Mapping, field: str, default: int) -> int:
    value = record.get(field, default)
    if isinstance(value, bool) or not isinstance(value, int):
        raise FontSdfBakeManifestError(f"{field} must be an integer")
    return value


def _codepoints(values: object) -> tuple[str, ...]:
    if not isinstance(values, Sequence) or isinstance(values, (str, bytes)):
        raise FontSdfBakeManifestError("codepoints must be an array")
    selected_ranges: list[tuple[int, int]] = []
    for value in values:
        if not isinstance(value, str):
            raise FontSdfBakeManifestError("codepoints must use U+XXXX notation")
        bounds = value.split("-", maxsplit=1)
        start = _codepoint_scalar(bounds[0], value)
        end = _codepoint_scalar(bounds[-1], value)
        if end < start:
            raise FontSdfBakeManifestError(f"codepoint range is reversed: {value!r}")
        if start <= 0xDFFF and end >= 0xD800:
            raise FontSdfBakeManifestError(
                f"range contains a surrogate, not a Unicode scalar: {value!r}"
            )
        selected_ranges.append((start, end))
    merged_ranges = _merge_codepoint_ranges(selected_ranges)
    return tuple(
        f"U+{codepoint:04X}"
        for start, end in merged_ranges
        for codepoint in range(start, end + 1)
    )


def _merge_codepoint_ranges(
    selected_ranges: list[tuple[int, int]],
) -> list[tuple[int, int]]:
    merged_ranges: list[tuple[int, int]] = []
    for start, end in sorted(selected_ranges):
        if merged_ranges and start <= merged_ranges[-1][1] + 1:
            previous_start, previous_end = merged_ranges[-1]
            merged_ranges[-1] = (previous_start, max(previous_end, end))
        else:
            merged_ranges.append((start, end))
    return merged_ranges


def _codepoint_scalar(value: str, source: str) -> int:
    if not value.upper().startswith("U+") or not value[2:]:
        raise FontSdfBakeManifestError("codepoints must use U+XXXX notation")
    try:
        codepoint = int(value[2:], 16)
        chr(codepoint)
    except (ValueError, OverflowError) as error:
        raise FontSdfBakeManifestError(f"invalid codepoint {source!r}") from error
    if 0xD800 <= codepoint <= 0xDFFF:
        raise FontSdfBakeManifestError(
            f"surrogate is not a Unicode scalar: {source!r}"
        )
    return codepoint


def _validate_hash(value: str, field: str) -> None:
    if len(value) != 64:
        raise FontSdfBakeManifestError(f"{field} must contain 64 hexadecimal digits")
    try:
        bytes.fromhex(value)
    except ValueError as error:
        raise FontSdfBakeManifestError(f"{field} must be hexadecimal") from error


def _quote_command(command: Sequence[str]) -> str:
    return subprocess.list2cmdline([str(value) for value in command])
