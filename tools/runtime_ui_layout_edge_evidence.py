"""Fail-closed evidence for parent-owned Runtime UI layout edges."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
from typing import Any


SCHEMA = "zircon.runtime.ui_layout_edge_evidence.v1"
SCENARIOS = frozenset(
    {
        "legacy_migration",
        "full_build",
        "child_dependency_patch",
        "parent_order_patch",
    }
)
REQUIRED_SOURCE_PATHS = (
    "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
    "zircon_runtime/src/ui/layout/pass/slot.rs",
    "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs",
)
_PREFIX = "ui.layout_edge."
_AGGREGATE_COUNTERS = (
    "operation_count",
    "edge_count",
    "legacy_slot_count",
    "retained_flat_slot_count",
    "affected_parent_count",
    "affected_parent_edge_count",
    "changed_child_count",
    "journal_mutation_count",
    "structure_mutation_count",
    "edge_projection_visit_count",
    "parent_child_visit_count",
    "changed_child_visit_count",
    "workspace_slot_visit_count",
    "unrelated_parent_slot_visit_count",
    "missing_edge_global_slot_visit_count",
    "fallback_repair_count",
    "parity_mismatch_count",
    "allocation_count",
    "allocation_bytes",
)
_EXPECTED_UNRELATED_SLOT_COUNTS = (64, 1_000, 10_000)
_HEX_40 = re.compile(r"[0-9a-fA-F]{40}\Z")
_HEX_64 = re.compile(r"[0-9a-fA-F]{64}\Z")


def evaluate_layout_edge_run(
    timeline: dict[str, object], scenario: str
) -> dict[str, object]:
    blockers: list[dict[str, object]] = []
    if scenario not in SCENARIOS:
        blockers.append({"code": "invalid_scenario", "scenario": scenario})

    raw_values = _counter_values(timeline)
    values: dict[str, int] = {}
    for suffix in _AGGREGATE_COUNTERS:
        name = f"{_PREFIX}{suffix}"
        samples = raw_values.get(name, [])
        if not samples:
            blockers.append({"code": "missing_counter", "counter": name})
            continue
        if len(samples) != 1:
            blockers.append(
                {
                    "code": "aggregate_counter_sample_count_mismatch",
                    "counter": name,
                    "actual": len(samples),
                }
            )
            continue
        if not _is_non_negative_integer(samples[0]):
            blockers.append(
                {
                    "code": "invalid_counter_value",
                    "counter": name,
                    "values": samples,
                }
            )
            continue
        values[suffix] = int(samples[0])

    duration_name = f"{_PREFIX}operation_duration_us"
    raw_durations = raw_values.get(duration_name, [])
    durations: list[int] = []
    if not raw_durations:
        blockers.append({"code": "missing_counter", "counter": duration_name})
    elif any(not _is_non_negative_integer(value) for value in raw_durations):
        blockers.append(
            {
                "code": "invalid_counter_value",
                "counter": duration_name,
                "values": raw_durations,
            }
        )
    else:
        durations = [int(value) for value in raw_durations]
        operation_count = values.get("operation_count")
        if operation_count is not None and len(durations) != operation_count:
            blockers.append(
                {
                    "code": "sample_count_mismatch",
                    "counter": duration_name,
                    "expected": operation_count,
                    "actual": len(durations),
                }
            )

    if values.get("operation_count") == 0:
        blockers.append({"code": "empty_measured_layout_edge_run"})
    if values.get("retained_flat_slot_count", 0) != 0:
        blockers.append(
            {
                "code": "retained_flat_slot_authority",
                "count": values.get("retained_flat_slot_count"),
            }
        )
    if values.get("journal_mutation_count") != values.get(
        "structure_mutation_count"
    ):
        blockers.append({"code": "mutation_journal_conservation_failed"})
    if values.get("unrelated_parent_slot_visit_count", 0) != 0:
        blockers.append({"code": "unrelated_parent_visit_detected"})
    if values.get("missing_edge_global_slot_visit_count", 0) != 0:
        blockers.append({"code": "runtime_flat_slot_scan_detected"})
    if values.get("fallback_repair_count", 0) != 0:
        blockers.append({"code": "defensive_repair_detected"})
    if values.get("parity_mismatch_count", 0) != 0:
        blockers.append({"code": "layout_parity_failed"})

    if all(name in values for name in _AGGREGATE_COUNTERS):
        blockers.extend(_validate_scenario(scenario, values))

    metrics = {
        "operation_visit_count": _operation_visit_count(scenario, values),
        "operation_duration_p95_us": _nearest_rank(durations, 0.95),
        "allocation_count": values.get("allocation_count", 0),
        "allocation_bytes": values.get("allocation_bytes", 0),
    }
    return {
        "schema": SCHEMA,
        "ready": not blockers,
        "scenario": scenario,
        "metrics": metrics,
        "blockers": blockers,
    }


def _validate_scenario(
    scenario: str, values: dict[str, int]
) -> list[dict[str, object]]:
    blockers: list[dict[str, object]] = []
    workspace_visits = values["workspace_slot_visit_count"]
    if scenario != "legacy_migration" and workspace_visits != 0:
        blockers.append({"code": "runtime_flat_slot_scan_detected"})
    if scenario != "legacy_migration" and values["legacy_slot_count"] != 0:
        blockers.append({"code": "runtime_legacy_slot_authority_detected"})

    if scenario == "legacy_migration":
        if (
            values["legacy_slot_count"] == 0
            or workspace_visits != values["legacy_slot_count"]
            or values["edge_projection_visit_count"]
            > values["edge_count"] + values["legacy_slot_count"]
            or values["changed_child_count"] != 0
            or values["journal_mutation_count"] != 0
        ):
            blockers.append({"code": "legacy_migration_conservation_failed"})
    elif scenario == "full_build":
        if (
            values["edge_projection_visit_count"] > values["edge_count"]
            or values["parent_child_visit_count"] > values["edge_count"]
            or values["changed_child_count"] != 0
            or values["journal_mutation_count"] != 0
        ):
            blockers.append({"code": "full_build_conservation_failed"})
    elif scenario == "child_dependency_patch":
        changed = values["changed_child_count"]
        if (
            changed == 0
            or values["changed_child_visit_count"] != changed
            or values["edge_projection_visit_count"] > changed
            or values["parent_child_visit_count"] != 0
            or values["affected_parent_edge_count"] != 0
            or values["journal_mutation_count"] != changed
        ):
            blockers.append({"code": "child_patch_locality_failed"})
    elif scenario == "parent_order_patch":
        parent_count = values["affected_parent_count"]
        affected_edges = values["affected_parent_edge_count"]
        if (
            parent_count == 0
            or affected_edges == 0
            or values["changed_child_count"] != 0
            or values["changed_child_visit_count"] != 0
            or values["edge_projection_visit_count"] > affected_edges
            or values["parent_child_visit_count"] > affected_edges
            or values["journal_mutation_count"] != parent_count
        ):
            blockers.append({"code": "parent_order_locality_failed"})
    return blockers


def _operation_visit_count(scenario: str, values: dict[str, int]) -> int:
    total = (
        values.get("edge_projection_visit_count", 0)
        + values.get("changed_child_visit_count", 0)
        + values.get("workspace_slot_visit_count", 0)
        + values.get("unrelated_parent_slot_visit_count", 0)
        + values.get("missing_edge_global_slot_visit_count", 0)
    )
    if scenario == "parent_order_patch":
        total += values.get("parent_child_visit_count", 0)
    return total


def _counter_values(timeline: dict[str, object]) -> dict[str, list[object]]:
    result: dict[str, list[object]] = {}
    raw_counters = timeline.get("counters", [])
    if not isinstance(raw_counters, list):
        return result
    for counter in raw_counters:
        if not isinstance(counter, dict):
            continue
        name = counter.get("name")
        if isinstance(name, str):
            result.setdefault(name, []).append(counter.get("value"))
    return result


def _nearest_rank(values: list[int], percentile: float) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered)))
    return ordered[rank - 1]


def evaluate_unrelated_slot_scaling(
    runs: list[dict[str, object]],
) -> dict[str, object]:
    blockers: list[dict[str, object]] = []
    indexed: dict[tuple[str, int], dict[str, object]] = {}
    expected_scenarios = ("child_dependency_patch", "parent_order_patch")
    for run in runs:
        scenario = run.get("scenario")
        slot_count = run.get("unrelated_slot_count")
        if (
            run.get("schema") != SCHEMA
            or scenario not in expected_scenarios
            or not _is_positive_integer(slot_count)
            or not run.get("ready")
        ):
            blockers.append({"code": "invalid_scaling_run", "run": run})
            continue
        key = (str(scenario), int(slot_count))
        if key in indexed:
            blockers.append(
                {
                    "code": "duplicate_unrelated_slot_scale_run",
                    "scenario": scenario,
                    "unrelated_slot_count": slot_count,
                }
            )
            continue
        indexed[key] = run

    for scenario in expected_scenarios:
        for slot_count in _EXPECTED_UNRELATED_SLOT_COUNTS:
            if (scenario, slot_count) not in indexed:
                blockers.append(
                    {
                        "code": "missing_unrelated_slot_scale_run",
                        "scenario": scenario,
                        "unrelated_slot_count": slot_count,
                    }
                )
        baseline_run = indexed.get((scenario, _EXPECTED_UNRELATED_SLOT_COUNTS[0]))
        baseline_work = _scaling_metric(baseline_run, "operation_visit_count")
        baseline_latency = _scaling_metric(
            baseline_run, "operation_duration_p95_us"
        )
        baseline_allocation = _scaling_metric(baseline_run, "allocation_bytes")
        for slot_count in _EXPECTED_UNRELATED_SLOT_COUNTS[1:]:
            run = indexed.get((scenario, slot_count))
            work = _scaling_metric(run, "operation_visit_count")
            latency = _scaling_metric(run, "operation_duration_p95_us")
            allocation = _scaling_metric(run, "allocation_bytes")
            if baseline_work is not None and work is not None and work != baseline_work:
                blockers.append(
                    {
                        "code": "local_work_scales_with_unrelated_slots",
                        "scenario": scenario,
                        "unrelated_slot_count": slot_count,
                    }
                )
            if (
                baseline_latency is not None
                and baseline_latency > 0
                and latency is not None
                and latency > baseline_latency * 1.10
            ):
                blockers.append(
                    {
                        "code": "local_latency_scales_with_unrelated_slots",
                        "scenario": scenario,
                        "unrelated_slot_count": slot_count,
                    }
                )
            if (
                baseline_allocation is not None
                and allocation is not None
                and allocation > baseline_allocation * 1.10
            ):
                blockers.append(
                    {
                        "code": "local_allocation_scales_with_unrelated_slots",
                        "scenario": scenario,
                        "unrelated_slot_count": slot_count,
                    }
                )
    return {"ready": not blockers, "blockers": blockers}


def _scaling_metric(
    run: dict[str, object] | None, metric: str
) -> int | float | None:
    if run is None:
        return None
    metrics = run.get("metrics")
    if not isinstance(metrics, dict):
        return None
    value = metrics.get(metric)
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(value)
        or value < 0
    ):
        return None
    return value


def validate_source_manifest(
    manifest: dict[str, object], scenario: str
) -> list[dict[str, object]]:
    blockers: list[dict[str, object]] = []
    if manifest.get("scenario") != f"runtime_ui_layout_edge_{scenario}":
        blockers.append({"code": "invalid_capture_contract"})
    repository = manifest.get("repository")
    repository = repository if isinstance(repository, dict) else {}
    git_binding = repository.get("git")
    git_binding = git_binding if isinstance(git_binding, dict) else {}
    if _HEX_40.fullmatch(str(git_binding.get("revision", ""))) is None or _HEX_64.fullmatch(
        str(git_binding.get("dirty_tree_sha256", ""))
    ) is None:
        blockers.append({"code": "invalid_source_binding"})

    raw_sources = repository.get("critical_source_files")
    sources = raw_sources if isinstance(raw_sources, list) else []
    sources_by_path = {
        str(source.get("relative_path", "")).replace("\\", "/"): source
        for source in sources
        if isinstance(source, dict)
    }
    for path in REQUIRED_SOURCE_PATHS:
        source = sources_by_path.get(path)
        if source is None:
            blockers.append(
                {"code": "missing_critical_source", "relative_path": path}
            )
            continue
        if (
            _HEX_64.fullmatch(str(source.get("sha256", ""))) is None
            or not _is_positive_integer(source.get("byte_length"))
        ):
            blockers.append(
                {"code": "invalid_source_fingerprint", "relative_path": path}
            )

    capture = manifest.get("capture")
    capture = capture if isinstance(capture, dict) else {}
    options = capture.get("options")
    options = options if isinstance(options, dict) else {}
    ordinal = options.get("run_ordinal")
    measured_count = options.get("measured_run_count")
    if (
        options.get("run_phase") != "measured"
        or not _is_positive_integer(ordinal)
        or not _is_positive_integer(measured_count)
        or int(ordinal) > int(measured_count)
    ):
        blockers.append({"code": "invalid_capture_contract"})
    return blockers


def build_layout_edge_report(
    timeline: dict[str, object],
    scenario: str,
    source_manifest: dict[str, object] | None,
) -> dict[str, object]:
    result = evaluate_layout_edge_run(timeline, scenario)
    if source_manifest is None:
        result["blockers"].append(
            {
                "code": "missing_source_manifest",
                "detail": "diagnostic output is not source-bound acceptance evidence",
            }
        )
    else:
        result["blockers"].extend(
            validate_source_manifest(source_manifest, scenario)
        )
    result["tool_binding"] = tool_binding()
    result["ready"] = not result["blockers"]
    return result


def tool_binding() -> dict[str, object]:
    path = Path(__file__).resolve()
    payload = path.read_bytes()
    return {
        "path": str(path),
        "sha256": hashlib.sha256(payload).hexdigest().upper(),
        "byte_length": len(payload),
    }


def _is_non_negative_integer(value: object) -> bool:
    return (
        not isinstance(value, bool)
        and isinstance(value, (int, float))
        and math.isfinite(value)
        and value >= 0
        and value == math.floor(value)
    )


def _is_positive_integer(value: object) -> bool:
    return _is_non_negative_integer(value) and int(value) > 0


def validate_output_path(path: Path) -> Path:
    resolved = path.resolve()
    if resolved.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("Layout edge evidence must be written below D:, E:, or F:.")
    return resolved


def _read_json(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeline", type=Path, required=True)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--scenario", choices=sorted(SCENARIOS), required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    output_path = validate_output_path(args.output)
    source_manifest = (
        _read_json(args.source_manifest) if args.source_manifest is not None else None
    )
    result = build_layout_edge_report(
        _read_json(args.timeline), args.scenario, source_manifest
    )
    result["timeline"] = str(args.timeline.resolve())
    result["source_manifest"] = (
        str(args.source_manifest.resolve())
        if args.source_manifest is not None
        else None
    )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(json.dumps({"ready": result["ready"], "output": str(output_path)}))
    return 0 if result["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
