"""Model the ordered-input fast path for retained UI batch planning."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(element_count: int = 32_768, ordered_frame_count: int = 4096) -> dict[str, object]:
    if element_count <= 0:
        raise ValueError("element_count must be positive")
    if ordered_frame_count <= 0:
        raise ValueError("ordered_frame_count must be positive")

    ordered_inputs = ordered_frame_count * element_count
    return {
        "schema": "zircon.runtime.ui_batch_plan_order_pressure.v1",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "element_count": element_count,
            "ordered_frame_count": ordered_frame_count,
        },
        "legacy_unconditional_sort": {
            "ordered_input_visits": ordered_inputs,
            "sort_invocations": ordered_frame_count,
            "sort_complexity": "O(N log N)",
        },
        "ordered_input_fast_path": {
            "ordered_input_visits": ordered_inputs,
            "sort_invocations": 0,
            "sort_complexity": "O(N) validation",
        },
        "delta": {
            "avoided_sort_invocations": ordered_frame_count,
            "ordered_input_visit_change": 0,
        },
        "excluded_from_model": [
            "unordered frame sorting",
            "batch-key construction",
            "allocator latency and RSS",
            "CPU, GPU, and frame timing",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--element-count", type=int, default=32_768)
    parser.add_argument("--ordered-frame-count", type=int, default=4096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.element_count, args.ordered_frame_count)
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
