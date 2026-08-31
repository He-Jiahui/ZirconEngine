import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    materialized_item_count: int,
    columns: int,
    scroll_update_count: int,
    large_seek_count: int,
) -> dict[str, Any]:
    values = {
        "materialized_item_count": materialized_item_count,
        "columns": columns,
        "scroll_update_count": scroll_update_count,
        "large_seek_count": large_seek_count,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("counts must be non-negative")
    if columns == 0:
        raise ValueError("columns must be positive")
    if materialized_item_count % columns != 0:
        raise ValueError("the optimized pool must contain complete physical rows")
    if large_seek_count > scroll_update_count:
        raise ValueError("large_seek_count cannot exceed scroll_update_count")

    one_row_scroll_count = scroll_update_count - large_seek_count
    retired_slot_rebinds = materialized_item_count * scroll_update_count
    row_modulo_slot_rebinds = (
        columns * one_row_scroll_count
        + materialized_item_count * large_seek_count
    )

    return {
        "schema": "zircon.editor.asset_browser_slot_reuse_pressure.v1",
        "inputs": values,
        "workload": {
            "one_row_scroll_count": one_row_scroll_count,
            "large_seek_count": large_seek_count,
        },
        "retired_window_relative_binding": {
            "slot_rebinds": retired_slot_rebinds,
            "slot_rebinds_per_one_row_scroll": materialized_item_count,
        },
        "row_modulo_binding": {
            "slot_rebinds": row_modulo_slot_rebinds,
            "slot_rebinds_per_one_row_scroll": columns,
            "preserved_slot_bindings": retired_slot_rebinds - row_modulo_slot_rebinds,
        },
        "delta": {
            "slot_rebind_reduction_ratio": _ratio(
                retired_slot_rebinds,
                row_modulo_slot_rebinds,
            ),
        },
        "interpretation": {
            "included": (
                "deterministic logical-item assignment changes in a complete-row "
                "Asset Browser physical slot pool"
            ),
            "excluded": (
                "item projection clones, equality checks, allocations, CPU time, input "
                "latency, RSS, render command patching, texture uploads, and GPU work"
            ),
            "runtime_cpu_measured": False,
            "allocator_or_rss_measured": False,
        },
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--materialized-item-count", type=int, default=54)
    parser.add_argument("--columns", type=int, default=6)
    parser.add_argument("--scroll-update-count", type=int, default=4_096)
    parser.add_argument("--large-seek-count", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        materialized_item_count=args.materialized_item_count,
        columns=args.columns,
        scroll_update_count=args.scroll_update_count,
        large_seek_count=args.large_seek_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded, encoding="utf-8")
    print(encoded, end="")


if __name__ == "__main__":
    main()
