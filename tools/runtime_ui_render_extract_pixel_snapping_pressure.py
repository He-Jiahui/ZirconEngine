import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    tree_node_count: int,
    changed_command_node_count: int,
    ancestor_depth: int,
    update_count: int,
) -> dict[str, Any]:
    values = {
        "tree_node_count": tree_node_count,
        "changed_command_node_count": changed_command_node_count,
        "ancestor_depth": ancestor_depth,
        "update_count": update_count,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("counts and depth must be non-negative")
    if changed_command_node_count > tree_node_count:
        raise ValueError("changed command nodes cannot exceed the tree")

    retired_node_visits = tree_node_count * update_count
    closure_nodes_per_update = min(
        tree_node_count,
        changed_command_node_count * ancestor_depth,
    )
    closure_node_visits = closure_nodes_per_update * update_count

    return {
        "schema": "zircon.runtime.ui_render_extract_pixel_snapping_pressure.v1",
        "inputs": values,
        "retired_full_tree_scan": {
            "node_visits": retired_node_visits,
            "nodes_per_update": tree_node_count,
        },
        "command_ancestor_closure": {
            "node_visits": closure_node_visits,
            "nodes_per_update_upper_bound": closure_nodes_per_update,
        },
        "delta": {
            "avoided_node_visits": retired_node_visits - closure_node_visits,
            "node_visit_reduction_ratio": _ratio(
                retired_node_visits,
                closure_node_visits,
            ),
        },
        "interpretation": {
            "included": (
                "deterministic UiTree node visits for pixel-snapping inheritance "
                "during local render patches"
            ),
            "excluded": (
                "BTreeMap lookup cost, command extraction, text preparation, render cache "
                "patching, allocations, CPU time, input latency, RSS, and GPU work"
            ),
            "cpu_or_latency_measured": False,
        },
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree-node-count", type=int, default=16_384)
    parser.add_argument("--changed-command-node-count", type=int, default=1)
    parser.add_argument("--ancestor-depth", type=int, default=8)
    parser.add_argument("--update-count", type=int, default=4_096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        tree_node_count=args.tree_node_count,
        changed_command_node_count=args.changed_command_node_count,
        ancestor_depth=args.ancestor_depth,
        update_count=args.update_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
