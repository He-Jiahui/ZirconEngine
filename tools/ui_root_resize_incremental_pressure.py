"""Deterministic work model for retained root-resize layout routing."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    total_node_count: int,
    resize_step_count: int,
    root_count: int,
    parent_size_dependent_child_count: int,
    interaction_dirty_node_count: int = 1,
) -> dict[str, int | float]:
    for name, value in (
        ("total_node_count", total_node_count),
        ("resize_step_count", resize_step_count),
        ("root_count", root_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if parent_size_dependent_child_count < 0:
        raise ValueError("parent_size_dependent_child_count must be non-negative")
    if interaction_dirty_node_count < 0:
        raise ValueError("interaction_dirty_node_count must be non-negative")
    if root_count > total_node_count:
        raise ValueError("root_count cannot exceed total_node_count")

    full_measure_probe_work = total_node_count * resize_step_count
    full_arrange_probe_work = total_node_count * resize_step_count
    incremental_measure_probe_work = 0
    incremental_arrange_probes_per_step = (
        root_count + parent_size_dependent_child_count
    )
    incremental_arrange_probe_work = (
        incremental_arrange_probes_per_step * resize_step_count
    )
    combined_patch_nodes_per_step = (
        incremental_arrange_probes_per_step + interaction_dirty_node_count
    )
    full_arranged_patch_work = total_node_count * resize_step_count
    full_hit_patch_work = total_node_count * resize_step_count
    full_render_patch_work = total_node_count * resize_step_count
    incremental_arranged_patch_work = combined_patch_nodes_per_step * resize_step_count
    incremental_hit_patch_work = combined_patch_nodes_per_step * resize_step_count
    incremental_render_patch_work = combined_patch_nodes_per_step * resize_step_count

    return {
        "total_node_count": total_node_count,
        "resize_step_count": resize_step_count,
        "root_count": root_count,
        "parent_size_dependent_child_count": parent_size_dependent_child_count,
        "interaction_dirty_node_count": interaction_dirty_node_count,
        "full_measure_probe_work": full_measure_probe_work,
        "full_arrange_probe_work": full_arrange_probe_work,
        "incremental_measure_probe_work": incremental_measure_probe_work,
        "incremental_arrange_probes_per_step": incremental_arrange_probes_per_step,
        "incremental_arrange_probe_work": incremental_arrange_probe_work,
        "eliminated_measure_probe_work": (
            full_measure_probe_work - incremental_measure_probe_work
        ),
        "eliminated_arrange_probe_work": (
            full_arrange_probe_work - incremental_arrange_probe_work
        ),
        "arrange_probe_reduction_ratio": (
            full_arrange_probe_work / incremental_arrange_probe_work
        ),
        "combined_patch_nodes_per_step": combined_patch_nodes_per_step,
        "full_arranged_patch_work": full_arranged_patch_work,
        "full_hit_patch_work": full_hit_patch_work,
        "full_render_patch_work": full_render_patch_work,
        "incremental_arranged_patch_work": incremental_arranged_patch_work,
        "incremental_hit_patch_work": incremental_hit_patch_work,
        "incremental_render_patch_work": incremental_render_patch_work,
        "combined_post_layout_reduction_ratio": (
            (full_arranged_patch_work + full_hit_patch_work + full_render_patch_work)
            /
            (
                incremental_arranged_patch_work
                + incremental_hit_patch_work
                + incremental_render_patch_work
            )
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--total-node-count", type=int, default=10_000)
    parser.add_argument("--resize-step-count", type=int, default=200)
    parser.add_argument("--root-count", type=int, default=1)
    parser.add_argument(
        "--parent-size-dependent-child-count", type=int, default=1
    )
    parser.add_argument("--interaction-dirty-node-count", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.total_node_count,
        args.resize_step_count,
        args.root_count,
        args.parent_size_dependent_child_count,
        args.interaction_dirty_node_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
