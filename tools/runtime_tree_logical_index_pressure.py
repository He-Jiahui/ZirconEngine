import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    node_count: int,
    single_interaction_count: int,
    range_interaction_count: int,
    range_width: int,
    legacy_full_passes_per_interaction: int,
) -> dict[str, Any]:
    values = (
        node_count,
        single_interaction_count,
        range_interaction_count,
        range_width,
        legacy_full_passes_per_interaction,
    )
    if any(value < 0 for value in values):
        raise ValueError("pressure inputs must be non-negative")
    if legacy_full_passes_per_interaction == 0:
        raise ValueError("legacy full-pass count must be positive")
    if range_width > node_count:
        raise ValueError("range width must not exceed the logical node count")

    old_single_visits = (
        node_count * single_interaction_count * legacy_full_passes_per_interaction
    )
    old_range_visits = (
        node_count * range_interaction_count * legacy_full_passes_per_interaction
    )
    new_range_visits = range_width * range_interaction_count
    interaction_count = single_interaction_count + range_interaction_count
    old_temporary_entries = (
        node_count * interaction_count * legacy_full_passes_per_interaction
    )
    old_total_visits = old_single_visits + old_range_visits
    new_total_visits = new_range_visits

    return {
        "model_scope": (
            "logical-node visits and temporary ID-index entries only; the target numbers are "
            "an architecture budget, not evidence that the production index is implemented"
        ),
        "node_count": node_count,
        "single_interaction_count": single_interaction_count,
        "range_interaction_count": range_interaction_count,
        "range_width": range_width,
        "legacy_full_passes_per_interaction": legacy_full_passes_per_interaction,
        "old_single_logical_node_visit_count": old_single_visits,
        "new_single_logical_node_visit_count": 0,
        "old_range_logical_node_visit_count": old_range_visits,
        "new_range_logical_node_visit_count": new_range_visits,
        "old_temporary_id_vector_entry_count": old_temporary_entries,
        "new_temporary_id_vector_entry_count": 0,
        "old_temporary_dedup_entry_count": old_temporary_entries,
        "new_temporary_dedup_entry_count": 0,
        "old_total_logical_node_visit_count": old_total_visits,
        "new_total_logical_node_visit_count": new_total_visits,
        "logical_visit_reduction_ratio": (
            old_total_visits / new_total_visits if new_total_visits > 0 else 0.0
        ),
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-count", type=int, default=100_000)
    parser.add_argument("--single-interaction-count", type=int, default=1_000)
    parser.add_argument("--range-interaction-count", type=int, default=1_000)
    parser.add_argument("--range-width", type=int, default=10)
    parser.add_argument("--legacy-full-passes-per-interaction", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        node_count=args.node_count,
        single_interaction_count=args.single_interaction_count,
        range_interaction_count=args.range_interaction_count,
        range_width=args.range_width,
        legacy_full_passes_per_interaction=args.legacy_full_passes_per_interaction,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
