#!/usr/bin/env python3
"""Model pointer feedback rebuilds moved from the callback to frame publication.

This is a deterministic work-count model, not product timing.
"""

import argparse
import json
import math
from pathlib import Path, PureWindowsPath


def pressure_report(pointer_events: int, events_per_frame: int) -> dict[str, object]:
    if pointer_events <= 0:
        raise ValueError("pointer_events must be positive")
    if events_per_frame <= 0:
        raise ValueError("events_per_frame must be positive")

    frame_count = math.ceil(pointer_events / events_per_frame)
    return {
        "schema": "zircon.editor.ui_pointer_feedback_deferred_pressure.v1",
        "evidence_kind": "deterministic_pipeline_work_count",
        "is_product_timing": False,
        "inputs": {
            "pointer_events": pointer_events,
            "events_per_frame": events_per_frame,
        },
        "current_event_owned_refresh": {
            "pointer_feedback_callbacks": pointer_events,
            "surface_rebuild_dirty_count": pointer_events,
            "workbench_projection_refresh_count": pointer_events,
        },
        "frame_owned_refresh": {
            "pointer_feedback_callbacks": pointer_events,
            "surface_rebuild_dirty_count": frame_count,
            "workbench_projection_refresh_count": frame_count,
            "deferred_feedback_count": pointer_events,
        },
        "comparison": {
            "frame_count": frame_count,
            "avoided_surface_rebuilds": pointer_events - frame_count,
            "refresh_reduction_ratio": pointer_events / frame_count,
        },
        "target_contract": [
            "pointer routing and semantic actions remain synchronous",
            "feedback property writes remain ordered within the input callback",
            "one frame recompute refreshes the dirty workbench surface before projection patch",
            "multiple feedback domains coalesce behind WORKBENCH_PROJECTION",
            "surface dirty state cannot be dropped on dispatch errors",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and input-to-present timing",
            "hit-test cell lookup and route dispatch",
            "host projection patch and native damage area",
            "GPU submission and present blocking",
        ],
    }


def pressure_suite() -> dict[str, object]:
    return {
        "schema": "zircon.editor.ui_pointer_feedback_deferred_pressure_suite.v1",
        "evidence_kind": "deterministic_pipeline_work_count",
        "is_product_timing": False,
        "scenarios": {
            str(events_per_frame): pressure_report(1_000, events_per_frame)
            for events_per_frame in (1, 4, 17)
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
