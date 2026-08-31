"""Deterministic copy-work model for event-lifetime UI dispatch routes."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_ui_dispatch_route_sharing_pressure.py",
    "zircon_runtime_interface/src/ui/dispatch/pointer/context.rs",
    "zircon_runtime_interface/src/ui/dispatch/navigation/context.rs",
    "zircon_runtime_interface/src/ui/surface/hit.rs",
    "zircon_runtime_interface/src/ui/surface/pointer/route.rs",
    "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs",
    "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs",
    "zircon_runtime/src/ui/dispatch/visited_node_set.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
)
BASELINE_GIT_REVISION = "5ffc4945095a6fc734bcbb2e632958026350b760"
HEAD_BASELINE_SHA256 = {
    "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs": (
        "C07AA16A9DA7B17C8EF74A9E3D196269F8E0E6C0AF7C5117BA1A8B8B7782D9BD"
    ),
    "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs": (
        "2AEEDA52EB071BB37B8765C4C48BD49B5B1E2243CF01FA14411789DB3A7F6D87"
    ),
    "zircon_runtime_interface/src/ui/dispatch/pointer/context.rs": (
        "B44C32614D94B2225EFC862045EE1922EB4A39C4DB04313D6BD3CDD13173F341"
    ),
    "zircon_runtime_interface/src/ui/dispatch/navigation/context.rs": (
        "9A130CEF2DFD9228BD3B67328497C43F227AE6487C44F38DA40550F2DBDF7066"
    ),
    "zircon_runtime_interface/src/ui/surface/hit.rs": (
        "99425457648D8A2DF8AEA5D2FFE0313B5770C6A79424E3B70665C007A2B10B87"
    ),
    "zircon_runtime_interface/src/ui/surface/pointer/route.rs": (
        "7BC036C1EA8050A380E0E5BA8802C27B376FD49B73479BEAD7BC514CE0D31514"
    ),
}


def run(
    event_count: int,
    route_depths: tuple[int, ...] = (1, 10, 100),
    handler_counts: tuple[int, ...] = (1, 4),
    stacked_candidate_count: int = 4,
    hover_transition_count: int = 2,
    node_identity_bytes: int = 8,
    handler_bearing_node_phase_count: int = 1,
    inline_visited_capacity: int = 16,
) -> dict[str, object]:
    if event_count <= 0:
        raise ValueError("event_count must be positive")
    if not route_depths or any(depth <= 0 for depth in route_depths):
        raise ValueError("route_depths must contain positive values")
    if not handler_counts or any(count <= 0 for count in handler_counts):
        raise ValueError("handler_counts must contain positive values")
    for name, value in (
        ("stacked_candidate_count", stacked_candidate_count),
        ("hover_transition_count", hover_transition_count),
    ):
        if value < 0:
            raise ValueError(f"{name} must be non-negative")
    if node_identity_bytes <= 0:
        raise ValueError("node_identity_bytes must be positive")
    if handler_bearing_node_phase_count <= 0:
        raise ValueError("handler_bearing_node_phase_count must be positive")
    if inline_visited_capacity <= 0:
        raise ValueError("inline_visited_capacity must be positive")

    cases = []
    for route_depth in route_depths:
        # PointerRoute retains the hit root-to-leaf path, hit bubble path, dispatch
        # bubble path, stacked candidates, and entered/left transition identities.
        pointer_identity_slots = (
            route_depth * 3 + stacked_candidate_count + hover_transition_count
        )
        pointer_non_empty_vectors = (
            3
            + int(stacked_candidate_count > 0)
            + int(hover_transition_count > 0)
        )
        navigation_identity_slots = route_depth
        pointer_candidate_identity_count = max(stacked_candidate_count, 1)
        pointer_shared_ancestry_unique_node_count = route_depth + stacked_candidate_count
        pointer_disjoint_ancestry_unique_node_upper_bound = (
            route_depth * max(stacked_candidate_count, 1)
        )
        navigation_unique_node_count = route_depth
        for handler_count in handler_counts:
            head_pointer_route_clones = event_count * (
                1 + handler_bearing_node_phase_count
            )
            head_navigation_route_clones = event_count * (
                1 + handler_bearing_node_phase_count
            )
            head_pointer_route_identity_copies = (
                head_pointer_route_clones * pointer_identity_slots
            )
            head_pointer_candidate_identity_copies = (
                event_count * pointer_candidate_identity_count
            )
            head_pointer_total_identity_copies = (
                head_pointer_route_identity_copies
                + head_pointer_candidate_identity_copies
            )
            head_navigation_route_identity_copies = (
                head_navigation_route_clones * navigation_identity_slots
            )
            head_navigation_candidate_identity_copies = (
                event_count * navigation_identity_slots
            )
            head_navigation_total_identity_copies = (
                head_navigation_route_identity_copies
                + head_navigation_candidate_identity_copies
            )
            visited_node_insert_count = event_count * route_depth
            cases.append(
                {
                    "route_depth": route_depth,
                    "handler_count": handler_count,
                    "handler_topology": {
                        "callbacks_per_node_phase": handler_count,
                        "handler_bearing_node_phase_count": (
                            handler_bearing_node_phase_count
                        ),
                    },
                    "head_pointer_route_clone_count": head_pointer_route_clones,
                    "candidate_pointer_route_clone_count": 0,
                    "head_pointer_route_identity_copies": (
                        head_pointer_route_identity_copies
                    ),
                    "head_pointer_candidate_identity_copies": (
                        head_pointer_candidate_identity_copies
                    ),
                    "head_pointer_total_identity_copies": (
                        head_pointer_total_identity_copies
                    ),
                    "candidate_pointer_total_identity_copies": 0,
                    "head_pointer_vector_allocations_lower_bound": (
                        head_pointer_route_clones * pointer_non_empty_vectors
                        + event_count
                    ),
                    "candidate_pointer_vector_allocations_lower_bound": 0,
                    "head_pointer_candidate_vector_copy_count": event_count,
                    "candidate_pointer_candidate_vector_copy_count": 0,
                    "head_pointer_visited_heap_allocation_count": event_count,
                    "candidate_pointer_visited_heap_allocation_count": (
                        event_count
                        if pointer_shared_ancestry_unique_node_count
                        > inline_visited_capacity
                        else 0
                    ),
                    "candidate_pointer_disjoint_ancestry_heap_allocation_upper_bound": (
                        event_count
                        if pointer_disjoint_ancestry_unique_node_upper_bound
                        > inline_visited_capacity
                        else 0
                    ),
                    "pointer_shared_ancestry_unique_node_count": (
                        pointer_shared_ancestry_unique_node_count
                    ),
                    "pointer_disjoint_ancestry_unique_node_upper_bound": (
                        pointer_disjoint_ancestry_unique_node_upper_bound
                    ),
                    "head_pointer_payload_bytes_lower_bound": (
                        head_pointer_total_identity_copies * node_identity_bytes
                    ),
                    "candidate_pointer_payload_bytes_lower_bound": 0,
                    "head_navigation_route_clone_count": (
                        head_navigation_route_clones
                    ),
                    "candidate_navigation_route_clone_count": 0,
                    "head_navigation_route_identity_copies": (
                        head_navigation_route_identity_copies
                    ),
                    "head_navigation_candidate_identity_copies": (
                        head_navigation_candidate_identity_copies
                    ),
                    "head_navigation_total_identity_copies": (
                        head_navigation_total_identity_copies
                    ),
                    "candidate_navigation_total_identity_copies": 0,
                    "head_navigation_vector_allocations_lower_bound": (
                        head_navigation_route_clones + event_count
                    ),
                    "candidate_navigation_vector_allocations_lower_bound": 0,
                    "head_navigation_candidate_vector_copy_count": event_count,
                    "candidate_navigation_candidate_vector_copy_count": 0,
                    "head_navigation_visited_heap_allocation_count": event_count,
                    "candidate_navigation_visited_heap_allocation_count": (
                        event_count
                        if navigation_unique_node_count > inline_visited_capacity
                        else 0
                    ),
                    "navigation_unique_node_count": navigation_unique_node_count,
                    "head_navigation_payload_bytes_lower_bound": (
                        head_navigation_total_identity_copies * node_identity_bytes
                    ),
                    "candidate_navigation_payload_bytes_lower_bound": 0,
                    "head_visited_node_insert_count": visited_node_insert_count,
                    "candidate_visited_node_insert_count": visited_node_insert_count,
                    "head_visited_set_initialization_count": event_count,
                    "candidate_visited_set_initialization_count": event_count,
                }
            )

    return {
        "schema": "zircon.runtime.ui_dispatch_route_sharing_pressure.v3",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_copy_work_lower_bound",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "route deep-clone count, copied node identities, minimum node payload "
                "bytes, clone-induced Vec allocations, candidate vector copies, "
                "visited-set heap allocations, and retained visited-node insert work"
            ),
            "excluded": (
                "allocator metadata, Vec capacities, route scalar fields, callback CPU, "
                "cache locality, RSS, and product input latency"
            ),
            "pointer_fixture": (
                "the primary visited-allocation fixture assumes stacked candidates share "
                "the target ancestry and each adds one unique leaf; the disjoint-ancestry "
                "field separately reports the conservative heap-fallback upper bound"
            ),
            "dynamic_acceptance_pending": (
                "managed Rust clone/allocation counters, dispatch CPU p95, and product "
                "capture/preview/direct/bubble/passthrough traces"
            ),
        },
        "inputs": {
            "event_count": event_count,
            "route_depths": list(route_depths),
            "handler_counts": list(handler_counts),
            "stacked_candidate_count": stacked_candidate_count,
            "hover_transition_count": hover_transition_count,
            "node_identity_bytes": node_identity_bytes,
            "handler_bearing_node_phase_count": handler_bearing_node_phase_count,
            "inline_visited_capacity": inline_visited_capacity,
        },
        "invariants": {
            "one_owned_route_per_result": True,
            "dispatch_context_borrows_route": True,
            "handler_count_changes_route_clone_bytes": False,
            "typical_route_visited_set_is_inline": True,
        },
        "cases": cases,
    }


def source_binding() -> dict[str, object]:
    source_sha256 = {
        relative_path: hashlib.sha256((ROOT / relative_path).read_bytes())
        .hexdigest()
        .upper()
        for relative_path in CRITICAL_SOURCE_FILES
    }
    head_source_sha256 = dict(HEAD_BASELINE_SHA256)
    manifest_lines = [
        f"worktree:{path}:{source_sha256[path]}" for path in sorted(source_sha256)
    ]
    manifest_lines.extend(
        f"head:{path}:{head_source_sha256[path]}"
        for path in sorted(head_source_sha256)
    )
    manifest_payload = "\n".join(manifest_lines).encode("utf-8")
    git_revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    candidate_contract = validate_candidate_source_contract()
    return {
        "git_revision": git_revision,
        "baseline_git_revision": BASELINE_GIT_REVISION,
        "critical_source_files": list(CRITICAL_SOURCE_FILES),
        "source_sha256": source_sha256,
        "head_baseline_files": list(HEAD_BASELINE_SHA256),
        "head_source_sha256": head_source_sha256,
        "source_manifest_sha256": hashlib.sha256(manifest_payload)
        .hexdigest()
        .upper(),
        "candidate_contract": candidate_contract,
    }


def validate_candidate_source_contract() -> dict[str, object]:
    pointer = (ROOT / "zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs").read_text(
        encoding="utf-8"
    )
    navigation = (
        ROOT / "zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs"
    ).read_text(encoding="utf-8")
    visited = (ROOT / "zircon_runtime/src/ui/dispatch/visited_node_set.rs").read_text(
        encoding="utf-8"
    )
    blockers: list[dict[str, str]] = []
    for relative_path, source in (
        ("zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs", pointer),
        ("zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs", navigation),
    ):
        if "UiDispatchVisitedNodeSet::with_expected_len" not in source:
            blockers.append(
                {
                    "code": "inline_visited_set_not_wired",
                    "relative_path": relative_path,
                }
            )
        if "HashSet::with_capacity" in source:
            blockers.append(
                {
                    "code": "dispatcher_direct_hashset_allocation_present",
                    "relative_path": relative_path,
                }
            )
    for anchor in (
        "UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY: usize = 16",
        "HashSet::with_capacity",
        "overflow.extend(self.inline.iter().copied())",
    ):
        if anchor not in visited:
            blockers.append(
                {
                    "code": "visited_set_fallback_contract_missing",
                    "relative_path": (
                        "zircon_runtime/src/ui/dispatch/visited_node_set.rs"
                    ),
                    "anchor": anchor,
                }
            )
    return {"ready": not blockers, "blockers": blockers}


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written to D:, E:, or F:")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-count", type=int, default=1_000_000)
    parser.add_argument("--route-depths", type=int, nargs="+", default=(1, 10, 100))
    parser.add_argument("--handler-counts", type=int, nargs="+", default=(1, 4))
    parser.add_argument("--stacked-candidate-count", type=int, default=4)
    parser.add_argument("--hover-transition-count", type=int, default=2)
    parser.add_argument("--node-identity-bytes", type=int, default=8)
    parser.add_argument("--handler-bearing-node-phase-count", type=int, default=1)
    parser.add_argument("--inline-visited-capacity", type=int, default=16)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.event_count,
        tuple(args.route_depths),
        tuple(args.handler_counts),
        args.stacked_candidate_count,
        args.hover_transition_count,
        args.node_identity_bytes,
        args.handler_bearing_node_phase_count,
        args.inline_visited_capacity,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        validate_output_path(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
