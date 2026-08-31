import argparse
import json
from pathlib import Path


def run(pointer_move_count: int) -> dict:
    if pointer_move_count < 0:
        raise ValueError("pointer_move_count must be non-negative")

    retired_runtime_surface_queries = pointer_move_count
    retired_workbench_index_queries = pointer_move_count
    shared_workbench_index_queries = pointer_move_count
    retired_total = retired_runtime_surface_queries + retired_workbench_index_queries
    avoided = retired_total - shared_workbench_index_queries

    return {
        "schema": "zircon.editor.workbench-pointer-single-hit-pressure.v1",
        "inputs": {"idle_pointer_move_count": pointer_move_count},
        "retired_double_hit_path": {
            "runtime_surface_hit_queries": retired_runtime_surface_queries,
            "workbench_index_queries": retired_workbench_index_queries,
            "total_spatial_queries": retired_total,
        },
        "published_identity_single_hit_path": {
            "runtime_surface_hit_queries": 0,
            "workbench_index_queries": shared_workbench_index_queries,
            "total_spatial_queries": shared_workbench_index_queries,
        },
        "delta": {
            "avoided_spatial_queries": avoided,
            "spatial_query_reduction_ratio": (
                retired_total / shared_workbench_index_queries
                if shared_workbench_index_queries
                else 0.0
            ),
        },
        "scope": {
            "idle_mouse_move_only": True,
            "latest_value_event_coalescing_implemented": True,
            "cpu_measured": False,
            "allocator_or_rss_measured": False,
            "input_latency_measured": False,
            "gpu_work_measured": False,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pointer-moves", type=int, default=65_536)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.pointer_moves)
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
        return
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(encoded, encoding="utf-8")


if __name__ == "__main__":
    main()
