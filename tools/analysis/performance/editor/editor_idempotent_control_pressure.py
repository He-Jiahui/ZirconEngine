import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Model presentation work for repeated controls with authoritative change gates."
    )
    parser.add_argument("--control-families", type=int, default=4)
    parser.add_argument("--interactions-per-control", type=int, default=4096)
    parser.add_argument("--host-nodes", type=int, default=32768)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    control_families = max(args.control_families, 0)
    interactions_per_control = max(args.interactions_per_control, 0)
    host_nodes = max(args.host_nodes, 0)
    retired_requests = control_families * interactions_per_control
    optimized_requests = control_families if interactions_per_control else 0
    retired_node_visits = retired_requests * host_nodes
    optimized_node_visits = optimized_requests * host_nodes

    result = {
        "schema": "zircon.editor.idempotent_control_pressure.v1",
        "inputs": {
            "control_families": control_families,
            "interactions_per_control": interactions_per_control,
            "host_nodes": host_nodes,
        },
        "retired": {
            "global_presentation_requests": retired_requests,
            "host_node_visits": retired_node_visits,
        },
        "optimized": {
            "global_presentation_requests": optimized_requests,
            "host_node_visits": optimized_node_visits,
        },
        "delta": {
            "eliminated_global_presentation_requests": (
                retired_requests - optimized_requests
            ),
            "eliminated_host_node_visits": retired_node_visits - optimized_node_visits,
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
