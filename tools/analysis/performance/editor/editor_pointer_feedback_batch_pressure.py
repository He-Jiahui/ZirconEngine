import argparse
import json
from pathlib import Path


def run(
    event_count: int,
    changed_stage_count: int = 2,
    trailing_candidate_count: int = 1,
) -> dict[str, int | float]:
    if event_count <= 0:
        raise ValueError("event_count must be positive")
    if changed_stage_count <= 0:
        raise ValueError("changed_stage_count must be positive")
    if trailing_candidate_count < 0:
        raise ValueError("trailing_candidate_count must be non-negative")

    old_dirty_summary_count = event_count * (
        changed_stage_count + trailing_candidate_count
    )
    old_refresh_count = event_count * changed_stage_count
    new_dirty_summary_count = 0
    new_pending_invalidation_count_checks = event_count
    new_refresh_count = event_count
    return {
        "event_count": event_count,
        "changed_stage_count": changed_stage_count,
        "trailing_candidate_count": trailing_candidate_count,
        "old_dirty_summary_count": old_dirty_summary_count,
        "new_dirty_summary_count": new_dirty_summary_count,
        "avoided_dirty_summary_count": old_dirty_summary_count
        - new_dirty_summary_count,
        "dirty_summary_elimination_percent": 100.0,
        "new_pending_invalidation_count_checks": new_pending_invalidation_count_checks,
        "old_surface_refresh_count": old_refresh_count,
        "new_surface_refresh_count": new_refresh_count,
        "avoided_surface_refresh_count": old_refresh_count - new_refresh_count,
        "surface_refresh_reduction_ratio": round(
            old_refresh_count / new_refresh_count, 4
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--event-count", type=int, default=65_536)
    parser.add_argument("--changed-stage-count", type=int, default=2)
    parser.add_argument("--trailing-candidate-count", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        args.event_count,
        args.changed_stage_count,
        args.trailing_candidate_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
