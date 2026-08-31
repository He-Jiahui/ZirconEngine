"""Deterministic hit-grid allocation budget model.

This models the bounded spatial-index shape, not product timing or RSS.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path


MAX_AXIS_CELLS = 128
MAX_CELL_COUNT = MAX_AXIS_CELLS * MAX_AXIS_CELLS


def run(
    entry_count: int,
    authored_width: float,
    authored_height: float,
    cell_size: float = 64.0,
    max_axis_cells: int = MAX_AXIS_CELLS,
) -> dict[str, int | float | bool | dict]:
    for name, value in (
        ("entry_count", entry_count),
        ("authored_width", authored_width),
        ("authored_height", authored_height),
        ("cell_size", cell_size),
        ("max_axis_cells", max_axis_cells),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    requested_columns = max(1, int((authored_width / cell_size) + 0.999999))
    requested_rows = max(1, int((authored_height / cell_size) + 0.999999))
    bounded_cell_size = max(
        cell_size,
        authored_width / max_axis_cells,
        authored_height / max_axis_cells,
    )
    bounded_columns = max(1, int((authored_width / bounded_cell_size) + 0.999999))
    bounded_rows = max(1, int((authored_height / bounded_cell_size) + 0.999999))
    wide_entry_fallback = entry_count > 0 and (
        bounded_columns * bounded_rows > 4_096
    )
    if wide_entry_fallback:
        bounded_columns = 1
        bounded_rows = 1
    bounded_cells = bounded_columns * bounded_rows
    return {
        "entry_count": entry_count,
        "authored_width": authored_width,
        "authored_height": authored_height,
        "cell_size": cell_size,
        "max_axis_cells": max_axis_cells,
        "requested_columns": requested_columns,
        "requested_rows": requested_rows,
        "unbounded_cell_count": requested_columns * requested_rows,
        "bounded_columns": bounded_columns,
        "bounded_rows": bounded_rows,
        "bounded_cell_count": bounded_cells,
        "wide_entry_fallback": wide_entry_fallback,
        "bounded_cell_count_limit": max_axis_cells * max_axis_cells,
        "interpretation": {
            "included": [
                "checked axis and cell-count budget shape",
                "wide-entry coarse-grid fallback",
            ],
            "excluded": [
                "entry membership bytes",
                "CPU timing",
                "allocator RSS",
                "product hit latency",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--entry-count", type=int, default=10_000)
    parser.add_argument("--authored-width", type=float, default=1_000_000.0)
    parser.add_argument("--authored-height", type=float, default=1_000_000.0)
    parser.add_argument("--cell-size", type=float, default=64.0)
    parser.add_argument("--max-axis-cells", type=int, default=MAX_AXIS_CELLS)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.entry_count,
        args.authored_width,
        args.authored_height,
        args.cell_size,
        args.max_axis_cells,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
