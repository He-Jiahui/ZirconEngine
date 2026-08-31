"""Model inherited-visibility work across Runtime UI render extraction."""

from __future__ import annotations

import argparse
import json
import math
from pathlib import Path


def _chain_ancestor_visits(node_count: int, chain_depth: int) -> int:
    full_chains, remainder = divmod(node_count, chain_depth)
    full_chain_visits = chain_depth * (chain_depth + 1) // 2
    remainder_visits = remainder * (remainder + 1) // 2
    return full_chains * full_chain_visits + remainder_visits


def run(
    render_extract_count: int = 4096,
    arranged_rebuild_count: int = 64,
    node_count: int = 16384,
    chain_depth: int = 256,
    visibility_query_passes: int = 3,
) -> dict[str, object]:
    if render_extract_count <= 0:
        raise ValueError("render_extract_count must be positive")
    if arranged_rebuild_count < 0:
        raise ValueError("arranged_rebuild_count must be non-negative")
    if node_count <= 0:
        raise ValueError("node_count must be positive")
    if chain_depth <= 0 or chain_depth > node_count:
        raise ValueError("chain_depth must be in 1..=node_count")
    if visibility_query_passes <= 0:
        raise ValueError("visibility_query_passes must be positive")

    ancestor_visits_per_pass = _chain_ancestor_visits(node_count, chain_depth)
    retired_ancestor_visits = (
        render_extract_count
        * visibility_query_passes
        * ancestor_visits_per_pass
    )
    retained_rebuild_node_visits = arranged_rebuild_count * node_count
    indexed_queries = render_extract_count * visibility_query_passes * node_count
    retained_work_units = retained_rebuild_node_visits + indexed_queries
    node_id_bytes = node_count * 8
    visibility_word_count = (node_count + 63) // 64
    visibility_word_bytes = visibility_word_count * 8
    lookup_comparisons_per_query = math.ceil(math.log2(node_count + 1))

    return {
        "schema": "zircon.runtime.ui_arranged_visibility_pressure.v1",
        "inputs": {
            "render_extract_count": render_extract_count,
            "arranged_rebuild_count": arranged_rebuild_count,
            "node_count": node_count,
            "chain_depth": chain_depth,
            "visibility_query_passes": visibility_query_passes,
        },
        "retired_parent_walks": {
            "ancestor_node_visits_per_pass": ancestor_visits_per_pass,
            "ancestor_node_visits": retired_ancestor_visits,
        },
        "retained_visibility_index": {
            "arranged_rebuild_node_visits": retained_rebuild_node_visits,
            "indexed_visibility_queries": indexed_queries,
            "combined_work_units": retained_work_units,
            "worst_case_binary_search_comparisons_per_query": (
                lookup_comparisons_per_query
            ),
            "retained_node_id_bytes": node_id_bytes,
            "retained_visibility_word_count": visibility_word_count,
            "retained_visibility_word_bytes": visibility_word_bytes,
            "retained_payload_bytes": node_id_bytes + visibility_word_bytes,
            "retained_payload_bytes_per_node": (
                node_id_bytes + visibility_word_bytes
            )
            / node_count,
        },
        "delta": {
            "avoided_visibility_work_units": (
                retired_ancestor_visits - retained_work_units
            ),
            "visibility_work_reduction_ratio": (
                retired_ancestor_visits / retained_work_units
            ),
        },
        "interpretation": {
            "included": "ancestor node visits in the retired per-node parent walk, one indexed query per node and visibility stage, one iterative node resolution per arranged-index rebuild, and compact node-id/bitset payload bytes",
            "excluded": "BTreeMap and binary-search latency, command generation, text shaping, popup anchor traversal other than trigger visibility, Vec headers/capacity, allocator cost, CPU time, RSS, cache effects, and malformed trees",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--render-extract-count", type=int, default=4096)
    parser.add_argument("--arranged-rebuild-count", type=int, default=64)
    parser.add_argument("--node-count", type=int, default=16384)
    parser.add_argument("--chain-depth", type=int, default=256)
    parser.add_argument("--visibility-query-passes", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.render_extract_count,
        args.arranged_rebuild_count,
        args.node_count,
        args.chain_depth,
        args.visibility_query_passes,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
