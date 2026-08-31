"""Deterministic work model for node-local retained measurement validity."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    clean_subtree_node_count: int,
    update_count: int,
    required_measured_node_count: int,
    root_direct_child_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("clean_subtree_node_count", clean_subtree_node_count),
        ("update_count", update_count),
        ("required_measured_node_count", required_measured_node_count),
        ("root_direct_child_count", root_direct_child_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    retired_forced_nodes_per_update = (
        required_measured_node_count + clean_subtree_node_count
    )
    local_validity_measured_nodes_per_update = required_measured_node_count
    # The planner probes the invalid parent and each direct child. A valid clean
    # child returns before descending into its retained subtree.
    local_validity_probe_nodes_per_update = 1 + root_direct_child_count
    retired_forced_measured_node_work = retired_forced_nodes_per_update * update_count
    local_validity_measured_node_work = (
        local_validity_measured_nodes_per_update * update_count
    )
    local_validity_probe_node_work = local_validity_probe_nodes_per_update * update_count
    eliminated_measured_node_work = (
        retired_forced_measured_node_work - local_validity_measured_node_work
    )

    return {
        "clean_subtree_node_count": clean_subtree_node_count,
        "update_count": update_count,
        "required_measured_node_count": required_measured_node_count,
        "root_direct_child_count": root_direct_child_count,
        "retired_forced_nodes_per_update": retired_forced_nodes_per_update,
        "local_validity_measured_nodes_per_update": (
            local_validity_measured_nodes_per_update
        ),
        "local_validity_probe_nodes_per_update": local_validity_probe_nodes_per_update,
        "retired_forced_measured_node_work": retired_forced_measured_node_work,
        "local_validity_measured_node_work": local_validity_measured_node_work,
        "local_validity_probe_node_work": local_validity_probe_node_work,
        "eliminated_measured_node_work": eliminated_measured_node_work,
        "measured_node_work_reduction_ratio": (
            retired_forced_measured_node_work / local_validity_measured_node_work
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--clean-subtree-node-count", type=int, default=10_000)
    parser.add_argument("--update-count", type=int, default=10_000)
    parser.add_argument("--required-measured-node-count", type=int, default=2)
    parser.add_argument("--root-direct-child-count", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.clean_subtree_node_count,
        args.update_count,
        args.required_measured_node_count,
        args.root_direct_child_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
