"""Fail-closed product evidence for bounded UI render dependency memory."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from pathlib import Path
from typing import Any


PREFIX = "ui.screen_space_ui_dependency_product"
PHASES = ("warmup", "pressure", "quiescent")
MAX_RETAINED_GENERATION_COUNT = 3
MAX_METADATA_BUDGET_BYTES = 8 * 1024 * 1024
MAX_IMAGE_POOL_BYTES = 64 * 1024 * 1024
MAX_END_MEMORY_GROWTH_BYTES = 64 * 1024 * 1024
MAX_PEAK_MEMORY_GROWTH_BYTES = 96 * 1024 * 1024
MIN_DELTA_CYCLES = 5
MIN_QUIESCENCE_MS = 2_000

REQUIRED_SOURCE_PATHS = (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
    "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs",
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
    "zircon_editor/src/ui/retained_host/ui_perf.rs",
    "tools/ui-profile-process-evidence.ps1",
)

PHASE_FIELDS = (
    "snapshot_count",
    "live_generation_count",
    "pending_retired_generation_count",
    "metadata_bytes",
    "source_payload_bytes",
    "binding_product_count",
    "unique_binding_identity_count",
    "image_shared_resident_bytes",
    "image_cache_resident_bytes",
    "image_cache_cpu_resident_bytes",
)

ACTION_COUNTERS = (
    "delta_publish_count",
    "retirement_count",
    "eviction_count",
    "global_binding_scan_count",
    "full_generation_payload_clone_bytes",
    "present_liveness_scan_count",
)


def _phase_counter(phase: str, field: str) -> str:
    return f"{PREFIX}.{phase}.{field}"


def _counter_values(timeline: dict[str, Any], name: str) -> list[Any]:
    counters = timeline.get("counters", [])
    if not isinstance(counters, list):
        return []
    return [
        entry.get("value")
        for entry in counters
        if isinstance(entry, dict) and entry.get("name") == name
    ]


def _nonnegative_integer(value: Any) -> int | None:
    if isinstance(value, bool):
        return None
    try:
        number = float(value)
    except (TypeError, ValueError, OverflowError):
        return None
    if not math.isfinite(number) or number < 0 or not number.is_integer():
        return None
    return int(number)


def _nonnegative_float(value: Any) -> float | None:
    if isinstance(value, bool):
        return None
    try:
        number = float(value)
    except (TypeError, ValueError, OverflowError):
        return None
    return number if math.isfinite(number) and number >= 0 else None


def validate_source_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    if not isinstance(manifest, dict) or manifest.get("schema_version") != 2:
        return [{"code": "invalid_source_manifest_schema"}]
    if manifest.get("scenario") != "render_dependency_memory_pressure":
        blockers.append(
            {
                "code": "invalid_source_manifest_scenario",
                "scenario": manifest.get("scenario"),
            }
        )
    options = manifest.get("capture", {}).get("options", {})
    if (
        options.get("run_phase") != "measured"
        or _nonnegative_integer(options.get("run_ordinal")) in (None, 0)
        or _nonnegative_integer(options.get("measured_run_count")) in (None, 0)
    ):
        blockers.append({"code": "invalid_capture_contract"})
    revision = manifest.get("repository", {}).get("git", {}).get("revision")
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-fA-F]{40}", revision):
        blockers.append({"code": "invalid_source_revision"})

    raw_files = manifest.get("repository", {}).get("critical_source_files", [])
    files = {
        entry.get("relative_path"): entry
        for entry in raw_files
        if isinstance(entry, dict) and isinstance(entry.get("relative_path"), str)
    }
    for path in REQUIRED_SOURCE_PATHS:
        entry = files.get(path)
        if entry is None:
            blockers.append({"code": "missing_critical_source", "path": path})
            continue
        digest = entry.get("sha256")
        if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-fA-F]{64}", digest):
            blockers.append({"code": "invalid_critical_source_hash", "path": path})
    return blockers


def _read_single_counter(
    timeline: dict[str, Any],
    name: str,
    blockers: list[dict[str, Any]],
) -> int | None:
    values = _counter_values(timeline, name)
    if not values:
        blockers.append({"code": "missing_counter", "counter": name})
        return None
    if len(values) != 1:
        blockers.append(
            {
                "code": "duplicate_snapshot_counter",
                "counter": name,
                "sample_count": len(values),
            }
        )
        return None
    value = _nonnegative_integer(values[0])
    if value is None:
        blockers.append({"code": "invalid_counter_value", "counter": name})
    return value


def _read_total_counter(
    timeline: dict[str, Any],
    name: str,
    blockers: list[dict[str, Any]],
) -> int | None:
    values = _counter_values(timeline, name)
    if not values:
        blockers.append({"code": "missing_counter", "counter": name})
        return None
    total = 0
    for raw in values:
        value = _nonnegative_integer(raw)
        if value is None:
            blockers.append({"code": "invalid_counter_value", "counter": name})
            return None
        total += value
    return total


def _evaluate_process_evidence(
    artifact: dict[str, Any],
) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    blockers: list[dict[str, Any]] = []
    interaction = artifact.get("interaction") if isinstance(artifact, dict) else None
    if not isinstance(interaction, dict):
        return {}, [{"code": "missing_interaction_evidence"}]

    requested_cycles = _nonnegative_integer(interaction.get("requested_delta_cycles"))
    completed_cycles = _nonnegative_integer(interaction.get("completed_delta_cycles"))
    if (
        interaction.get("scenario") != "render_dependency_memory_pressure"
        or requested_cycles is None
        or requested_cycles < MIN_DELTA_CYCLES
        or completed_cycles != requested_cycles
        or interaction.get("same_resource_identity_set") is not True
    ):
        blockers.append({"code": "invalid_memory_pressure_interaction"})

    integer_fields = (
        "process_id",
        "logical_processor_count",
        "start_working_set_bytes",
        "end_working_set_bytes",
        "peak_working_set_bytes",
        "start_private_bytes",
        "end_private_bytes",
        "peak_private_bytes",
        "quiescence_process_id",
        "quiescence_working_set_bytes",
        "quiescence_private_bytes",
    )
    float_fields = (
        "elapsed_ms",
        "processor_time_delta_ms",
        "cpu_core_utilization_percent",
        "cpu_system_utilization_percent",
        "quiescence_requested_ms",
        "quiescence_elapsed_ms",
    )
    integers = {
        name: _nonnegative_integer(interaction.get(name)) for name in integer_fields
    }
    floats = {name: _nonnegative_float(interaction.get(name)) for name in float_fields}
    if any(value is None for value in (*integers.values(), *floats.values())):
        blockers.append({"code": "invalid_process_evidence_value"})
        return {
            "requested_delta_cycles": requested_cycles,
            "completed_delta_cycles": completed_cycles,
        }, blockers

    process_id = integers["process_id"]
    logical_processors = integers["logical_processor_count"]
    elapsed_ms = floats["elapsed_ms"]
    cpu_ms = floats["processor_time_delta_ms"]
    requested_quiescence_ms = floats["quiescence_requested_ms"]
    elapsed_quiescence_ms = floats["quiescence_elapsed_ms"]
    if (
        process_id == 0
        or logical_processors == 0
        or elapsed_ms == 0
        or interaction.get("quiescence_sampled") is not True
        or integers["quiescence_process_id"] != process_id
        or requested_quiescence_ms < MIN_QUIESCENCE_MS
        or elapsed_quiescence_ms < requested_quiescence_ms
    ):
        blockers.append({"code": "invalid_process_quiescence"})

    memory_fields = (
        "start_working_set_bytes",
        "end_working_set_bytes",
        "peak_working_set_bytes",
        "quiescence_working_set_bytes",
        "start_private_bytes",
        "end_private_bytes",
        "peak_private_bytes",
        "quiescence_private_bytes",
    )
    if any(integers[name] == 0 for name in memory_fields):
        blockers.append({"code": "invalid_process_memory_sample"})
    if (
        integers["peak_working_set_bytes"]
        < max(
            integers["start_working_set_bytes"],
            integers["end_working_set_bytes"],
            integers["quiescence_working_set_bytes"],
        )
        or integers["peak_private_bytes"]
        < max(
            integers["start_private_bytes"],
            integers["end_private_bytes"],
            integers["quiescence_private_bytes"],
        )
    ):
        blockers.append({"code": "invalid_process_memory_peak"})

    expected_core = (cpu_ms / elapsed_ms) * 100.0 if elapsed_ms else math.inf
    expected_system = expected_core / logical_processors if logical_processors else math.inf
    core_tolerance = max(0.1, expected_core * 0.01)
    system_tolerance = max(0.1, expected_system * 0.01)
    if (
        abs(floats["cpu_core_utilization_percent"] - expected_core) > core_tolerance
        or abs(floats["cpu_system_utilization_percent"] - expected_system)
        > system_tolerance
        or floats["cpu_core_utilization_percent"] > 100.0
    ):
        blockers.append({"code": "invalid_process_cpu_evidence"})

    working_start = integers["start_working_set_bytes"]
    private_start = integers["start_private_bytes"]
    if (
        integers["end_working_set_bytes"] - working_start
        > MAX_END_MEMORY_GROWTH_BYTES
        or integers["quiescence_working_set_bytes"] - working_start
        > MAX_END_MEMORY_GROWTH_BYTES
        or integers["peak_working_set_bytes"] - working_start
        > MAX_PEAK_MEMORY_GROWTH_BYTES
        or integers["end_private_bytes"] - private_start
        > MAX_END_MEMORY_GROWTH_BYTES
        or integers["quiescence_private_bytes"] - private_start
        > MAX_END_MEMORY_GROWTH_BYTES
        or integers["peak_private_bytes"] - private_start
        > MAX_PEAK_MEMORY_GROWTH_BYTES
    ):
        blockers.append({"code": "process_memory_growth_budget_exceeded"})

    return {
        "requested_delta_cycles": requested_cycles,
        "completed_delta_cycles": completed_cycles,
        "process_id": process_id,
        "working_set_growth_bytes": integers["end_working_set_bytes"] - working_start,
        "quiescent_working_set_growth_bytes": (
            integers["quiescence_working_set_bytes"] - working_start
        ),
        "peak_working_set_growth_bytes": (
            integers["peak_working_set_bytes"] - working_start
        ),
        "private_growth_bytes": integers["end_private_bytes"] - private_start,
        "quiescent_private_growth_bytes": (
            integers["quiescence_private_bytes"] - private_start
        ),
        "peak_private_growth_bytes": integers["peak_private_bytes"] - private_start,
    }, blockers


def evaluate_memory_evidence(
    timeline: dict[str, Any], interaction_artifact: dict[str, Any]
) -> dict[str, Any]:
    blockers: list[dict[str, Any]] = []
    process, process_blockers = _evaluate_process_evidence(interaction_artifact)
    blockers.extend(process_blockers)

    singleton_names = (
        f"{PREFIX}.max_retained_generation_count",
        f"{PREFIX}.metadata_budget_bytes",
    )
    singleton = {
        name: _read_single_counter(timeline, name, blockers)
        for name in singleton_names
    }
    actions = {
        field: _read_total_counter(timeline, f"{PREFIX}.{field}", blockers)
        for field in ACTION_COUNTERS
    }
    phases: dict[str, dict[str, int | None]] = {}
    for phase in PHASES:
        phases[phase] = {
            field: _read_single_counter(
                timeline, _phase_counter(phase, field), blockers
            )
            for field in PHASE_FIELDS
        }

    max_generations = singleton[f"{PREFIX}.max_retained_generation_count"]
    metadata_budget = singleton[f"{PREFIX}.metadata_budget_bytes"]
    if max_generations is not None and (
        max_generations == 0 or max_generations > MAX_RETAINED_GENERATION_COUNT
    ):
        blockers.append(
            {
                "code": "invalid_retained_generation_bound",
                "observed": max_generations,
                "maximum": MAX_RETAINED_GENERATION_COUNT,
            }
        )
    if metadata_budget is not None and (
        metadata_budget == 0 or metadata_budget > MAX_METADATA_BUDGET_BYTES
    ):
        blockers.append(
            {
                "code": "invalid_metadata_budget",
                "observed": metadata_budget,
                "maximum": MAX_METADATA_BUDGET_BYTES,
            }
        )

    for phase, snapshot in phases.items():
        if any(value is None for value in snapshot.values()):
            continue
        if snapshot["snapshot_count"] != 1:
            blockers.append(
                {
                    "code": "invalid_phase_snapshot_count",
                    "phase": phase,
                    "observed": snapshot["snapshot_count"],
                }
            )
        if (
            max_generations is not None
            and snapshot["live_generation_count"] > max_generations
        ):
            blockers.append(
                {
                    "code": "retained_generation_bound_exceeded",
                    "phase": phase,
                    "observed": snapshot["live_generation_count"],
                    "maximum": max_generations,
                }
            )
        if snapshot["pending_retired_generation_count"] > max(
            0, snapshot["live_generation_count"] - 1
        ):
            blockers.append(
                {
                    "code": "pending_retirement_count_incoherent",
                    "phase": phase,
                    "live_generation_count": snapshot["live_generation_count"],
                    "pending_retired_generation_count": snapshot[
                        "pending_retired_generation_count"
                    ],
                }
            )
        if (
            metadata_budget is not None
            and snapshot["metadata_bytes"] > metadata_budget
        ):
            blockers.append(
                {
                    "code": "metadata_budget_exceeded",
                    "phase": phase,
                    "observed": snapshot["metadata_bytes"],
                    "maximum": metadata_budget,
                }
            )
        if snapshot["binding_product_count"] > snapshot["unique_binding_identity_count"]:
            blockers.append(
                {
                    "code": "binding_identity_conservation_failed",
                    "phase": phase,
                    "bindings": snapshot["binding_product_count"],
                    "identities": snapshot["unique_binding_identity_count"],
                }
            )
        for field in (
            "image_shared_resident_bytes",
            "image_cache_resident_bytes",
            "image_cache_cpu_resident_bytes",
        ):
            if snapshot[field] > MAX_IMAGE_POOL_BYTES:
                blockers.append(
                    {
                        "code": "image_pool_budget_exceeded",
                        "phase": phase,
                        "pool": field,
                        "observed": snapshot[field],
                        "maximum": MAX_IMAGE_POOL_BYTES,
                    }
                )

    completed_cycles = process.get("completed_delta_cycles")
    delta_publishes = actions.get("delta_publish_count")
    if (
        completed_cycles is not None
        and delta_publishes is not None
        and delta_publishes < completed_cycles
    ):
        blockers.append(
            {
                "code": "insufficient_delta_pressure_activity",
                "completed_delta_cycles": completed_cycles,
                "delta_publish_count": delta_publishes,
            }
        )
    if (
        delta_publishes is not None
        and actions.get("retirement_count") is not None
        and actions["retirement_count"] < delta_publishes
    ):
        blockers.append(
            {
                "code": "insufficient_generation_retirement",
                "delta_publish_count": delta_publishes,
                "retirement_count": actions["retirement_count"],
            }
        )
    forbidden = {
        field: actions[field]
        for field in (
            "global_binding_scan_count",
            "full_generation_payload_clone_bytes",
            "present_liveness_scan_count",
        )
        if actions.get(field) not in (None, 0)
    }
    if forbidden:
        blockers.append(
            {"code": "forbidden_global_dependency_work", "counters": forbidden}
        )

    warmup = phases["warmup"]
    pressure = phases["pressure"]
    quiescent = phases["quiescent"]
    if all(value is not None for value in warmup.values()) and (
        warmup["live_generation_count"] != 1
        or warmup["pending_retired_generation_count"] != 0
    ):
        blockers.append(
            {
                "code": "warmup_generation_not_canonical",
                "live_generation_count": warmup["live_generation_count"],
                "pending_retired_generation_count": warmup[
                    "pending_retired_generation_count"
                ],
            }
        )
    if all(value is not None for value in pressure.values()) and pressure[
        "live_generation_count"
    ] < 2:
        blockers.append({"code": "pressure_did_not_overlap_generations"})
    if all(
        warmup[field] is not None and pressure[field] is not None
        for field in ("metadata_bytes", "source_payload_bytes")
    ) and (
        pressure["metadata_bytes"] <= warmup["metadata_bytes"]
        or pressure["source_payload_bytes"] <= warmup["source_payload_bytes"]
    ):
        blockers.append(
            {
                "code": "pressure_memory_did_not_overlap",
                "warmup_metadata_bytes": warmup["metadata_bytes"],
                "pressure_metadata_bytes": pressure["metadata_bytes"],
                "warmup_source_payload_bytes": warmup["source_payload_bytes"],
                "pressure_source_payload_bytes": pressure["source_payload_bytes"],
            }
        )
    if all(value is not None for value in quiescent.values()) and (
        quiescent["live_generation_count"] != 1
        or quiescent["pending_retired_generation_count"] != 0
    ):
        blockers.append(
            {
                "code": "quiescent_generation_retirement_incomplete",
                "live_generation_count": quiescent["live_generation_count"],
                "pending_retired_generation_count": quiescent[
                    "pending_retired_generation_count"
                ],
            }
        )

    identity_counts = {
        phase: phases[phase]["unique_binding_identity_count"] for phase in PHASES
    }
    if all(value is not None for value in identity_counts.values()) and len(
        set(identity_counts.values())
    ) != 1:
        blockers.append(
            {"code": "resource_identity_set_changed", "counts": identity_counts}
        )

    recovery_fields = (
        "metadata_bytes",
        "source_payload_bytes",
        "binding_product_count",
        "unique_binding_identity_count",
        "image_shared_resident_bytes",
        "image_cache_resident_bytes",
        "image_cache_cpu_resident_bytes",
    )
    if all(warmup[field] is not None and quiescent[field] is not None for field in recovery_fields):
        growth = {
            field: quiescent[field] - warmup[field]
            for field in recovery_fields
            if quiescent[field] > warmup[field]
        }
        if growth:
            blockers.append(
                {"code": "quiescent_residency_not_recovered", "growth": growth}
            )

    return {
        "schema": "zircon.runtime.ui_render_dependency_memory_evidence.v1",
        "ready": not blockers,
        "blockers": blockers,
        "phases": phases,
        "actions": actions,
        "process": process,
        "budgets": {
            "max_retained_generation_count": MAX_RETAINED_GENERATION_COUNT,
            "max_metadata_bytes": MAX_METADATA_BUDGET_BYTES,
            "max_image_pool_bytes": MAX_IMAGE_POOL_BYTES,
            "max_end_process_memory_growth_bytes": MAX_END_MEMORY_GROWTH_BYTES,
            "max_peak_process_memory_growth_bytes": MAX_PEAK_MEMORY_GROWTH_BYTES,
        },
        "scope": {
            "driver_gpu_residency_measured": False,
            "rhi_accounted_image_payload_bytes_measured": True,
            "description": (
                "source-bound warmup, pressure and quiescent snapshots for renderer "
                "dependency metadata, segment payload versions, binding products, "
                "RHI-accounted image payload bytes, and same-process working/private "
                "bytes; driver allocator overhead and total OS GPU residency remain "
                "outside this gate"
            ),
        },
    }


def validate_output_path(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if resolved.drive.casefold() not in {"d:", "e:", "f:"}:
        raise ValueError("performance artifacts must be written under D:, E:, or F:")
    return resolved


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def _read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile-dir", type=Path, required=True)
    parser.add_argument("--timeline", type=Path)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--interaction-evidence", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    profile_dir = validate_output_path(args.profile_dir)
    timeline_path = (
        validate_output_path(args.timeline)
        if args.timeline is not None
        else profile_dir / "timeline.zrtrace.json"
    )
    interaction_path = (
        validate_output_path(args.interaction_evidence)
        if args.interaction_evidence is not None
        else profile_dir / "ui_interaction_evidence.json"
    )
    source_manifest_path = (
        validate_output_path(args.source_manifest)
        if args.source_manifest is not None
        else profile_dir / "source_manifest.json"
    )
    if not timeline_path.is_file():
        raise FileNotFoundError(f"timeline artifact is missing: {timeline_path}")
    interaction = _read_json(interaction_path) if interaction_path.is_file() else {}
    result = evaluate_memory_evidence(_read_json(timeline_path), interaction)
    if not interaction_path.is_file():
        for blocker in result["blockers"]:
            if blocker.get("code") == "missing_interaction_evidence":
                blocker["path"] = str(interaction_path)
                break
    if source_manifest_path.is_file():
        result["blockers"].extend(
            validate_source_manifest(_read_json(source_manifest_path))
        )
    else:
        result["blockers"].append(
            {"code": "missing_source_manifest", "path": str(source_manifest_path)}
        )
    result["ready"] = not result["blockers"]
    result["profile_binding"] = {
        "timeline_path": str(timeline_path),
        "timeline_sha256": _sha256(timeline_path),
        "interaction_evidence_path": str(interaction_path),
        "interaction_evidence_sha256": (
            _sha256(interaction_path) if interaction_path.is_file() else None
        ),
        "source_manifest_path": str(source_manifest_path),
        "source_manifest_sha256": (
            _sha256(source_manifest_path) if source_manifest_path.is_file() else None
        ),
        "tool_sha256": _sha256(Path(__file__)),
    }
    output_path = validate_output_path(args.output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps({"ready": result["ready"], "output": str(output_path)}))
    return 0 if result["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
