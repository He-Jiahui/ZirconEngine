"""Deterministic work model for Plugins18 and Plugins20 hot paths."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins_texture_native_pressure.py",
    "zircon_plugins/texture_importer/runtime/src/mipgen/kernel.rs",
    "zircon_plugins/native_dynamic_fixture/native/src/lib.rs",
    "zircon_plugins/native_dynamic_fixture/native/src/tests.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/texture_importer/runtime/src/mipgen/kernel.rs": (
        "9c2080db17cff857cf1bd47971d2a0e34b3ed175"
    ),
    "zircon_plugins/native_dynamic_fixture/native/src/lib.rs": (
        "df69c9d312e87580c1845abc15dc37f834fda03d"
    ),
    "zircon_plugins/native_dynamic_fixture/native/src/tests.rs": (
        "cab0068aba9e863f2393b6edd1ea384c49e33ff5"
    ),
}


def run(
    target_texels: int = 16_384,
    legacy_kaiser_weight_evaluations: int = 487_305,
    cached_kaiser_weight_evaluations: int = 1_274,
    native_source_bytes: int = 131_101,
    native_encodes_per_sample: int = 8,
) -> dict[str, object]:
    for name, value in (
        ("target_texels", target_texels),
        ("legacy_kaiser_weight_evaluations", legacy_kaiser_weight_evaluations),
        ("cached_kaiser_weight_evaluations", cached_kaiser_weight_evaluations),
        ("native_source_bytes", native_source_bytes),
        ("native_encodes_per_sample", native_encodes_per_sample),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if cached_kaiser_weight_evaluations > legacy_kaiser_weight_evaluations:
        raise ValueError("cached Kaiser work cannot exceed the legacy workload")

    return {
        "schema": "zircon.plugins.texture_native_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "Kaiser weight and normalizer evaluations, target texels, native response-sized "
                "buffers, source-text clone bytes, intermediate metadata buffers, and byte limits"
            ),
            "excluded": (
                "filter floating-point cost, allocator metadata, JSON serializer costs, cache "
                "locality, wall-clock duration, GPU texture work, IPC latency, and RSS"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating release benchmarks for Kaiser P95 <= 25% of inline "
                "and bounded native response P95 <= 110% of legacy"
            ),
        },
        "inputs": {
            "target_texels": target_texels,
            "native_source_bytes": native_source_bytes,
            "native_encodes_per_sample": native_encodes_per_sample,
            "sample_pairs": 21,
        },
        "kaiser_axis_cache": {
            "baseline_target_texel_count": target_texels,
            "candidate_target_texel_count": target_texels,
            "baseline_weight_evaluation_count": legacy_kaiser_weight_evaluations,
            "candidate_weight_evaluation_count": cached_kaiser_weight_evaluations,
            "weight_evaluation_reduction_percent": reduction_percent(
                legacy_kaiser_weight_evaluations,
                cached_kaiser_weight_evaluations,
            ),
            "baseline_normalizer_evaluation_count": 1,
            "candidate_normalizer_evaluation_count": 1,
        },
        "native_response": {
            "baseline_full_response_buffer_count": native_encodes_per_sample * 2,
            "candidate_full_response_buffer_count": native_encodes_per_sample,
            "full_response_buffer_reduction_percent": 50.0,
            "baseline_source_text_clone_bytes": (
                native_source_bytes * native_encodes_per_sample
            ),
            "candidate_source_text_clone_bytes": 0,
            "baseline_intermediate_metadata_buffer_count": native_encodes_per_sample,
            "candidate_intermediate_metadata_buffer_count": 0,
            "maximum_request_metadata_bytes": 64 * 1024,
            "maximum_request_source_bytes": 256 * 1024,
            "maximum_host_output_bytes": 1024 * 1024,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "kaiser_candidate_p95_maximum_legacy_ratio": 0.25,
            "native_candidate_p95_maximum_legacy_ratio": 1.10,
            "release_timing_pending": True,
        },
        "invariants": {
            "kaiser_output_byte_parity": True,
            "kaiser_normalizer_count_preserved": True,
            "native_response_metadata_parity": True,
            "native_checked_length_admission_preserved": True,
        },
    }


def reduction_percent(baseline: int, candidate: int) -> float:
    return (baseline - candidate) * 100.0 / baseline


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
    parser.add_argument("--target-texels", type=int, default=16_384)
    parser.add_argument(
        "--legacy-kaiser-weight-evaluations", type=int, default=487_305
    )
    parser.add_argument(
        "--cached-kaiser-weight-evaluations", type=int, default=1_274
    )
    parser.add_argument("--native-source-bytes", type=int, default=131_101)
    parser.add_argument("--native-encodes-per-sample", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.target_texels,
        args.legacy_kaiser_weight_evaluations,
        args.cached_kaiser_weight_evaluations,
        args.native_source_bytes,
        args.native_encodes_per_sample,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
