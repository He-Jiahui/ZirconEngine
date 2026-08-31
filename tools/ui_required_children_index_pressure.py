"""Deterministic pressure model for incremental required-child selection.

This models the relation queried by UiLayoutPassEngineContext without importing the
Rust workspace, so it remains usable while the managed Cargo lane is unavailable.
It verifies that the parent-indexed path preserves the selected child lists and
quantifies the eliminated parent-by-required scan work.
"""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path


def run(parent_count: int, required_count: int) -> dict[str, object]:
    required_parents = [index % parent_count for index in range(required_count)]

    old_started = time.perf_counter()
    old_by_parent = {
        parent: [
            node_id
            for node_id, required_parent in enumerate(required_parents)
            if required_parent == parent
        ]
        for parent in range(parent_count)
    }
    old_elapsed = time.perf_counter() - old_started

    indexed_started = time.perf_counter()
    indexed_by_parent: dict[int, list[int]] = {}
    for node_id, parent in enumerate(required_parents):
        indexed_by_parent.setdefault(parent, []).append(node_id)
    new_elapsed = time.perf_counter() - indexed_started

    assert all(
        old_by_parent[parent] == indexed_by_parent.get(parent, [])
        for parent in range(parent_count)
    )

    old_checks = parent_count * required_count
    indexed_checks = required_count
    return {
        "parent_count": parent_count,
        "required_count": required_count,
        "old_scan_checks": old_checks,
        "indexed_build_checks": indexed_checks,
        "eliminated_scan_checks": old_checks - indexed_checks,
        "scan_reduction_ratio": old_checks / indexed_checks,
        "old_model_seconds": old_elapsed,
        "indexed_model_seconds": new_elapsed,
        "semantic_lists_match": True,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--parents", type=int, default=4096)
    parser.add_argument("--required", type=int, default=16384)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.parents, args.required)
    encoded = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
