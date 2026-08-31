#!/usr/bin/env python3
"""Model global UiSlot scans against parent-owned edge authority.

This is a deterministic worst-case access-count model, not measured product timing.
"""

import argparse
import hashlib
import json
from pathlib import Path, PureWindowsPath
import subprocess


SOURCE_PATHS = (
    "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
    "zircon_runtime/src/ui/layout/pass/slot.rs",
    "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/Children.h",
)


def pressure_report(
    unslotted_children: int,
    global_slots: int,
    changed_children: int,
    changed_parents: int,
) -> dict[str, object]:
    values = {
        "unslotted_children": unslotted_children,
        "global_slots": global_slots,
        "changed_children": changed_children,
        "changed_parents": changed_parents,
    }
    for name, value in values.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if changed_children > unslotted_children:
        raise ValueError("changed_children must not exceed unslotted_children")

    missing_edge_global_slot_visits = unslotted_children * global_slots
    full_edge_projection_visits = unslotted_children + global_slots
    rejected_full_units = (
        full_edge_projection_visits
        + unslotted_children
        + missing_edge_global_slot_visits
    )
    achieved_full_units = full_edge_projection_visits + unslotted_children

    local_parent_child_visits = changed_parents * unslotted_children
    local_missing_edge_global_slot_visits = (
        local_parent_child_visits * global_slots
    )
    rejected_local_dependency_units = (
        local_parent_child_visits + local_missing_edge_global_slot_visits
    )
    achieved_exact_dependency_units = changed_children
    parent_topology_rebuild_units = local_parent_child_visits

    rejected_local_order_units = (
        global_slots
        + local_parent_child_visits
        + local_missing_edge_global_slot_visits
    )
    achieved_local_order_units = local_parent_child_visits

    return {
        "schema": "zircon.runtime.ui_layout_slot_index_pressure.v3",
        "evidence_kind": "deterministic_worst_case_access_model",
        "is_product_timing": False,
        "inputs": values,
        "modeled_case": (
            "children have no matching container slot and every global slot belongs "
            "to another edge"
        ),
        "rejected_workspace_scan_full_index_build": {
            "edge_projection_visits": full_edge_projection_visits,
            "parent_child_visits": unslotted_children,
            "missing_edge_global_slot_visits": missing_edge_global_slot_visits,
            "operation_units": rejected_full_units,
        },
        "achieved_tree_edge_full_index_build": {
            "edge_projection_visits": full_edge_projection_visits,
            "parent_child_visits": unslotted_children,
            "missing_edge_global_slot_visits": 0,
            "operation_units": achieved_full_units,
        },
        "rejected_workspace_scan_local_dependency_patch": {
            "parent_child_visits": local_parent_child_visits,
            "missing_edge_global_slot_visits": (
                local_missing_edge_global_slot_visits
            ),
            "operation_units": rejected_local_dependency_units,
        },
        "achieved_exact_child_dependency_patch": {
            "changed_child_visits": changed_children,
            "missing_edge_global_slot_visits": 0,
            "operation_units": achieved_exact_dependency_units,
        },
        "achieved_parent_topology_dependency_rebuild": {
            "parent_child_visits": local_parent_child_visits,
            "missing_edge_global_slot_visits": 0,
            "operation_units": parent_topology_rebuild_units,
        },
        "rejected_workspace_scan_local_parent_order_patch": {
            "workspace_wide_slot_visits": global_slots,
            "parent_child_visits": local_parent_child_visits,
            "missing_edge_global_slot_visits": (
                local_missing_edge_global_slot_visits
            ),
            "operation_units": rejected_local_order_units,
        },
        "achieved_parent_local_order_patch": {
            "workspace_wide_slot_visits": 0,
            "parent_child_visits": local_parent_child_visits,
            "missing_edge_global_slot_visits": 0,
            "operation_units": achieved_local_order_units,
        },
        "comparison": {
            "rejected_to_achieved_full_build_reduction_ratio": (
                rejected_full_units / achieved_full_units
            ),
            "rejected_to_achieved_local_dependency_reduction_ratio": (
                rejected_local_dependency_units / achieved_exact_dependency_units
            ),
            "parent_topology_to_exact_child_dependency_ratio": (
                parent_topology_rebuild_units / achieved_exact_dependency_units
            ),
            "rejected_to_achieved_local_parent_order_reduction_ratio": (
                rejected_local_order_units / achieved_local_order_units
            ),
        },
        "achieved_contract": [
            "the serialized flat slot carrier is private behind UiTree mutation APIs",
            "UiTree owns the authoritative edge-to-slot lookup",
            "missing slot lookup is authoritative and never scans the workspace slot table",
            "a child property change patches only that child's dependency membership",
            "parent topology or container changes use an explicit parent-local rebuild",
            "parent order changes visit only that parent's edge collection",
        ],
        "remaining_target_contract": [
            "child identity and layout slot share one parent-owned edge record",
            "runtime mutation records exact changed edges and parent identities",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and latency timing",
            "BTree lookup and child-order sort constants",
            "serialization migration and compatibility adapter cost",
            "layout measurement, arrangement, hit-test, paint, and GPU work",
            "matching-slot early exits and average-case slot distribution",
        ],
    }


def source_binding(repo_root: Path) -> dict[str, object]:
    root = repo_root.resolve()
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    files = []
    for relative_path in SOURCE_PATHS:
        path = root / relative_path
        files.append(
            {
                "path": relative_path,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest().upper(),
            }
        )
    return {"workspace_head": head, "files": files}


def pressure_suite(global_slots: int, repo_root: Path | None = None) -> dict[str, object]:
    root = repo_root or Path(__file__).resolve().parents[1]
    return {
        "schema": "zircon.runtime.ui_layout_slot_index_pressure_suite.v3",
        "evidence_kind": "deterministic_worst_case_access_model",
        "is_product_timing": False,
        "source_binding": source_binding(root),
        "scenarios": {
            str(child_count): pressure_report(child_count, global_slots, 1, 1)
            for child_count in (64, 1_000, 10_000)
        },
    }


def validate_output_path(output: str) -> Path:
    path = Path(output).resolve()
    if PureWindowsPath(str(path)).drive.upper() == "C:":
        raise ValueError("performance artifacts must not be written to the C drive")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--global-slots", type=int, default=10_000)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_suite(args.global_slots)
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        output_path = validate_output_path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8", newline="\n")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
