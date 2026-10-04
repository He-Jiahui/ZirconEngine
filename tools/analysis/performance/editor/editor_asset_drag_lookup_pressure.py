#!/usr/bin/env python3
"""Model the lookup work at Asset Browser drag start.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import json


def pressure_report(visible_assets: int, drag_starts: int) -> dict[str, object]:
    if visible_assets <= 0:
        raise ValueError("visible_assets must be positive")
    if drag_starts <= 0:
        raise ValueError("drag_starts must be positive")

    average_linear_visits_per_drag = (visible_assets + 1) / 2
    legacy_item_visits = average_linear_visits_per_drag * drag_starts
    indexed_operations = 2 * drag_starts
    return {
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "visible_assets": visible_assets,
            "drag_starts": drag_starts,
            "lookup_distribution": "uniform_existing_uuid",
        },
        "legacy": {
            "algorithm": "linear_iter_find",
            "expected_item_visits": legacy_item_visits,
            "complexity": "O(N) per drag start",
        },
        "indexed": {
            "algorithm": "shared_uuid_hash_index_then_chunk_get",
            "logical_lookup_operations": indexed_operations,
            "complexity": "expected O(1) per drag start",
            "additional_index_allocations": 0,
        },
        "expected_work_reduction_ratio": legacy_item_visits / indexed_operations,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--visible-assets", type=int, default=100_000)
    parser.add_argument("--drag-starts", type=int, default=1_000)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(args.visible_assets, args.drag_starts)
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
