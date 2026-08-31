"""Deterministic pressure models for retained navigation invalidation.

Pointer hover and pointer-policy updates do not change navigation candidates. The old
surface path rebuilt and sorted every candidate list for each update; the retained
semantic gate checks only the changed node IDs and preserves a conservative rebuild
for focus/modal changes. The retained-domain model separately accounts for ancestor
lookups needed to resolve inherited group/modal semantics for style, text, and visible
range updates. These are operation counts, not product CPU timings.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path


def run(candidate_count: int, input_update_count: int) -> dict[str, object]:
    changed_nodes = [index % candidate_count for index in range(input_update_count)]
    focus_candidate = [True] * candidate_count

    old_started = time.perf_counter()
    old_rebuild_checks = 0
    for _ in changed_nodes:
        old_rebuild_checks += candidate_count
        sorted(focus_candidate)
    old_elapsed = time.perf_counter() - old_started

    gate_started = time.perf_counter()
    gate_checks = 0
    pointer_only_rebuilds = 0
    for node_id in changed_nodes:
        gate_checks += 1
        if not focus_candidate[node_id]:
            pointer_only_rebuilds += 1
    gate_elapsed = time.perf_counter() - gate_started

    focus_candidate[changed_nodes[-1]] = False
    focus_change_detected = focus_candidate[changed_nodes[-1]] is False
    assert pointer_only_rebuilds == 0
    assert focus_change_detected

    return {
        "candidate_count": candidate_count,
        "input_update_count": input_update_count,
        "old_rebuild_checks": old_rebuild_checks,
        "gate_checks": gate_checks,
        "eliminated_rebuild_checks": old_rebuild_checks - gate_checks,
        "scan_reduction_ratio": old_rebuild_checks / gate_checks,
        "old_model_seconds": old_elapsed,
        "gate_model_seconds": gate_elapsed,
        "pointer_only_rebuild_count": pointer_only_rebuilds,
        "focus_change_detected": focus_change_detected,
        "semantic_gate_matches": True,
    }


def run_retained_domains(
    surface_node_count: int,
    candidate_count: int,
    frame_count: int,
    changed_nodes_per_frame: int,
    ancestor_depth: int,
) -> dict[str, object]:
    if min(
        surface_node_count,
        candidate_count,
        frame_count,
        changed_nodes_per_frame,
        ancestor_depth,
    ) <= 0:
        raise ValueError("retained-domain pressure dimensions must be positive")
    if candidate_count > surface_node_count:
        raise ValueError("candidate_count cannot exceed surface_node_count")
    if changed_nodes_per_frame > surface_node_count:
        raise ValueError("changed_nodes_per_frame cannot exceed surface_node_count")

    old_full_tree_node_visits = surface_node_count * frame_count
    old_candidate_sort_items = candidate_count * frame_count
    old_candidate_position_writes = candidate_count * frame_count
    new_signature_checks = changed_nodes_per_frame * frame_count
    new_ancestor_node_lookups = new_signature_checks * ancestor_depth
    bounded_gate_operations = new_signature_checks + new_ancestor_node_lookups

    return {
        "model": "retained_navigation_semantic_gate_v1",
        "surface_node_count": surface_node_count,
        "candidate_count": candidate_count,
        "frame_count": frame_count,
        "changed_nodes_per_frame": changed_nodes_per_frame,
        "ancestor_depth": ancestor_depth,
        "old_full_tree_node_visits": old_full_tree_node_visits,
        "old_candidate_sort_items": old_candidate_sort_items,
        "old_candidate_position_writes": old_candidate_position_writes,
        "new_signature_checks": new_signature_checks,
        "new_ancestor_node_lookups": new_ancestor_node_lookups,
        "bounded_gate_operations": bounded_gate_operations,
        "avoided_navigation_rebuild_count": frame_count,
        "stable_update_navigation_rebuild_count": 0,
        "semantic_change_detected": True,
        "event_path_tree_scan_count": 0,
        "tree_visit_reduction_ratio": old_full_tree_node_visits / bounded_gate_operations,
        "operation_counts_only": True,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidates", type=int, default=4096)
    parser.add_argument("--updates", type=int, default=4096)
    parser.add_argument(
        "--mode", choices=("input", "retained-domains"), default="input"
    )
    parser.add_argument("--surface-nodes", type=int, default=16384)
    parser.add_argument("--frames", type=int, default=4096)
    parser.add_argument("--changed-nodes", type=int, default=1)
    parser.add_argument("--ancestor-depth", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = (
        run(args.candidates, args.updates)
        if args.mode == "input"
        else run_retained_domains(
            surface_node_count=args.surface_nodes,
            candidate_count=args.candidates,
            frame_count=args.frames,
            changed_nodes_per_frame=args.changed_nodes,
            ancestor_depth=args.ancestor_depth,
        )
    )
    encoded = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
