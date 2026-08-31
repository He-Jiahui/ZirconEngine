"""Model removed transient-vector and redundant command-probe work.

This is a deterministic operation-count model, not measured product timing.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(present_count: int = 4096, commands_per_present: int = 32768) -> dict[str, object]:
    if present_count <= 0:
        raise ValueError("present_count must be positive")
    if commands_per_present < 0:
        raise ValueError("commands_per_present must be non-negative")

    retired_allocations = present_count * 2
    direct_adoption_allocations = present_count
    retired_header_moves = present_count * commands_per_present
    redundant_compaction_command_visits = present_count * commands_per_present
    return {
        "schema": "zircon.editor.chrome_command_stream_allocation_pressure.v2",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "present_count": present_count,
            "commands_per_present": commands_per_present,
        },
        "retired_extract_then_extend": {
            "chrome_command_vector_allocations": retired_allocations,
            "inter_vector_command_header_moves": retired_header_moves,
        },
        "direct_vector_adoption": {
            "chrome_command_vector_allocations": direct_adoption_allocations,
            "inter_vector_command_header_moves": 0,
        },
        "retired_redundant_image_compaction_probe": {
            "command_visits": redundant_compaction_command_visits,
        },
        "explicit_compaction_state": {
            "redundant_command_visits": 0,
        },
        "delta": {
            "avoided_vector_allocations": retired_allocations
            - direct_adoption_allocations,
            "allocation_reduction_ratio": retired_allocations
            / direct_adoption_allocations,
            "avoided_inter_vector_command_header_moves": retired_header_moves,
            "avoided_redundant_compaction_command_visits": (
                redundant_compaction_command_visits
            ),
        },
        "excluded_from_model": [
            "recording and chrome command materialization",
            "icon atlas discovery and rewrite scans",
            "first image-resource compaction",
            "host-to-runtime command conversion",
            "runtime image/style compaction and GPU batching",
            "CPU, allocator, RSS, latency, power, and GPU timing",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--present-count", type=int, default=4096)
    parser.add_argument("--commands-per-present", type=int, default=32768)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.present_count, args.commands_per_present)
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
