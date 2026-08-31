"""Model retained hit-route publication, patching, and route-only payload work."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    full_rebuild_count: int = 64,
    geometry_patch_count: int = 65_536,
    noop_input_patch_count: int = 4_096,
    semantic_input_patch_count: int = 64,
    node_count: int = 16_384,
    hit_entry_count: int = 12_288,
    chain_depth: int = 256,
    input_affected_node_count: int = 8,
    semantic_affected_node_count: int = 256,
) -> dict[str, object]:
    positive_inputs = {
        "full_rebuild_count": full_rebuild_count,
        "geometry_patch_count": geometry_patch_count,
        "noop_input_patch_count": noop_input_patch_count,
        "semantic_input_patch_count": semantic_input_patch_count,
        "node_count": node_count,
        "hit_entry_count": hit_entry_count,
        "chain_depth": chain_depth,
        "input_affected_node_count": input_affected_node_count,
        "semantic_affected_node_count": semantic_affected_node_count,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if hit_entry_count > node_count:
        raise ValueError("hit_entry_count must not exceed node_count")
    if chain_depth > node_count:
        raise ValueError("chain_depth must not exceed node_count")

    ancestor_pass_count = 3
    retired_full_rebuild_ancestor_visits = (
        full_rebuild_count
        * hit_entry_count
        * chain_depth
        * ancestor_pass_count
    )
    retired_geometry_patch_ancestor_visits = (
        geometry_patch_count * chain_depth * ancestor_pass_count
    )
    retired_noop_input_patch_ancestor_visits = (
        noop_input_patch_count
        * input_affected_node_count
        * chain_depth
        * ancestor_pass_count
    )
    retired_semantic_input_patch_ancestor_visits = (
        semantic_input_patch_count
        * semantic_affected_node_count
        * chain_depth
        * ancestor_pass_count
    )
    retired_work_units = (
        retired_full_rebuild_ancestor_visits
        + retired_geometry_patch_ancestor_visits
        + retired_noop_input_patch_ancestor_visits
        + retired_semantic_input_patch_ancestor_visits
    )

    retained_full_route_resolution_work = full_rebuild_count * node_count * 2
    retained_full_entry_admission_work = full_rebuild_count * hit_entry_count
    retained_geometry_route_lookups = geometry_patch_count
    retained_noop_input_work = (
        noop_input_patch_count * input_affected_node_count * 2
    )
    retained_semantic_input_work = semantic_input_patch_count * (
        node_count + semantic_affected_node_count * 2
    )
    retained_work_units = (
        retained_full_route_resolution_work
        + retained_full_entry_admission_work
        + retained_geometry_route_lookups
        + retained_noop_input_work
        + retained_semantic_input_work
    )

    node_id_bytes = 8
    vec_header_bytes = 24
    estimated_route_node_bytes = 16
    retired_route_node_id_bytes = (
        hit_entry_count * chain_depth * node_id_bytes
    )
    retired_route_vec_header_bytes = hit_entry_count * vec_header_bytes
    retired_route_payload_bytes = (
        retired_route_node_id_bytes + retired_route_vec_header_bytes
    )
    retained_route_payload_bytes = node_count * estimated_route_node_bytes

    return {
        "schema": "zircon.runtime.ui_hit_route_index_pressure.v1",
        "inputs": positive_inputs,
        "retired_per_entry_routes": {
            "ancestor_passes_per_entry_or_patch": ancestor_pass_count,
            "full_rebuild_ancestor_visits": retired_full_rebuild_ancestor_visits,
            "geometry_patch_ancestor_visits": retired_geometry_patch_ancestor_visits,
            "noop_input_patch_ancestor_visits": (
                retired_noop_input_patch_ancestor_visits
            ),
            "semantic_input_patch_ancestor_visits": (
                retired_semantic_input_patch_ancestor_visits
            ),
            "combined_work_units": retired_work_units,
            "route_node_id_bytes": retired_route_node_id_bytes,
            "route_vec_header_bytes": retired_route_vec_header_bytes,
            "route_only_payload_bytes": retired_route_payload_bytes,
        },
        "retained_route_index": {
            "full_route_resolution_work": retained_full_route_resolution_work,
            "full_entry_admission_work": retained_full_entry_admission_work,
            "geometry_route_lookups": retained_geometry_route_lookups,
            "noop_input_precompute_and_entry_work": retained_noop_input_work,
            "semantic_input_work_including_snapshot_cow": (
                retained_semantic_input_work
            ),
            "combined_work_units": retained_work_units,
            "estimated_route_node_bytes": estimated_route_node_bytes,
            "route_only_payload_bytes": retained_route_payload_bytes,
            "route_only_payload_bytes_per_node": estimated_route_node_bytes,
            "noop_input_route_table_clone_count": 0,
        },
        "delta": {
            "avoided_work_units": retired_work_units - retained_work_units,
            "work_reduction_ratio": retired_work_units / retained_work_units,
            "avoided_route_only_payload_bytes": (
                retired_route_payload_bytes - retained_route_payload_bytes
            ),
            "route_only_payload_reduction_ratio": (
                retired_route_payload_bytes / retained_route_payload_bytes
            ),
        },
        "interpretation": {
            "included": "three retired ancestor walks per hit entry or patched node; two iterative visits per node plus one admission per hit entry on full publication; indexed geometry lookup; no-op input precompute; one full route-table COW per true semantic mutation while immutable projected/frame snapshots share the prior Arc; and estimated route-only payload bytes",
            "excluded": "BTreeMap lookup latency, cell membership work, entry/control string payload, Vec capacity above length, allocator latency, CPU time, RSS, cache effects, popup geometry projection, event-time selected-path construction, malformed-tree frequency, and chunked/persistent route-table designs not yet implemented",
            "scope": "deterministic deep-tree algorithm and route-only payload model, not a measured editor latency or process-memory result",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--full-rebuild-count", type=int, default=64)
    parser.add_argument("--geometry-patch-count", type=int, default=65_536)
    parser.add_argument("--noop-input-patch-count", type=int, default=4_096)
    parser.add_argument("--semantic-input-patch-count", type=int, default=64)
    parser.add_argument("--node-count", type=int, default=16_384)
    parser.add_argument("--hit-entry-count", type=int, default=12_288)
    parser.add_argument("--chain-depth", type=int, default=256)
    parser.add_argument("--input-affected-node-count", type=int, default=8)
    parser.add_argument("--semantic-affected-node-count", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.full_rebuild_count,
        args.geometry_patch_count,
        args.noop_input_patch_count,
        args.semantic_input_patch_count,
        args.node_count,
        args.hit_entry_count,
        args.chain_depth,
        args.input_affected_node_count,
        args.semantic_affected_node_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
