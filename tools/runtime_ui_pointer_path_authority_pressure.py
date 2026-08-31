"""Deterministic ownership-work model for pointer hit and dispatch paths."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_ui_pointer_path_authority_pressure.py",
    "zircon_runtime_interface/src/ui/surface/hit.rs",
    "zircon_runtime_interface/src/ui/surface/pointer/route.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
    "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h",
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp",
)
HEAD_BASELINE_FILES = (
    "zircon_runtime_interface/src/ui/surface/hit.rs",
    "zircon_runtime_interface/src/ui/surface/pointer/route.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
)


def run(
    event_count: int,
    route_depths: tuple[int, ...] = (1, 16, 64, 100),
    node_identity_bytes: int = 8,
) -> dict[str, object]:
    if event_count <= 0:
        raise ValueError("event_count must be positive")
    if not route_depths or any(depth <= 0 for depth in route_depths):
        raise ValueError("route_depths must contain positive values")
    if node_identity_bytes <= 0:
        raise ValueError("node_identity_bytes must be positive")

    cases = []
    for route_depth in route_depths:
        ordinary_head_node_writes = event_count * route_depth * 3
        ordinary_candidate_node_writes = event_count * route_depth
        captured_head_node_writes = event_count * route_depth * 3
        captured_candidate_node_writes = event_count * route_depth * 2
        cases.append(
            {
                "route_depth": route_depth,
                "ordinary": {
                    "head_owned_path_sequences_per_event": 3,
                    "candidate_owned_path_sequences_per_event": 1,
                    "head_vec_allocations_lower_bound": event_count * 3,
                    "candidate_vec_allocations_lower_bound": event_count,
                    "head_node_identity_writes": ordinary_head_node_writes,
                    "candidate_node_identity_writes": ordinary_candidate_node_writes,
                    "head_payload_bytes_lower_bound": (
                        ordinary_head_node_writes * node_identity_bytes
                    ),
                    "candidate_payload_bytes_lower_bound": (
                        ordinary_candidate_node_writes * node_identity_bytes
                    ),
                },
                "captured_with_physical_hit": {
                    "head_owned_path_sequences_per_event": 3,
                    "candidate_owned_path_sequences_per_event": 2,
                    "head_vec_allocations_lower_bound": event_count * 3,
                    "candidate_vec_allocations_lower_bound": event_count * 2,
                    "head_node_identity_writes": captured_head_node_writes,
                    "candidate_node_identity_writes": captured_candidate_node_writes,
                    "head_payload_bytes_lower_bound": (
                        captured_head_node_writes * node_identity_bytes
                    ),
                    "candidate_payload_bytes_lower_bound": (
                        captured_candidate_node_writes * node_identity_bytes
                    ),
                },
                "retained_algorithmic_work": {
                    "hit_route_parent_visits": event_count * route_depth,
                    "captured_dispatch_parent_visits": event_count * route_depth,
                    "candidate_in_place_reverse_items_upper_bound": (
                        event_count * route_depth * 2
                    ),
                    "dispatch_handler_route_visits": "unchanged",
                },
            }
        )

    return {
        "schema": "zircon.runtime.ui_pointer_path_authority_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_path_ownership_work_lower_bound",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "owned path sequence count, Vec allocation lower bound, node identity "
                "writes, minimum payload bytes, retained parent visits, and in-place reversals"
            ),
            "excluded": (
                "allocator metadata, Vec spare capacity, cache locality, callback CPU, "
                "serialization, RSS, and product input latency"
            ),
            "dynamic_acceptance_pending": (
                "managed allocation counters, route-direction parity, capture/product traces, "
                "and pointer dispatch CPU p95"
            ),
        },
        "inputs": {
            "event_count": event_count,
            "route_depths": list(route_depths),
            "node_identity_bytes": node_identity_bytes,
        },
        "invariants": {
            "canonical_path_order": "root_to_leaf",
            "bubble_is_reverse_iteration": True,
            "ordinary_dispatch_reuses_hit_path": True,
            "captured_dispatch_preserves_physical_hit_path": True,
            "legacy_serde_path_arrays_preserved": True,
        },
        "cases": cases,
    }


def source_binding() -> dict[str, object]:
    worktree_sha256 = {
        path: hashlib.sha256((ROOT / path).read_bytes()).hexdigest().upper()
        for path in CRITICAL_SOURCE_FILES
    }
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    head_sha256 = {}
    for path in HEAD_BASELINE_FILES:
        content = subprocess.run(
            ["git", "show", f"HEAD:{path}"],
            cwd=ROOT,
            check=True,
            capture_output=True,
        ).stdout
        head_sha256[path] = hashlib.sha256(content).hexdigest().upper()
    manifest = [f"worktree:{path}:{digest}" for path, digest in worktree_sha256.items()]
    manifest.extend(f"head:{path}:{digest}" for path, digest in head_sha256.items())
    manifest_sha256 = hashlib.sha256("\n".join(sorted(manifest)).encode()).hexdigest().upper()
    return {
        "git_revision": revision,
        "worktree_sha256": worktree_sha256,
        "head_baseline_sha256": head_sha256,
        "manifest_sha256": manifest_sha256,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-count", type=int, default=1_000_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = run(args.event_count)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")


if __name__ == "__main__":
    main()
