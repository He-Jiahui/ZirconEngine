"""Model command ownership work across Runtime UI publication generations."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    update_count: int = 4_096,
    surface_count: int = 64,
    commands_per_surface: int = 4_096,
    changed_surface_count: int = 1,
    changed_commands_per_surface: int = 1,
    command_segment_size: int = 64,
    directory_fanout: int = 32,
) -> dict[str, object]:
    positive_inputs = {
        "update_count": update_count,
        "surface_count": surface_count,
        "commands_per_surface": commands_per_surface,
        "changed_surface_count": changed_surface_count,
        "changed_commands_per_surface": changed_commands_per_surface,
        "command_segment_size": command_segment_size,
        "directory_fanout": directory_fanout,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if changed_surface_count > surface_count:
        raise ValueError("changed_surface_count must not exceed surface_count")
    if changed_commands_per_surface > commands_per_surface:
        raise ValueError(
            "changed_commands_per_surface must not exceed commands_per_surface"
        )

    commands_per_submission = surface_count * commands_per_surface
    changed_surface_command_clones = (
        update_count * changed_surface_count * commands_per_surface
    )
    aggregate_flat_command_clones = update_count * commands_per_submission
    flat_total_command_clones = changed_surface_command_clones + aggregate_flat_command_clones
    command_segment_count = (
        commands_per_surface + command_segment_size - 1
    ) // command_segment_size
    touched_command_segments = min(
        command_segment_count,
        (changed_commands_per_surface + command_segment_size - 1)
        // command_segment_size,
    )
    cloned_commands_per_changed_surface = min(
        commands_per_surface, touched_command_segments * command_segment_size
    )
    persistent_command_clones = (
        update_count * changed_surface_count * cloned_commands_per_changed_surface
    )
    directory_depth = 1
    directory_nodes_at_level = command_segment_count
    while directory_nodes_at_level > directory_fanout:
        directory_nodes_at_level = (
            directory_nodes_at_level + directory_fanout - 1
        ) // directory_fanout
        directory_depth += 1
    persistent_directory_node_clones = (
        update_count * changed_surface_count * touched_command_segments * directory_depth
    )
    segment_handle_publications = update_count * surface_count
    required_renderer_command_visits = update_count * commands_per_submission

    return {
        "schema": "zircon.runtime.ui_segmented_submission_pressure.v2",
        "inputs": positive_inputs,
        "legacy_flat_aggregate_submission": {
            "changed_surface_command_clones": changed_surface_command_clones,
            "aggregate_flat_command_vector_allocations": update_count,
            "aggregate_flat_command_clones": aggregate_flat_command_clones,
            "combined_command_clone_events": flat_total_command_clones,
            "temporary_segment_handle_publications": segment_handle_publications,
            "required_renderer_command_visits": required_renderer_command_visits,
        },
        "surface_flat_segmented_submission": {
            "changed_surface_command_clones": changed_surface_command_clones,
            "aggregate_flat_command_vector_allocations": 0,
            "aggregate_flat_command_clones": 0,
            "combined_command_clone_events": changed_surface_command_clones,
            "node_id_projection_command_clones": changed_surface_command_clones,
            "segment_table_snapshot_count": update_count,
            "segment_handle_publications": segment_handle_publications,
            "required_renderer_command_visits": required_renderer_command_visits,
        },
        "persistent_command_submission": {
            "command_segment_count_per_surface": command_segment_count,
            "touched_command_segments_per_changed_surface": touched_command_segments,
            "directory_depth": directory_depth,
            "surface_command_clones": persistent_command_clones,
            "surface_directory_node_clones": persistent_directory_node_clones,
            "runtime_node_id_projection_command_clones": 0,
            "aggregate_flat_command_vector_allocations": 0,
            "aggregate_flat_command_clones": 0,
            "segment_table_snapshot_count": update_count,
            "segment_handle_publications": segment_handle_publications,
            "required_renderer_command_visits": required_renderer_command_visits,
        },
        "delta": {
            "avoided_aggregate_flat_command_vector_allocations": update_count,
            "avoided_command_clone_events": aggregate_flat_command_clones,
            "surface_publication_command_clone_reduction_ratio": (
                changed_surface_command_clones / persistent_command_clones
            ),
            "legacy_total_command_clone_reduction_ratio": (
                flat_total_command_clones / persistent_command_clones
            ),
            "renderer_command_visit_reduction": 0,
        },
        "interpretation": {
            "included": "legacy whole-surface command copies, legacy aggregate flattening, persistent 64-command leaf copies and directory path copies, zero-copy Runtime node-id projection handles, ordered Arc-backed submission publication, and the renderer's still-required command visits",
            "excluded": "command payload bytes, deep String or asset payload clone cost, Arc atomic latency, allocator latency, CPU time, RSS, GPU time, cache effects, renderer batch conversion cost, stable frames that hit the aggregate cache, sparse changes spanning more leaves than the packed lower-bound model, and command-count-changing full-snapshot fallback",
            "scope": "deterministic ownership-work model for one or more changed surfaces per aggregate update, not a measured editor latency, memory, or frame-rate result",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update-count", type=int, default=4_096)
    parser.add_argument("--surface-count", type=int, default=64)
    parser.add_argument("--commands-per-surface", type=int, default=4_096)
    parser.add_argument("--changed-surface-count", type=int, default=1)
    parser.add_argument("--changed-commands-per-surface", type=int, default=1)
    parser.add_argument("--command-segment-size", type=int, default=64)
    parser.add_argument("--directory-fanout", type=int, default=32)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.update_count,
        args.surface_count,
        args.commands_per_surface,
        args.changed_surface_count,
        args.changed_commands_per_surface,
        args.command_segment_size,
        args.directory_fanout,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
