"""Validate a measured Zircon render performance sidecar without starting Cargo."""

from __future__ import annotations

import argparse
import binascii
import json
import math
import struct
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping


RENDER_MEASUREMENT_EVIDENCE_SCHEMA = "zircon_render_measurement_evidence_v1"
DEFAULT_ARTIFACT_ROOT = Path("docs/tests/runtime/render")
WORKLOAD_NAMES = frozenset(
    {
        "control_shared_material",
        "stress_unique_materials",
        "stress_culled_materials",
    }
)
VARIANTS = frozenset({"standard", "bindless"})
_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


@dataclass(frozen=True)
class RenderMeasurementEvidence:
    report_path: Path
    workload_name: str
    variant: str
    accepted_for_default: bool


def validate_render_measurement_evidence(
    report_path: str | Path,
    *,
    require_artifacts: bool = False,
    artifact_root: str | Path | None = None,
) -> RenderMeasurementEvidence:
    """Validate one Render19 sidecar and its decision-critical invariants."""

    path = Path(report_path)
    document = _read_json_object(path, "sidecar")
    _require_exact_keys(
        document,
        {
            "schema",
            "source",
            "adapter",
            "workload",
            "observations",
            "material_binds",
            "calibration",
            "artifacts",
            "decision",
        },
        path,
        "sidecar",
    )
    _require_exact_value(document, "schema", RENDER_MEASUREMENT_EVIDENCE_SCHEMA, path)
    _validate_source(document["source"], path)
    adapter = _validate_adapter(document["adapter"], path)
    workload_name, variant = _validate_workload(document["workload"], path)
    observations = _validate_observations(document["observations"], path)
    material_binds = _validate_material_binds(document["material_binds"], path)
    calibration = _validate_calibration(document["calibration"], path)
    if calibration["counter_set_count"] != material_binds["aggregate_set_count"]:
        raise RuntimeError(
            "Render measurement calibration counter_set_count does not equal the "
            f"aggregate material-bind count: path={path}"
        )
    artifacts = _validate_artifacts(document["artifacts"], path)
    decision = _validate_decision(document["decision"], path)
    _validate_default_gate(
        adapter,
        variant,
        observations,
        material_binds,
        calibration,
        artifacts,
        decision,
        path,
    )
    if require_artifacts:
        root = _resolve_artifact_root(path, artifact_root)
        _validate_artifact_files(artifacts, root, path)
    return RenderMeasurementEvidence(
        report_path=path,
        workload_name=workload_name,
        variant=variant,
        accepted_for_default=decision["accepted_for_default"],
    )


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except OSError as error:
        raise RuntimeError(f"Render measurement {label} is unavailable: {path}") from error
    except json.JSONDecodeError as error:
        raise RuntimeError(f"Render measurement {label} is not valid JSON: {path}") from error
    if not isinstance(document, dict):
        raise RuntimeError(f"Render measurement {label} must be a JSON object: path={path}")
    return document


def _require_exact_keys(
    value: Mapping[str, Any], expected: set[str], path: Path, label: str
) -> None:
    actual = set(value)
    missing = sorted(expected - actual)
    unexpected = sorted(actual - expected)
    if missing or unexpected:
        details = []
        if missing:
            details.append(f"missing={','.join(missing)}")
        if unexpected:
            details.append(f"unexpected={','.join(unexpected)}")
        raise RuntimeError(
            f"Render measurement {label} fields do not match the contract: "
            f"{' '.join(details)} path={path}"
        )


def _require_exact_value(
    value: Mapping[str, Any], field: str, expected: str, path: Path
) -> None:
    if value[field] != expected:
        raise RuntimeError(
            f"Render measurement requires {field}={expected!r}: "
            f"actual={value[field]!r} path={path}"
        )


def _require_object(value: Any, label: str, path: Path) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        raise RuntimeError(f"Render measurement {label} must be an object: path={path}")
    return value


def _require_non_empty_string(value: Any, label: str, path: Path) -> str:
    if not isinstance(value, str) or not value.strip():
        raise RuntimeError(
            f"Render measurement {label} must be a non-empty string: path={path}"
        )
    return value


def _require_non_negative_integer(value: Any, label: str, path: Path) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise RuntimeError(
            f"Render measurement {label} must be a non-negative integer: path={path}"
        )
    return value


def _require_positive_integer(value: Any, label: str, path: Path) -> int:
    integer = _require_non_negative_integer(value, label, path)
    if integer == 0:
        raise RuntimeError(f"Render measurement {label} must be positive: path={path}")
    return integer


def _require_non_negative_number(value: Any, label: str, path: Path) -> float | int:
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(value)
        or value < 0
    ):
        raise RuntimeError(
            f"Render measurement {label} must be a non-negative number: path={path}"
        )
    return value


def _validate_source(value: Any, path: Path) -> None:
    source = _require_object(value, "source", path)
    _require_exact_keys(
        source,
        {"revision", "source_fingerprint", "session_id", "validation_ticket"},
        path,
        "source",
    )
    for field in source:
        _require_non_empty_string(source[field], f"source.{field}", path)


def _validate_adapter(value: Any, path: Path) -> Mapping[str, Any]:
    adapter = _require_object(value, "adapter", path)
    _require_exact_keys(
        adapter,
        {
            "name",
            "backend",
            "driver",
            "requested_features",
            "limits",
            "bindless_gate",
            "slot_capacity",
        },
        path,
        "adapter",
    )
    for field in ("name", "backend", "driver", "bindless_gate"):
        _require_non_empty_string(adapter[field], f"adapter.{field}", path)
    if not isinstance(adapter["requested_features"], list) or not all(
        isinstance(feature, str) and feature for feature in adapter["requested_features"]
    ):
        raise RuntimeError(
            "Render measurement adapter.requested_features must be an array of strings: "
            f"path={path}"
        )
    if not isinstance(adapter["limits"], dict):
        raise RuntimeError(f"Render measurement adapter.limits must be an object: path={path}")
    _require_non_negative_integer(adapter["slot_capacity"], "adapter.slot_capacity", path)
    return adapter


def _validate_workload(value: Any, path: Path) -> tuple[str, str]:
    workload = _require_object(value, "workload", path)
    _require_exact_keys(
        workload,
        {
            "name",
            "variant",
            "resolution",
            "quality_profile",
            "camera_fingerprint",
            "warmup_frames",
            "sampled_frames",
        },
        path,
        "workload",
    )
    name = workload["name"]
    if name not in WORKLOAD_NAMES:
        raise RuntimeError(
            f"Render measurement workload.name is not a protocol workload: {name!r} path={path}"
        )
    variant = workload["variant"]
    if variant not in VARIANTS:
        raise RuntimeError(
            f"Render measurement workload.variant is not supported: {variant!r} path={path}"
        )
    resolution = _require_object(workload["resolution"], "workload.resolution", path)
    _require_exact_keys(resolution, {"width", "height"}, path, "workload.resolution")
    _require_positive_integer(resolution["width"], "workload.resolution.width", path)
    _require_positive_integer(resolution["height"], "workload.resolution.height", path)
    _require_non_empty_string(workload["quality_profile"], "workload.quality_profile", path)
    _require_non_empty_string(workload["camera_fingerprint"], "workload.camera_fingerprint", path)
    if workload["warmup_frames"] != 30 or workload["sampled_frames"] != 120:
        raise RuntimeError(
            "Render measurement workload must use the protocol's 30 warm-up and 120 sampled frames: "
            f"path={path}"
        )
    return name, variant


def _validate_statistic(value: Any, label: str, path: Path) -> Mapping[str, Any]:
    statistic = _require_object(value, label, path)
    _require_exact_keys(statistic, {"median", "p95", "mad"}, path, label)
    for field in statistic:
        _require_non_negative_number(statistic[field], f"{label}.{field}", path)
    if statistic["p95"] < statistic["median"]:
        raise RuntimeError(
            f"Render measurement {label}.p95 must not be below its median: path={path}"
        )
    return statistic


def _validate_observations(value: Any, path: Path) -> Mapping[str, Any]:
    observations = _require_object(value, "observations", path)
    _require_exact_keys(
        observations,
        {
            "valid_frame_count",
            "excluded_pending_timing_count",
            "excluded_unavailable_timing_count",
            "cpu_mesh_encode_ns",
            "gpu_frame_ns",
            "board_power_w",
            "power_telemetry",
        },
        path,
        "observations",
    )
    valid = _require_positive_integer(
        observations["valid_frame_count"], "observations.valid_frame_count", path
    )
    pending = _require_non_negative_integer(
        observations["excluded_pending_timing_count"],
        "observations.excluded_pending_timing_count",
        path,
    )
    unavailable = _require_non_negative_integer(
        observations["excluded_unavailable_timing_count"],
        "observations.excluded_unavailable_timing_count",
        path,
    )
    if valid + pending + unavailable != 120:
        raise RuntimeError(
            "Render measurement timing observations must account for all 120 sampled frames: "
            f"path={path}"
        )
    _validate_statistic(observations["cpu_mesh_encode_ns"], "observations.cpu_mesh_encode_ns", path)
    _validate_statistic(observations["gpu_frame_ns"], "observations.gpu_frame_ns", path)
    telemetry = _require_object(observations["power_telemetry"], "observations.power_telemetry", path)
    _require_exact_keys(
        telemetry,
        {"probe", "sampling_interval_ms", "ac_power"},
        path,
        "observations.power_telemetry",
    )
    if telemetry["probe"] not in {"available", "unavailable"}:
        raise RuntimeError(
            "Render measurement observations.power_telemetry.probe must be available or unavailable: "
            f"path={path}"
        )
    _require_positive_integer(
        telemetry["sampling_interval_ms"],
        "observations.power_telemetry.sampling_interval_ms",
        path,
    )
    if not isinstance(telemetry["ac_power"], bool):
        raise RuntimeError(
            "Render measurement observations.power_telemetry.ac_power must be a boolean: "
            f"path={path}"
        )
    board_power = observations["board_power_w"]
    if board_power == "power_unavailable":
        if telemetry["probe"] != "unavailable":
            raise RuntimeError(
                "Render measurement power_unavailable requires an unavailable telemetry probe: "
                f"path={path}"
            )
    else:
        if telemetry["probe"] != "available":
            raise RuntimeError(
                "Render measurement board power statistics require an available telemetry probe: "
                f"path={path}"
            )
        _validate_statistic(board_power, "observations.board_power_w", path)
    return observations


def _validate_material_binds(value: Any, path: Path) -> Mapping[str, Any]:
    material_binds = _require_object(value, "material_binds", path)
    _require_exact_keys(
        material_binds,
        {"aggregate_set_count", "aggregate_skip_count", "main_mesh", "shadow"},
        path,
        "material_binds",
    )
    for domain in ("main_mesh", "shadow"):
        domain_counts = _require_object(material_binds[domain], f"material_binds.{domain}", path)
        _require_exact_keys(domain_counts, {"set_count", "skip_count"}, path, f"material_binds.{domain}")
        _require_non_negative_integer(
            domain_counts["set_count"], f"material_binds.{domain}.set_count", path
        )
        _require_non_negative_integer(
            domain_counts["skip_count"], f"material_binds.{domain}.skip_count", path
        )
    for aggregate in ("aggregate_set_count", "aggregate_skip_count"):
        _require_non_negative_integer(material_binds[aggregate], f"material_binds.{aggregate}", path)
    if material_binds["aggregate_set_count"] != (
        material_binds["main_mesh"]["set_count"] + material_binds["shadow"]["set_count"]
    ):
        raise RuntimeError(
            "Render measurement material-binds aggregate_set_count does not equal main_mesh plus shadow: "
            f"path={path}"
        )
    if material_binds["aggregate_skip_count"] != (
        material_binds["main_mesh"]["skip_count"] + material_binds["shadow"]["skip_count"]
    ):
        raise RuntimeError(
            "Render measurement material-binds aggregate_skip_count does not equal main_mesh plus shadow: "
            f"path={path}"
        )
    return material_binds


def _validate_calibration(value: Any, path: Path) -> Mapping[str, Any]:
    calibration = _require_object(value, "calibration", path)
    _require_exact_keys(
        calibration,
        {"captured_frame", "renderdoc_group2_event_count", "counter_set_count", "matched"},
        path,
        "calibration",
    )
    _require_positive_integer(calibration["captured_frame"], "calibration.captured_frame", path)
    _require_non_negative_integer(
        calibration["renderdoc_group2_event_count"],
        "calibration.renderdoc_group2_event_count",
        path,
    )
    _require_non_negative_integer(
        calibration["counter_set_count"], "calibration.counter_set_count", path
    )
    if not isinstance(calibration["matched"], bool):
        raise RuntimeError(f"Render measurement calibration.matched must be a boolean: path={path}")
    if calibration["matched"] != (
        calibration["renderdoc_group2_event_count"] == calibration["counter_set_count"]
    ):
        raise RuntimeError(
            "Render measurement calibration.matched disagrees with its RenderDoc and counter values: "
            f"path={path}"
        )
    return calibration


def _validate_artifacts(value: Any, path: Path) -> Mapping[str, Any]:
    artifacts = _require_object(value, "artifacts", path)
    _require_exact_keys(
        artifacts,
        {
            "png_path",
            "png_pixel_comparison",
            "rdc_cold_path",
            "rdc_warm_path",
            "graph_dump_path",
        },
        path,
        "artifacts",
    )
    for field, suffix in (("png_path", ".png"), ("rdc_cold_path", ".rdc"), ("rdc_warm_path", ".rdc")):
        artifact_path = _require_relative_artifact_path(artifacts[field], f"artifacts.{field}", path)
        if artifact_path.suffix.lower() != suffix:
            raise RuntimeError(
                f"Render measurement artifacts.{field} must use {suffix}: path={path}"
            )
    _require_relative_artifact_path(artifacts["graph_dump_path"], "artifacts.graph_dump_path", path)
    comparison = _require_object(artifacts["png_pixel_comparison"], "artifacts.png_pixel_comparison", path)
    _require_exact_keys(comparison, {"passed", "max_channel_error", "reason"}, path, "artifacts.png_pixel_comparison")
    if not isinstance(comparison["passed"], bool):
        raise RuntimeError(
            f"Render measurement artifacts.png_pixel_comparison.passed must be a boolean: path={path}"
        )
    _require_non_negative_integer(
        comparison["max_channel_error"],
        "artifacts.png_pixel_comparison.max_channel_error",
        path,
    )
    _require_non_empty_string(
        comparison["reason"], "artifacts.png_pixel_comparison.reason", path
    )
    return artifacts


def _require_relative_artifact_path(value: Any, label: str, path: Path) -> Path:
    raw_path = _require_non_empty_string(value, label, path)
    artifact_path = Path(raw_path)
    if (
        artifact_path.is_absolute()
        or artifact_path.drive
        or any(part == ".." for part in artifact_path.parts)
    ):
        raise RuntimeError(
            f"Render measurement {label} must be a relative path below the artifact root: path={path}"
        )
    return artifact_path


def _validate_decision(value: Any, path: Path) -> Mapping[str, Any]:
    decision = _require_object(value, "decision", path)
    _require_exact_keys(
        decision,
        {"noise_threshold", "control_result", "stress_result", "accepted_for_default", "rationale"},
        path,
        "decision",
    )
    _validate_statistic(decision["noise_threshold"], "decision.noise_threshold", path)
    for field in ("control_result", "stress_result", "rationale"):
        _require_non_empty_string(decision[field], f"decision.{field}", path)
    if not isinstance(decision["accepted_for_default"], bool):
        raise RuntimeError(
            f"Render measurement decision.accepted_for_default must be a boolean: path={path}"
        )
    return decision


def _validate_default_gate(
    adapter: Mapping[str, Any],
    variant: str,
    observations: Mapping[str, Any],
    material_binds: Mapping[str, Any],
    calibration: Mapping[str, Any],
    artifacts: Mapping[str, Any],
    decision: Mapping[str, Any],
    path: Path,
) -> None:
    if not decision["accepted_for_default"]:
        return
    failures = []
    if variant != "bindless":
        failures.append("workload.variant=bindless")
    if adapter["bindless_gate"] != "eligible":
        failures.append("adapter.bindless_gate=eligible")
    if observations["board_power_w"] == "power_unavailable":
        failures.append("measured board_power_w")
    if not calibration["matched"]:
        failures.append("calibration.matched=true")
    if not artifacts["png_pixel_comparison"]["passed"]:
        failures.append("artifacts.png_pixel_comparison.passed=true")
    if decision["control_result"] != "not_worse":
        failures.append("decision.control_result=not_worse")
    if decision["stress_result"] != "improved":
        failures.append("decision.stress_result=improved")
    if material_binds["main_mesh"]["set_count"] == 0:
        failures.append("material_binds.main_mesh.set_count>0")
    if failures:
        raise RuntimeError(
            "Render measurement cannot accept bindless as the default: "
            f"missing={','.join(failures)} path={path}"
        )


def _resolve_artifact_root(report_path: Path, artifact_root: str | Path | None) -> Path:
    root = Path(artifact_root) if artifact_root is not None else DEFAULT_ARTIFACT_ROOT
    try:
        return root.resolve(strict=True)
    except OSError as error:
        raise RuntimeError(
            f"Render measurement artifact root is unavailable: root={root} report={report_path}"
        ) from error


def _validate_artifact_files(
    artifacts: Mapping[str, Any], artifact_root: Path, report_path: Path
) -> None:
    for field in ("png_path", "rdc_cold_path", "rdc_warm_path", "graph_dump_path"):
        resolved = (artifact_root / Path(artifacts[field])).resolve()
        if artifact_root not in resolved.parents:
            raise RuntimeError(
                f"Render measurement artifacts.{field} escapes the artifact root: report={report_path}"
            )
        if not resolved.is_file() or resolved.stat().st_size == 0:
            raise RuntimeError(
                f"Render measurement artifacts.{field} is unavailable or empty: artifact={resolved}"
            )
        if field == "png_path":
            _validate_png(resolved, report_path)


def _validate_png(path: Path, report_path: Path) -> None:
    try:
        contents = path.read_bytes()
    except OSError as error:
        raise RuntimeError(f"Render measurement PNG is unavailable: artifact={path}") from error
    if len(contents) < len(_PNG_SIGNATURE) or not contents.startswith(_PNG_SIGNATURE):
        raise RuntimeError(f"Render measurement PNG has an invalid signature: artifact={path}")
    offset = len(_PNG_SIGNATURE)
    saw_ihdr = False
    saw_idat = False
    while offset < len(contents):
        if offset + 12 > len(contents):
            raise RuntimeError(f"Render measurement PNG is truncated: artifact={path}")
        length = struct.unpack(">I", contents[offset : offset + 4])[0]
        chunk_type = contents[offset + 4 : offset + 8]
        chunk_end = offset + 12 + length
        if chunk_end > len(contents):
            raise RuntimeError(f"Render measurement PNG is truncated: artifact={path}")
        payload = contents[offset + 8 : offset + 8 + length]
        actual_crc = struct.unpack(">I", contents[offset + 8 + length : chunk_end])[0]
        expected_crc = binascii.crc32(chunk_type + payload) & 0xFFFFFFFF
        if actual_crc != expected_crc:
            raise RuntimeError(f"Render measurement PNG has an invalid chunk CRC: artifact={path}")
        if chunk_type == b"IHDR":
            if saw_ihdr or offset != len(_PNG_SIGNATURE) or length != 13:
                raise RuntimeError(f"Render measurement PNG has an invalid IHDR chunk: artifact={path}")
            width, height = struct.unpack(">II", payload[:8])
            if width == 0 or height == 0:
                raise RuntimeError(
                    f"Render measurement PNG has an invalid image size: artifact={path} report={report_path}"
                )
            saw_ihdr = True
        elif chunk_type == b"IDAT":
            if not saw_ihdr:
                raise RuntimeError(f"Render measurement PNG IDAT precedes IHDR: artifact={path}")
            saw_idat = True
        elif chunk_type == b"IEND":
            if length != 0 or not saw_ihdr or not saw_idat or chunk_end != len(contents):
                raise RuntimeError(f"Render measurement PNG has an invalid IEND chunk: artifact={path}")
            return
        offset = chunk_end
    raise RuntimeError(f"Render measurement PNG is missing IEND: artifact={path}")


def _parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate a measured Zircon Render19 performance sidecar."
    )
    parser.add_argument("report", type=Path, help="Render19 JSON sidecar")
    parser.add_argument(
        "--require-artifacts",
        action="store_true",
        help="require the referenced PNG, RDC, and graph-dump files below the artifact root",
    )
    parser.add_argument(
        "--artifact-root",
        type=Path,
        help="artifact root; defaults to docs/tests/runtime/render",
    )
    return parser.parse_args()


def main() -> int:
    arguments = _parse_arguments()
    try:
        evidence = validate_render_measurement_evidence(
            arguments.report,
            require_artifacts=arguments.require_artifacts,
            artifact_root=arguments.artifact_root,
        )
    except (OSError, RuntimeError, ValueError) as error:
        print(f"Render measurement validation failed: {error}", file=sys.stderr)
        return 1
    print(
        json.dumps(
            {
                "accepted_for_default": evidence.accepted_for_default,
                "report": str(evidence.report_path),
                "schema": RENDER_MEASUREMENT_EVIDENCE_SCHEMA,
                "variant": evidence.variant,
                "workload": evidence.workload_name,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
