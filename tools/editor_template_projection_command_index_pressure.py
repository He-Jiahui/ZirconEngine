"""Model retained template projection command visits before and after node indexing."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    update_count: int = 4096,
    render_command_count: int = 65_536,
    changed_control_count: int = 8,
    changed_geometry_node_count: int = 8,
    commands_per_node: int = 2,
) -> dict[str, object]:
    if update_count <= 0:
        raise ValueError("update_count must be positive")
    if render_command_count <= 0:
        raise ValueError("render_command_count must be positive")
    if changed_control_count < 0 or changed_geometry_node_count < 0:
        raise ValueError("changed counts must be non-negative")
    if commands_per_node <= 0:
        raise ValueError("commands_per_node must be positive")

    # The retired text-update path made three topology passes and one geometry pass.
    retired_visits_per_update = 4 * render_command_count
    indexed_visits_per_update = commands_per_node * (
        changed_control_count + changed_geometry_node_count
    )
    retired_visits = retired_visits_per_update * update_count
    indexed_visits = indexed_visits_per_update * update_count
    return {
        "schema": "zircon.editor.template_projection_command_index_pressure.v1",
        "inputs": {
            "update_count": update_count,
            "render_command_count": render_command_count,
            "changed_control_count": changed_control_count,
            "changed_geometry_node_count": changed_geometry_node_count,
            "commands_per_node": commands_per_node,
        },
        "retired": {
            "command_visits_per_update": retired_visits_per_update,
            "command_visits": retired_visits,
        },
        "indexed": {
            "command_visits_per_update": indexed_visits_per_update,
            "command_visits": indexed_visits,
        },
        "delta": {
            "avoided_command_visits": retired_visits - indexed_visits,
            "work_reduction_ratio": retired_visits / indexed_visits
            if indexed_visits
            else 0.0,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update-count", type=int, default=4096)
    parser.add_argument("--render-command-count", type=int, default=65_536)
    parser.add_argument("--changed-control-count", type=int, default=8)
    parser.add_argument("--changed-geometry-node-count", type=int, default=8)
    parser.add_argument("--commands-per-node", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        update_count=args.update_count,
        render_command_count=args.render_command_count,
        changed_control_count=args.changed_control_count,
        changed_geometry_node_count=args.changed_geometry_node_count,
        commands_per_node=args.commands_per_node,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
