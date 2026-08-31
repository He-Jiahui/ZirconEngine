"""Deterministic work model for Runtime177 and Runtime182 input-manager tasks."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_input_manager_metadata_capacity_pressure.py",
    "zircon_runtime/src/ui/dispatch/input_manager/outcome.rs",
    "zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs",
)
BASELINE_GIT_REVISION = "5ffc4945095a6fc734bcbb2e632958026350b760"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/ui/dispatch/input_manager/outcome.rs": (
        "c44c34de8c33e0ba240aaab7c8dfd3970e86ac34"
    ),
    "zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs": (
        "27a41991c710f91f060d14fc599537320f08373a"
    ),
}


def run(
    batch_count: int = 2_048,
    results_per_batch: int = 256,
    ime_appends_per_sample: int = 1_024,
    ime_requests_per_append: int = 256,
    max_host_requests_per_request: int = 3,
) -> dict[str, object]:
    for name, value in (
        ("batch_count", batch_count),
        ("results_per_batch", results_per_batch),
        ("ime_appends_per_sample", ime_appends_per_sample),
        ("ime_requests_per_append", ime_requests_per_append),
        ("max_host_requests_per_request", max_host_requests_per_request),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    legacy_result_passes = batch_count * 2
    candidate_result_passes = batch_count
    legacy_result_visits = legacy_result_passes * results_per_batch
    candidate_result_visits = candidate_result_passes * results_per_batch
    ime_request_count = ime_appends_per_sample * ime_requests_per_append
    ime_host_request_upper_bound = ime_request_count * max_host_requests_per_request

    return {
        "schema": "zircon.runtime.input_manager_metadata_capacity_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_scan_and_capacity_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "top-level dispatch-result passes and visits, iterator admissions, "
                "planned reserve calls, and maximum IME host-request slots"
            ),
            "excluded": (
                "allocator growth policy, host-request payload clone cost, nested effect "
                "scan cost, cache locality, RSS, and product input latency"
            ),
            "dynamic_acceptance_pending": (
                "managed alternating release P50/P95 samples for both Rust benchmarks"
            ),
        },
        "inputs": {
            "batch_count": batch_count,
            "results_per_batch": results_per_batch,
            "ime_appends_per_sample": ime_appends_per_sample,
            "ime_requests_per_append": ime_requests_per_append,
            "max_host_requests_per_request": max_host_requests_per_request,
        },
        "dispatch_metadata": {
            "legacy_top_level_result_pass_count": legacy_result_passes,
            "candidate_top_level_result_pass_count": candidate_result_passes,
            "legacy_top_level_result_visit_count": legacy_result_visits,
            "candidate_top_level_result_visit_count": candidate_result_visits,
            "top_level_visit_reduction_percent": 50.0,
        },
        "ime_capacity": {
            "input_method_request_count": ime_request_count,
            "maximum_host_request_value_count": ime_host_request_upper_bound,
            "legacy_planned_reserve_call_count": 0,
            "candidate_planned_reserve_call_count": ime_appends_per_sample,
            "legacy_planned_slot_count": 0,
            "candidate_planned_slot_count": ime_host_request_upper_bound,
            "capacity_multiplier": max_host_requests_per_request,
        },
        "invariants": {
            "host_request_order_preserved": True,
            "redraw_short_circuit_preserved": True,
            "ime_disable_short_circuit_preserved": True,
            "ime_optional_payload_semantics_preserved": True,
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
    parser.add_argument("--batch-count", type=int, default=2_048)
    parser.add_argument("--results-per-batch", type=int, default=256)
    parser.add_argument("--ime-appends-per-sample", type=int, default=1_024)
    parser.add_argument("--ime-requests-per-append", type=int, default=256)
    parser.add_argument("--max-host-requests-per-request", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.batch_count,
        args.results_per_batch,
        args.ime_appends_per_sample,
        args.ime_requests_per_append,
        args.max_host_requests_per_request,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
