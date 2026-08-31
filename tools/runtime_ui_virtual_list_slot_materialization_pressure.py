import argparse
import json
import math
from pathlib import Path
from typing import Any


def run(
    *,
    logical_count: int,
    row_subtree_node_count: int,
    viewport_extent: float,
    item_extent: float,
    overscan: int,
    scroll_update_count: int,
    large_seek_count: int,
) -> dict[str, Any]:
    values = {
        "logical_count": logical_count,
        "row_subtree_node_count": row_subtree_node_count,
        "overscan": overscan,
        "scroll_update_count": scroll_update_count,
        "large_seek_count": large_seek_count,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("counts and overscan must be non-negative")
    if item_extent <= 0.0 or not math.isfinite(item_extent):
        raise ValueError("item_extent must be finite and positive")
    if viewport_extent <= 0.0 or not math.isfinite(viewport_extent):
        raise ValueError("viewport_extent must be finite and positive")
    if large_seek_count > scroll_update_count:
        raise ValueError("large_seek_count cannot exceed scroll_update_count")

    visible_count = math.ceil(viewport_extent / item_extent)
    partial_boundary_slot_count = 1
    slot_count = min(
        logical_count,
        visible_count + partial_boundary_slot_count + overscan * 2,
    )
    retained_child_node_count = logical_count * row_subtree_node_count
    bounded_slot_node_count = slot_count * row_subtree_node_count
    retained_scroll_child_visits = logical_count * scroll_update_count
    planner_slot_visits = slot_count * scroll_update_count
    one_row_scroll_count = scroll_update_count - large_seek_count
    planner_rebind_count_upper_bound = (
        one_row_scroll_count + large_seek_count * slot_count
    )

    return {
        "schema_version": 1,
        "scope": "fixed_extent_slot_planner_only",
        **values,
        "viewport_extent": viewport_extent,
        "item_extent": item_extent,
        "visible_count": visible_count,
        "partial_boundary_slot_count": partial_boundary_slot_count,
        "slot_count": slot_count,
        "retained_child_node_count": retained_child_node_count,
        "bounded_slot_node_count": bounded_slot_node_count,
        "slot_node_count_reduction_ratio": _ratio(
            retained_child_node_count, bounded_slot_node_count
        ),
        "retained_scroll_child_visits": retained_scroll_child_visits,
        "planner_slot_visits": planner_slot_visits,
        "planner_visit_reduction_ratio": _ratio(
            retained_scroll_child_visits, planner_slot_visits
        ),
        "planner_rebind_count_upper_bound": planner_rebind_count_upper_bound,
        "surface_materializer_wired": False,
        "layout_render_hit_reduction_measured": False,
        "cpu_or_rss_measured": False,
    }


def _ratio(retired: int, retained: int) -> float | None:
    if retained == 0:
        return None
    return round(retired / retained, 2)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--logical-count", type=int, default=100_000)
    parser.add_argument("--row-subtree-node-count", type=int, default=6)
    parser.add_argument("--viewport-extent", type=float, default=800.0)
    parser.add_argument("--item-extent", type=float, default=24.0)
    parser.add_argument("--overscan", type=int, default=3)
    parser.add_argument("--scroll-update-count", type=int, default=4_096)
    parser.add_argument("--large-seek-count", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        logical_count=args.logical_count,
        row_subtree_node_count=args.row_subtree_node_count,
        viewport_extent=args.viewport_extent,
        item_extent=args.item_extent,
        overscan=args.overscan,
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
