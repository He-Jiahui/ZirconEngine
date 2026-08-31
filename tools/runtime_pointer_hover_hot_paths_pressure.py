"""Deterministic work model for pointer-hover route retention and diffing."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_pointer_hover_hot_paths_pressure.py",
    "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
)
BASELINE_GIT_REVISION = "5ffc4945095a6fc734bcbb2e632958026350b760"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs": (
        "a042804a567e258151a66c6635abd2a52c20e0ba"
    ),
    "zircon_runtime/src/ui/surface/surface/event_routing.rs": (
        "9315b899d3f7cd79e2f6c0b2604e634f0332092b"
    ),
}


def run(
    event_count: int,
    route_depth: int = 512,
    small_route_depth: int = 8,
    node_identity_bytes: int = 8,
) -> dict[str, object]:
    if event_count <= 0:
        raise ValueError("event_count must be positive")
    if route_depth <= 0:
        raise ValueError("route_depth must be positive")
    if small_route_depth <= 0:
        raise ValueError("small_route_depth must be positive")
    if small_route_depth * small_route_depth > 64:
        raise ValueError("small_route_depth must remain within the linear budget")
    if route_depth * route_depth <= 64:
        raise ValueError("route_depth must exercise the indexed branch")
    if node_identity_bytes <= 0:
        raise ValueError("node_identity_bytes must be positive")

    legacy_hover_node_copies = event_count * route_depth
    legacy_large_diff_comparisons = event_count * 2 * route_depth * route_depth
    indexed_large_diff_operations = event_count * 4 * route_depth
    small_diff_comparisons = event_count * 2 * small_route_depth * small_route_depth

    return {
        "schema": "zircon.runtime.pointer_hover_hot_paths_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_operation_and_copy_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "stable-route clone, node-copy, Vec-allocation, comparison, and "
                "large-path membership-operation counts"
            ),
            "excluded": (
                "allocator metadata, hash cost, cache locality, output-vector work, "
                "branch prediction, RSS, and product input latency"
            ),
            "dynamic_acceptance_pending": (
                "managed release P50/P95 benchmark and current-source product input profile"
            ),
        },
        "inputs": {
            "event_count": event_count,
            "route_depth": route_depth,
            "small_route_depth": small_route_depth,
            "node_identity_bytes": node_identity_bytes,
            "hover_diff_linear_comparison_budget": 64,
            "workload": "stable retained route plus disjoint hover-diff routes",
        },
        "retained_hover_path": {
            "legacy_route_clone_count": event_count,
            "candidate_route_clone_count": 0,
            "legacy_node_copy_count": legacy_hover_node_copies,
            "candidate_node_copy_count": 0,
            "candidate_node_comparison_count": legacy_hover_node_copies,
            "legacy_vec_allocations_lower_bound": event_count,
            "candidate_vec_allocation_count": 0,
            "legacy_payload_bytes_lower_bound": (
                legacy_hover_node_copies * node_identity_bytes
            ),
            "candidate_payload_bytes_lower_bound": 0,
        },
        "hover_diff": {
            "small_path": {
                "route_depth": small_route_depth,
                "legacy_node_comparison_count": small_diff_comparisons,
                "candidate_node_comparison_count": small_diff_comparisons,
                "legacy_membership_allocations": 0,
                "candidate_membership_allocations": 0,
            },
            "large_path": {
                "route_depth": route_depth,
                "legacy_node_comparison_count": legacy_large_diff_comparisons,
                "candidate_membership_insert_count": event_count * 2 * route_depth,
                "candidate_membership_lookup_count": event_count * 2 * route_depth,
                "candidate_membership_operation_count": indexed_large_diff_operations,
                "legacy_membership_allocations": 0,
                "candidate_membership_allocations": event_count,
                "work_reduction_numerator": (
                    legacy_large_diff_comparisons - indexed_large_diff_operations
                ),
                "work_reduction_denominator": legacy_large_diff_comparisons,
                "work_reduction_percent": (
                    100.0
                    * (
                        legacy_large_diff_comparisons
                        - indexed_large_diff_operations
                    )
                    / legacy_large_diff_comparisons
                ),
            },
        },
        "invariants": {
            "stable_route_order_preserved": True,
            "small_hover_diff_adds_no_membership_allocation": True,
            "large_hover_diff_reuses_one_table_across_both_phases": True,
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
        "head_baseline_files": list(HEAD_BASELINE_GIT_BLOBS),
        "head_baseline_git_blobs": dict(HEAD_BASELINE_GIT_BLOBS),
        "source_manifest_sha256": hashlib.sha256(manifest_payload)
        .hexdigest()
        .upper(),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-count", type=int, default=1_000_000)
    parser.add_argument("--route-depth", type=int, default=512)
    parser.add_argument("--small-route-depth", type=int, default=8)
    parser.add_argument("--node-identity-bytes", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.event_count,
        args.route_depth,
        args.small_route_depth,
        args.node_identity_bytes,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
