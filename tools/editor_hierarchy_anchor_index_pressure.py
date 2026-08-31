"""Deterministic work model for generation-owned Hierarchy viewport anchors."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def run(
    template_node_count: int = 10_000,
    pane_paint_count: int = 2_000,
    metadata_generation_count: int = 1,
    anchor_candidate_count: int = 2,
) -> dict[str, int | float | str]:
    for name, value in (
        ("template_node_count", template_node_count),
        ("pane_paint_count", pane_paint_count),
        ("metadata_generation_count", metadata_generation_count),
        ("anchor_candidate_count", anchor_candidate_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    legacy_scanned_nodes_per_paint = template_node_count
    legacy_template_node_visits = pane_paint_count * legacy_scanned_nodes_per_paint

    target_generation_node_visits = metadata_generation_count * template_node_count
    target_metadata_queries = pane_paint_count
    target_candidate_index_visits = pane_paint_count * anchor_candidate_count
    target_overlay_trie_depth = math.ceil(math.log2(template_node_count))
    target_overlay_trie_node_visits_per_candidate = target_overlay_trie_depth + 1
    target_live_row_probe_visits = (
        target_candidate_index_visits * target_overlay_trie_node_visits_per_candidate
    )
    target_combined_work_units = (
        target_generation_node_visits
        + target_metadata_queries
        + target_candidate_index_visits
        + target_live_row_probe_visits
    )

    return {
        "schema_version": 1,
        "interpretation": (
            "deterministic worst-case late/missing-visible-anchor template-node/index/trie "
            "work model; not CPU, allocation, layout, render, GPU, or latency evidence"
        ),
        "template_node_count": template_node_count,
        "pane_paint_count": pane_paint_count,
        "metadata_generation_count": metadata_generation_count,
        "anchor_candidate_count": anchor_candidate_count,
        "legacy_scenario": "first visible anchor is last or absent",
        "legacy_scanned_nodes_per_paint": legacy_scanned_nodes_per_paint,
        "legacy_template_node_visits": legacy_template_node_visits,
        "target_generation_node_visits": target_generation_node_visits,
        "target_metadata_queries": target_metadata_queries,
        "target_candidate_index_visits": target_candidate_index_visits,
        "target_overlay_trie_depth": target_overlay_trie_depth,
        "target_overlay_trie_node_visits_per_candidate": (
            target_overlay_trie_node_visits_per_candidate
        ),
        "target_live_row_probe_visits": target_live_row_probe_visits,
        "target_combined_work_units": target_combined_work_units,
        "eliminated_work_units": legacy_template_node_visits - target_combined_work_units,
        "work_reduction_ratio": legacy_template_node_visits / target_combined_work_units,
        "target_generation_complexity": "O(N) per metadata generation",
        "target_paint_complexity": (
            "O(A) contiguous model; O(A log N) worst-case persistent row overlay"
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
    parser.add_argument("--metadata-generation-count", type=int, default=1)
    parser.add_argument("--anchor-candidate-count", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        template_node_count=args.template_node_count,
        pane_paint_count=args.pane_paint_count,
        metadata_generation_count=args.metadata_generation_count,
        anchor_candidate_count=args.anchor_candidate_count,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
