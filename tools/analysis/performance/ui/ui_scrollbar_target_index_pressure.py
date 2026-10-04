"""Deterministic work model for scrollbar target lookup during pointer drag."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    node_count: int = 10_000,
    pointer_move_count: int = 2_000,
    average_bucket_candidate_count: int = 1,
    dirty_node_patch_count: int = 0,
) -> dict[str, int | float | str]:
    for name, value in (
        ("node_count", node_count),
        ("pointer_move_count", pointer_move_count),
        ("average_bucket_candidate_count", average_bucket_candidate_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if dirty_node_patch_count < 0:
        raise ValueError("dirty_node_patch_count must be non-negative")
    if average_bucket_candidate_count > node_count:
        raise ValueError("average_bucket_candidate_count cannot exceed node_count")

    legacy_node_visits = node_count * pointer_move_count
    indexed_cold_build_node_visits = node_count
    indexed_dirty_patch_node_visits = dirty_node_patch_count
    indexed_exact_candidate_checks = (
        average_bucket_candidate_count * pointer_move_count
    )
    indexed_combined_work = (
        indexed_cold_build_node_visits
        + indexed_dirty_patch_node_visits
        + indexed_exact_candidate_checks
    )

    return {
        "schema_version": 1,
        "interpretation": (
            "deterministic lookup-work model; not CPU, allocation, or latency evidence"
        ),
        "node_count": node_count,
        "pointer_move_count": pointer_move_count,
        "average_bucket_candidate_count": average_bucket_candidate_count,
        "dirty_node_patch_count": dirty_node_patch_count,
        "legacy_full_tree_node_visits": legacy_node_visits,
        "indexed_cold_build_node_visits": indexed_cold_build_node_visits,
        "indexed_dirty_patch_node_visits": indexed_dirty_patch_node_visits,
        "indexed_exact_candidate_checks": indexed_exact_candidate_checks,
        "indexed_combined_work_units": indexed_combined_work,
        "eliminated_work_units": legacy_node_visits - indexed_combined_work,
        "work_reduction_ratio": legacy_node_visits / indexed_combined_work,
        "steady_state_lookup_complexity": "O(K), K = exact hash-bucket candidates",
        "cold_or_replacement_complexity": "O(N) once, then O(K) per lookup",
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-count", type=int, default=10_000)
    parser.add_argument("--pointer-move-count", type=int, default=2_000)
    parser.add_argument("--average-bucket-candidate-count", type=int, default=1)
    parser.add_argument("--dirty-node-patch-count", type=int, default=0)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        node_count=args.node_count,
        pointer_move_count=args.pointer_move_count,
        average_bucket_candidate_count=args.average_bucket_candidate_count,
        dirty_node_patch_count=args.dirty_node_patch_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
