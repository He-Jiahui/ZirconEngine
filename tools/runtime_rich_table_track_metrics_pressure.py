#!/usr/bin/env python3
"""Model RichTable track geometry query work.

This is a deterministic algorithm-pressure model, not measured product timing.
It compares repeated span summation with one gap-aware prefix authority.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def pressure_report(
    cell_count: int = 10_000,
    column_count: int = 256,
    row_count: int = 1_000,
    average_column_span: int = 32,
    average_row_span: int = 4,
    column_span_queries_per_cell: int = 3,
    row_span_queries_per_cell: int = 2,
    origin_queries_per_cell: int = 5,
) -> dict[str, object]:
    inputs = {
        "cell_count": cell_count,
        "column_count": column_count,
        "row_count": row_count,
        "average_column_span": average_column_span,
        "average_row_span": average_row_span,
        "column_span_queries_per_cell": column_span_queries_per_cell,
        "row_span_queries_per_cell": row_span_queries_per_cell,
        "origin_queries_per_cell": origin_queries_per_cell,
    }
    for name, value in inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if average_column_span > column_count:
        raise ValueError("average_column_span must not exceed column_count")
    if average_row_span > row_count:
        raise ValueError("average_row_span must not exceed row_count")

    track_count = column_count + row_count
    span_query_count = cell_count * (
        column_span_queries_per_cell + row_span_queries_per_cell
    )
    origin_query_count = cell_count * origin_queries_per_cell

    repeated_span_track_visits = cell_count * (
        column_span_queries_per_cell * average_column_span
        + row_span_queries_per_cell * average_row_span
    )
    repeated_origin_build_visits = track_count
    repeated_total_extent_visits = track_count
    repeated_total_work = (
        repeated_span_track_visits
        + repeated_origin_build_visits
        + repeated_total_extent_visits
        + origin_query_count
    )

    prefix_build_visits = track_count
    prefix_span_queries = span_query_count
    prefix_total_extent_queries = 2
    prefix_total_work = (
        prefix_build_visits
        + prefix_span_queries
        + prefix_total_extent_queries
        + origin_query_count
    )

    f32_bytes = 4
    previous_geometry_payload_bytes = 2 * track_count * f32_bytes
    prefix_geometry_payload_bytes = (
        track_count + column_count + 1 + row_count + 1
    ) * f32_bytes

    return {
        "schema": "zircon.runtime.rich_table_track_metrics_pressure.v1",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": inputs,
        "repeated_span_summation": {
            "origin_build_track_visits": repeated_origin_build_visits,
            "span_query_count": span_query_count,
            "span_track_visits": repeated_span_track_visits,
            "origin_query_count": origin_query_count,
            "total_extent_track_visits": repeated_total_extent_visits,
            "combined_work_units": repeated_total_work,
            "complexity": "O(C + R + cells * (column_span + row_span))",
        },
        "gap_aware_prefix_metrics": {
            "prefix_build_track_visits": prefix_build_visits,
            "span_query_count": prefix_span_queries,
            "span_query_work_units": prefix_span_queries,
            "origin_query_count": origin_query_count,
            "total_extent_query_count": prefix_total_extent_queries,
            "combined_work_units": prefix_total_work,
            "complexity": "O(C + R + cells)",
        },
        "delta": {
            "avoided_track_work_units": repeated_total_work - prefix_total_work,
            "combined_work_reduction_ratio": round(
                repeated_total_work / prefix_total_work, 6
            ),
            "span_work_reduction_ratio": round(
                repeated_span_track_visits / prefix_span_queries, 6
            ),
        },
        "geometry_payload_estimate": {
            "previous_extents_plus_origins_bytes": previous_geometry_payload_bytes,
            "prefix_metrics_extents_plus_prefix_bytes": prefix_geometry_payload_bytes,
            "delta_bytes": prefix_geometry_payload_bytes
            - previous_geometry_payload_bytes,
            "note": "Payload estimate counts f32 arrays only; Vec headers and allocator overhead are excluded.",
        },
        "interpretation": {
            "included": "worst-case styled cells visiting provisional, box and final geometry; average column/row spans; origin construction/lookups; and final table extent",
            "excluded": "text shaping, row/column sizing, clipping, allocation latency, cache behavior, actual CPU/RSS, and GPU submission",
            "correctness_contract": "origin, span extent and total extent share one gap-aware prefix authority for horizontal and vertical writing modes",
            "required_product_evidence": "current-source rich-table layout CPU, span query count, prefix build count, fallback count, allocator bytes and p50/p95/p99 for large-span tables",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cell-count", type=int, default=10_000)
    parser.add_argument("--column-count", type=int, default=256)
    parser.add_argument("--row-count", type=int, default=1_000)
    parser.add_argument("--average-column-span", type=int, default=32)
    parser.add_argument("--average-row-span", type=int, default=4)
    parser.add_argument("--column-span-queries-per-cell", type=int, default=3)
    parser.add_argument("--row-span-queries-per-cell", type=int, default=2)
    parser.add_argument("--origin-queries-per-cell", type=int, default=5)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        cell_count=args.cell_count,
        column_count=args.column_count,
        row_count=args.row_count,
        average_column_span=args.average_column_span,
        average_row_span=args.average_row_span,
        column_span_queries_per_cell=args.column_span_queries_per_cell,
        row_span_queries_per_cell=args.row_span_queries_per_cell,
        origin_queries_per_cell=args.origin_queries_per_cell,
    )
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        if args.output.drive.upper() == "C:":
            raise ValueError("profile artifacts must not be written to C:")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
