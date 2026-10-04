"""Model the batch-level temporary allocation removed from render cache reuse."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(batch_count: int = 4096, elements_per_batch: int = 8) -> dict[str, object]:
    if batch_count <= 0:
        raise ValueError("batch_count must be positive")
    if elements_per_batch <= 0:
        raise ValueError("elements_per_batch must be positive")

    source_index_visits = batch_count * elements_per_batch
    return {
        "schema": "zircon.runtime.ui_render_cache_batch_reuse_pressure.v1",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "batch_count": batch_count,
            "elements_per_batch": elements_per_batch,
        },
        "legacy_collect": {
            "temporary_source_vec_allocations": batch_count,
            "source_index_visits": source_index_visits,
        },
        "borrowed_all_check": {
            "temporary_source_vec_allocations": 0,
            "source_index_visits": source_index_visits,
        },
        "reason_first_short_circuit": {
            "dirty_frame_source_index_visits": 0,
            "dirty_frame_status": "rebuilt",
        },
        "delta": {
            "avoided_temporary_source_vec_allocations": batch_count,
            "source_index_visit_change": 0,
            "avoided_dirty_frame_source_index_visits": source_index_visits,
            "temporary_allocation_reduction_ratio": "unbounded",
        },
        "excluded_from_model": [
            "paint element construction",
            "batch planning and sorting",
            "allocator latency and RSS",
            "CPU, GPU, and frame timing",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--batch-count", type=int, default=4096)
    parser.add_argument("--elements-per-batch", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.batch_count, args.elements_per_batch)
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
