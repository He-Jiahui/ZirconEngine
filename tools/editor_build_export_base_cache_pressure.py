import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    target_count: int,
    diagnostic_count: int,
    preset_path_count: int,
    stable_cache_hit_count: int,
    source_change_count: int = 1,
    wizard_view_model_payload_item_count: int = 2_048,
    stable_wizard_projection_count: int = 1_000,
    wizard_projection_node_count: int = 512,
    wizard_owned_payload_slots_per_node: int = 13,
) -> dict[str, Any]:
    values = (
        target_count,
        diagnostic_count,
        preset_path_count,
        stable_cache_hit_count,
        source_change_count,
        wizard_view_model_payload_item_count,
        stable_wizard_projection_count,
        wizard_projection_node_count,
        wizard_owned_payload_slots_per_node,
    )
    if any(value < 0 for value in values):
        raise ValueError("pressure inputs must be non-negative")

    payload_items_per_generation = target_count + diagnostic_count + preset_path_count
    old_payload_item_clone_count = payload_items_per_generation * stable_cache_hit_count
    new_arc_clone_count = stable_cache_hit_count
    ownership_operation_reduction_ratio = (
        old_payload_item_clone_count / new_arc_clone_count
        if new_arc_clone_count > 0
        else 0.0
    )
    metadata_probes_per_identity_check = preset_path_count + 2
    old_stable_hit_metadata_probe_count = (
        metadata_probes_per_identity_check * stable_cache_hit_count
    )
    new_stable_hit_metadata_probe_count = 0
    watcher_setup_count = 1
    watcher_setup_filesystem_probe_count = 2 + source_change_count
    old_wizard_view_model_item_clone_count = (
        wizard_view_model_payload_item_count * stable_wizard_projection_count
    )
    wizard_node_payload_slot_transfer_count = (
        wizard_projection_node_count
        * wizard_owned_payload_slots_per_node
        * stable_wizard_projection_count
    )

    return {
        "model_scope": (
            "top-level payload item clone and owned-slot move operations only; excludes "
            "nested string bytes, elapsed time, allocation size, filesystem metadata "
            "latency, and OS watcher cost"
        ),
        "target_count": target_count,
        "diagnostic_count": diagnostic_count,
        "preset_path_count": preset_path_count,
        "stable_cache_hit_count": stable_cache_hit_count,
        "source_change_count": source_change_count,
        "wizard_view_model_payload_item_count": wizard_view_model_payload_item_count,
        "stable_wizard_projection_count": stable_wizard_projection_count,
        "wizard_projection_node_count": wizard_projection_node_count,
        "wizard_owned_payload_slots_per_node": wizard_owned_payload_slots_per_node,
        "payload_items_per_generation": payload_items_per_generation,
        "old_payload_item_clone_count": old_payload_item_clone_count,
        "new_payload_item_clone_count": 0,
        "new_arc_clone_count": new_arc_clone_count,
        "payload_item_clone_avoidance_count": old_payload_item_clone_count,
        "ownership_operation_reduction_ratio": ownership_operation_reduction_ratio,
        "metadata_probes_per_identity_check": metadata_probes_per_identity_check,
        "old_stable_hit_metadata_probe_count": old_stable_hit_metadata_probe_count,
        "new_stable_hit_metadata_probe_count": new_stable_hit_metadata_probe_count,
        "stable_hit_metadata_probe_avoidance_count": old_stable_hit_metadata_probe_count,
        "watcher_epoch_load_count": stable_cache_hit_count,
        "watcher_setup_count": watcher_setup_count,
        "source_epoch_refresh_count": source_change_count,
        "watcher_setup_filesystem_probe_count": watcher_setup_filesystem_probe_count,
        "old_wizard_view_model_item_clone_count": old_wizard_view_model_item_clone_count,
        "new_wizard_view_model_item_clone_count": 0,
        "new_wizard_view_model_borrow_count": stable_wizard_projection_count,
        "wizard_view_model_item_clone_avoidance_count": (
            old_wizard_view_model_item_clone_count
        ),
        "old_wizard_node_payload_slot_clone_count": (
            wizard_node_payload_slot_transfer_count
        ),
        "new_wizard_node_payload_slot_clone_count": 0,
        "new_wizard_node_payload_slot_move_count": (
            wizard_node_payload_slot_transfer_count
        ),
    }


def write_result(output: Path, result: dict[str, Any]) -> None:
    if output.drive.casefold() == "c:":
        raise ValueError("profile artifacts must not be written to the C drive")
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target-count", type=int, default=1_000)
    parser.add_argument("--diagnostic-count", type=int, default=128)
    parser.add_argument("--preset-path-count", type=int, default=1_000)
    parser.add_argument("--stable-cache-hit-count", type=int, default=1_000)
    parser.add_argument("--source-change-count", type=int, default=1)
    parser.add_argument("--wizard-view-model-payload-item-count", type=int, default=2_048)
    parser.add_argument("--stable-wizard-projection-count", type=int, default=1_000)
    parser.add_argument("--wizard-projection-node-count", type=int, default=512)
    parser.add_argument("--wizard-owned-payload-slots-per-node", type=int, default=13)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        target_count=args.target_count,
        diagnostic_count=args.diagnostic_count,
        preset_path_count=args.preset_path_count,
        stable_cache_hit_count=args.stable_cache_hit_count,
        source_change_count=args.source_change_count,
        wizard_view_model_payload_item_count=args.wizard_view_model_payload_item_count,
        stable_wizard_projection_count=args.stable_wizard_projection_count,
        wizard_projection_node_count=args.wizard_projection_node_count,
        wizard_owned_payload_slots_per_node=args.wizard_owned_payload_slots_per_node,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
