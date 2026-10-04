"""Deterministic work model for local workbench projection invalidation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    interaction_count: int,
    host_node_count: int,
    changed_row_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("interaction_count", interaction_count),
        ("host_node_count", host_node_count),
        ("changed_row_count", changed_row_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if changed_row_count > host_node_count:
        raise ValueError("changed_row_count must not exceed host_node_count")

    old_full_recompute_node_visits = interaction_count * host_node_count
    new_projection_row_visits = interaction_count * changed_row_count
    old_toast_snapshot_collections = interaction_count * 2
    new_toast_snapshot_collections = interaction_count

    return {
        "interaction_count": interaction_count,
        "host_node_count": host_node_count,
        "changed_row_count": changed_row_count,
        "old_full_recompute_node_visits": old_full_recompute_node_visits,
        "new_projection_row_visits": new_projection_row_visits,
        "eliminated_node_or_row_visits": (
            old_full_recompute_node_visits - new_projection_row_visits
        ),
        "old_toast_snapshot_collections": old_toast_snapshot_collections,
        "new_toast_snapshot_collections": new_toast_snapshot_collections,
        "eliminated_toast_snapshot_collections": (
            old_toast_snapshot_collections - new_toast_snapshot_collections
        ),
        "work_reduction_ratio": (
            old_full_recompute_node_visits / new_projection_row_visits
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--interaction-count", type=int, default=4_096)
    parser.add_argument("--host-node-count", type=int, default=32_768)
    parser.add_argument("--changed-row-count", type=int, default=24)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.interaction_count,
        args.host_node_count,
        args.changed_row_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
