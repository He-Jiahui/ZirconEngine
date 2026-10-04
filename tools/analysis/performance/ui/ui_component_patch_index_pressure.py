"""Deterministic pressure model for template component patch projection.

The old projection path searched the retained tree once per patch. The indexed
path groups patches by control id, then visits the retained tree once in
preorder. This model checks first-match semantics and reports the eliminated
tree comparisons without importing the Rust workspace.
"""

from __future__ import annotations

import argparse
import json
import time
from collections import defaultdict
from pathlib import Path


def run(node_count: int, patch_count: int) -> dict[str, object]:
    node_controls = [f"control-{index}" for index in range(node_count)]
    patches = [
        f"control-{node_count - 1 - (index % 7)}" if index % 11 else "missing-control"
        for index in range(patch_count)
    ]

    old_started = time.perf_counter()
    old_matches: list[int | None] = []
    old_checks = 0
    for control_id in patches:
        match_index = None
        for index, node_control in enumerate(node_controls):
            old_checks += 1
            if node_control == control_id:
                match_index = index
                break
        old_matches.append(match_index)
    old_elapsed = time.perf_counter() - old_started

    indexed_started = time.perf_counter()
    patches_by_control: dict[str, list[int]] = defaultdict(list)
    for patch_index, control_id in enumerate(patches):
        patches_by_control[control_id].append(patch_index)
    new_matches: dict[int, list[int]] = {}
    for index, control_id in enumerate(node_controls):
        patch_indexes = patches_by_control.pop(control_id, [])
        if patch_indexes:
            new_matches[index] = patch_indexes
    new_elapsed = time.perf_counter() - indexed_started

    indexed_match_list = [
        next((index for index, patch_indexes in new_matches.items() if patch_index in patch_indexes), None)
        for patch_index in range(patch_count)
    ]
    assert old_matches == indexed_match_list

    indexed_checks = node_count + patch_count
    return {
        "node_count": node_count,
        "patch_count": patch_count,
        "old_scan_checks": old_checks,
        "indexed_checks": indexed_checks,
        "eliminated_scan_checks": old_checks - indexed_checks,
        "scan_reduction_ratio": old_checks / indexed_checks,
        "old_model_seconds": old_elapsed,
        "indexed_model_seconds": new_elapsed,
        "first_match_semantics_match": True,
        "unmatched_patch_count": sum(match is None for match in old_matches),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--nodes", type=int, default=16384)
    parser.add_argument("--patches", type=int, default=4096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.nodes, args.patches)
    encoded = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
