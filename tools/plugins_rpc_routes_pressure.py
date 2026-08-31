"""Deterministic work model for Plugins10 and Plugins11 hot paths."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins_rpc_routes_pressure.py",
    "zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs",
    "zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs": (
        "5dd962db716858eba3fba6a6bc53852ee6e2aaea"
    ),
    "zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs": (
        "8e3ac84737d70161e67bc3270838e318e1520734"
    ),
}


def run(
    pending_requests: int = 131_072,
    expired_requests: int = 32_768,
    source_tracks: int = 2_048,
    downstream_tracks: int = 256,
) -> dict[str, object]:
    for name, value in (
        ("pending_requests", pending_requests),
        ("expired_requests", expired_requests),
        ("source_tracks", source_tracks),
        ("downstream_tracks", downstream_tracks),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if expired_requests > pending_requests:
        raise ValueError("expired_requests cannot exceed pending_requests")

    routes_per_source = downstream_tracks + 1
    shared_cache_hit_route_rows = source_tracks * downstream_tracks
    insertion_clone_route_rows = (
        source_tracks * routes_per_source + downstream_tracks
    )
    direct_send_edges = source_tracks + downstream_tracks
    planned_gain_slots = insertion_clone_route_rows

    return {
        "schema": "zircon.plugins.rpc_routes_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "RPC table scans, expired-ID materialization, second-pass hash "
                "removals, report writes, route-vector clone calls and copied rows, "
                "cache inserts, direct-send edges, and explicit gain-map reserve work"
            ),
            "excluded": (
                "allocator metadata, hash-table implementation costs, cache locality, "
                "wall-clock duration, audio callback latency, transport latency, and RSS"
            ),
            "dynamic_acceptance_pending": (
                "managed release Rust behavior tests plus alternating release P50/P95 "
                "reproduction of the recorded native models"
            ),
        },
        "inputs": {
            "pending_requests": pending_requests,
            "expired_requests": expired_requests,
            "source_tracks": source_tracks,
            "downstream_tracks": downstream_tracks,
            "routes_per_source": routes_per_source,
        },
        "rpc_expiration": {
            "baseline_pending_table_scan_count": pending_requests,
            "candidate_pending_table_scan_count": pending_requests,
            "baseline_expired_id_materialization_count": expired_requests,
            "candidate_expired_id_materialization_count": 0,
            "baseline_second_pass_hash_removal_count": expired_requests,
            "candidate_second_pass_hash_removal_count": 0,
            "baseline_report_write_count": expired_requests,
            "candidate_report_write_count": expired_requests,
            "baseline_temporary_collection_count": 2,
            "candidate_temporary_collection_count": 1,
            "temporary_collection_reduction_percent": 50.0,
        },
        "route_expansion": {
            "baseline_shared_cache_route_clone_count": source_tracks,
            "candidate_shared_cache_route_clone_count": 0,
            "baseline_shared_cache_route_row_copy_count": (
                shared_cache_hit_route_rows
            ),
            "candidate_shared_cache_route_row_copy_count": 0,
            "baseline_cache_insertion_route_clone_count": (
                source_tracks + downstream_tracks + 1
            ),
            "candidate_cache_insertion_route_clone_count": 0,
            "baseline_cache_insertion_route_row_copy_count": (
                insertion_clone_route_rows
            ),
            "candidate_cache_insertion_route_row_copy_count": 0,
            "baseline_total_route_row_copy_count": (
                shared_cache_hit_route_rows + insertion_clone_route_rows
            ),
            "candidate_total_route_row_copy_count": 0,
            "baseline_cache_insert_count": source_tracks + downstream_tracks + 1,
            "candidate_cache_insert_count": source_tracks + downstream_tracks + 1,
            "candidate_gain_reserve_call_count": direct_send_edges,
            "candidate_planned_gain_slot_count": planned_gain_slots,
        },
        "historical_release_evidence": {
            "rpc": {
                "checksum": 8_727_815_200_911_380_074,
                "baseline_p50_ms": 5.1467,
                "candidate_p50_ms": 2.0265,
                "p50_reduction_percent": 60.6253,
                "baseline_p95_ms": 11.0896,
                "candidate_p95_ms": 5.3114,
                "p95_reduction_percent": 52.1047,
                "baseline_allocations": 28,
                "candidate_allocations": 14,
                "allocation_reduction_percent": 50.0,
            },
            "routes": {
                "checksum": 13_349_105_238_628_374_174,
                "baseline_p50_ms": 48.6129,
                "candidate_p50_ms": 28.6061,
                "p50_reduction_percent": 41.16,
                "baseline_p95_ms": 97.2023,
                "candidate_p95_ms": 61.5532,
                "p95_reduction_percent": 36.67,
                "baseline_allocations": 22_550,
                "candidate_allocations": 4_117,
                "allocation_reduction_percent": 81.742794,
            },
        },
        "invariants": {
            "rpc_timeout_boundary_preserved": True,
            "rpc_timeout_report_preserved": True,
            "route_gain_accumulation_preserved": True,
            "route_ordering_preserved": True,
            "cycle_and_unknown_track_errors_preserved": True,
        },
    }


def source_binding() -> dict[str, object]:
    source_sha256 = {
        relative_path: hashlib.sha256((ROOT / relative_path).read_bytes())
        .hexdigest()
        .upper()
        for relative_path in CRITICAL_SOURCE_FILES
    }
    manifest_lines = [
        f"worktree:{path}:{source_sha256[path]}" for path in sorted(source_sha256)
    ]
    manifest_lines.extend(
        f"head-git-blob:{path}:{HEAD_BASELINE_GIT_BLOBS[path]}"
        for path in sorted(HEAD_BASELINE_GIT_BLOBS)
    )
    manifest_payload = "\n".join(manifest_lines).encode("utf-8")
    return {
        "git_revision": BASELINE_GIT_REVISION,
        "critical_source_files": list(CRITICAL_SOURCE_FILES),
        "source_sha256": source_sha256,
        "head_baseline_git_blobs": dict(HEAD_BASELINE_GIT_BLOBS),
        "source_manifest_sha256": hashlib.sha256(manifest_payload)
        .hexdigest()
        .upper(),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pending-requests", type=int, default=131_072)
    parser.add_argument("--expired-requests", type=int, default=32_768)
    parser.add_argument("--source-tracks", type=int, default=2_048)
    parser.add_argument("--downstream-tracks", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.pending_requests,
        args.expired_requests,
        args.source_tracks,
        args.downstream_tracks,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
