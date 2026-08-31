"""Deterministic work model for Runtime340 and Runtime341 validation paths."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_path_module_validation_pressure.py",
    "zircon_runtime/src/core/framework/scene/entity_path.rs",
    "zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs",
)
BASELINE_GIT_REVISION = "630d66c362013e3b5b72f97362ad56fc54ff6d8c"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/core/framework/scene/entity_path.rs": (
        "a0c00d7bff922c2d83affadf698c97054dae9bb6"
    ),
    "zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs": (
        "99cac76fcb28638c1344122d976221b2dc129475"
    ),
}


def run(
    path_checks_per_sample: int = 8_192,
    path_segments: int = 1_024,
    segment_bytes: int = 7,
    module_checks_per_sample: int = 8_192,
    field_padding_bytes: int = 4_096,
) -> dict[str, object]:
    for name, value in (
        ("path_checks_per_sample", path_checks_per_sample),
        ("path_segments", path_segments),
        ("segment_bytes", segment_bytes),
        ("module_checks_per_sample", module_checks_per_sample),
        ("field_padding_bytes", field_padding_bytes),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    path_bytes = path_segments * segment_bytes + path_segments - 1
    reserved_slots_per_path = max(path_bytes // 2, 1)
    retained_segment_count = path_checks_per_sample * path_segments

    return {
        "schema": "zircon.runtime.path_module_validation_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_reservation_and_trim_call_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "path input scans, explicit reserve calls and planned segment slots, "
                "retained segment ownership count, module trim calls, and heap allocations"
            ),
            "excluded": (
                "Vec growth policy, allocator metadata, string allocation payload, trim "
                "implementation cost, cache locality, RSS, and product latency"
            ),
            "dynamic_acceptance_pending": (
                "managed alternating release P50/P95 samples for both Rust benchmarks"
            ),
        },
        "inputs": {
            "path_checks_per_sample": path_checks_per_sample,
            "path_segments": path_segments,
            "segment_bytes": segment_bytes,
            "path_bytes": path_bytes,
            "module_checks_per_sample": module_checks_per_sample,
            "field_padding_bytes": field_padding_bytes,
        },
        "entity_path": {
            "baseline_input_scan_count": path_checks_per_sample,
            "candidate_input_scan_count": path_checks_per_sample,
            "baseline_explicit_reserve_call_count": 0,
            "candidate_explicit_reserve_call_count": path_checks_per_sample,
            "baseline_planned_segment_slot_count": 0,
            "candidate_planned_segment_slot_count": (
                path_checks_per_sample * reserved_slots_per_path
            ),
            "planned_slots_per_path": reserved_slots_per_path,
            "baseline_segment_ownership_allocation_count": retained_segment_count,
            "candidate_segment_ownership_allocation_count": retained_segment_count,
        },
        "module_field": {
            "baseline_trim_call_count": module_checks_per_sample * 2,
            "candidate_trim_call_count": module_checks_per_sample,
            "trim_call_reduction_percent": 50.0,
            "baseline_success_heap_allocations": 0,
            "candidate_success_heap_allocations": 0,
        },
        "invariants": {
            "entity_path_segment_order_preserved": True,
            "entity_path_empty_error_preserved": True,
            "module_field_decisions_preserved": True,
            "module_field_error_text_preserved": True,
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
    parser.add_argument("--path-checks-per-sample", type=int, default=8_192)
    parser.add_argument("--path-segments", type=int, default=1_024)
    parser.add_argument("--segment-bytes", type=int, default=7)
    parser.add_argument("--module-checks-per-sample", type=int, default=8_192)
    parser.add_argument("--field-padding-bytes", type=int, default=4_096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.path_checks_per_sample,
        args.path_segments,
        args.segment_bytes,
        args.module_checks_per_sample,
        args.field_padding_bytes,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
