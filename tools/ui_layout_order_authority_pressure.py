"""Deterministic work model for generation-owned retained UI child order."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def run(
    child_count: int,
    layout_update_count: int,
    topology_change_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("child_count", child_count),
        ("layout_update_count", layout_update_count),
        ("topology_change_count", topology_change_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if topology_change_count > layout_update_count:
        raise ValueError(
            "topology_change_count must not exceed layout_update_count"
        )

    comparison_levels_per_sort = max(1, math.ceil(math.log2(child_count)))
    comparison_work_per_sort = child_count * comparison_levels_per_sort
    current_order_sort_count = layout_update_count
    generation_owned_order_sort_count = topology_change_count
    current_order_comparison_work = (
        current_order_sort_count * comparison_work_per_sort
    )
    generation_owned_order_comparison_work = (
        generation_owned_order_sort_count * comparison_work_per_sort
    )
    eliminated_order_comparison_work = (
        current_order_comparison_work - generation_owned_order_comparison_work
    )

    return {
        "child_count": child_count,
        "layout_update_count": layout_update_count,
        "topology_change_count": topology_change_count,
        "comparison_levels_per_sort": comparison_levels_per_sort,
        "comparison_work_per_sort": comparison_work_per_sort,
        "current_order_sort_count": current_order_sort_count,
        "generation_owned_order_sort_count": generation_owned_order_sort_count,
        "current_order_comparison_work": current_order_comparison_work,
        "generation_owned_order_comparison_work": (
            generation_owned_order_comparison_work
        ),
        "eliminated_order_comparison_work": eliminated_order_comparison_work,
        "order_comparison_reduction_ratio": (
            current_order_comparison_work
            / generation_owned_order_comparison_work
        ),
        # Parent containers may still need every child's desired size or frame.
        # The order authority removes repeated ordering, not this semantic work.
        "required_child_aggregation_work": child_count * layout_update_count,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--child-count", type=int, default=10_000)
    parser.add_argument("--layout-update-count", type=int, default=10_000)
    parser.add_argument("--topology-change-count", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.child_count,
        args.layout_update_count,
        args.topology_change_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
