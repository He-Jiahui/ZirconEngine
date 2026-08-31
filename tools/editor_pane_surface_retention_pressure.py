import argparse
import json
from pathlib import Path
from typing import Any


PANE_PIPELINE_STAGE_COUNT = 4


def run(
    *,
    pane_count: int,
    nodes_per_pane: int,
    stable_update_count: int,
    changed_update_count: int,
    changed_panes_per_update: int,
) -> dict[str, Any]:
    if pane_count <= 0:
        raise ValueError("pane_count must be positive")
    if nodes_per_pane <= 0:
        raise ValueError("nodes_per_pane must be positive")
    if stable_update_count < 0 or changed_update_count < 0:
        raise ValueError("update counts must be non-negative")
    if not 0 <= changed_panes_per_update <= pane_count:
        raise ValueError("changed_panes_per_update must be within pane_count")
    if changed_update_count > 0 and changed_panes_per_update == 0:
        raise ValueError("changed updates must identify at least one changed pane")

    baseline_initial_surface_build_count = pane_count
    baseline_stable_surface_build_count = pane_count * stable_update_count
    baseline_changed_surface_build_count = pane_count * changed_update_count
    baseline_total_surface_build_count = (
        baseline_initial_surface_build_count
        + baseline_stable_surface_build_count
        + baseline_changed_surface_build_count
    )

    retained_initial_surface_build_count = pane_count
    retained_stable_surface_build_count = 0
    retained_changed_surface_build_count = (
        changed_update_count * changed_panes_per_update
    )
    retained_total_surface_build_count = (
        retained_initial_surface_build_count
        + retained_changed_surface_build_count
    )

    work_per_surface = nodes_per_pane * PANE_PIPELINE_STAGE_COUNT
    baseline_node_stage_visit_count = baseline_total_surface_build_count * work_per_surface
    retained_node_stage_visit_count = retained_total_surface_build_count * work_per_surface
    stable_surface_build_avoidance_count = (
        baseline_stable_surface_build_count - retained_stable_surface_build_count
    )
    stable_surface_build_avoidance_percent = (
        100.0
        if baseline_stable_surface_build_count > 0
        else 0.0
    )

    return {
        "model_scope": "deterministic pane pipeline operation counts; not elapsed time or memory",
        "pane_pipeline_stage_count": PANE_PIPELINE_STAGE_COUNT,
        "pane_count": pane_count,
        "nodes_per_pane": nodes_per_pane,
        "stable_update_count": stable_update_count,
        "changed_update_count": changed_update_count,
        "changed_panes_per_update": changed_panes_per_update,
        "baseline_initial_surface_build_count": baseline_initial_surface_build_count,
        "baseline_stable_surface_build_count": baseline_stable_surface_build_count,
        "baseline_changed_surface_build_count": baseline_changed_surface_build_count,
        "baseline_total_surface_build_count": baseline_total_surface_build_count,
        "retained_initial_surface_build_count": retained_initial_surface_build_count,
        "retained_stable_surface_build_count": retained_stable_surface_build_count,
        "retained_changed_surface_build_count": retained_changed_surface_build_count,
        "retained_total_surface_build_count": retained_total_surface_build_count,
        "retained_unchanged_pane_rebuild_count": 0,
        "stable_surface_build_avoidance_count": stable_surface_build_avoidance_count,
        "stable_surface_build_avoidance_percent": stable_surface_build_avoidance_percent,
        "baseline_node_stage_visit_count": baseline_node_stage_visit_count,
        "retained_node_stage_visit_count": retained_node_stage_visit_count,
        "eliminated_node_stage_visit_count": (
            baseline_node_stage_visit_count - retained_node_stage_visit_count
        ),
        "node_stage_visit_reduction_ratio": (
            baseline_node_stage_visit_count / retained_node_stage_visit_count
        ),
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pane-count", type=int, default=64)
    parser.add_argument("--nodes-per-pane", type=int, default=2_048)
    parser.add_argument("--stable-update-count", type=int, default=1_000)
    parser.add_argument("--changed-update-count", type=int, default=1_000)
    parser.add_argument("--changed-panes-per-update", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        pane_count=args.pane_count,
        nodes_per_pane=args.nodes_per_pane,
        stable_update_count=args.stable_update_count,
        changed_update_count=args.changed_update_count,
        changed_panes_per_update=args.changed_panes_per_update,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
