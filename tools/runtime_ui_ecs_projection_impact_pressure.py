"""Model source-entry work in Runtime UI ECS projection impact aggregation."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    dirty_node_count: int = 10_000,
    dirty_domain_count: int = 8,
    runtime_stage_count: int = 10,
    active_stage_membership_count: int = 60_000,
) -> dict[str, object]:
    positive_inputs = {
        "dirty_node_count": dirty_node_count,
        "dirty_domain_count": dirty_domain_count,
        "runtime_stage_count": runtime_stage_count,
        "active_stage_membership_count": active_stage_membership_count,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    maximum_memberships = dirty_node_count * runtime_stage_count
    if active_stage_membership_count > maximum_memberships:
        raise ValueError(
            "active_stage_membership_count must not exceed dirty nodes times stages"
        )

    old_domain_entry_reads = dirty_node_count * (1 + dirty_domain_count)
    old_schedule_entry_reads = dirty_node_count * (2 + runtime_stage_count)
    old_source_entry_reads = old_domain_entry_reads + old_schedule_entry_reads
    retained_source_entry_reads = dirty_node_count * 2

    old_schedule_mask_derivations = 1 + dirty_node_count * runtime_stage_count
    retained_schedule_mask_derivations = dirty_node_count

    return {
        "schema": "zircon.runtime.ui_ecs_projection_impact_pressure.v1",
        "inputs": positive_inputs,
        "repeated_scan_aggregation": {
            "domain_source_entry_reads": old_domain_entry_reads,
            "schedule_source_entry_reads": old_schedule_entry_reads,
            "total_source_entry_reads": old_source_entry_reads,
            "schedule_mask_derivations": old_schedule_mask_derivations,
            "per_node_stage_reason_vector_allocations": active_stage_membership_count,
            "intermediate_entry_slots": dirty_node_count * 2,
        },
        "single_pass_bucket_aggregation": {
            "domain_source_entry_reads": dirty_node_count,
            "schedule_source_entry_reads": dirty_node_count,
            "total_source_entry_reads": retained_source_entry_reads,
            "schedule_mask_derivations": retained_schedule_mask_derivations,
            "per_node_stage_reason_vector_allocations": 0,
            "intermediate_entry_slots": 0,
            "bounded_domain_buckets": dirty_domain_count,
            "bounded_schedule_buckets": runtime_stage_count,
        },
        "delta": {
            "avoided_source_entry_reads": (
                old_source_entry_reads - retained_source_entry_reads
            ),
            "source_entry_read_reduction_ratio": round(
                old_source_entry_reads / retained_source_entry_reads, 6
            ),
            "avoided_schedule_mask_derivations": (
                old_schedule_mask_derivations - retained_schedule_mask_derivations
            ),
            "avoided_per_node_stage_reason_vector_allocations": (
                active_stage_membership_count
            ),
            "avoided_intermediate_entry_slots": dirty_node_count * 2,
        },
        "interpretation": {
            "included": "source-entry reads, schedule-mask derivations, temporary reason-vector allocations, and materialized intermediate entry slots in the two impact aggregators",
            "excluded": "actual CPU and allocator latency, cache behavior, branch prediction, bucket pushes, final node sorting/deduplication, serialization, snapshot construction, and downstream consumers",
            "scope": "deterministic operation-count model; all runtime stages are active in the repeated-scan implementation, while both implementations preserve the same eight domain checks, ten stage checks, bucket membership writes, and final deterministic output ordering",
        },
    }


def _reject_c_drive(path: Path) -> None:
    if path.drive.casefold() == "c:":
        raise ValueError("performance artifacts must not be written to C drive")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dirty-node-count", type=int, default=10_000)
    parser.add_argument("--dirty-domain-count", type=int, default=8)
    parser.add_argument("--runtime-stage-count", type=int, default=10)
    parser.add_argument("--active-stage-membership-count", type=int, default=60_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.dirty_node_count,
        args.dirty_domain_count,
        args.runtime_stage_count,
        args.active_stage_membership_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        _reject_c_drive(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
