#!/usr/bin/env python3
"""Model event-owned UI refresh against frame-cadence publication.

This is a deterministic pipeline execution count, not product timing.
"""

import argparse
import json
import math
from pathlib import Path, PureWindowsPath


def pressure_report(changed_events: int, changed_events_per_frame: int) -> dict[str, object]:
    if changed_events <= 0:
        raise ValueError("changed_events must be positive")
    if changed_events_per_frame <= 0:
        raise ValueError("changed_events_per_frame must be positive")

    frame_count = math.ceil(changed_events / changed_events_per_frame)
    current_refresh_stages = changed_events * 2
    target_refresh_stages = frame_count * 2

    return {
        "schema": "zircon.editor.pointer_frame_cadence_refresh_pressure.v1",
        "evidence_kind": "deterministic_pipeline_execution_count",
        "is_product_timing": False,
        "inputs": {
            "changed_events": changed_events,
            "changed_events_per_frame": changed_events_per_frame,
        },
        "modeled_case": (
            "one continuously changing visual property key; input route and semantic "
            "state remain synchronous"
        ),
        "frame_count": frame_count,
        "current_event_owned_refresh": {
            "input_route_count": changed_events,
            "semantic_state_write_count": changed_events,
            "surface_rebuild_dirty_count": changed_events,
            "host_projection_refresh_count": changed_events,
            "frame_commit_count": frame_count,
            "expensive_refresh_stage_executions": current_refresh_stages,
        },
        "frame_cadence_publication": {
            "input_route_count": changed_events,
            "visual_journal_write_count": changed_events,
            "coalesced_visual_write_count": changed_events - frame_count,
            "published_visual_state_count": frame_count,
            "surface_rebuild_dirty_count": frame_count,
            "host_projection_refresh_count": frame_count,
            "frame_commit_count": frame_count,
            "expensive_refresh_stage_executions": target_refresh_stages,
        },
        "comparison": {
            "refresh_stage_reduction_ratio": (
                current_refresh_stages / target_refresh_stages
            ),
            "avoided_surface_refreshes": changed_events - frame_count,
            "avoided_projection_refreshes": changed_events - frame_count,
        },
        "target_contract": [
            "hit routing, capture, focus, press/release, and actions remain synchronous",
            "coalescible visual/value properties use a latest-value keyed journal",
            "one frame transaction drains the journal before render publication",
            "surface rebuild and host projection refresh execute at most once per frame",
            "generation and input sequence identify the semantic state represented by a frame",
            "no pointer callback invokes rebuild_dirty or host projection extraction directly",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and latency timing",
            "style node/property work inside each refresh",
            "hit-test, business action, and callback cost",
            "multiple independent visual keys changed in the same frame",
            "native event arrival jitter, GPU submission, and present blocking",
        ],
    }


def pressure_suite() -> dict[str, object]:
    return {
        "schema": "zircon.editor.pointer_frame_cadence_refresh_pressure_suite.v1",
        "evidence_kind": "deterministic_pipeline_execution_count",
        "is_product_timing": False,
        "scenarios": {
            str(events_per_frame): pressure_report(1_000, events_per_frame)
            for events_per_frame in (4, 8, 17)
        },
    }


def validate_output_path(output: str) -> Path:
    path = Path(output).resolve()
    if PureWindowsPath(str(path)).drive.upper() == "C:":
        raise ValueError("performance artifacts must not be written to the C drive")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    args = parser.parse_args()

    payload = json.dumps(pressure_suite(), indent=2, sort_keys=True) + "\n"
    if args.output:
        output_path = validate_output_path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8", newline="\n")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
