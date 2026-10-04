"""Deterministic work model for runtime render-path surface-frame publication."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    surface_count: int,
    render_command_count: int,
    hit_entry_count: int,
    hit_cell_entry_count: int,
    interactive_update_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("surface_count", surface_count),
        ("render_command_count", render_command_count),
        ("hit_entry_count", hit_entry_count),
        ("hit_cell_entry_count", hit_cell_entry_count),
        ("interactive_update_count", interactive_update_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    old_clone_work_per_update = surface_count * (
        render_command_count + hit_entry_count + hit_cell_entry_count
    )
    old_total_clone_work = old_clone_work_per_update * interactive_update_count
    new_generation_check_count = surface_count * interactive_update_count

    return {
        "surface_count": surface_count,
        "render_command_count_per_surface": render_command_count,
        "hit_entry_count_per_surface": hit_entry_count,
        "hit_cell_entry_count_per_surface": hit_cell_entry_count,
        "interactive_update_count": interactive_update_count,
        "old_clone_work_per_update": old_clone_work_per_update,
        "old_total_clone_work": old_total_clone_work,
        "new_surface_frame_materialization_count": 0,
        "new_generation_check_count": new_generation_check_count,
        "eliminated_clone_work": old_total_clone_work,
        "work_reduction_ratio": old_total_clone_work / new_generation_check_count,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--surface-count", type=int, default=4)
    parser.add_argument("--render-command-count", type=int, default=32_768)
    parser.add_argument("--hit-entry-count", type=int, default=16_384)
    parser.add_argument("--hit-cell-entry-count", type=int, default=65_536)
    parser.add_argument("--interactive-update-count", type=int, default=4_096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.surface_count,
        args.render_command_count,
        args.hit_entry_count,
        args.hit_cell_entry_count,
        args.interactive_update_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
