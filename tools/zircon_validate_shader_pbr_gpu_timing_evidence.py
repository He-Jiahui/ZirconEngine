"""Validate a calibrated Zircon PBR viewer GPU timing distribution."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping


GPU_TIMING_EVIDENCE_SCHEMA = "zircon_shader_pbr_viewer_gpu_timing_evidence_v2"
GPU_TIMING_WARMUP_SAMPLE_COUNT = 5
GPU_TIMING_MEASURED_SAMPLE_COUNT = 31
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
_PASS_AGGREGATE_PATTERN = re.compile(
    r"pass\.([a-z][a-z0-9_]*)\.(min|median|p95|max)_us\Z"
)
_TOTAL_AGGREGATE_PATTERN = re.compile(r"total\.(min|median|p95|max)_us\Z")
_SAMPLE_STANDARD_PATTERN = re.compile(
    r"sample\.([0-9]{3})\.(frame_generation|total_us)\Z"
)
_SAMPLE_PASS_PATTERN = re.compile(
    r"sample\.([0-9]{3})\.pass\.([a-z][a-z0-9_]*)_us\Z"
)
_SHA256_HEXDIGEST_PATTERN = re.compile(r"[0-9a-f]{64}\Z")
_MAX_U32 = (1 << 32) - 1
_MAX_U64 = (1 << 64) - 1
_MEASURED_STANDARD_FIELDS = frozenset(
    {
        "schema",
        "status",
        "screenshot",
        "screenshot_sha256",
        "screenshot_frame_generation",
        "warmup_sample_count",
        "warmup_first_frame_generation",
        "warmup_last_frame_generation",
        "measured_sample_count",
        "first_measured_frame_generation",
        "last_measured_frame_generation",
        "timestamp_period_ns_bits",
        "timestamp_period_ns",
        "timestamp_frequency_hz",
        "percentile_policy",
        "outlier_policy",
        "pass_coverage",
    }
)


@dataclass(frozen=True)
class GpuTimingDistribution:
    min_us: int
    median_us: int
    p95_us: int
    max_us: int
    samples_us: tuple[int, ...]


@dataclass(frozen=True)
class GpuTimingEvidence:
    report_path: Path
    screenshot_frame_generation: int
    first_measured_frame_generation: int
    last_measured_frame_generation: int
    measured_sample_count: int
    timestamp_period_ns: float
    timestamp_frequency_hz: float
    total_distribution: GpuTimingDistribution
    pass_distributions: Mapping[str, GpuTimingDistribution]

    @property
    def pass_median_gpu_time_us(self) -> Mapping[str, int]:
        return {
            pass_name: distribution.median_us
            for pass_name, distribution in self.pass_distributions.items()
        }


def validate_gpu_timing_evidence(
    report_path: str | Path, screenshot_path: str | Path
) -> GpuTimingEvidence:
    """Validate one measured HDRI viewer distribution without starting the engine."""

    path = Path(report_path)
    screenshot = Path(screenshot_path)
    if not screenshot.is_file():
        raise RuntimeError(f"GPU timing evidence screenshot is unavailable: {screenshot}")
    fields = _read_fields(path)
    _require_exact_value(fields, "schema", GPU_TIMING_EVIDENCE_SCHEMA, path)
    _require_exact_value(fields, "status", "measured", path)
    _require_measured_fields(fields, path)
    _validate_screenshot_identity(fields, screenshot, path)
    _require_exact_value(fields, "percentile_policy", "nearest_rank", path)
    _require_exact_value(
        fields, "outlier_policy", "none_all_samples_retained", path
    )

    screenshot_generation = _require_positive_u64(
        fields, "screenshot_frame_generation", path
    )
    warmup_count = _require_positive_u64(fields, "warmup_sample_count", path)
    measured_count = _require_positive_u64(fields, "measured_sample_count", path)
    if warmup_count != GPU_TIMING_WARMUP_SAMPLE_COUNT:
        raise RuntimeError(
            "GPU timing evidence warmup policy was downgraded: "
            f"expected={GPU_TIMING_WARMUP_SAMPLE_COUNT} actual={warmup_count} path={path}"
        )
    if measured_count != GPU_TIMING_MEASURED_SAMPLE_COUNT:
        raise RuntimeError(
            "GPU timing evidence sample policy was downgraded: "
            f"expected={GPU_TIMING_MEASURED_SAMPLE_COUNT} actual={measured_count} path={path}"
        )
    warmup_first = _require_positive_u64(
        fields, "warmup_first_frame_generation", path
    )
    warmup_last = _require_positive_u64(fields, "warmup_last_frame_generation", path)
    first_measured = _require_positive_u64(
        fields, "first_measured_frame_generation", path
    )
    last_measured = _require_positive_u64(
        fields, "last_measured_frame_generation", path
    )
    expected_generations = (
        screenshot_generation + 1,
        screenshot_generation + warmup_count,
        screenshot_generation + warmup_count + 1,
        screenshot_generation + warmup_count + measured_count,
    )
    if max(expected_generations) > _MAX_U64 or (
        warmup_first,
        warmup_last,
        first_measured,
        last_measured,
    ) != expected_generations:
        raise RuntimeError(
            "GPU timing evidence generations are not one consecutive post-screenshot range: "
            f"path={path}"
        )

    timestamp_period_ns, timestamp_frequency_hz = _read_timestamp_calibration(
        fields, path
    )
    pass_names = _read_pass_coverage(fields, path)
    _require_exact_distribution_fields(fields, pass_names, path)
    sample_generations, total_samples, pass_samples = _read_samples(
        fields, pass_names, path
    )
    if sample_generations != list(range(first_measured, last_measured + 1)):
        raise RuntimeError(
            f"GPU timing evidence raw samples are not consecutive: path={path}"
        )

    total_distribution = _validated_distribution(fields, "total", total_samples, path)
    pass_distributions = {
        pass_name: _validated_distribution(
            fields, f"pass.{pass_name}", pass_samples[pass_name], path
        )
        for pass_name in pass_names
    }
    return GpuTimingEvidence(
        path,
        screenshot_generation,
        first_measured,
        last_measured,
        measured_count,
        timestamp_period_ns,
        timestamp_frequency_hz,
        total_distribution,
        pass_distributions,
    )


def _require_measured_fields(fields: Mapping[str, str], path: Path) -> None:
    unexpected_fields = sorted(
        field
        for field in fields
        if field not in _MEASURED_STANDARD_FIELDS
        and _PASS_AGGREGATE_PATTERN.fullmatch(field) is None
        and _TOTAL_AGGREGATE_PATTERN.fullmatch(field) is None
        and _SAMPLE_STANDARD_PATTERN.fullmatch(field) is None
        and _SAMPLE_PASS_PATTERN.fullmatch(field) is None
    )
    if unexpected_fields:
        raise RuntimeError(
            "GPU timing evidence contains unexpected measured fields: "
            f"fields={', '.join(unexpected_fields)} path={path}"
        )


def _validate_screenshot_identity(
    fields: Mapping[str, str], screenshot: Path, path: Path
) -> None:
    reported_screenshot = fields.get("screenshot")
    if reported_screenshot != screenshot.name:
        raise RuntimeError(
            "GPU timing evidence screenshot does not match the paired PNG: "
            f"expected={screenshot.name!r}, actual={reported_screenshot!r}, path={path}"
        )
    reported_sha256 = fields.get("screenshot_sha256")
    if (
        reported_sha256 is None
        or _SHA256_HEXDIGEST_PATTERN.fullmatch(reported_sha256) is None
    ):
        raise RuntimeError(f"GPU timing evidence SHA-256 is malformed: path={path}")
    actual_sha256 = _screenshot_sha256(screenshot)
    if reported_sha256 != actual_sha256:
        raise RuntimeError(
            "GPU timing evidence screenshot SHA-256 does not match the paired PNG: "
            f"expected={reported_sha256}, actual={actual_sha256}, path={path}"
        )


def _read_timestamp_calibration(
    fields: Mapping[str, str], path: Path
) -> tuple[float, float]:
    period_bits = _require_non_negative_integer(
        fields, "timestamp_period_ns_bits", path, _MAX_U32
    )
    calibrated_period = struct.unpack("!f", period_bits.to_bytes(4, "big"))[0]
    period = _require_positive_finite_float(fields, "timestamp_period_ns", path)
    frequency = _require_positive_finite_float(fields, "timestamp_frequency_hz", path)
    if not math.isfinite(calibrated_period) or calibrated_period <= 0.0:
        raise RuntimeError(f"GPU timing timestamp period bits are invalid: path={path}")
    if not math.isclose(period, calibrated_period, rel_tol=1e-9, abs_tol=5e-10):
        raise RuntimeError(
            f"GPU timing timestamp period does not match its raw calibration bits: path={path}"
        )
    expected_frequency = 1_000_000_000.0 / calibrated_period
    if not math.isclose(frequency, expected_frequency, rel_tol=1e-9, abs_tol=0.001):
        raise RuntimeError(
            f"GPU timing timestamp frequency does not match its calibrated period: path={path}"
        )
    return period, frequency


def _read_pass_coverage(fields: Mapping[str, str], path: Path) -> tuple[str, ...]:
    raw_coverage = fields.get("pass_coverage")
    if raw_coverage is None:
        raise RuntimeError(f"GPU timing evidence is missing pass coverage: path={path}")
    pass_names = tuple(raw_coverage.split(","))
    if (
        not pass_names
        or len(set(pass_names)) != len(pass_names)
        or any(_PASS_NAME_PATTERN.fullmatch(name) is None for name in pass_names)
        or pass_names != tuple(sorted(pass_names))
    ):
        raise RuntimeError(f"GPU timing evidence pass coverage is malformed: path={path}")
    unknown = sorted(set(pass_names).difference(_ALLOWED_DIRECT_GPU_PASSES))
    if unknown:
        raise RuntimeError(
            "GPU timing evidence contains an unknown direct pass: "
            f"passes={', '.join(unknown)} path={path}"
        )
    missing = sorted(set(REQUIRED_HDRI_DIRECT_GPU_PASSES).difference(pass_names))
    if missing:
        raise RuntimeError(
            "GPU timing evidence is missing required HDRI direct passes: "
            f"{', '.join(missing)} path={path}"
        )
    return pass_names


def _require_exact_distribution_fields(
    fields: Mapping[str, str], pass_names: tuple[str, ...], path: Path
) -> None:
    expected_fields = set(_MEASURED_STANDARD_FIELDS)
    expected_fields.update(
        f"total.{statistic}_us" for statistic in ("min", "median", "p95", "max")
    )
    for pass_name in pass_names:
        expected_fields.update(
            f"pass.{pass_name}.{statistic}_us"
            for statistic in ("min", "median", "p95", "max")
        )
    for index in range(GPU_TIMING_MEASURED_SAMPLE_COUNT):
        prefix = f"sample.{index:03}"
        expected_fields.update((f"{prefix}.frame_generation", f"{prefix}.total_us"))
        expected_fields.update(
            f"{prefix}.pass.{pass_name}_us" for pass_name in pass_names
        )

    actual_fields = set(fields)
    if actual_fields != expected_fields:
        missing = sorted(expected_fields.difference(actual_fields))
        unexpected = sorted(actual_fields.difference(expected_fields))
        raise RuntimeError(
            "GPU timing evidence does not contain the exact field set for its declared "
            f"distribution: missing={missing} unexpected={unexpected} path={path}"
        )


def _read_samples(
    fields: Mapping[str, str], pass_names: tuple[str, ...], path: Path
) -> tuple[list[int], list[int], dict[str, list[int]]]:
    generations: list[int] = []
    totals: list[int] = []
    pass_samples = {pass_name: [] for pass_name in pass_names}
    for index in range(GPU_TIMING_MEASURED_SAMPLE_COUNT):
        prefix = f"sample.{index:03}"
        generation = _require_positive_u64(fields, f"{prefix}.frame_generation", path)
        total = _require_non_negative_integer(fields, f"{prefix}.total_us", path, _MAX_U64)
        values = []
        for pass_name in pass_names:
            value = _require_non_negative_integer(
                fields, f"{prefix}.pass.{pass_name}_us", path, _MAX_U64
            )
            values.append(value)
            pass_samples[pass_name].append(value)
        if sum(values) > _MAX_U64 or total != sum(values):
            raise RuntimeError(
                f"GPU timing sample total does not match its pass samples: index={index} path={path}"
            )
        generations.append(generation)
        totals.append(total)
    return generations, totals, pass_samples


def _validated_distribution(
    fields: Mapping[str, str], prefix: str, samples: list[int], path: Path
) -> GpuTimingDistribution:
    expected = {
        "min": min(samples),
        "median": _nearest_rank(samples, 50),
        "p95": _nearest_rank(samples, 95),
        "max": max(samples),
    }
    actual = {
        statistic: _require_non_negative_integer(
            fields, f"{prefix}.{statistic}_us", path, _MAX_U64
        )
        for statistic in ("min", "median", "p95", "max")
    }
    if actual != expected:
        raise RuntimeError(
            "GPU timing aggregate does not match retained raw samples: "
            f"prefix={prefix} expected={expected} actual={actual} path={path}"
        )
    return GpuTimingDistribution(
        actual["min"],
        actual["median"],
        actual["p95"],
        actual["max"],
        tuple(samples),
    )


def _nearest_rank(samples: list[int], percentile: int) -> int:
    sorted_samples = sorted(samples)
    rank = (len(sorted_samples) * percentile + 99) // 100
    return sorted_samples[rank - 1]


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
    parsed = _require_non_negative_integer(fields, field, path, _MAX_U64)
    if parsed == 0:
        raise RuntimeError(
            "GPU timing evidence positive u64 is out of range: "
            f"field={field} value={parsed} path={path}"
        )
    return parsed


def _require_non_negative_integer(
    fields: Mapping[str, str], field: str, path: Path, maximum: int
) -> int:
    value = fields.get(field)
    if value is None or _NON_NEGATIVE_INTEGER_PATTERN.fullmatch(value) is None:
        raise RuntimeError(
            "GPU timing evidence non-negative integer is malformed: "
            f"field={field} path={path}"
        )
    parsed = int(value)
    if parsed > maximum:
        raise RuntimeError(
            "GPU timing evidence integer is out of range: "
            f"field={field} value={parsed} path={path}"
        )
    return parsed


def _require_positive_finite_float(
    fields: Mapping[str, str], field: str, path: Path
) -> float:
    value = fields.get(field)
    try:
        parsed = float(value) if value is not None else math.nan
    except ValueError as error:
        raise RuntimeError(
            f"GPU timing evidence floating-point field is malformed: field={field} path={path}"
        ) from error
    if not math.isfinite(parsed) or parsed <= 0.0:
        raise RuntimeError(
            f"GPU timing evidence floating-point field must be finite and positive: field={field} path={path}"
        )
    return parsed


def _screenshot_sha256(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        raise RuntimeError(f"GPU timing evidence screenshot is unavailable: {path}") from error


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a calibrated Zircon PBR viewer GPU timing distribution."
    )
    parser.add_argument(
        "report", type=Path, help="GPU timing report written by zircon_shader_pbr_viewer"
    )
    parser.add_argument(
        "png", type=Path, help="matching Ready-frame PNG written by zircon_shader_pbr_viewer"
    )
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
                "first_measured_frame_generation": evidence.first_measured_frame_generation,
                "gpu_timing_report": str(evidence.report_path),
                "last_measured_frame_generation": evidence.last_measured_frame_generation,
                "measured_sample_count": evidence.measured_sample_count,
                "pass_distributions_us": {
                    pass_name: {
                        "max": distribution.max_us,
                        "median": distribution.median_us,
                        "min": distribution.min_us,
                        "p95": distribution.p95_us,
                    }
                    for pass_name, distribution in evidence.pass_distributions.items()
                },
                "png": str(arguments.png),
                "schema": GPU_TIMING_EVIDENCE_SCHEMA,
                "screenshot_frame_generation": evidence.screenshot_frame_generation,
                "timestamp_frequency_hz": evidence.timestamp_frequency_hz,
                "timestamp_period_ns": evidence.timestamp_period_ns,
                "total_distribution_us": {
                    "max": evidence.total_distribution.max_us,
                    "median": evidence.total_distribution.median_us,
                    "min": evidence.total_distribution.min_us,
                    "p95": evidence.total_distribution.p95_us,
                },
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
