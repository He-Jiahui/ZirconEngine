import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Model repeated asset-control invalidation work before and after idempotence gating."
    )
    parser.add_argument("--interactions", type=int, default=4096)
    parser.add_argument("--host-nodes", type=int, default=32768)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    interactions = max(args.interactions, 0)
    host_nodes = max(args.host_nodes, 0)
    distinct_changes = min(interactions, 1)
    retired_node_visits = interactions * host_nodes
    optimized_node_visits = distinct_changes * host_nodes
    eliminated_node_visits = retired_node_visits - optimized_node_visits

    result = {
        "schema": "zircon.editor.asset_idempotent_interaction_pressure.v1",
        "inputs": {
            "interactions": interactions,
            "host_nodes": host_nodes,
            "distinct_changes": distinct_changes,
        },
        "retired": {
            "global_presentation_requests": interactions,
            "visible_preview_refresh_requests": interactions,
            "host_node_visits": retired_node_visits,
        },
        "optimized": {
            "global_presentation_requests": distinct_changes,
            "visible_preview_refresh_requests": distinct_changes,
            "host_node_visits": optimized_node_visits,
        },
        "delta": {
            "eliminated_global_presentation_requests": interactions - distinct_changes,
            "eliminated_visible_preview_refresh_requests": interactions - distinct_changes,
            "eliminated_host_node_visits": eliminated_node_visits,
            "node_visit_reduction_ratio": (
                retired_node_visits / max(optimized_node_visits, 1)
            ),
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    main()
