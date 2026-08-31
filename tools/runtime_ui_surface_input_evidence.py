"""Validate retained Runtime UI Surface input publication evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
from typing import Any


SCHEMA = "zircon.runtime.ui_surface_input_evidence.v1"
ROUTE_CLASSES = frozenset(
    {"pointer_uncaptured", "pointer_captured", "focused", "unrouted"}
)
REQUIRED_SOURCE_PATHS = (
    "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
    "zircon_runtime/src/ui/surface/frame_hit_test.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
)
_AGGREGATE_COUNTERS = (
    "event_count",
    "directory_query_count",
    "capture_direct_route_count",
    "focus_direct_route_count",
    "unrouted_reject_count",
    "tree_scan_count",
    "render_command_scan_count",
    "publication_patch_count",
    "publication_full_rebuild_count",
)
_EVENT_COUNTERS = (
    "candidate_surface_count",
    "dispatched_surface_count",
    "event_clone_count",
    "event_rebuild_count",
    "text_owner_sync_count",
    "warm_path_allocation_count",
    "input_to_damage_us",
    "input_to_present_us",
)
_PREFIX = "ui.surface_set.input."
_HEX_40 = re.compile(r"[0-9a-fA-F]{40}\Z")
_HEX_64 = re.compile(r"[0-9a-fA-F]{64}\Z")
_EXPECTED_SURFACE_COUNTS = (1, 4, 16, 64)


def evaluate_input_run(
    timeline: dict[str, object], route_class: str, surface_count: int
) -> dict[str, object]:
    blockers: list[dict[str, object]] = []
    if route_class not in ROUTE_CLASSES:
        blockers.append(
            {"code": "invalid_route_class", "route_class": route_class}
        )
    if not _is_positive_integer(surface_count):
        blockers.append(
            {"code": "invalid_surface_count", "value": surface_count}
        )

    raw_values = _counter_values(timeline)
    aggregates: dict[str, int] = {}
    for suffix in _AGGREGATE_COUNTERS:
        name = f"{_PREFIX}{suffix}"
        values = raw_values.get(name, [])
        if not values:
            blockers.append({"code": "missing_counter", "counter": name})
            continue
        if len(values) != 1:
            blockers.append(
                {
                    "code": "aggregate_counter_sample_count_mismatch",
                    "counter": name,
                    "actual": len(values),
                }
            )
            continue
        if not _is_non_negative_integer(values[0]):
            blockers.append(
                {
                    "code": "invalid_counter_value",
                    "counter": name,
                    "values": values,
                }
            )
            continue
        aggregates[suffix] = int(values[0])

    event_count = aggregates.get("event_count")
    if event_count is not None and event_count == 0:
        blockers.append({"code": "empty_measured_input_run"})

    samples: dict[str, list[int]] = {}
    for suffix in _EVENT_COUNTERS:
        name = f"{_PREFIX}{suffix}"
        values = raw_values.get(name, [])
        if not values:
            blockers.append({"code": "missing_counter", "counter": name})
            continue
        if any(not _is_non_negative_integer(value) for value in values):
            blockers.append(
                {
                    "code": "invalid_counter_value",
                    "counter": name,
                    "values": values,
                }
            )
            continue
        samples[suffix] = [int(value) for value in values]
        if event_count is not None and len(values) != event_count:
            blockers.append(
                {
                    "code": "sample_count_mismatch",
                    "counter": name,
                    "expected": event_count,
                    "actual": len(values),
                }
            )

    metrics = _run_metrics(samples)
    if aggregates.get("tree_scan_count", 0) != 0 or aggregates.get(
        "render_command_scan_count", 0
    ) != 0:
        blockers.append(
            {
                "code": "event_time_global_scan_detected",
                "tree_scan_count": aggregates.get("tree_scan_count"),
                "render_command_scan_count": aggregates.get(
                    "render_command_scan_count"
                ),
            }
        )
    if any(samples.get("event_rebuild_count", [])):
        blockers.append(
            {
                "code": "event_time_rebuild_detected",
                "count": sum(samples["event_rebuild_count"]),
            }
        )
    if aggregates.get("publication_patch_count", 0) != 0 or aggregates.get(
        "publication_full_rebuild_count", 0
    ) != 0:
        blockers.append(
            {
                "code": "event_time_publication_detected",
                "patch_count": aggregates.get("publication_patch_count"),
                "full_rebuild_count": aggregates.get(
                    "publication_full_rebuild_count"
                ),
            }
        )
    if metrics.get("input_to_damage_p95_us", 0) > 1_000:
        blockers.append(
            {
                "code": "input_to_damage_p95_exceeded",
                "actual_us": metrics["input_to_damage_p95_us"],
                "budget_us": 1_000,
            }
        )
    if metrics.get("input_to_present_p95_us", 0) > 9_000:
        blockers.append(
            {
                "code": "input_to_present_p95_exceeded",
                "actual_us": metrics["input_to_present_p95_us"],
                "budget_us": 9_000,
            }
        )

    if event_count is not None and all(
        len(samples.get(suffix, [])) == event_count for suffix in _EVENT_COUNTERS
    ):
        if _is_positive_integer(surface_count) and any(
            value > surface_count
            for value in samples["candidate_surface_count"]
        ):
            blockers.append(
                {
                    "code": "candidate_count_exceeds_surface_count",
                    "surface_count": surface_count,
                    "max_candidate_count": max(
                        samples["candidate_surface_count"], default=0
                    ),
                }
            )
        blockers.extend(
            _validate_route_contract(route_class, event_count, aggregates, samples)
        )

    return {
        "schema": SCHEMA,
        "ready": not blockers,
        "route_class": route_class,
        "surface_count": surface_count,
        "metrics": metrics,
        "blockers": blockers,
    }


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


def _run_metrics(samples: dict[str, list[int]]) -> dict[str, int]:
    return {
        "candidate_surface_count": sum(samples.get("candidate_surface_count", [])),
        "candidate_surface_p95": _nearest_rank(
            samples.get("candidate_surface_count", []), 0.95
        ),
        "dispatched_surface_count": sum(
            samples.get("dispatched_surface_count", [])
        ),
        "event_clone_count": sum(samples.get("event_clone_count", [])),
        "event_rebuild_count": sum(samples.get("event_rebuild_count", [])),
        "text_owner_sync_count": sum(samples.get("text_owner_sync_count", [])),
        "warm_path_allocation_count": sum(
            samples.get("warm_path_allocation_count", [])
        ),
        "input_to_damage_p95_us": _nearest_rank(
            samples.get("input_to_damage_us", []), 0.95
        ),
        "input_to_present_p95_us": _nearest_rank(
            samples.get("input_to_present_us", []), 0.95
        ),
    }


def _nearest_rank(values: list[int], percentile: float) -> int:
    if not values:
        return 0
    ordered = sorted(values)
    rank = max(1, math.ceil(percentile * len(ordered)))
    return ordered[rank - 1]


def _validate_route_contract(
    route_class: str,
    event_count: int,
    aggregates: dict[str, int],
    samples: dict[str, list[int]],
) -> list[dict[str, object]]:
    blockers: list[dict[str, object]] = []
    candidates = samples["candidate_surface_count"]
    dispatched = samples["dispatched_surface_count"]
    clones = samples["event_clone_count"]
    text_syncs = samples["text_owner_sync_count"]
    allocations = samples["warm_path_allocation_count"]

    if route_class == "pointer_uncaptured":
        if aggregates["directory_query_count"] != event_count:
            blockers.append({"code": "directory_query_conservation_failed"})
        if any(dispatch > candidate for dispatch, candidate in zip(dispatched, candidates)):
            blockers.append({"code": "dispatch_conservation_failed"})
        if any(
            clone > max(dispatch - 1, 0)
            for clone, dispatch in zip(clones, dispatched)
        ):
            blockers.append({"code": "clone_conservation_failed"})
        if any(allocation > clone for allocation, clone in zip(allocations, clones)):
            blockers.append({"code": "unexpected_warm_path_allocation"})
        if any(sync > dispatch for sync, dispatch in zip(text_syncs, dispatched)):
            blockers.append({"code": "text_owner_sync_conservation_failed"})
        if _nearest_rank(candidates, 0.95) > 2:
            blockers.append(
                {
                    "code": "candidate_surface_p95_exceeded",
                    "actual": _nearest_rank(candidates, 0.95),
                    "budget": 2,
                }
            )
        if any(
            aggregates[name] != 0
            for name in (
                "capture_direct_route_count",
                "focus_direct_route_count",
                "unrouted_reject_count",
            )
        ):
            blockers.append({"code": "route_class_conservation_failed"})
        return blockers

    direct_counter = {
        "pointer_captured": "capture_direct_route_count",
        "focused": "focus_direct_route_count",
        "unrouted": "unrouted_reject_count",
    }[route_class]
    if aggregates[direct_counter] != event_count:
        blockers.append({"code": "direct_route_conservation_failed"})
    if aggregates["directory_query_count"] != 0 or any(candidates):
        blockers.append({"code": "unexpected_directory_query"})
    for name in (
        "capture_direct_route_count",
        "focus_direct_route_count",
        "unrouted_reject_count",
    ):
        if name != direct_counter and aggregates[name] != 0:
            blockers.append({"code": "route_class_conservation_failed"})

    expected_dispatch = 0 if route_class == "unrouted" else 1
    if any(value != expected_dispatch for value in dispatched):
        blockers.append({"code": "direct_route_fanout_detected"})
    if any(clones):
        blockers.append({"code": "direct_route_clone_detected"})
    if any(allocations):
        blockers.append({"code": "unexpected_warm_path_allocation"})
    expected_sync = 0 if route_class == "unrouted" else 1
    if any(value > expected_sync for value in text_syncs):
        blockers.append({"code": "text_owner_sync_conservation_failed"})
    return blockers


def evaluate_surface_scaling(runs: list[dict[str, object]]) -> dict[str, object]:
    blockers: list[dict[str, object]] = []
    indexed: dict[tuple[str, int], dict[str, object]] = {}
    for run in runs:
        route_class = run.get("route_class")
        surface_count = run.get("surface_count")
        if (
            run.get("schema") != SCHEMA
            or route_class not in {"focused", "pointer_uncaptured"}
            or not _is_positive_integer(surface_count)
            or not run.get("ready")
        ):
            blockers.append({"code": "invalid_scaling_run", "run": run})
            continue
        key = (str(route_class), int(surface_count))
        if key in indexed:
            blockers.append(
                {
                    "code": "duplicate_surface_scale_run",
                    "route_class": route_class,
                    "surface_count": surface_count,
                }
            )
            continue
        indexed[key] = run
        if route_class == "pointer_uncaptured":
            candidate_p95 = _scaling_metric(run, "candidate_surface_p95")
            if candidate_p95 != 1:
                blockers.append(
                    {
                        "code": "noncanonical_pointer_candidate_set",
                        "surface_count": surface_count,
                        "candidate_surface_p95": candidate_p95,
                    }
                )

    for route_class in ("focused", "pointer_uncaptured"):
        for surface_count in _EXPECTED_SURFACE_COUNTS:
            if (route_class, surface_count) not in indexed:
                blockers.append(
                    {
                        "code": "missing_surface_scale_run",
                        "route_class": route_class,
                        "surface_count": surface_count,
                    }
                )

    for route_class, ratio, code in (
        ("focused", 1.05, "focused_scaling_regression"),
        ("pointer_uncaptured", 1.10, "pointer_scaling_regression"),
    ):
        for metric in ("input_to_damage_p95_us", "input_to_present_p95_us"):
            baseline = _scaling_metric(indexed.get((route_class, 1)), metric)
            if baseline is None or baseline <= 0:
                blockers.append(
                    {
                        "code": "invalid_scaling_baseline",
                        "route_class": route_class,
                        "metric": metric,
                    }
                )
                continue
            for surface_count in _EXPECTED_SURFACE_COUNTS[1:]:
                value = _scaling_metric(
                    indexed.get((route_class, surface_count)), metric
                )
                if value is not None and value > baseline * ratio:
                    blockers.append(
                        {
                            "code": code,
                            "metric": metric,
                            "surface_count": surface_count,
                            "baseline_us": baseline,
                            "actual_us": value,
                            "ratio_budget": ratio,
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
    manifest: dict[str, object], route_class: str
) -> list[dict[str, object]]:
    blockers: list[dict[str, object]] = []
    if manifest.get("scenario") != f"runtime_ui_{route_class}":
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


def build_input_report(
    timeline: dict[str, object],
    route_class: str,
    surface_count: int,
    source_manifest: dict[str, object] | None,
) -> dict[str, object]:
    result = evaluate_input_run(timeline, route_class, surface_count)
    if source_manifest is None:
        result["blockers"].append(
            {
                "code": "missing_source_manifest",
                "detail": "diagnostic output is not source-bound acceptance evidence",
            }
        )
    else:
        result["blockers"].extend(
            validate_source_manifest(source_manifest, route_class)
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
        raise ValueError("Runtime UI input evidence must be written below D:, E:, or F:.")
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
    parser.add_argument("--route-class", choices=sorted(ROUTE_CLASSES), required=True)
    parser.add_argument("--surface-count", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    output_path = validate_output_path(args.output)
    source_manifest = (
        _read_json(args.source_manifest) if args.source_manifest is not None else None
    )
    result = build_input_report(
        _read_json(args.timeline),
        args.route_class,
        args.surface_count,
        source_manifest,
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
