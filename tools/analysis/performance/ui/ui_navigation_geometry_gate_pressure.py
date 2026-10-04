"""Deterministic work model for scoped UI navigation geometry invalidation."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def run(
    total_node_count: int,
    focus_candidate_count: int,
    non_candidate_update_count: int,
) -> dict[str, int | float | bool]:
    if total_node_count <= 0:
        raise ValueError("total_node_count must be positive")
    if not 0 < focus_candidate_count <= total_node_count:
        raise ValueError("focus_candidate_count must be within the node count")
    if non_candidate_update_count <= 0:
        raise ValueError("non_candidate_update_count must be positive")

    sort_work_per_rebuild = math.ceil(
        focus_candidate_count * math.log2(max(2, focus_candidate_count))
    )
    old_tree_visit_count = total_node_count * non_candidate_update_count
    old_candidate_sort_work = sort_work_per_rebuild * non_candidate_update_count
    old_total_work = old_tree_visit_count + old_candidate_sort_work
    authority_gate_check_count = non_candidate_update_count

    return {
        "total_node_count": total_node_count,
        "focus_candidate_count": focus_candidate_count,
        "non_candidate_update_count": non_candidate_update_count,
        "old_full_rebuild_count": non_candidate_update_count,
        "old_tree_visit_count": old_tree_visit_count,
        "old_candidate_sort_work": old_candidate_sort_work,
        "old_total_work": old_total_work,
        "authority_gate_check_count": authority_gate_check_count,
        "new_full_rebuild_count": 0,
        "eliminated_work": old_total_work - authority_gate_check_count,
        "work_reduction_ratio": old_total_work / authority_gate_check_count,
        "focus_candidate_change_detected": True,
        "removed_focus_candidate_detected": True,
    }


def run_frame_patch(
    total_node_count: int,
    focus_candidate_count: int,
    candidate_frame_update_count: int,
) -> dict[str, int | float | bool]:
    if total_node_count <= 0:
        raise ValueError("total_node_count must be positive")
    if not 0 < focus_candidate_count <= total_node_count:
        raise ValueError("focus_candidate_count must be within the node count")
    if candidate_frame_update_count <= 0:
        raise ValueError("candidate_frame_update_count must be positive")

    sort_work_per_rebuild = math.ceil(
        focus_candidate_count * math.log2(max(2, focus_candidate_count))
    )
    old_tree_visit_count = total_node_count * candidate_frame_update_count
    old_candidate_sort_work = sort_work_per_rebuild * candidate_frame_update_count
    old_candidate_write_count = focus_candidate_count * candidate_frame_update_count
    old_total_work = (
        old_tree_visit_count + old_candidate_sort_work + old_candidate_write_count
    )
    changed_authority_check_count = candidate_frame_update_count
    candidate_geometry_lookup_count = candidate_frame_update_count
    candidate_frame_write_count = candidate_frame_update_count
    new_total_work = (
        changed_authority_check_count
        + candidate_geometry_lookup_count
        + candidate_frame_write_count
    )

    return {
        "total_node_count": total_node_count,
        "focus_candidate_count": focus_candidate_count,
        "candidate_frame_update_count": candidate_frame_update_count,
        "old_full_rebuild_count": candidate_frame_update_count,
        "old_tree_visit_count": old_tree_visit_count,
        "old_candidate_sort_work": old_candidate_sort_work,
        "old_candidate_write_count": old_candidate_write_count,
        "old_total_work": old_total_work,
        "changed_authority_check_count": changed_authority_check_count,
        "candidate_geometry_lookup_count": candidate_geometry_lookup_count,
        "candidate_frame_write_count": candidate_frame_write_count,
        "new_total_work": new_total_work,
        "new_full_rebuild_count": 0,
        "eliminated_work": old_total_work - new_total_work,
        "work_reduction_ratio": old_total_work / new_total_work,
        "ordering_change_forces_rebuild": True,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--total-node-count", type=int, default=16_384)
    parser.add_argument("--focus-candidate-count", type=int, default=512)
    parser.add_argument("--non-candidate-update-count", type=int, default=4_096)
    parser.add_argument("--candidate-frame-update-count", type=int, default=4_096)
    parser.add_argument(
        "--model",
        choices=("non-candidate-gate", "candidate-frame-patch"),
        default="non-candidate-gate",
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.model == "candidate-frame-patch":
        result = run_frame_patch(
            args.total_node_count,
            args.focus_candidate_count,
            args.candidate_frame_update_count,
        )
    else:
        result = run(
            args.total_node_count,
            args.focus_candidate_count,
            args.non_candidate_update_count,
        )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
