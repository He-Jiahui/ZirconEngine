"""Deterministic work model for Plugins06 editor provider composition."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins06_editor_provider_pressure.py",
    "tools/zircon-validation-plugins06-editor-provider-batch.ps1",
    "zircon_app/Cargo.toml",
    "zircon_app/src/entry/mod.rs",
    "zircon_app/src/entry/first_party_editor_plugins.rs",
)
BASELINE_GIT_REVISION = "050d8e6c36cd1bf4f3ab0d8fc4df0864c1c29a3f"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_app/Cargo.toml": "dda6b39caf881405be774fcc10fafee1c15b0325",
    "zircon_app/src/entry/mod.rs": "95d23453483f3301a5aa197c16ac2828ad78fb2d",
    "zircon_app/src/entry/first_party_editor_plugins.rs": (
        "063e5391e7321bcc58f7220224b68ad23d66afc4"
    ),
}


def run(
    provider_configurations: int = 2,
    samples_per_provider: int = 21,
    iterations_per_sample: int = 1_024,
    maximum_p95_microseconds: int = 250_000,
) -> dict[str, object]:
    for name, value in (
        ("provider_configurations", provider_configurations),
        ("samples_per_provider", samples_per_provider),
        ("iterations_per_sample", iterations_per_sample),
        ("maximum_p95_microseconds", maximum_p95_microseconds),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    projections_per_provider = samples_per_provider * iterations_per_sample
    return {
        "schema": "zircon.plugins.editor_provider_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_composition_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "feature configurations, manifest-to-registration projections, "
                "registration cardinality, and provider-neutral branch structure"
            ),
            "excluded": (
                "Cargo compile time, allocator constants, cache locality, wall-clock "
                "duration, Editor startup, rendering, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "two managed no-default-feature release configurations, each emitting "
                "21 nearest-rank P50/P95 samples with P95 <= 250000us"
            ),
        },
        "inputs": {
            "provider_configurations": provider_configurations,
            "samples_per_provider": samples_per_provider,
            "iterations_per_sample": iterations_per_sample,
            "maximum_p95_microseconds": maximum_p95_microseconds,
        },
        "provider_composition": {
            "baseline_neural_only_registration_count": 0,
            "candidate_neural_only_registration_count": 1,
            "baseline_navigation_only_registration_count": 1,
            "candidate_navigation_only_registration_count": 1,
            "baseline_empty_fallback_branch_count": 1,
            "candidate_empty_fallback_branch_count": 0,
            "candidate_provider_neutral_catalog_branch_count": 1,
            "target_editor_host_provider_count": 2,
        },
        "validation_workload": {
            "manifest_projection_count_per_provider": projections_per_provider,
            "manifest_projection_count_all_providers": (
                provider_configurations * projections_per_provider
            ),
            "registration_count_per_projection": 1,
        },
        "acceptance": {
            "sample_order": "sequential-single-branch",
            "percentile_method": "nearest_rank",
            "provider_p95_maximum_microseconds": maximum_p95_microseconds,
            "navigation_release_timing_pending": True,
            "neural_release_timing_pending": True,
        },
        "invariants": {
            "navigation_only_provider_preserved": True,
            "neural_only_provider_enabled": True,
            "target_editor_host_enables_both_providers": True,
            "provider_registration_cardinality_preserved": True,
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
    parser.add_argument("--provider-configurations", type=int, default=2)
    parser.add_argument("--samples-per-provider", type=int, default=21)
    parser.add_argument("--iterations-per-sample", type=int, default=1_024)
    parser.add_argument("--maximum-p95-microseconds", type=int, default=250_000)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.provider_configurations,
        args.samples_per_provider,
        args.iterations_per_sample,
        args.maximum_p95_microseconds,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
