"""Deterministic candidate-work model for retained UI hit-grid pointer queries."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    total_entry_count: int,
    average_cell_candidate_count: int,
    pointer_query_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("total_entry_count", total_entry_count),
        ("average_cell_candidate_count", average_cell_candidate_count),
        ("pointer_query_count", pointer_query_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if average_cell_candidate_count > total_entry_count:
        raise ValueError("average_cell_candidate_count cannot exceed total_entry_count")

    old_full_surface_candidate_checks = total_entry_count * pointer_query_count
    new_cell_candidate_checks = average_cell_candidate_count * pointer_query_count

    return {
        "total_entry_count": total_entry_count,
        "average_cell_candidate_count": average_cell_candidate_count,
        "pointer_query_count": pointer_query_count,
        "old_full_surface_candidate_checks": old_full_surface_candidate_checks,
        "new_cell_candidate_checks": new_cell_candidate_checks,
        "new_event_time_sort_count": 0,
        "eliminated_candidate_checks": (
            old_full_surface_candidate_checks - new_cell_candidate_checks
        ),
        "candidate_check_reduction_ratio": (
            old_full_surface_candidate_checks / new_cell_candidate_checks
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--total-entry-count", type=int, default=65_536)
    parser.add_argument("--average-cell-candidate-count", type=int, default=32)
    parser.add_argument("--pointer-query-count", type=int, default=1_000_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.total_entry_count,
        args.average_cell_candidate_count,
        args.pointer_query_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
