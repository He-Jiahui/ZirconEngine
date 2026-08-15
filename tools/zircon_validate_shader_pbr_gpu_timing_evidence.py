"""Validate a measured Zircon PBR viewer direct GPU timing report."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping


GPU_TIMING_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_gpu_timing_evidence_v1"
REQUIRED_HDRI_DIRECT_GPU_PASSES = (
    "direct_gpu_scene_upload",
    "direct_scene_content",
    "direct_output_transfer",
    "direct_overlays",
)
OPTIONAL_DIRECT_GPU_PASSES = (
    "direct_realtime_ibl",
    "direct_ui",
)
_ALLOWED_DIRECT_GPU_PASSES = frozenset(
    REQUIRED_HDRI_DIRECT_GPU_PASSES + OPTIONAL_DIRECT_GPU_PASSES
)
_NON_NEGATIVE_INTEGER_PATTERN = re.compile(r"[0-9]+\Z")
_PASS_NAME_PATTERN = re.compile(r"[a-z][a-z0-9_]*\Z")
_SHA256_HEXDIGEST_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
_MAX_U64 = (1 << 64) - 1
_MEASURED_STANDARD_FIELDS = frozenset(
    {"schema", "status", "screenshot", "screenshot_sha256", "frame_generation"}
)


@dataclass(frozen=True)
class GpuTimingEvidence:
    report_path: Path
    frame_generation: int
    pass_gpu_time_us: Mapping[str, int]


def validate_gpu_timing_evidence(
    report_path: str | Path, screenshot_path: str | Path
) -> GpuTimingEvidence:
    """Validate one measured HDRI viewer GPU timing report without starting the engine."""

    path = Path(report_path)
    screenshot = Path(screenshot_path)
    if not screenshot.is_file():
        raise RuntimeError(f"GPU timing evidence screenshot is unavailable: {screenshot}")
    fields = _read_fields(path)
    _require_exact_value(fields, "schema", GPU_TIMING_EVIDENCE_SCHEMA, path)
    _require_exact_value(fields, "status", "measured", path)
    _require_measured_fields(fields, path)
    reported_screenshot = fields.get("screenshot")
    if reported_screenshot != screenshot.name:
        raise RuntimeError(
            "GPU timing evidence screenshot does not match the paired PNG: "
            f"expected={screenshot.name!r}, actual={reported_screenshot!r}, path={path}"
        )
    reported_screenshot_sha256 = fields.get("screenshot_sha256")
    if (
        reported_screenshot_sha256 is None
        or _SHA256_HEXDIGEST_PATTERN.fullmatch(reported_screenshot_sha256) is None
    ):
        raise RuntimeError(f"GPU timing evidence SHA-256 is malformed: path={path}")
    actual_screenshot_sha256 = _screenshot_sha256(screenshot)
    if reported_screenshot_sha256 != actual_screenshot_sha256:
        raise RuntimeError(
            "GPU timing evidence screenshot SHA-256 does not match the paired PNG: "
            f"expected={reported_screenshot_sha256}, actual={actual_screenshot_sha256}, path={path}"
        )
    frame_generation = _require_positive_u64(fields, "frame_generation", path)
    pass_gpu_time_us = _read_pass_gpu_time_us(fields, path)
    missing_passes = [
        pass_name
        for pass_name in REQUIRED_HDRI_DIRECT_GPU_PASSES
        if pass_name not in pass_gpu_time_us
    ]
    if missing_passes:
        raise RuntimeError(
            "GPU timing evidence is missing required HDRI direct passes: "
            f"{', '.join(missing_passes)} path={path}"
        )
    return GpuTimingEvidence(path, frame_generation, pass_gpu_time_us)


def _require_measured_fields(fields: Mapping[str, str], path: Path) -> None:
    unexpected_fields = sorted(
        field
        for field in fields
        if field not in _MEASURED_STANDARD_FIELDS and not field.startswith("pass.")
    )
    if unexpected_fields:
        raise RuntimeError(
            "GPU timing evidence contains unexpected measured fields: "
            f"fields={', '.join(unexpected_fields)} path={path}"
        )


def _read_fields(path: Path) -> dict[str, str]:
    try:
        report = path.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"GPU timing evidence is unavailable: {path}") from error
    fields: dict[str, str] = {}
    for line_number, line in enumerate(report.splitlines(), start=1):
        if not line:
            continue
        key, separator, value = line.partition("=")
        if not separator or not key or key != key.strip() or not value:
            raise RuntimeError(
                "GPU timing evidence contains an invalid field: "
                f"line={line_number} path={path}"
            )
        if key in fields:
            raise RuntimeError(
                "GPU timing evidence repeats a field: " f"field={key} path={path}"
            )
        fields[key] = value
    return fields


def _require_exact_value(
    fields: Mapping[str, str], field: str, expected: str, path: Path
) -> None:
    actual = fields.get(field)
    if actual != expected:
        raise RuntimeError(
            f"GPU timing evidence requires {field}={expected}: "
            f"actual={actual!r} path={path}"
        )


def _require_positive_u64(fields: Mapping[str, str], field: str, path: Path) -> int:
    value = fields.get(field)
    if value is None or _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(value) is None:
        raise RuntimeError(
            "GPU timing evidence positive integer is malformed: "
            f"field={field} path={path}"
        )
    parsed = int(value)
    if parsed == 0 or parsed > _MAX_U64:
        raise RuntimeError(
            "GPU timing evidence positive u64 is out of range: "
            f"field={field} value={parsed} path={path}"
        )
    return parsed


def _read_pass_gpu_time_us(fields: Mapping[str, str], path: Path) -> dict[str, int]:
    pass_gpu_time_us: dict[str, int] = {}
    for field, value in fields.items():
        if not field.startswith("pass."):
            continue
        pass_name = field.removeprefix("pass.")
        if _PASS_NAME_PATTERN.fullmatch(pass_name) is None:
            raise RuntimeError(
                "GPU timing evidence pass name is malformed: "
                f"field={field} path={path}"
            )
        if pass_name not in _ALLOWED_DIRECT_GPU_PASSES:
            raise RuntimeError(
                "GPU timing evidence contains an unknown direct pass: "
                f"field={field} path={path}"
            )
        if _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(value) is None:
            raise RuntimeError(
                "GPU timing evidence pass duration is malformed: "
                f"field={field} path={path}"
            )
        gpu_time_us = int(value)
        if gpu_time_us > _MAX_U64:
            raise RuntimeError(
                "GPU timing evidence pass duration exceeds u64: "
                f"field={field} value={gpu_time_us} path={path}"
            )
        pass_gpu_time_us[pass_name] = gpu_time_us
    return pass_gpu_time_us


def _screenshot_sha256(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise RuntimeError(f"GPU timing evidence screenshot is unavailable: {path}") from error


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a measured Zircon PBR viewer direct GPU timing report."
    )
    parser.add_argument("report", type=Path, help="GPU timing report written by zircon_shader_pbr_viewer")
    parser.add_argument("png", type=Path, help="matching Ready-frame PNG written by zircon_shader_pbr_viewer")
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        evidence = validate_gpu_timing_evidence(arguments.report, arguments.png)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"PBR viewer GPU timing validation failed: {error}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "frame_generation": evidence.frame_generation,
                "gpu_timing_report": str(evidence.report_path),
                "png": str(arguments.png),
                "pass_gpu_time_us": evidence.pass_gpu_time_us,
                "schema": GPU_TIMING_EVIDENCE_SCHEMA,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
