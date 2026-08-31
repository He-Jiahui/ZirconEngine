"""Deterministic work model for the four Plugins15 AI runtime optimizations."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins15_ai_runtime_pressure.py",
    "tools/zircon-validation-plugins15-ai-runtime-batch.ps1",
    "zircon_plugins/ai/runtime/src/manager/snapshot.rs",
    "zircon_plugins/ai/runtime/src/manager/state.rs",
    "zircon_plugins/ai/runtime/src/perception/scan.rs",
    "zircon_plugins/ai/runtime/src/perception/stimuli.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/ai/runtime/src/manager/snapshot.rs": (
        "1c44f3e4f245a1b96d72bf7c15592cb82e1d4886"
    ),
    "zircon_plugins/ai/runtime/src/manager/state.rs": (
        "75f877cba0fc575a2548b6c1a9d655b9ecb217ad"
    ),
    "zircon_plugins/ai/runtime/src/perception/scan.rs": (
        "0c6e9e934334f86b33b2aa866f16e38cf885a0eb"
    ),
    "zircon_plugins/ai/runtime/src/perception/stimuli.rs": (
        "f2e55ce701f458ec01d6a9be5610c0463adcf766"
    ),
}


def run(
    tree_count: int = 256,
    nodes_per_tree: int = 32,
    tree_acquisitions: int = 32,
    stimulus_count: int = 8_192,
    world_node_count: int = 4_096,
    total_agent_count: int = 8_192,
    active_agent_count: int = 256,
) -> dict[str, object]:
    for name, value in (
        ("tree_count", tree_count),
        ("nodes_per_tree", nodes_per_tree),
        ("tree_acquisitions", tree_acquisitions),
        ("stimulus_count", stimulus_count),
        ("world_node_count", world_node_count),
        ("total_agent_count", total_agent_count),
        ("active_agent_count", active_agent_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if active_agent_count > total_agent_count:
        raise ValueError("active_agent_count cannot exceed total_agent_count")

    baseline_tree_clones = tree_count * tree_acquisitions
    baseline_node_copies = baseline_tree_clones * nodes_per_tree
    return {
        "schema": "zircon.plugins.ai_runtime_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "compiled-tree clone and node-copy counts, snapshot sort passes, "
                "World projections, projected node records, agent projections, "
                "global key unions, and descriptor-catalog clones"
            ),
            "excluded": (
                "allocator and comparison constants, cache locality, lock contention, "
                "wall-clock duration, full behavior execution, I/O, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "one managed release Cargo batch running four 21-pair alternating "
                "benchmarks with per-record nearest-rank P95 gates"
            ),
        },
        "inputs": {
            "tree_count": tree_count,
            "nodes_per_tree": nodes_per_tree,
            "tree_acquisitions": tree_acquisitions,
            "stimulus_count": stimulus_count,
            "world_node_count": world_node_count,
            "total_agent_count": total_agent_count,
            "active_agent_count": active_agent_count,
            "sample_pairs": 21,
        },
        "compiled_tree_generation": {
            "baseline_top_level_tree_clone_count": baseline_tree_clones,
            "candidate_top_level_tree_clone_count": 0,
            "baseline_compiled_node_copy_count": baseline_node_copies,
            "candidate_compiled_node_copy_count": 0,
            "baseline_catalog_generation_rebuild_count": tree_acquisitions,
            "candidate_catalog_generation_rebuild_count": 0,
            "candidate_arc_handle_clone_count": tree_acquisitions,
        },
        "ordered_stimuli": {
            "baseline_snapshot_sort_count": 1,
            "candidate_snapshot_sort_count": 0,
            "baseline_sort_input_element_count": stimulus_count,
            "candidate_sort_input_element_count": 0,
            "baseline_cloned_stimulus_count": stimulus_count,
            "candidate_cloned_stimulus_count": stimulus_count,
        },
        "single_pass_sampling": {
            "baseline_world_projection_count": 2,
            "candidate_world_projection_count": 1,
            "baseline_projected_node_record_count": world_node_count * 2,
            "candidate_projected_node_record_count": world_node_count,
            "baseline_redundant_sample_sort_count": 2,
            "candidate_redundant_sample_sort_count": 0,
        },
        "targeted_debug_snapshot": {
            "baseline_agent_projection_count": total_agent_count,
            "candidate_agent_projection_count": active_agent_count,
            "baseline_global_key_union_count": 1,
            "candidate_global_key_union_count": 0,
            "baseline_behavior_tree_catalog_clone_count": 1,
            "candidate_behavior_tree_catalog_clone_count": 0,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "compiled_tree_p95_maximum_legacy_ratio": 0.10,
            "ordered_stimuli_p95_maximum_legacy_ratio": 0.75,
            "single_pass_sampling_p95_maximum_legacy_ratio": 0.75,
            "targeted_debug_snapshot_p95_maximum_legacy_ratio": 0.25,
            "release_timing_pending": True,
        },
        "invariants": {
            "compiled_tree_registry_order_preserved": True,
            "stimulus_sense_source_order_preserved": True,
            "receiver_source_entity_order_preserved": True,
            "targeted_world_and_entity_filtering_preserved": True,
        },
    }


def source_binding() -> dict[str, object]:
    source_sha256 = {
        relative_path: hashlib.sha256((ROOT / relative_path).read_bytes())
        .hexdigest()
        .upper()
        for relative_path in CRITICAL_SOURCE_FILES
    }
    manifest_lines = [
        f"worktree:{path}:{source_sha256[path]}" for path in sorted(source_sha256)
    ]
    manifest_lines.extend(
        f"head-git-blob:{path}:{HEAD_BASELINE_GIT_BLOBS[path]}"
        for path in sorted(HEAD_BASELINE_GIT_BLOBS)
    )
    manifest_payload = "\n".join(manifest_lines).encode("utf-8")
    return {
        "git_revision": BASELINE_GIT_REVISION,
        "critical_source_files": list(CRITICAL_SOURCE_FILES),
        "source_sha256": source_sha256,
        "head_baseline_git_blobs": dict(HEAD_BASELINE_GIT_BLOBS),
        "source_manifest_sha256": hashlib.sha256(manifest_payload)
        .hexdigest()
        .upper(),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--tree-count", type=int, default=256)
    parser.add_argument("--nodes-per-tree", type=int, default=32)
    parser.add_argument("--tree-acquisitions", type=int, default=32)
    parser.add_argument("--stimulus-count", type=int, default=8_192)
    parser.add_argument("--world-node-count", type=int, default=4_096)
    parser.add_argument("--total-agent-count", type=int, default=8_192)
    parser.add_argument("--active-agent-count", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.tree_count,
        args.nodes_per_tree,
        args.tree_acquisitions,
        args.stimulus_count,
        args.world_node_count,
        args.total_agent_count,
        args.active_agent_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
