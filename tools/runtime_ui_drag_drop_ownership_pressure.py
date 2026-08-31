#!/usr/bin/env python3
"""Model drag-over ownership and optional diagnostics for the current candidate.

This is deterministic source-contract evidence, not compiled behavior, allocator telemetry,
RSS evidence, or product input timing.
"""

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMPLEMENTATION_SOURCES = (
    "zircon_runtime_interface/src/ui/dispatch/input/event.rs",
    "zircon_runtime_interface/src/ui/dispatch/input/effect.rs",
    "zircon_runtime_interface/src/ui/window/input/normalization.rs",
    "zircon_runtime/src/ui/surface/input/drag_drop.rs",
    "zircon_runtime/src/ui/surface/input/effect.rs",
    "zircon_runtime/src/ui/surface/input/effect/transaction.rs",
    "zircon_runtime/src/ui/surface/input/state/drag_drop.rs",
    "zircon_runtime/src/ui/surface/mutation_snapshot.rs",
)
REFERENCE_SOURCES = (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/DragAndDrop.h",
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp",
)


def source_bindings(paths: tuple[str, ...]) -> list[dict[str, object]]:
    bindings = []
    for relative_path in paths:
        payload = (ROOT / relative_path).read_bytes()
        bindings.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest().upper(),
            }
        )
    return bindings


def pressure_report(
    drag_update_events: int,
    target_transitions: int,
    payload_bytes: int,
    payload_string_fields: int,
    route_depth: int,
    focus_depth: int,
    popup_entries: int,
    surface_snapshot_entries: int,
) -> dict[str, object]:
    positive_inputs = {
        "drag_update_events": drag_update_events,
        "payload_bytes": payload_bytes,
        "payload_string_fields": payload_string_fields,
        "route_depth": route_depth,
        "surface_snapshot_entries": surface_snapshot_entries,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    for name, value in {
        "target_transitions": target_transitions,
        "focus_depth": focus_depth,
        "popup_entries": popup_entries,
    }.items():
        if value < 0:
            raise ValueError(f"{name} must be non-negative")
    if target_transitions > drag_update_events:
        raise ValueError("target_transitions cannot exceed drag_update_events")

    prior_payload_deep_clones = drag_update_events * 6
    prior_payload_string_allocations = prior_payload_deep_clones * payload_string_fields
    prior_payload_bytes_copied = prior_payload_deep_clones * payload_bytes
    candidate_arc_clones = drag_update_events * 3

    prior_surface_snapshots = drag_update_events
    candidate_surface_snapshots = target_transitions
    prior_snapshot_clone_units = prior_surface_snapshots * surface_snapshot_entries
    candidate_snapshot_clone_units = candidate_surface_snapshots * surface_snapshot_entries

    route_node_writes_per_projection = route_depth * 3 + focus_depth
    prior_route_node_writes = drag_update_events * route_node_writes_per_projection * 2
    full_route_node_writes = drag_update_events * route_node_writes_per_projection
    prior_route_step_writes = drag_update_events * (route_depth + 1)
    prior_popup_string_clones = drag_update_events * popup_entries * 2

    return {
        "evidence_kind": "deterministic_source_bound_pressure_model",
        "implementation_evidence": False,
        "implementation_source_contract": True,
        "is_product_timing": False,
        "source_binding": {
            "implementation": source_bindings(IMPLEMENTATION_SOURCES),
            "primary_reference": source_bindings(REFERENCE_SOURCES),
        },
        "reference_contract": {
            "payload_authority": (
                "Unreal FDragDropEvent retains a TSharedPtr<FDragDropOperation>, while Slate "
                "routes drag events by const reference and keeps one operation on the user"
            ),
            "inference": (
                "Zircon models the same immutable shared-operation ownership with Arc while "
                "preserving its existing serialized payload object"
            ),
        },
        "inputs": {
            "drag_update_events": drag_update_events,
            "target_transitions": target_transitions,
            "payload_bytes": payload_bytes,
            "payload_string_fields": payload_string_fields,
            "route_depth": route_depth,
            "focus_depth": focus_depth,
            "popup_entries": popup_entries,
            "surface_snapshot_entries": surface_snapshot_entries,
        },
        "prior_owned_payload_baseline": {
            "payload_deep_clones": prior_payload_deep_clones,
            "payload_string_allocations": prior_payload_string_allocations,
            "minimum_payload_bytes_copied": prior_payload_bytes_copied,
        },
        "candidate_shared_payload": {
            "payload_deep_clones": 0,
            "payload_string_allocations": 0,
            "minimum_payload_bytes_copied": 0,
            "arc_reference_clones": candidate_arc_clones,
        },
        "steady_target_transaction_snapshot": {
            "prior_full_surface_snapshots": prior_surface_snapshots,
            "candidate_full_surface_snapshots": candidate_surface_snapshots,
            "eliminated_full_surface_snapshots": (
                prior_surface_snapshots - candidate_surface_snapshots
            ),
            "prior_retained_entry_clone_units": prior_snapshot_clone_units,
            "candidate_retained_entry_clone_units": candidate_snapshot_clone_units,
            "eliminated_retained_entry_clone_units": (
                prior_snapshot_clone_units - candidate_snapshot_clone_units
            ),
        },
        "optional_diagnostics_projection": {
            "prior_route_node_writes": prior_route_node_writes,
            "candidate_summary_route_node_writes": 0,
            "candidate_full_route_node_writes": full_route_node_writes,
            "prior_route_step_writes": prior_route_step_writes,
            "candidate_summary_route_step_writes": 0,
            "prior_popup_string_clones": prior_popup_string_clones,
            "candidate_summary_popup_string_clones": 0,
        },
        "retained_work": [
            "one projected hit-grid lookup per drag update",
            "pointer/session ownership validation",
            "Arc reference-count updates for event, applied effect, and retained state",
            "full rollback snapshot when the drop target changes or lifecycle state transitions",
            "explicit Full diagnostics projection when requested",
        ],
        "excluded_from_model": [
            "CPU and allocator timing",
            "exact Rust type sizes and Arc atomic instruction cost",
            "tree/style entry byte sizes",
            "target hit-test and runtime-style invalidation cost",
            "managed Rust behavior tests and product drag latency",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--drag-update-events", type=int, default=100_000)
    parser.add_argument("--target-transitions", type=int, default=100)
    parser.add_argument("--payload-bytes", type=int, default=4096)
    parser.add_argument("--payload-string-fields", type=int, default=8)
    parser.add_argument("--route-depth", type=int, default=8)
    parser.add_argument("--focus-depth", type=int, default=6)
    parser.add_argument("--popup-entries", type=int, default=2)
    parser.add_argument("--surface-snapshot-entries", type=int, default=50_000)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(
        drag_update_events=args.drag_update_events,
        target_transitions=args.target_transitions,
        payload_bytes=args.payload_bytes,
        payload_string_fields=args.payload_string_fields,
        route_depth=args.route_depth,
        focus_depth=args.focus_depth,
        popup_entries=args.popup_entries,
        surface_snapshot_entries=args.surface_snapshot_entries,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
