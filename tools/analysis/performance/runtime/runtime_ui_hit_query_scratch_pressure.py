"""Deterministic storage-allocation model for retained UI hit queries."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    entry_count: int,
    average_candidate_count: int,
    pointer_query_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("entry_count", entry_count),
        ("average_candidate_count", average_candidate_count),
        ("pointer_query_count", pointer_query_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if average_candidate_count > entry_count:
        raise ValueError("average_candidate_count cannot exceed entry_count")

    # The old frame helper creates a fresh scratch cell for every radius query.
    # The retained index initializes storage once and clears/reuses it thereafter.
    old_scratch_initialization_count = pointer_query_count
    new_scratch_initialization_count = 1
    old_storage_allocation_count = pointer_query_count * 2
    new_storage_allocation_count = 2
    old_storage_slot_count = pointer_query_count * (
        entry_count + average_candidate_count
    )
    new_storage_slot_count = entry_count + average_candidate_count

    return {
        "entry_count": entry_count,
        "average_candidate_count": average_candidate_count,
        "pointer_query_count": pointer_query_count,
        "old_scratch_initialization_count": old_scratch_initialization_count,
        "new_scratch_initialization_count": new_scratch_initialization_count,
        "old_storage_allocation_count": old_storage_allocation_count,
        "new_storage_allocation_count": new_storage_allocation_count,
        "old_storage_slot_count": old_storage_slot_count,
        "new_storage_slot_count": new_storage_slot_count,
        "avoided_scratch_initializations": (
            old_scratch_initialization_count - new_scratch_initialization_count
        ),
        "avoided_storage_allocations": (
            old_storage_allocation_count - new_storage_allocation_count
        ),
        "storage_slot_reduction_ratio": (
            old_storage_slot_count / new_storage_slot_count
        ),
        "interpretation": {
            "included": [
                "scratch initialization count",
                "marks/candidate storage allocation events",
                "retained storage slot upper bound",
            ],
            "excluded": [
                "CPU timing",
                "allocator RSS",
                "GPU work",
                "candidate scan count",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--entry-count", type=int, default=65_536)
    parser.add_argument("--average-candidate-count", type=int, default=32)
    parser.add_argument("--pointer-query-count", type=int, default=1_000_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.entry_count,
        args.average_candidate_count,
        args.pointer_query_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
