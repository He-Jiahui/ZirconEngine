"""Deterministic work model for retained Asset tree paint metadata."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def run(
    template_node_count: int = 10_000,
    pane_paint_count: int = 2_000,
    activity_hover_paint_count: int = 1_000,
    metadata_generation_count: int = 1,
) -> dict[str, int | float | str]:
    for name, value in (
        ("template_node_count", template_node_count),
        ("pane_paint_count", pane_paint_count),
        ("activity_hover_paint_count", activity_hover_paint_count),
        ("metadata_generation_count", metadata_generation_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    legacy_activity_count_node_visits = pane_paint_count * template_node_count
    legacy_browser_count_node_visits = pane_paint_count * template_node_count
    legacy_activity_hover_node_visits = (
        activity_hover_paint_count * template_node_count
    )
    legacy_combined_work_units = (
        legacy_activity_count_node_visits
        + legacy_browser_count_node_visits
        + legacy_activity_hover_node_visits
    )

    target_generation_node_visits = metadata_generation_count * template_node_count
    target_count_queries = pane_paint_count * 2
    target_hover_index_queries = activity_hover_paint_count
    target_overlay_trie_depth = math.ceil(math.log2(template_node_count))
    target_overlay_trie_node_visits_per_query = target_overlay_trie_depth + 1
    target_hover_live_row_probe_visits = (
        activity_hover_paint_count * target_overlay_trie_node_visits_per_query
    )
    target_combined_work_units = (
        target_generation_node_visits
        + target_count_queries
        + target_hover_index_queries
        + target_hover_live_row_probe_visits
    )

    return {
        "schema_version": 1,
        "interpretation": (
            "deterministic template-node/query work model; not CPU, allocation, "
            "layout, render, GPU, or latency evidence"
        ),
        "template_node_count": template_node_count,
        "pane_paint_count": pane_paint_count,
        "activity_hover_paint_count": activity_hover_paint_count,
        "metadata_generation_count": metadata_generation_count,
        "legacy_activity_count_node_visits": legacy_activity_count_node_visits,
        "legacy_browser_count_node_visits": legacy_browser_count_node_visits,
        "legacy_activity_hover_node_visits": legacy_activity_hover_node_visits,
        "legacy_combined_work_units": legacy_combined_work_units,
        "target_generation_node_visits": target_generation_node_visits,
        "target_count_queries": target_count_queries,
        "target_hover_index_queries": target_hover_index_queries,
        "target_overlay_trie_depth": target_overlay_trie_depth,
        "target_overlay_trie_node_visits_per_query": (
            target_overlay_trie_node_visits_per_query
        ),
        "target_hover_live_row_probe_visits": target_hover_live_row_probe_visits,
        "target_combined_work_units": target_combined_work_units,
        "eliminated_work_units": legacy_combined_work_units - target_combined_work_units,
        "work_reduction_ratio": legacy_combined_work_units / target_combined_work_units,
        "target_publication_complexity": "O(N) per metadata generation",
        "target_paint_count_complexity": "O(1) per pane paint",
        "target_hover_frame_complexity": (
            "O(1) contiguous model; O(log N) worst-case persistent row overlay"
        ),
    }


def write_result(path: Path, result: dict[str, int | float | str]) -> None:
    resolved = path.resolve()
    if resolved.drive.upper() == "C:":
        raise ValueError("pressure artifacts must not be written to C:")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    resolved.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template-node-count", type=int, default=10_000)
    parser.add_argument("--pane-paint-count", type=int, default=2_000)
    parser.add_argument("--activity-hover-paint-count", type=int, default=1_000)
    parser.add_argument("--metadata-generation-count", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        template_node_count=args.template_node_count,
        pane_paint_count=args.pane_paint_count,
        activity_hover_paint_count=args.activity_hover_paint_count,
        metadata_generation_count=args.metadata_generation_count,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
