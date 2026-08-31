"""Model map work removed from retained runtime UI style-delta classification."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    node_update_count: int = 4096,
    attributes_per_map: int = 256,
    changed_existing_key_count: int = 2,
    early_domain_resolution_rank: int = 2,
) -> dict[str, object]:
    if node_update_count <= 0:
        raise ValueError("node_update_count must be positive")
    if attributes_per_map <= 0:
        raise ValueError("attributes_per_map must be positive")
    if not 0 <= changed_existing_key_count <= attributes_per_map:
        raise ValueError("changed_existing_key_count must fit within the attribute map")
    if not 0 < early_domain_resolution_rank <= attributes_per_map:
        raise ValueError("early_domain_resolution_rank must fit within the attribute map")

    chained_key_visits = node_update_count * attributes_per_map * 2
    btree_get_calls = chained_key_visits * 2
    temporary_changed_key_clones = (
        node_update_count * changed_existing_key_count * 2
    )
    retired_decision_work = chained_key_visits + btree_get_calls

    full_merge_key_comparisons = node_update_count * attributes_per_map
    full_merge_value_comparisons = node_update_count * attributes_per_map
    full_merge_decision_work = full_merge_key_comparisons + full_merge_value_comparisons

    early_merge_key_comparisons = node_update_count * early_domain_resolution_rank
    early_merge_value_comparisons = node_update_count * early_domain_resolution_rank
    early_merge_decision_work = early_merge_key_comparisons + early_merge_value_comparisons

    return {
        "schema": "zircon.runtime.ui_style_delta_merge_pressure.v1",
        "interpretation": {
            "work_unit": "one explicit source-level key visit, map get call, key comparison, or value comparison",
            "excluded": "BTreeMap internal comparisons, allocator latency, CPU time, cache effects, Value clone work, and style recomputation outside delta classification",
            "early_exit_case": "the first text-affecting and non-render-only changed keys are both known by early_domain_resolution_rank",
        },
        "inputs": {
            "node_update_count": node_update_count,
            "attributes_per_map": attributes_per_map,
            "changed_existing_key_count": changed_existing_key_count,
            "early_domain_resolution_rank": early_domain_resolution_rank,
        },
        "retired_chain_filter": {
            "chained_key_visits": chained_key_visits,
            "btree_get_calls": btree_get_calls,
            "temporary_changed_key_clones": temporary_changed_key_clones,
            "decision_work_units": retired_decision_work,
        },
        "ordered_merge_full_scan": {
            "key_comparisons": full_merge_key_comparisons,
            "value_comparisons": full_merge_value_comparisons,
            "temporary_changed_key_clones": 0,
            "decision_work_units": full_merge_decision_work,
            "decision_work_reduction_ratio": retired_decision_work
            / full_merge_decision_work,
        },
        "ordered_merge_early_exit": {
            "key_comparisons": early_merge_key_comparisons,
            "value_comparisons": early_merge_value_comparisons,
            "temporary_changed_key_clones": 0,
            "decision_work_units": early_merge_decision_work,
            "decision_work_reduction_ratio": retired_decision_work
            / early_merge_decision_work,
        },
        "delta": {
            "avoided_temporary_changed_key_clones": temporary_changed_key_clones,
            "full_scan_avoided_decision_work_units": retired_decision_work
            - full_merge_decision_work,
            "early_exit_avoided_decision_work_units": retired_decision_work
            - early_merge_decision_work,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-update-count", type=int, default=4096)
    parser.add_argument("--attributes-per-map", type=int, default=256)
    parser.add_argument("--changed-existing-key-count", type=int, default=2)
    parser.add_argument("--early-domain-resolution-rank", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        node_update_count=args.node_update_count,
        attributes_per_map=args.attributes_per_map,
        changed_existing_key_count=args.changed_existing_key_count,
        early_domain_resolution_rank=args.early_domain_resolution_rank,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
