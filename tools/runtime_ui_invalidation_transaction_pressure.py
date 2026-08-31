"""Deterministic clone/allocation model for UI invalidation transactions."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(change_count: int, transaction_count: int) -> dict[str, int | float | dict]:
    if change_count <= 0:
        raise ValueError("change_count must be positive")
    if transaction_count <= 0:
        raise ValueError("transaction_count must be positive")

    old_change_clones = change_count * transaction_count
    old_temporary_vectors = transaction_count
    return {
        "change_count": change_count,
        "transaction_count": transaction_count,
        "old_change_clones": old_change_clones,
        "new_change_clones": 0,
        "eliminated_change_clones": old_change_clones,
        "old_temporary_vectors": old_temporary_vectors,
        "new_temporary_vectors": 0,
        "interpretation": {
            "included": [
                "UiInvalidationChange clones before applying a transaction",
                "temporary change vectors",
            ],
            "excluded": [
                "tree validation",
                "dirty propagation",
                "invalidation map writes",
                "CPU timing",
                "allocator RSS",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--change-count", type=int, default=16_384)
    parser.add_argument("--transaction-count", type=int, default=120)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.change_count, args.transaction_count)
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
