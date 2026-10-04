"""Deterministic draw-order traversal model for owner-text prewarming."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    node_count: int,
    full_extract_count: int,
    eligible_request_count: int,
    overlap_threshold: int = 8,
) -> dict[str, int | float | bool | dict]:
    for name, value in (
        ("node_count", node_count),
        ("full_extract_count", full_extract_count),
        ("eligible_request_count", eligible_request_count),
        ("overlap_threshold", overlap_threshold),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    admission_scan_enabled = eligible_request_count >= overlap_threshold
    draw_order_visits_per_extract = node_count
    old_collection_visits = draw_order_visits_per_extract * full_extract_count
    old_admission_visits = (
        old_collection_visits if admission_scan_enabled else 0
    )
    new_collection_visits = old_collection_visits
    old_render_collection_visits = old_collection_visits
    new_render_collection_visits = old_render_collection_visits
    old_total_visits = (
        old_collection_visits + old_admission_visits + old_render_collection_visits
    )
    new_total_visits = new_collection_visits + new_render_collection_visits
    return {
        "node_count": node_count,
        "full_extract_count": full_extract_count,
        "eligible_request_count": eligible_request_count,
        "overlap_threshold": overlap_threshold,
        "admission_scan_enabled": admission_scan_enabled,
        "old_owner_prewarm_collection_visits": old_collection_visits,
        "old_overlap_admission_visits": old_admission_visits,
        "new_owner_prewarm_collection_visits": new_collection_visits,
        "eliminated_overlap_admission_visits": old_admission_visits,
        "old_render_command_collection_visits": old_render_collection_visits,
        "new_render_command_collection_visits": new_render_collection_visits,
        "old_total_draw_order_visits": old_total_visits,
        "new_total_draw_order_visits": new_total_visits,
        "eliminated_draw_order_visits": old_total_visits - new_total_visits,
        "draw_order_visit_reduction_ratio": old_total_visits / new_total_visits,
        "interpretation": {
            "included": [
                "owner-text prewarm request collection",
                "legacy overlap-admission draw-order scan",
                "render-command draw-order collection",
            ],
            "excluded": [
                "text shaping and cache work",
                "render command construction details",
                "CPU timing",
                "allocator RSS",
                "GPU work",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-count", type=int, default=100_000)
    parser.add_argument("--full-extract-count", type=int, default=1_000)
    parser.add_argument("--eligible-request-count", type=int, default=32)
    parser.add_argument("--overlap-threshold", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.node_count,
        args.full_extract_count,
        args.eligible_request_count,
        args.overlap_threshold,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
