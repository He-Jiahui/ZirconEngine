import argparse
import json
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(
        description=(
            "Model downstream invalidation work for repeated viewport setting commands."
        )
    )
    parser.add_argument("--setter-families", type=int, default=10)
    parser.add_argument("--interactions-per-setter", type=int, default=4096)
    parser.add_argument("--downstream-nodes", type=int, default=32768)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    setter_families = max(args.setter_families, 0)
    interactions_per_setter = max(args.interactions_per_setter, 0)
    downstream_nodes = max(args.downstream_nodes, 0)
    retired_requests = setter_families * interactions_per_setter
    optimized_requests = setter_families if interactions_per_setter else 0

    # Each accepted setting change currently invalidates both render and reflection.
    retired_work_units = retired_requests * downstream_nodes * 2
    optimized_work_units = optimized_requests * downstream_nodes * 2

    result = {
        "schema": "zircon.editor.viewport_idempotent_invalidation_pressure.v1",
        "inputs": {
            "setter_families": setter_families,
            "interactions_per_setter": interactions_per_setter,
            "downstream_nodes": downstream_nodes,
            "invalidation_domains": ["render", "reflection"],
        },
        "retired": {
            "render_invalidation_requests": retired_requests,
            "reflection_invalidation_requests": retired_requests,
            "modeled_downstream_work_units": retired_work_units,
        },
        "optimized": {
            "render_invalidation_requests": optimized_requests,
            "reflection_invalidation_requests": optimized_requests,
            "modeled_downstream_work_units": optimized_work_units,
        },
        "delta": {
            "eliminated_render_invalidation_requests": (
                retired_requests - optimized_requests
            ),
            "eliminated_reflection_invalidation_requests": (
                retired_requests - optimized_requests
            ),
            "eliminated_modeled_downstream_work_units": (
                retired_work_units - optimized_work_units
            ),
            "work_reduction_ratio": (
                retired_work_units / max(optimized_work_units, 1)
            ),
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    main()
