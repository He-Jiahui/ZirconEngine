"""Deterministic reference-count work model for Plugins14 tiled bake plans."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins14_tiled_plan_pressure.py",
    "zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/navigation/runtime/src/manager/bake/task_pool.rs": (
        "0dc1ed4709d6bac99b5bdf1122d7b3f25f68a525"
    ),
}


def run(
    clones_per_sample: int = 200_000,
    sample_pairs: int = 21,
) -> dict[str, object]:
    for name, value in (
        ("clones_per_sample", clones_per_sample),
        ("sample_pairs", sample_pairs),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    legacy_pairs = clones_per_sample * 4
    candidate_pairs = clones_per_sample
    return {
        "schema": "zircon.plugins.tiled_plan_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_reference_count_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "plan clone count, Arc increment/decrement pairs, modeled atomic reference-count "
                "operations, payload observations, and harvest plan copies"
            ),
            "excluded": (
                "atomic contention, cache-line migration, allocator metadata, wall-clock duration, "
                "tile baking, scheduler latency, cancellation, publication, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating release benchmark with candidate P95 no more "
                "than 80% of legacy P95"
            ),
        },
        "inputs": {
            "clones_per_sample": clones_per_sample,
            "sample_pairs": sample_pairs,
            "legacy_arc_refcount_pairs_per_clone": 4,
            "candidate_arc_refcount_pairs_per_clone": 1,
        },
        "work": {
            "baseline_arc_refcount_pair_count": legacy_pairs,
            "candidate_arc_refcount_pair_count": candidate_pairs,
            "arc_refcount_pair_reduction_percent": 75.0,
            "baseline_modeled_atomic_rmw_count": legacy_pairs * 2,
            "candidate_modeled_atomic_rmw_count": candidate_pairs * 2,
            "atomic_rmw_reduction_percent": 75.0,
            "baseline_plan_payload_observation_count": clones_per_sample,
            "candidate_plan_payload_observation_count": clones_per_sample,
            "baseline_completed_plan_copy_count": 0,
            "candidate_completed_plan_copy_count": 0,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "candidate_p95_maximum_legacy_ratio": 0.8,
            "release_timing_pending": True,
        },
        "invariants": {
            "tile_payload_observation_parity": True,
            "dispatch_completes_before_harvest": True,
            "worker_references_released_before_completion": True,
            "zero_copy_plan_publication_preserved": True,
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
    parser.add_argument("--clones-per-sample", type=int, default=200_000)
    parser.add_argument("--sample-pairs", type=int, default=21)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.clones_per_sample, args.sample_pairs)
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
