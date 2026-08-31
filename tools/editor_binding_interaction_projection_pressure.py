"""Model UI Asset binding interactions before and after selected-authority queries."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def model_pressure(
    interactions_per_family: int = 4096,
    binding_count: int = 256,
    payload_entry_count: int = 128,
    schema_item_count: int = 256,
    event_option_count: int = 16,
    action_kind_count: int = 3,
    route_suggestion_count: int = 4,
    action_suggestion_count: int = 3,
    payload_suggestion_count: int = 8,
) -> dict[str, object]:
    inputs = {
        "interactions_per_family": max(interactions_per_family, 0),
        "binding_count": max(binding_count, 0),
        "payload_entry_count": max(payload_entry_count, 0),
        "schema_item_count": max(schema_item_count, 0),
        "event_option_count": max(event_option_count, 0),
        "action_kind_count": max(action_kind_count, 0),
        "route_suggestion_count": max(route_suggestion_count, 0),
        "action_suggestion_count": max(action_suggestion_count, 0),
        "payload_suggestion_count": max(payload_suggestion_count, 0),
    }
    full_projection_items = (
        inputs["binding_count"]
        + inputs["event_option_count"]
        + inputs["action_kind_count"]
        + inputs["route_suggestion_count"]
        + inputs["action_suggestion_count"]
        + 2 * inputs["payload_entry_count"]
        + inputs["payload_suggestion_count"]
        + inputs["schema_item_count"]
    )

    # Seven interaction families used to build all inspector fields. Suggestion actions then
    # generated their selected suggestion list a second time inside the mutation helper.
    retired_items_per_cycle = (
        7 * full_projection_items
        + inputs["route_suggestion_count"]
        + inputs["action_suggestion_count"]
        + inputs["payload_suggestion_count"]
    )
    selected_items_per_cycle = (
        3
        + inputs["route_suggestion_count"]
        + inputs["action_suggestion_count"]
        + inputs["payload_entry_count"]
        + inputs["payload_suggestion_count"]
    )
    interactions = inputs["interactions_per_family"]
    retired_items = retired_items_per_cycle * interactions
    selected_items = selected_items_per_cycle * interactions

    return {
        "schema": "zircon.editor.binding_interaction_projection_pressure.v1",
        "inputs": inputs,
        "retired": {
            "full_inspector_projections": 7 * interactions,
            "materialized_items": retired_items,
        },
        "selected_authority": {
            "full_inspector_projections": 0,
            "materialized_items": selected_items,
        },
        "delta": {
            "eliminated_full_inspector_projections": 7 * interactions,
            "eliminated_materialized_items": retired_items - selected_items,
            "work_reduction_ratio": (
                retired_items / selected_items if selected_items else 0.0
            ),
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--interactions-per-family", type=int, default=4096)
    parser.add_argument("--bindings", type=int, default=256)
    parser.add_argument("--payload-entries", type=int, default=128)
    parser.add_argument("--schema-items", type=int, default=256)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = model_pressure(
        interactions_per_family=args.interactions_per_family,
        binding_count=args.bindings,
        payload_entry_count=args.payload_entries,
        schema_item_count=args.schema_items,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    main()
