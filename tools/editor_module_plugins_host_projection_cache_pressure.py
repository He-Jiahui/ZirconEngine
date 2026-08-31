from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    *,
    plugin_count: int,
    presentation_apply_count: int,
    projection_generation_change_count: int,
) -> dict[str, object]:
    if plugin_count < 0:
        raise ValueError("plugin_count must be non-negative")
    if presentation_apply_count <= 0:
        raise ValueError("presentation_apply_count must be positive")
    if projection_generation_change_count < 0:
        raise ValueError("projection_generation_change_count must be non-negative")
    rebuild_count = min(
        presentation_apply_count,
        projection_generation_change_count + 1,
    )
    old_plugin_row_reads = plugin_count * presentation_apply_count
    new_plugin_row_reads = plugin_count * rebuild_count
    old_status_mapping_reads = old_plugin_row_reads
    new_status_mapping_reads = 0
    old_total_source_row_reads = old_plugin_row_reads + old_status_mapping_reads
    new_total_source_row_reads = new_plugin_row_reads + new_status_mapping_reads
    avoided = old_total_source_row_reads - new_total_source_row_reads

    return {
        "inputs": {
            "plugin_count": plugin_count,
            "presentation_apply_count": presentation_apply_count,
            "projection_generation_change_count": projection_generation_change_count,
            "projection_rebuild_count": rebuild_count,
        },
        "retired_full_projection": {
            "plugin_row_reads": old_plugin_row_reads,
            "status_mapping_reads": old_status_mapping_reads,
            "total_source_row_reads": old_total_source_row_reads,
        },
        "generation_cached_projection": {
            "plugin_row_reads": new_plugin_row_reads,
            "status_mapping_reads": new_status_mapping_reads,
            "total_source_row_reads": new_total_source_row_reads,
            "stable_lookup_source_row_reads": 0,
            "maximum_resident_pane_entries": 8,
        },
        "delta": {
            "avoided_source_row_reads": avoided,
            "source_row_read_reduction_ratio": (
                old_total_source_row_reads / new_total_source_row_reads
                if new_total_source_row_reads
                else None
            ),
        },
        "interpretation": {
            "model": "deterministic source-row visit count",
            "runtime_cpu_measured": False,
            "rss_measured": False,
            "allocator_calls_measured": False,
            "template_projection_measured": False,
            "host_node_count_measured": False,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plugin-count", type=int, default=1_000)
    parser.add_argument("--presentation-apply-count", type=int, default=4_096)
    parser.add_argument("--projection-generation-change-count", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = run(
        plugin_count=args.plugin_count,
        presentation_apply_count=args.presentation_apply_count,
        projection_generation_change_count=args.projection_generation_change_count,
    )
    rendered = json.dumps(report, indent=2, sort_keys=True)
    if args.output is None:
        print(rendered)
        return
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
