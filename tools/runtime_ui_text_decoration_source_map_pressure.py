#!/usr/bin/env python3
"""Model eager versus touched-line text-decoration source-map materialization.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import json


def pressure_report(
    line_count: int,
    touched_line_count: int,
    decoration_count: int,
    clusters_per_line: int,
) -> dict[str, object]:
    values = {
        "line_count": line_count,
        "touched_line_count": touched_line_count,
        "decoration_count": decoration_count,
        "clusters_per_line": clusters_per_line,
    }
    for name, value in values.items():
        if value < 0:
            raise ValueError(f"{name} must be non-negative")
    if line_count == 0:
        raise ValueError("line_count must be positive")
    if touched_line_count > line_count:
        raise ValueError("touched_line_count cannot exceed line_count")
    if decoration_count == 0 and touched_line_count != 0:
        raise ValueError("zero decorations cannot touch text lines")

    eager_cluster_projection_visits = line_count * clusters_per_line
    lazy_cluster_projection_visits = touched_line_count * clusters_per_line
    line_range_probes = line_count * decoration_count
    avoided_maps = line_count - touched_line_count
    avoided_cluster_visits = (
        eager_cluster_projection_visits - lazy_cluster_projection_visits
    )

    return {
        "inputs": values,
        "eager_all_line_maps": {
            "source_map_constructions": line_count,
            "cluster_projection_visits": eager_cluster_projection_visits,
            "retained_map_entries": line_count,
            "line_range_probes": line_range_probes,
        },
        "lazy_touched_line_maps": {
            "source_map_constructions": touched_line_count,
            "cluster_projection_visits": lazy_cluster_projection_visits,
            "retained_map_entries": touched_line_count,
            "line_range_probes": line_range_probes,
        },
        "avoided": {
            "source_map_constructions": avoided_maps,
            "cluster_projection_visits": avoided_cluster_visits,
            "retained_map_entries": avoided_maps,
        },
        "construction_reduction_ratio": (
            line_count / touched_line_count if touched_line_count else None
        ),
        "retained_work": [
            "decoration-major line range probes",
            "source-map span projection for intersecting lines",
            "caret source-map construction",
            "decoration command allocation",
        ],
    }


def pressure_suite(
    line_counts: list[int],
    touched_line_count: int,
    decoration_count: int,
    clusters_per_line: int,
) -> dict[str, object]:
    if not line_counts:
        raise ValueError("line_counts must not be empty")
    return {
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "scenarios": [
            pressure_report(
                line_count,
                touched_line_count,
                decoration_count,
                clusters_per_line,
            )
            for line_count in line_counts
        ],
        "excluded_from_model": [
            "CPU and allocator timing",
            "hash-table implementation overhead",
            "glyph shaping and rasterization",
            "render command submission and present",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--line-counts", type=int, nargs="+", default=[128, 4096, 65536])
    parser.add_argument("--touched-line-count", type=int, default=1)
    parser.add_argument("--decoration-count", type=int, default=3)
    parser.add_argument("--clusters-per-line", type=int, default=32)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_suite(
        args.line_counts,
        args.touched_line_count,
        args.decoration_count,
        args.clusters_per_line,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
