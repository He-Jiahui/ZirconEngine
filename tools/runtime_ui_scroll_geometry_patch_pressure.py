"""Deterministic post-layout work model for retained scroll geometry patches."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    total_node_count: int,
    hit_entry_count: int,
    scroll_update_count: int,
    affected_node_count: int,
    inactive_stable_entry_count: int,
    modeled_entry_bytes: int,
) -> dict[str, int | float | bool | str]:
    if total_node_count <= 0:
        raise ValueError("total_node_count must be positive")
    if not 0 <= hit_entry_count <= total_node_count:
        raise ValueError("hit_entry_count must be within the node count")
    if scroll_update_count <= 0:
        raise ValueError("scroll_update_count must be positive")
    if not 0 < affected_node_count <= total_node_count:
        raise ValueError("affected_node_count must be within the node count")
    if not 0 <= inactive_stable_entry_count <= hit_entry_count:
        raise ValueError("inactive_stable_entry_count must be within hit entries")
    if modeled_entry_bytes <= 0:
        raise ValueError("modeled_entry_bytes must be positive")

    retired_arranged_visits = total_node_count * scroll_update_count
    retired_hit_entry_visits = hit_entry_count * scroll_update_count
    retired_post_layout_work = retired_arranged_visits + retired_hit_entry_visits
    retained_arranged_patch_visits = affected_node_count * scroll_update_count
    retained_hit_entry_patch_visits = affected_node_count * scroll_update_count
    retained_post_layout_work = (
        retained_arranged_patch_visits + retained_hit_entry_patch_visits
    )
    modeled_stable_entry_payload_bytes = (
        inactive_stable_entry_count * modeled_entry_bytes
    )

    return {
        "schema_version": 1,
        "scope": "post_layout_arranged_hit_only",
        "total_node_count": total_node_count,
        "hit_entry_count": hit_entry_count,
        "scroll_update_count": scroll_update_count,
        "affected_node_count": affected_node_count,
        "inactive_stable_entry_count": inactive_stable_entry_count,
        "modeled_entry_bytes": modeled_entry_bytes,
        "retired_arranged_visits": retired_arranged_visits,
        "retired_hit_entry_visits": retired_hit_entry_visits,
        "retired_post_layout_work": retired_post_layout_work,
        "retained_arranged_patch_visits": retained_arranged_patch_visits,
        "retained_hit_entry_patch_visits": retained_hit_entry_patch_visits,
        "retained_post_layout_work": retained_post_layout_work,
        "eliminated_post_layout_work": (
            retired_post_layout_work - retained_post_layout_work
        ),
        "post_layout_work_reduction_ratio": (
            retired_post_layout_work / retained_post_layout_work
        ),
        "modeled_stable_entry_payload_bytes": modeled_stable_entry_payload_bytes,
        "full_arranged_rebuilds_modeled_after_patch": 0,
        "full_hit_rebuilds_modeled_after_patch": 0,
        "layout_child_iteration_modeled": False,
        "variable_render_command_patch_modeled": False,
        "true_instance_virtualization_modeled": False,
        "cpu_or_rss_measured": False,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--total-node-count", type=int, default=16_384)
    parser.add_argument("--hit-entry-count", type=int, default=12_288)
    parser.add_argument("--scroll-update-count", type=int, default=4_096)
    parser.add_argument("--affected-node-count", type=int, default=64)
    parser.add_argument("--inactive-stable-entry-count", type=int, default=12_224)
    parser.add_argument("--modeled-entry-bytes", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.total_node_count,
        args.hit_entry_count,
        args.scroll_update_count,
        args.affected_node_count,
        args.inactive_stable_entry_count,
        args.modeled_entry_bytes,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
