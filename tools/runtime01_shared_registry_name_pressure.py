"""Deterministic work model for Runtime01 registry-name clone storage."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime01_shared_registry_name_pressure.py",
    "tools/zircon-validation-runtime01-shared-registry-name-batch.ps1",
    "zircon_runtime/src/core/runtime/descriptors/registry_name.rs",
)
BASELINE_GIT_REVISION = "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/core/runtime/descriptors/registry_name.rs": (
        "7d19185511e98d306a885d48446b1acda4e40820"
    )
}


def run(names: int = 65_536, clones_per_name: int = 8) -> dict[str, object]:
    if names <= 0:
        raise ValueError("names must be positive")
    if clones_per_name <= 0:
        raise ValueError("clones_per_name must be positive")

    clone_count = names * clones_per_name
    payload_bytes = sum(
        len(f"Runtime.Feature{index}.Manager.Service{index}")
        for index in range(names)
    ) * clones_per_name
    return {
        "schema": "zircon.runtime.registry_name_clone_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "owned registry-name clone count, cloned payload bytes, and immutable "
                "payload allocation count"
            ),
            "excluded": (
                "allocator constants, Arc atomic-operation cost, graph construction, "
                "resolution, lifecycle transitions, and product frame latency"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating release benchmark with shared clone "
                "P50/P95 no greater than 50% of legacy"
            ),
        },
        "inputs": {
            "names": names,
            "clones_per_name": clones_per_name,
            "sample_pairs": 21,
        },
        "clone_work": {
            "owned_clone_count": clone_count,
            "baseline_payload_allocation_count": clone_count,
            "candidate_payload_allocation_count": 0,
            "baseline_cloned_payload_bytes": payload_bytes,
            "candidate_cloned_payload_bytes": 0,
            "payload_allocation_reduction_percent": 100.0,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "shared_p50_maximum_legacy_ratio": 0.5,
            "shared_p95_maximum_legacy_ratio": 0.5,
            "release_timing_pending": True,
        },
        "invariants": {
            "validated_name_bytes_preserved": True,
            "cached_offsets_preserved": True,
            "equality_hash_borrow_display_serde_preserved": True,
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
    parser.add_argument("--names", type=int, default=65_536)
    parser.add_argument("--clones-per-name", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.names, args.clones_per_name)
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
