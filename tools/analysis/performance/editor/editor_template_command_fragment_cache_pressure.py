#!/usr/bin/env python3
"""Model warm region-paint command materialization with retained fragments.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import json


def pressure_report(
    repaint_events: int,
    candidate_nodes_per_repaint: int,
    changed_nodes_per_repaint: int,
    commands_per_node: int,
    visible_nodes: int,
    commands_per_changed_fragment: int | None = None,
) -> dict[str, object]:
    if commands_per_changed_fragment is None:
        commands_per_changed_fragment = commands_per_node
    values = {
        "repaint_events": repaint_events,
        "candidate_nodes_per_repaint": candidate_nodes_per_repaint,
        "changed_nodes_per_repaint": changed_nodes_per_repaint,
        "commands_per_node": commands_per_node,
        "commands_per_changed_fragment": commands_per_changed_fragment,
        "visible_nodes": visible_nodes,
    }
    for name, value in values.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if changed_nodes_per_repaint > candidate_nodes_per_repaint:
        raise ValueError(
            "changed_nodes_per_repaint must not exceed candidate_nodes_per_repaint"
        )
    if candidate_nodes_per_repaint > visible_nodes:
        raise ValueError("candidate_nodes_per_repaint must not exceed visible_nodes")
    if commands_per_changed_fragment > commands_per_node:
        raise ValueError("commands_per_changed_fragment must not exceed commands_per_node")

    current_node_visits = repaint_events * candidate_nodes_per_repaint
    current_command_materializations = current_node_visits * commands_per_node
    retained_fragment_lookups = current_node_visits
    retained_changed_node_rebuilds = repaint_events * changed_nodes_per_repaint
    retained_command_materializations = (
        retained_changed_node_rebuilds * commands_per_changed_fragment
    )
    eliminated_command_materializations = (
        current_command_materializations - retained_command_materializations
    )

    return {
        "schema": "zircon.editor.template_command_fragment_cache_pressure.v2",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": values,
        "cold_fragment_fill": {
            "node_rebuilds": visible_nodes,
            "command_materializations": visible_nodes * commands_per_node,
        },
        "current_region_rebuild": {
            "candidate_node_visits": current_node_visits,
            "command_materializations": current_command_materializations,
        },
        "warm_retained_fragments": {
            "fragment_lookups": retained_fragment_lookups,
            "changed_node_rebuilds": retained_changed_node_rebuilds,
            "changed_fragment_rebuilds": retained_changed_node_rebuilds,
            "command_materializations": retained_command_materializations,
        },
        "delta": {
            "eliminated_command_materializations": (
                eliminated_command_materializations
            ),
            "command_materialization_reduction_ratio": (
                current_command_materializations
                / retained_command_materializations
            ),
            "eliminated_command_materialization_fraction": (
                eliminated_command_materializations
                / current_command_materializations
            ),
            "candidate_node_visit_reduction": 0,
        },
        "excluded_from_model": [
            "CPU, allocator, RSS, and latency timing",
            "fragment lookup and ordered merge cost",
            "fragment role discovery and publication cost",
            "command payload byte size",
            "text shaping and image decode cost",
            "GPU upload, batching, and present cost",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repaint-events", type=int, default=4096)
    parser.add_argument("--candidate-nodes-per-repaint", type=int, default=12)
    parser.add_argument("--changed-nodes-per-repaint", type=int, default=1)
    parser.add_argument("--commands-per-node", type=int, default=4)
    parser.add_argument("--commands-per-changed-fragment", type=int, default=1)
    parser.add_argument("--visible-nodes", type=int, default=10_000)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(
        args.repaint_events,
        args.candidate_nodes_per_repaint,
        args.changed_nodes_per_repaint,
        args.commands_per_node,
        args.visible_nodes,
        args.commands_per_changed_fragment,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
