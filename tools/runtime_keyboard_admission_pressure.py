"""Deterministic scan model for Runtime337 and Runtime338 keyboard admission."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/runtime_keyboard_admission_pressure.py",
    "zircon_runtime/src/ui/surface/input/keyboard_action.rs",
    "zircon_runtime/src/ui/surface/input/keyboard_navigation.rs",
)
BASELINE_GIT_REVISION = "630d66c362013e3b5b72f97362ad56fc54ff6d8c"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_runtime/src/ui/surface/input/keyboard_action.rs": (
        "9e6b49d08bb56da126d4e6942a96a69eb1f2943b"
    ),
    "zircon_runtime/src/ui/surface/input/keyboard_navigation.rs": (
        "897ba18d086ca8df10f04c3cacd139625c140b77"
    ),
}


def run(
    text_checks_per_sample: int = 8_192,
    text_characters: int = 4_096,
    direction_checks_per_sample: int = 262_144,
    baseline_direction_candidates: int = 12,
    normalized_stack_bytes: int = 16,
) -> dict[str, object]:
    for name, value in (
        ("text_checks_per_sample", text_checks_per_sample),
        ("text_characters", text_characters),
        ("direction_checks_per_sample", direction_checks_per_sample),
        ("baseline_direction_candidates", baseline_direction_candidates),
        ("normalized_stack_bytes", normalized_stack_bytes),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    baseline_text_scans = text_checks_per_sample * 2
    candidate_text_scans = text_checks_per_sample
    baseline_text_character_visits = baseline_text_scans * text_characters
    candidate_text_character_visits = candidate_text_scans * text_characters
    baseline_direction_passes = (
        direction_checks_per_sample * baseline_direction_candidates
    )
    candidate_direction_passes = direction_checks_per_sample

    return {
        "schema": "zircon.runtime.keyboard_admission_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_scan_and_normalization_pass_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "all-whitespace Unicode character scans and visits, direction-key "
                "normalization passes, heap allocations, and fixed stack storage"
            ),
            "excluded": (
                "per-candidate early comparison exits, Unicode classification cost, "
                "branch prediction, cache locality, and product input latency"
            ),
            "dynamic_acceptance_pending": (
                "managed alternating release P50/P95 samples for both Rust benchmarks"
            ),
        },
        "inputs": {
            "text_checks_per_sample": text_checks_per_sample,
            "text_characters": text_characters,
            "direction_checks_per_sample": direction_checks_per_sample,
            "baseline_direction_candidates": baseline_direction_candidates,
            "normalized_stack_bytes": normalized_stack_bytes,
        },
        "keyboard_text": {
            "baseline_character_scan_count": baseline_text_scans,
            "candidate_character_scan_count": candidate_text_scans,
            "baseline_character_visit_count": baseline_text_character_visits,
            "candidate_character_visit_count": candidate_text_character_visits,
            "character_visit_reduction_percent": 50.0,
            "baseline_accepted_text_allocations": 0,
            "candidate_accepted_text_allocations": 0,
        },
        "direction_key": {
            "baseline_normalization_pass_count": baseline_direction_passes,
            "candidate_normalization_pass_count": candidate_direction_passes,
            "normalization_pass_reduction_percent": (
                100.0
                * (baseline_direction_passes - candidate_direction_passes)
                / baseline_direction_passes
            ),
            "baseline_heap_allocations": 0,
            "candidate_heap_allocations": 0,
            "candidate_stack_storage_bytes": normalized_stack_bytes,
        },
        "invariants": {
            "control_character_rejection_preserved": True,
            "unicode_whitespace_rejection_preserved": True,
            "accepted_text_remains_borrowed": True,
            "direction_key_code_fallback_preserved": True,
            "direction_separator_and_case_folding_preserved": True,
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
    parser.add_argument("--text-checks-per-sample", type=int, default=8_192)
    parser.add_argument("--text-characters", type=int, default=4_096)
    parser.add_argument("--direction-checks-per-sample", type=int, default=262_144)
    parser.add_argument("--baseline-direction-candidates", type=int, default=12)
    parser.add_argument("--normalized-stack-bytes", type=int, default=16)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.text_checks_per_sample,
        args.text_characters,
        args.direction_checks_per_sample,
        args.baseline_direction_candidates,
        args.normalized_stack_bytes,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
