import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    pointer_move_count: int,
    candidate_change_count: int,
    mean_bubble_depth: int,
    timer_tick_count: int,
) -> dict[str, Any]:
    values = {
        "pointer_move_count": pointer_move_count,
        "candidate_change_count": candidate_change_count,
        "mean_bubble_depth": mean_bubble_depth,
        "timer_tick_count": timer_tick_count,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("counts must be non-negative")
    if candidate_change_count > pointer_move_count:
        raise ValueError("candidate changes cannot exceed pointer moves")

    retired_metadata_string_allocations = pointer_move_count * 2
    retired_bubble_route_allocations = pointer_move_count
    retired_candidate_clone_string_allocations = candidate_change_count * 2
    retired_tooltip_id_allocations = candidate_change_count
    retired_timer_tick_allocations = timer_tick_count * 3
    retired_allocation_site_executions = (
        retired_metadata_string_allocations
        + retired_bubble_route_allocations
        + retired_candidate_clone_string_allocations
        + retired_tooltip_id_allocations
        + retired_timer_tick_allocations
    )

    borrowed_candidate_string_allocations = candidate_change_count * 2
    borrowed_candidate_tooltip_id_allocations = candidate_change_count
    borrowed_candidate_allocation_site_executions = (
        borrowed_candidate_string_allocations
        + borrowed_candidate_tooltip_id_allocations
    )

    return {
        "schema": "zircon.editor.workbench_icon_tooltip_candidate_pressure.v1",
        "inputs": values,
        "retired_owned_candidate_path": {
            "allocation_site_executions": retired_allocation_site_executions,
            "metadata_string_allocations": retired_metadata_string_allocations,
            "bubble_route_allocations": retired_bubble_route_allocations,
            "candidate_clone_string_allocations": (
                retired_candidate_clone_string_allocations
            ),
            "tooltip_id_allocations": retired_tooltip_id_allocations,
            "timer_tick_candidate_and_id_allocations": retired_timer_tick_allocations,
            "bubble_route_element_writes": pointer_move_count * mean_bubble_depth,
        },
        "borrowed_candidate_path": {
            "allocation_site_executions": (
                borrowed_candidate_allocation_site_executions
            ),
            "candidate_change_string_allocations": (
                borrowed_candidate_string_allocations
            ),
            "tooltip_id_allocations": borrowed_candidate_tooltip_id_allocations,
            "bubble_route_element_writes": 0,
        },
        "delta": {
            "avoided_allocation_site_executions": (
                retired_allocation_site_executions
                - borrowed_candidate_allocation_site_executions
            ),
            "allocation_site_execution_reduction_ratio": _ratio(
                retired_allocation_site_executions,
                borrowed_candidate_allocation_site_executions,
            ),
            "avoided_bubble_route_element_writes": (
                pointer_move_count * mean_bubble_depth
            ),
        },
        "interpretation": {
            "included": (
                "deterministic owned String, formatted tooltip id, and bubble-route "
                "Vec allocation-site executions for tooltip-bearing pointer moves"
            ),
            "excluded": (
                "allocator implementation, hit-grid query cost, ancestor node reads, "
                "event coalescing, CPU time, input latency, RSS, paint, and GPU work"
            ),
            "runtime_cpu_measured": False,
            "allocator_or_rss_measured": False,
        },
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pointer-move-count", type=int, default=65_536)
    parser.add_argument("--candidate-change-count", type=int, default=64)
    parser.add_argument("--mean-bubble-depth", type=int, default=6)
    parser.add_argument("--timer-tick-count", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        pointer_move_count=args.pointer_move_count,
        candidate_change_count=args.candidate_change_count,
        mean_bubble_depth=args.mean_bubble_depth,
        timer_tick_count=args.timer_tick_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
