#!/usr/bin/env python3
"""Model a rejected trailing resize reflow against frame-cadence publication.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import hashlib
import json
import math
from pathlib import Path, PureWindowsPath
import subprocess


REPO_ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/events/resize.rs",
    "zircon_editor/src/ui/retained_host/host_contract/window/event_loop/redraw.rs",
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/shell_metrics.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp",
)


def _positive(name: str, value: int | float) -> None:
    if value <= 0:
        raise ValueError(f"{name} must be positive")


def _non_negative(name: str, value: int | float) -> None:
    if value < 0:
        raise ValueError(f"{name} must be non-negative")


def _frame_cadence_slots(
    resize_events: int,
    event_interval_ms: float,
    frame_interval_ms: float,
) -> tuple[int, float]:
    slots: set[int] = set()
    maximum_delay_ms = 0.0
    for event_index in range(resize_events):
        event_time_ms = event_index * event_interval_ms
        slot = math.ceil((event_time_ms / frame_interval_ms) - 1.0e-12)
        commit_time_ms = slot * frame_interval_ms
        slots.add(slot)
        maximum_delay_ms = max(maximum_delay_ms, commit_time_ms - event_time_ms)
    return len(slots), maximum_delay_ms


def pressure_report(
    resize_events: int,
    event_interval_ms: float,
    rejected_trailing_debounce_ms: float,
    frame_interval_ms: float,
    semantic_nodes: int,
    total_layout_nodes: int,
    total_hit_entries: int,
    affected_layout_nodes: int,
    affected_hit_entries: int,
    damage_regions: int,
) -> dict[str, object]:
    for name, value in {
        "resize_events": resize_events,
        "event_interval_ms": event_interval_ms,
        "frame_interval_ms": frame_interval_ms,
        "semantic_nodes": semantic_nodes,
        "total_layout_nodes": total_layout_nodes,
        "total_hit_entries": total_hit_entries,
        "damage_regions": damage_regions,
    }.items():
        _positive(name, value)
    for name, value in {
        "rejected_trailing_debounce_ms": rejected_trailing_debounce_ms,
        "affected_layout_nodes": affected_layout_nodes,
        "affected_hit_entries": affected_hit_entries,
    }.items():
        _non_negative(name, value)
    if affected_layout_nodes > total_layout_nodes:
        raise ValueError("affected_layout_nodes must not exceed total_layout_nodes")
    if affected_hit_entries > total_hit_entries:
        raise ValueError("affected_hit_entries must not exceed total_hit_entries")

    interaction_duration_ms = (resize_events - 1) * event_interval_ms
    mismatched_geometry_window_ms = (
        interaction_duration_ms + rejected_trailing_debounce_ms
    )
    cadence_commits, maximum_cadence_delay_ms = _frame_cadence_slots(
        resize_events,
        event_interval_ms,
        frame_interval_ms,
    )

    rejected_operation_units = (
        semantic_nodes + total_layout_nodes + total_hit_entries + damage_regions
    )
    target_operation_units_per_commit = (
        affected_layout_nodes + affected_hit_entries + damage_regions
    )
    target_operation_units = cadence_commits * target_operation_units_per_commit

    inputs = {
        "resize_events": resize_events,
        "event_interval_ms": event_interval_ms,
        "rejected_trailing_debounce_ms": rejected_trailing_debounce_ms,
        "frame_interval_ms": frame_interval_ms,
        "semantic_nodes": semantic_nodes,
        "total_layout_nodes": total_layout_nodes,
        "total_hit_entries": total_hit_entries,
        "affected_layout_nodes": affected_layout_nodes,
        "affected_hit_entries": affected_hit_entries,
        "damage_regions": damage_regions,
    }
    return {
        "schema": "zircon.editor.window_resize_reflow_pressure.v2",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": inputs,
        "rejected_trailing_debounce": {
            "interaction_duration_ms": round(interaction_duration_ms, 3),
            "mismatched_geometry_window_ms": round(
                mismatched_geometry_window_ms, 3
            ),
            "mismatched_geometry_frame_budgets": math.ceil(
                mismatched_geometry_window_ms / frame_interval_ms
            ),
            "native_resize_events": resize_events,
            "full_frame_redraw_requests": resize_events + 1,
            "retained_geometry_commits": 1,
            "coalesced_resize_events": resize_events - 1,
            "semantic_projection_visits": semantic_nodes,
        },
        "rejected_final_reflow": {
            "semantic_projection_visits": semantic_nodes,
            "layout_node_visits": total_layout_nodes,
            "hit_entry_visits": total_hit_entries,
            "damage_region_merges": damage_regions,
            "geometry_operation_units": rejected_operation_units,
        },
        "frame_cadence_geometry_publication": {
            "retained_geometry_commits": cadence_commits,
            "coalesced_resize_events": resize_events - cadence_commits,
            "max_event_to_geometry_commit_ms": round(
                maximum_cadence_delay_ms, 3
            ),
            "semantic_projection_visits": 0,
            "layout_node_visits": cadence_commits * affected_layout_nodes,
            "hit_entry_visits": cadence_commits * affected_hit_entries,
            "damage_region_merges": cadence_commits * damage_regions,
            "geometry_operation_units_per_commit": (
                target_operation_units_per_commit
            ),
            "geometry_operation_units": target_operation_units,
        },
        "comparison": {
            "rejected_final_reflow_to_frame_cadence_operation_ratio": (
                rejected_operation_units / target_operation_units
            ),
            "semantic_projection_visits_eliminated": semantic_nodes,
            "target_commit_count_is_frame_bounded": cadence_commits <= resize_events,
        },
        "model_contract": [
            "rejected trailing-debounce baseline presents stale retained geometry",
            "target publishes at most one latest geometry generation per frame cadence slot",
            "target never rebuilds semantic presentation for an ordinary size change",
            "geometry, hit-test rows, and damage publish as one generation",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, GPU, and input-to-visible timing",
            "platform event-loop redraw coalescing and compositor scheduling",
            "layout dependency propagation and cache lookup constants",
            "surface pixel fill, swapchain configuration, and command submission cost",
            "responsive breakpoints that legitimately require structural fallback",
        ],
    }


def pressure_suite(
    semantic_nodes: int,
    total_layout_nodes: int,
    total_hit_entries: int,
    affected_layout_nodes: int,
    affected_hit_entries: int,
    damage_regions: int,
) -> dict[str, object]:
    common = {
        "rejected_trailing_debounce_ms": 80.0,
        "frame_interval_ms": 1000.0 / 60.0,
        "semantic_nodes": semantic_nodes,
        "total_layout_nodes": total_layout_nodes,
        "total_hit_entries": total_hit_entries,
        "affected_layout_nodes": affected_layout_nodes,
        "affected_hit_entries": affected_hit_entries,
        "damage_regions": damage_regions,
    }
    return {
        "schema": "zircon.editor.window_resize_reflow_pressure_suite.v2",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "source_binding": source_binding(),
        "scenarios": {
            "default_profile_fixture": pressure_report(
                resize_events=25,
                event_interval_ms=40.0,
                **common,
            ),
            "high_frequency_stress": pressure_report(
                resize_events=2_000,
                event_interval_ms=4.0,
                **common,
            ),
        },
    }


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def source_binding() -> dict[str, object]:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    return {
        "git_revision": revision,
        "files": [
            {"path": relative_path, "sha256": _sha256(REPO_ROOT / relative_path)}
            for relative_path in SOURCE_PATHS
        ],
    }


def validate_output_path(output: str) -> Path:
    path = Path(output).resolve()
    drive = PureWindowsPath(str(path)).drive.upper()
    if drive not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written to D:, E:, or F:")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--semantic-nodes", type=int, default=10_000)
    parser.add_argument("--total-layout-nodes", type=int, default=10_000)
    parser.add_argument("--total-hit-entries", type=int, default=10_000)
    parser.add_argument("--affected-layout-nodes", type=int, default=64)
    parser.add_argument("--affected-hit-entries", type=int, default=64)
    parser.add_argument("--damage-regions", type=int, default=8)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_suite(
        args.semantic_nodes,
        args.total_layout_nodes,
        args.total_hit_entries,
        args.affected_layout_nodes,
        args.affected_hit_entries,
        args.damage_regions,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        output_path = validate_output_path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8", newline="\n")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
