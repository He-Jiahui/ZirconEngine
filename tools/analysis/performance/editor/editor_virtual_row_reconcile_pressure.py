"""Model retained-tree scans in editor template virtual-row reconciliation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    reconcile_count: int = 4096,
    surface_node_count: int = 16384,
    physical_slot_capacity: int = 41,
    changed_slot_count_per_scroll: int = 1,
) -> dict[str, object]:
    if reconcile_count <= 0:
        raise ValueError("reconcile_count must be positive")
    if surface_node_count <= 0:
        raise ValueError("surface_node_count must be positive")
    if physical_slot_capacity <= 0:
        raise ValueError("physical_slot_capacity must be positive")
    if not 0 <= changed_slot_count_per_scroll <= physical_slot_capacity:
        raise ValueError(
            "changed_slot_count_per_scroll must be between zero and physical_slot_capacity"
        )

    retired_shared_inventory_passes = 2
    retired_shared_inventory_node_visits = (
        reconcile_count
        * surface_node_count
        * retired_shared_inventory_passes
    )
    physical_slot_binding_visits = reconcile_count * physical_slot_capacity
    changed_slot_metadata_rebinds = reconcile_count * changed_slot_count_per_scroll
    avoided_unchanged_slot_metadata_rebinds = (
        physical_slot_binding_visits - changed_slot_metadata_rebinds
    )
    retained_tree_to_slot_visit_ratio = (
        retired_shared_inventory_node_visits / physical_slot_binding_visits
    )

    return {
        "schema": "zircon.editor.virtual_row_reconcile_pressure.v3",
        "interpretation": {
            "scenario": "steady-state reconciliation of a virtual list whose logical model can be much larger than its physical slot capacity",
            "retired_passes": "indexed parent lookup fallback plus the shared virtual-row inventory each scanned the retained tree",
            "prototype_slot_pool": "surface indexes resolve controls incrementally; reconciliation checks V physical slots and metadata binding touches only changed slots",
            "excluded": "initial prototype capture, capacity-change subtree creation, renderer cost, allocation latency, and CPU time",
        },
        "inputs": {
            "reconcile_count": reconcile_count,
            "surface_node_count": surface_node_count,
            "physical_slot_capacity": physical_slot_capacity,
            "changed_slot_count_per_scroll": changed_slot_count_per_scroll,
        },
        "retired_shared_inventory": {
            "full_tree_passes_per_reconcile": retired_shared_inventory_passes,
            "retained_tree_node_visits": retired_shared_inventory_node_visits,
        },
        "prototype_slot_pool": {
            "full_tree_passes_per_reconcile": 0,
            "retained_tree_node_visits": 0,
            "physical_slot_binding_visits": physical_slot_binding_visits,
            "changed_slot_metadata_rebinds": changed_slot_metadata_rebinds,
            "logical_growth_nodes_created": 0,
        },
        "delta": {
            "avoided_retained_tree_node_visits": retired_shared_inventory_node_visits,
            "avoided_unchanged_slot_metadata_rebinds": avoided_unchanged_slot_metadata_rebinds,
            "retained_tree_to_slot_visit_ratio": retained_tree_to_slot_visit_ratio,
            "logical_growth_nodes_created": 0,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--reconcile-count", type=int, default=4096)
    parser.add_argument("--surface-node-count", type=int, default=16384)
    parser.add_argument("--physical-slot-capacity", type=int, default=41)
    parser.add_argument("--changed-slot-count-per-scroll", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        reconcile_count=args.reconcile_count,
        surface_node_count=args.surface_node_count,
        physical_slot_capacity=args.physical_slot_capacity,
        changed_slot_count_per_scroll=args.changed_slot_count_per_scroll,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
