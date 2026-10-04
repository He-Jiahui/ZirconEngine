"""Deterministic critical-path model for deferred editor maintenance stages."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    interaction_count: int,
    maintenance_stage_count: int,
    committed_frame_stage_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("interaction_count", interaction_count),
        ("maintenance_stage_count", maintenance_stage_count),
        ("committed_frame_stage_count", committed_frame_stage_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    old_critical_path_stage_visits = interaction_count * (
        maintenance_stage_count + committed_frame_stage_count
    )
    new_critical_path_stage_visits = interaction_count * committed_frame_stage_count
    deferred_maintenance_stage_visits = interaction_count * maintenance_stage_count

    return {
        "interaction_count": interaction_count,
        "maintenance_stage_count": maintenance_stage_count,
        "committed_frame_stage_count": committed_frame_stage_count,
        "old_critical_path_stage_visits": old_critical_path_stage_visits,
        "new_critical_path_stage_visits": new_critical_path_stage_visits,
        "deferred_maintenance_stage_visits": deferred_maintenance_stage_visits,
        "eliminated_critical_path_stage_visits": (
            old_critical_path_stage_visits - new_critical_path_stage_visits
        ),
        "critical_path_stage_reduction_ratio": (
            old_critical_path_stage_visits / new_critical_path_stage_visits
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--interaction-count", type=int, default=4_096)
    parser.add_argument("--maintenance-stage-count", type=int, default=24)
    parser.add_argument("--committed-frame-stage-count", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.interaction_count,
        args.maintenance_stage_count,
        args.committed_frame_stage_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
