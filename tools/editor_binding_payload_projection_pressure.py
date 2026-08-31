"""Deterministic work model for UI Asset binding payload projection."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def subtree_node_count(branching_factor: int, depth: int) -> int:
    branching = max(branching_factor, 0)
    levels = max(depth, 0)
    nodes = 1
    for _ in range(levels):
        nodes = 1 + branching * nodes
    return nodes


def summed_subtree_clone_count(branching_factor: int, depth: int) -> int:
    branching = max(branching_factor, 0)
    levels = max(depth, 0)
    subtree_nodes = 1
    clone_nodes = 1
    for _ in range(levels):
        subtree_nodes = 1 + branching * subtree_nodes
        clone_nodes = subtree_nodes + branching * clone_nodes
    return clone_nodes


def model_pressure(
    projection_count: int = 4096,
    root_field_count: int = 16,
    branching_factor: int = 4,
    depth: int = 5,
) -> dict[str, object]:
    projections = max(projection_count, 0)
    root_fields = max(root_field_count, 0)
    branching = max(branching_factor, 0)
    levels = max(depth, 0)
    nodes_per_field = subtree_node_count(branching, levels)
    payload_value_nodes = root_fields * nodes_per_field

    # The retired path cloned one synthetic root table, its complete payload, and then every
    # flattened entry's complete subtree. The borrowed path still visits each value once.
    root_clone_nodes = (1 + payload_value_nodes) if root_fields else 0
    entry_clone_nodes = root_fields * summed_subtree_clone_count(branching, levels)
    flatten_clone_nodes_per_projection = root_clone_nodes + entry_clone_nodes
    schema_clone_nodes_per_projection = payload_value_nodes
    retired_clone_nodes_per_projection = (
        flatten_clone_nodes_per_projection + schema_clone_nodes_per_projection
    )
    retired_clone_nodes = projections * retired_clone_nodes_per_projection
    borrowed_visits = 2 * projections * payload_value_nodes

    return {
        "schema": "zircon.editor.binding_payload_projection_pressure.v1",
        "inputs": {
            "projection_count": projections,
            "root_field_count": root_fields,
            "branching_factor": branching,
            "depth": levels,
        },
        "payload": {
            "value_nodes_per_projection": payload_value_nodes,
        },
        "retired_owned_projection": {
            "value_node_clone_operations": retired_clone_nodes,
            "value_node_clone_operations_per_projection": retired_clone_nodes_per_projection,
            "flatten_value_node_clone_operations_per_projection": (
                flatten_clone_nodes_per_projection
            ),
            "schema_value_node_clone_operations_per_projection": (
                schema_clone_nodes_per_projection
            ),
        },
        "borrowed_projection": {
            "value_node_clone_operations": 0,
            "value_node_visits": borrowed_visits,
        },
        "delta": {
            "eliminated_value_node_clone_operations": retired_clone_nodes,
            "retired_clone_to_borrowed_visit_ratio": (
                retired_clone_nodes / borrowed_visits if borrowed_visits else 0.0
            ),
        },
        "excluded": [
            "payload path string construction",
            "final display string formatting",
            "schema output string construction",
            "suggestion projection",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--projections", type=int, default=4096)
    parser.add_argument("--root-fields", type=int, default=16)
    parser.add_argument("--branching-factor", type=int, default=4)
    parser.add_argument("--depth", type=int, default=5)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = model_pressure(
        projection_count=args.projections,
        root_field_count=args.root_fields,
        branching_factor=args.branching_factor,
        depth=args.depth,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    main()
