#!/usr/bin/env python3
"""Model work removed from the unrouted raw mouse-motion dispatch path.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import json


def pressure_report(
    motion_events: int,
    focus_path_nodes: int,
    root_targets: int,
    popup_stack_entries: int,
) -> dict[str, object]:
    for name, value in (
        ("motion_events", motion_events),
        ("focus_path_nodes", focus_path_nodes),
        ("root_targets", root_targets),
        ("popup_stack_entries", popup_stack_entries),
    ):
        if value < 0 or (name == "motion_events" and value == 0):
            raise ValueError(f"{name} must be positive" if name == "motion_events" else f"{name} must be non-negative")

    focus_vector_count = 3 if focus_path_nodes else 0
    root_vector_count = 1 if root_targets else 0
    popup_vector_count = 1 if popup_stack_entries else 0
    legacy_identity_copies_per_event = (
        3 * focus_path_nodes + root_targets + popup_stack_entries
    )
    legacy_trace_vector_allocations_per_event = (
        focus_vector_count + root_vector_count + popup_vector_count
    )

    return {
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "motion_events": motion_events,
            "focus_path_nodes": focus_path_nodes,
            "root_targets": root_targets,
            "popup_stack_entries": popup_stack_entries,
        },
        "legacy_generic_route_annotation": {
            "event_payload_clones": motion_events,
            "focused_route_queries": motion_events if focus_path_nodes else 0,
            "route_identity_copies": (
                legacy_identity_copies_per_event * motion_events
            ),
            "route_trace_vector_allocations": (
                legacy_trace_vector_allocations_per_event * motion_events
            ),
        },
        "unrouted_fast_path": {
            "event_payload_clones": 0,
            "focused_route_queries": 0,
            "route_identity_copies": 0,
            "route_trace_vector_allocations": 0,
            "retained_diagnostic_note_string_allocations": 2 * motion_events,
        },
        "eliminated": {
            "event_payload_clones": motion_events,
            "focused_route_queries": motion_events if focus_path_nodes else 0,
            "route_identity_copies": legacy_identity_copies_per_event * motion_events,
            "route_trace_vector_allocations": (
                legacy_trace_vector_allocations_per_event * motion_events
            ),
        },
        "excluded_from_model": [
            "allocator and CPU timing",
            "public diagnostic note allocation",
            "final route-authority annotation",
            "window event normalization and delivery",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--motion-events", type=int, default=100_000)
    parser.add_argument("--focus-path-nodes", type=int, default=12)
    parser.add_argument("--root-targets", type=int, default=4)
    parser.add_argument("--popup-stack-entries", type=int, default=3)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(
        args.motion_events,
        args.focus_path_nodes,
        args.root_targets,
        args.popup_stack_entries,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
