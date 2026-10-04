import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    tree_node_count: int,
    accessibility_node_count: int,
    hidden_relation_target_count: int,
) -> dict[str, Any]:
    values = {
        "tree_node_count": tree_node_count,
        "accessibility_node_count": accessibility_node_count,
        "hidden_relation_target_count": hidden_relation_target_count,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("counts must be non-negative")
    if accessibility_node_count > tree_node_count:
        raise ValueError("accessibility nodes cannot exceed tree nodes")
    if hidden_relation_target_count > accessibility_node_count:
        raise ValueError("hidden relation targets cannot exceed accessibility nodes")

    tree_edges = max(0, tree_node_count - 1)
    chain_depth_sum = tree_node_count * tree_edges // 2
    retired_visibility_parent_edge_visits = chain_depth_sum * 2
    retired_child_flatten_edge_visits = chain_depth_sum
    retained_visibility_parent_edge_visits = tree_edges
    retained_child_flatten_edge_visits = tree_edges
    retired_structural_edge_visits = (
        retired_visibility_parent_edge_visits + retired_child_flatten_edge_visits
    )
    retained_structural_edge_visits = (
        retained_visibility_parent_edge_visits + retained_child_flatten_edge_visits
    )

    return {
        "schema": "zircon.runtime.ui_accessibility_extraction_pressure.v1",
        "inputs": values,
        "workload": {
            "shape": "deep_chain_with_only_the_root_published",
            "snapshot_build_count": 1,
        },
        "retired_repeated_traversal": {
            "visibility_parent_edge_visits": retired_visibility_parent_edge_visits,
            "child_flatten_edge_visits": retired_child_flatten_edge_visits,
            "structural_edge_visits": retired_structural_edge_visits,
            "hidden_target_vector_probe_upper_bound": (
                accessibility_node_count * hidden_relation_target_count
            ),
        },
        "indexed_extraction": {
            "visibility_parent_edge_visits": retained_visibility_parent_edge_visits,
            "child_flatten_edge_visits": retained_child_flatten_edge_visits,
            "structural_edge_visits": retained_structural_edge_visits,
            "hidden_target_map_lookups": hidden_relation_target_count,
        },
        "delta": {
            "avoided_structural_edge_visits": (
                retired_structural_edge_visits - retained_structural_edge_visits
            ),
            "structural_edge_visit_reduction_ratio": _ratio(
                retired_structural_edge_visits,
                retained_structural_edge_visits,
            ),
        },
        "interpretation": {
            "included": (
                "deterministic parent/child edge visits for effective-hidden resolution "
                "and accessibility child flattening in an adversarial valid tree"
            ),
            "excluded": (
                "BTree lookup comparisons, relation/name resolution, serialization, budget "
                "accounting, allocations, CPU time, input latency, RSS, AccessKit work, "
                "OS accessibility traffic, and incremental publication"
            ),
            "runtime_cpu_measured": False,
            "allocator_or_rss_measured": False,
            "incremental_publication_implemented": False,
        },
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree-node-count", type=int, default=16_384)
    parser.add_argument("--accessibility-node-count", type=int, default=8_192)
    parser.add_argument("--hidden-relation-target-count", type=int, default=128)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        tree_node_count=args.tree_node_count,
        accessibility_node_count=args.accessibility_node_count,
        hidden_relation_target_count=args.hidden_relation_target_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
