"""Model redundant command-vector ownership work in the Runtime UI render cache."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    update_count: int = 4096,
    commands_per_update: int = 32768,
    cache_entry_count: int = 16384,
    changed_commands_per_update: int = 8,
    modeled_command_payload_bytes: int = 512,
) -> dict[str, object]:
    if update_count <= 0:
        raise ValueError("update_count must be positive")
    if commands_per_update < 0:
        raise ValueError("commands_per_update must be non-negative")
    if cache_entry_count < 0:
        raise ValueError("cache_entry_count must be non-negative")
    if not 0 <= changed_commands_per_update <= commands_per_update:
        raise ValueError(
            "changed_commands_per_update must be within the command count"
        )
    if modeled_command_payload_bytes < 0:
        raise ValueError("modeled_command_payload_bytes must be non-negative")

    command_header_moves = update_count * commands_per_update
    retired_cache_entry_visits = update_count * cache_entry_count * 2
    single_pass_cache_entry_visits = update_count * cache_entry_count
    retained_payload_bytes = commands_per_update * modeled_command_payload_bytes
    local_patch_clone_events = update_count * changed_commands_per_update
    return {
        "schema": "zircon.runtime.ui_render_cache_command_buffer_pressure.v2",
        "inputs": {
            "update_count": update_count,
            "commands_per_update": commands_per_update,
            "cache_entry_count": cache_entry_count,
            "changed_commands_per_update": changed_commands_per_update,
            "modeled_command_payload_bytes": modeled_command_payload_bytes,
        },
        "retired_rematerialization": {
            "additional_command_vector_allocations": update_count,
            "inter_vector_command_header_moves": command_header_moves,
        },
        "borrowed_input_update": {
            "additional_command_vector_allocations": 0,
            "inter_vector_command_header_moves": 0,
        },
        "retired_stale_reconciliation": {
            "full_cache_entry_passes_per_update": 2,
            "cache_entry_visits": retired_cache_entry_visits,
        },
        "single_pass_stale_reconciliation": {
            "full_cache_entry_passes_per_update": 1,
            "cache_entry_visits": single_pass_cache_entry_visits,
        },
        "retired_geometry_patchable_refresh": {
            "temporary_node_id_vector_allocations": update_count,
            "inter_vector_node_id_moves": single_pass_cache_entry_visits,
        },
        "direct_geometry_patchable_refresh": {
            "temporary_node_id_vector_allocations": 0,
            "inter_vector_node_id_moves": 0,
        },
        "retired_command_range_reindex": {
            "additional_full_command_passes_per_update": 1,
            "command_visits": command_header_moves,
        },
        "inline_command_range_publication": {
            "additional_full_command_passes_per_update": 0,
            "node_state_publication_visits": single_pass_cache_entry_visits,
        },
        "retired_full_command_mirror": {
            "retained_command_count": commands_per_update,
            "retained_modeled_payload_bytes": retained_payload_bytes,
            "cold_build_command_clones": commands_per_update,
            "local_patch_command_clones": local_patch_clone_events,
        },
        "compact_derived_metadata": {
            "retained_full_command_count": 0,
            "retained_modeled_payload_bytes": 0,
            "cold_build_command_clones": 0,
            "local_patch_command_clones": 0,
        },
        "retired_surface_serialization": {
            "command_record_count": commands_per_update * 2,
        },
        "compact_surface_serialization": {
            "command_record_count": commands_per_update,
        },
        "delta": {
            "avoided_command_vector_allocations": update_count,
            "avoided_inter_vector_command_header_moves": command_header_moves,
            "avoided_cache_entry_visits": (
                retired_cache_entry_visits - single_pass_cache_entry_visits
            ),
            "cache_entry_visit_reduction_ratio": (
                retired_cache_entry_visits / single_pass_cache_entry_visits
                if single_pass_cache_entry_visits
                else 1.0
            ),
            "avoided_geometry_node_id_vector_allocations": update_count,
            "avoided_inter_vector_node_id_moves": single_pass_cache_entry_visits,
            "avoided_command_range_work_units": (
                command_header_moves - single_pass_cache_entry_visits
            ),
            "command_range_work_reduction_ratio": (
                command_header_moves / single_pass_cache_entry_visits
                if single_pass_cache_entry_visits
                else 1.0
            ),
            "avoided_retained_payload_bytes": retained_payload_bytes,
            "avoided_cold_build_command_clones": commands_per_update,
            "avoided_local_patch_command_clones": local_patch_clone_events,
            "serialization_command_record_ratio": (
                2.0 if commands_per_update else 1.0
            ),
        },
        "interpretation": {
            "included": "the second output command Vec previously allocated by UiSurfaceRenderCache::update, moves into that Vec, stable-bucket collect plus retain passes, the geometry-patchable intermediate node-id Vec, command-range reindex work versus active-node state publication, full-command cache mirror counts, explicitly modeled deep payload bytes, cold-build/local-patch command clone events, and serialized command record counts",
            "excluded": "render extraction, BTreeMap/HashSet/bucket allocation overhead, stale descriptor and truncation work, allocator latency, measured CPU time, measured RSS, command header bytes, and real command payload-size distribution",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--update-count", type=int, default=4096)
    parser.add_argument("--commands-per-update", type=int, default=32768)
    parser.add_argument("--cache-entry-count", type=int, default=16384)
    parser.add_argument("--changed-commands-per-update", type=int, default=8)
    parser.add_argument("--modeled-command-payload-bytes", type=int, default=512)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.update_count,
        args.commands_per_update,
        args.cache_entry_count,
        args.changed_commands_per_update,
        args.modeled_command_payload_bytes,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
