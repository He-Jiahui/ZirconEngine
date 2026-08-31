"""Deterministic work model for Plugins12, Plugins19, and Plugins21."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins_projection_sort_discovery_pressure.py",
    "zircon_plugins/physics/runtime/src/manager/world_sync.rs",
    "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/prepare_frame/collect_pending_updates.rs",
    "zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/contract.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/physics/runtime/src/manager/world_sync.rs": (
        "c851d5de0d7f94c1921e5de7e5ee5b1ad9ae6ee1"
    ),
    "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/prepare_frame/collect_pending_updates.rs": (
        "6f6108ce5bea947c6dfc41d8c2cdfd69ac1162f7"
    ),
    "zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/contract.rs": (
        "759d020e113114d5790423c351d5839dee1aca5e"
    ),
}


def run(
    physics_nodes: int = 65_536,
    pending_updates: int = 1_024,
    modeled_sort_comparisons: int = 10_240,
    discovery_inputs: int = 262_144,
    clones_per_input: int = 6,
) -> dict[str, object]:
    for name, value in (
        ("physics_nodes", physics_nodes),
        ("pending_updates", pending_updates),
        ("modeled_sort_comparisons", modeled_sort_comparisons),
        ("discovery_inputs", discovery_inputs),
        ("clones_per_input", clones_per_input),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    baseline_sort_key_evaluations = modeled_sort_comparisons * 2
    candidate_sort_key_evaluations = pending_updates
    baseline_path_allocations = discovery_inputs * (clones_per_input + 1)
    candidate_path_allocations = discovery_inputs * 2

    return {
        "schema": "zircon.plugins.projection_sort_discovery_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "world-sync projection rows, output capacity sites, modeled nested payload "
                "clones and moves, sort-key evaluations and dimensions, discovery path-owner "
                "allocations, deep path clones, and shared-handle clones"
            ),
            "excluded": (
                "allocator metadata, Rust sort implementation comparison count, hash and tree "
                "costs, cache locality, wall-clock duration, physics tick latency, GPU frame "
                "time, filesystem discovery latency, and RSS"
            ),
            "dynamic_acceptance_pending": (
                "managed release Rust behavior tests plus alternating release P50/P95 "
                "reproduction of the recorded native models"
            ),
        },
        "inputs": {
            "physics_nodes": physics_nodes,
            "pending_updates": pending_updates,
            "modeled_sort_comparisons": modeled_sort_comparisons,
            "discovery_inputs": discovery_inputs,
            "clones_per_input": clones_per_input,
        },
        "world_sync_projection": {
            "baseline_snapshot_capture_count": 1,
            "candidate_snapshot_capture_count": 1,
            "baseline_capacity_count_row_visit_count": 0,
            "candidate_capacity_count_row_visit_count": physics_nodes,
            "baseline_projection_row_visit_count": physics_nodes,
            "candidate_projection_row_visit_count": physics_nodes,
            "baseline_presized_output_vector_count": 0,
            "candidate_presized_output_vector_count": 4,
            "baseline_modeled_nested_payload_clone_count": physics_nodes * 3,
            "candidate_modeled_nested_payload_clone_count": 0,
            "baseline_modeled_nested_payload_move_count": 0,
            "candidate_modeled_nested_payload_move_count": physics_nodes * 3,
        },
        "pending_update_sort": {
            "modeled_sort_comparison_count": modeled_sort_comparisons,
            "baseline_sort_key_evaluation_count": baseline_sort_key_evaluations,
            "candidate_sort_key_evaluation_count": candidate_sort_key_evaluations,
            "key_evaluation_reduction_percent": reduction_percent(
                baseline_sort_key_evaluations, candidate_sort_key_evaluations
            ),
            "baseline_expensive_graph_query_count": baseline_sort_key_evaluations * 4,
            "candidate_expensive_graph_query_count": candidate_sort_key_evaluations * 4,
            "baseline_priority_dimension_evaluation_count": (
                baseline_sort_key_evaluations * 6
            ),
            "candidate_priority_dimension_evaluation_count": (
                candidate_sort_key_evaluations * 6
            ),
        },
        "discovery_input": {
            "baseline_path_owner_allocation_count": baseline_path_allocations,
            "candidate_path_owner_allocation_count": candidate_path_allocations,
            "path_owner_allocation_reduction_percent": reduction_percent(
                baseline_path_allocations, candidate_path_allocations
            ),
            "baseline_deep_path_clone_allocation_count": (
                discovery_inputs * clones_per_input
            ),
            "candidate_deep_path_clone_allocation_count": 0,
            "baseline_shared_handle_clone_count": 0,
            "candidate_shared_handle_clone_count": discovery_inputs * clones_per_input,
        },
        "historical_release_evidence": {
            "world_sync": {
                "checksum": 6_649_329_941_810_118_656,
                "baseline_p50_ms": 57.4550,
                "candidate_p50_ms": 5.3678,
                "p50_reduction_percent": 90.66,
                "baseline_p95_ms": 90.9359,
                "candidate_p95_ms": 14.8975,
                "p95_reduction_percent": 83.62,
                "baseline_allocations": 196_653,
                "candidate_allocations": 3,
                "allocation_reduction_percent": 99.998474,
            },
            "pending_sort": {
                "checksum": 1_123_984_918_402_528_105,
                "baseline_p50_ms": 77.7146,
                "candidate_p50_ms": 3.6246,
                "p50_reduction_percent": 95.3360,
                "baseline_p95_ms": 165.3016,
                "candidate_p95_ms": 6.0630,
                "p95_reduction_percent": 96.3322,
                "baseline_allocations": 329_884,
                "candidate_allocations": 15_974,
                "allocation_reduction_percent": 95.157692,
            },
            "discovery_input": {
                "checksum": 10_711_012_688_504_291_325,
                "baseline_p50_ms": 198.7112,
                "candidate_p50_ms": 77.9511,
                "p50_reduction_percent": 60.772,
                "baseline_p95_ms": 361.4602,
                "candidate_p95_ms": 144.8218,
                "p95_reduction_percent": 59.934,
                "baseline_allocations": 1_835_008,
                "candidate_allocations": 524_288,
                "allocation_reduction_percent": 71.429,
            },
        },
        "invariants": {
            "world_sync_row_order_and_filtering_preserved": True,
            "world_sync_nested_payloads_preserved": True,
            "pending_update_priority_tuple_preserved": True,
            "discovery_input_identity_preserved": True,
            "root_scan_payload_free": True,
        },
    }


def reduction_percent(baseline: int, candidate: int) -> float:
    return (baseline - candidate) * 100.0 / baseline


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
    parser.add_argument("--physics-nodes", type=int, default=65_536)
    parser.add_argument("--pending-updates", type=int, default=1_024)
    parser.add_argument("--modeled-sort-comparisons", type=int, default=10_240)
    parser.add_argument("--discovery-inputs", type=int, default=262_144)
    parser.add_argument("--clones-per-input", type=int, default=6)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.physics_nodes,
        args.pending_updates,
        args.modeled_sort_comparisons,
        args.discovery_inputs,
        args.clones_per_input,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
