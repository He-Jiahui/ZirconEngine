"""Deterministic candidate-work model for responsive resize gating."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    responsive_candidate_count: int,
    resize_step_count: int,
    threshold_crossing_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("responsive_candidate_count", responsive_candidate_count),
        ("resize_step_count", resize_step_count),
        ("threshold_crossing_count", threshold_crossing_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if threshold_crossing_count > resize_step_count:
        raise ValueError("threshold_crossing_count cannot exceed resize_step_count")

    old_candidate_visits = responsive_candidate_count * resize_step_count
    # One initial responsive resolution plus one pass per crossed threshold.
    new_candidate_visits = responsive_candidate_count * (threshold_crossing_count + 1)
    return {
        "responsive_candidate_count": responsive_candidate_count,
        "resize_step_count": resize_step_count,
        "threshold_crossing_count": threshold_crossing_count,
        "old_candidate_visits": old_candidate_visits,
        "new_candidate_visits": new_candidate_visits,
        "eliminated_candidate_visits": old_candidate_visits - new_candidate_visits,
        "candidate_visit_reduction_ratio": old_candidate_visits / new_candidate_visits,
        "interpretation": {
            "included": [
                "responsive candidate traversal",
                "one initial resolution",
                "threshold-crossing resolutions",
            ],
            "excluded": [
                "layout measurement",
                "arrangement",
                "paint extraction",
                "CPU timing",
                "RSS",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--responsive-candidate-count", type=int, default=10_000)
    parser.add_argument("--resize-step-count", type=int, default=200)
    parser.add_argument("--threshold-crossing-count", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.responsive_candidate_count,
        args.resize_step_count,
        args.threshold_crossing_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
