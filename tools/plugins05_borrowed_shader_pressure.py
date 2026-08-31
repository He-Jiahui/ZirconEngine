"""Deterministic clone-work model for Plugins05 borrowed shader validation."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins05_borrowed_shader_pressure.py",
    "zircon_runtime/src/asset/importer/contract.rs",
    "zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs",
    "zircon_plugins/asset_importers/shader/runtime/src/lib.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/asset/importer/contract.rs": (
        "b517b891ae7d9d8f4de63b1ed811848afdd7f844"
    ),
    "zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs": (
        "034ebc714674cad4a0a88570eef3583997c447b9"
    ),
    "zircon_plugins/asset_importers/shader/runtime/src/lib.rs": (
        "3097ef7cb1ae2d89c6cacad7e7df1986041ea804"
    ),
}


def run(source_bytes: int = 1_048_576, iterations_per_sample: int = 32) -> dict[str, object]:
    for name, value in (
        ("source_bytes", source_bytes),
        ("iterations_per_sample", iterations_per_sample),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    return {
        "schema": "zircon.plugins.borrowed_shader_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_clone_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "valid UTF-8 view calls, syntactically invalid shader parse attempts, owned "
                "source clone bytes, source clone allocations, and provider borrow sites"
            ),
            "excluded": (
                "Naga parse cost, diagnostic formatting, allocator metadata, cache locality, "
                "accepted-asset ownership, wall-clock duration, pipeline compilation, and RSS"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating WGSL release benchmark with borrowed P95 no more "
                "than 85% of owned P95"
            ),
        },
        "inputs": {
            "source_bytes": source_bytes,
            "iterations_per_sample": iterations_per_sample,
            "sample_pairs": 21,
        },
        "validation": {
            "baseline_utf8_view_count": iterations_per_sample,
            "candidate_utf8_view_count": iterations_per_sample,
            "baseline_invalid_parse_attempt_count": iterations_per_sample,
            "candidate_invalid_parse_attempt_count": iterations_per_sample,
            "baseline_source_clone_byte_count": source_bytes * iterations_per_sample,
            "candidate_source_clone_byte_count": 0,
            "source_clone_byte_reduction_percent": 100.0,
            "baseline_source_clone_allocation_count": iterations_per_sample,
            "candidate_source_clone_allocation_count": 0,
        },
        "providers": {
            "wgsl_borrowed_validation_sites": 2,
            "glsl_borrowed_validation_sites": 1,
            "accepted_asset_ownership_retained": True,
        },
        "acceptance": {
            "sample_order": "alternating_owned_first_even",
            "percentile_method": "nearest_rank",
            "borrowed_p95_maximum_owned_ratio": 0.85,
            "release_timing_pending": True,
        },
        "invariants": {
            "source_byte_identity_preserved": True,
            "invalid_utf8_error_chain_preserved": True,
            "shader_diagnostics_preserved": True,
            "accepted_shader_source_owned": True,
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
    parser.add_argument("--source-bytes", type=int, default=1_048_576)
    parser.add_argument("--iterations-per-sample", type=int, default=32)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.source_bytes, args.iterations_per_sample)
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
