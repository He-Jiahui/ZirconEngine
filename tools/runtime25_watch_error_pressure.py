"""Deterministic work model for Runtime25 watch-error tail admission."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime25_watch_error_pressure.py",
    "tools/zircon-validation-runtime25-watch-error-batch.ps1",
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs",
)
BASELINE_GIT_REVISION = "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/asset/pipeline/manager/project_asset_manager/watch_dispatch.rs": (
        "e489f5f5c6fe152464b83869efce0fa154412b38"
    )
}


def run(items: int = 200_000, capacity: int = 64) -> dict[str, object]:
    if items <= 0:
        raise ValueError("items must be positive")
    if capacity <= 0:
        raise ValueError("capacity must be positive")
    if items < capacity:
        raise ValueError("items must be at least capacity")

    overflow_count = items - capacity
    legacy_moves = overflow_count * (capacity - 1)
    return {
        "schema": "zircon.runtime.watch_error_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "bounded admissions, overflow count, legacy prefix record moves, "
                "optimized record moves, and queue retention"
            ),
            "excluded": (
                "allocator constants, payload size, mutex contention, filesystem events, "
                "wall-clock duration, reconciliation execution, I/O, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating release benchmark with nearest-rank "
                "optimized P95 <= 75% of legacy"
            ),
        },
        "inputs": {
            "items": items,
            "capacity": capacity,
            "sample_pairs": 21,
        },
        "tail_admission": {
            "overflow_admission_count": overflow_count,
            "baseline_prefix_record_move_count": legacy_moves,
            "candidate_prefix_record_move_count": 0,
            "baseline_retained_error_count": capacity,
            "candidate_retained_error_count": capacity,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "optimized_p95_maximum_legacy_ratio": 0.75,
            "release_timing_pending": True,
        },
        "invariants": {
            "oldest_error_eviction_preserved": True,
            "fifo_publication_preserved": True,
            "overflow_requires_reconciliation": True,
            "error_payloads_not_cloned_by_take_work": True,
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
    parser.add_argument("--items", type=int, default=200_000)
    parser.add_argument("--capacity", type=int, default=64)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.items, args.capacity)
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
