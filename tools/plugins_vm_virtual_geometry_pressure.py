"""Deterministic work model for Plugins16 and Plugins17 hot paths."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins_vm_virtual_geometry_pressure.py",
    "zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs",
    "zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs",
    "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/zr_vm_language/runtime/src/call_site/script_call_table.rs": (
        "c984522e2b5b4729bb747484693e2fe791e24a11"
    ),
    "zircon_plugins/zr_vm_language/runtime/src/call_site/tests.rs": (
        "c712e773a7e2263f871c0d78ef8a3605a7d03d24"
    ),
    "zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/pending_completion/apply_gpu_page_table_entries.rs": (
        "cea56884a27a27cca6f1fa135d6527b8b8a0157b"
    ),
}


def run(
    reflected_fields: int = 4_096,
    token_rounds: int = 32,
    candidate_pages: int = 2_048,
    hot_pages: int = 64,
    hierarchy_edges: int = 4_096,
) -> dict[str, object]:
    for name, value in (
        ("reflected_fields", reflected_fields),
        ("token_rounds", token_rounds),
        ("candidate_pages", candidate_pages),
        ("hot_pages", hot_pages),
        ("hierarchy_edges", hierarchy_edges),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    token_dispatches = reflected_fields * token_rounds
    legacy_descendant_scans = candidate_pages * hierarchy_edges
    legacy_slot_operations = candidate_pages * hot_pages
    return {
        "schema": "zircon.plugins.vm_virtual_geometry_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "call-table atomic operations, retained token hash entries, token hash lookups, "
                "direct indexes, repeated hierarchy-edge scans, slot-owner operations, and "
                "shared hot-ancestor preparation"
            ),
            "excluded": (
                "hash and tree constant factors, atomic contention, cache locality, wall-clock "
                "duration, VM execution, GPU residency, VRAM, I/O, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "managed 21-pair alternating release benchmarks with direct-token P95 <= 50% "
                "and indexed hot-inheritance P95 <= 25% of legacy"
            ),
        },
        "inputs": {
            "reflected_fields": reflected_fields,
            "token_rounds": token_rounds,
            "candidate_pages": candidate_pages,
            "hot_pages": hot_pages,
            "hierarchy_edges": hierarchy_edges,
            "sample_pairs": 21,
        },
        "callsite_tokens": {
            "baseline_compile_atomic_operation_count": reflected_fields,
            "candidate_compile_atomic_operation_count": 1,
            "baseline_retained_token_hash_entry_count": reflected_fields,
            "candidate_retained_token_hash_entry_count": 0,
            "baseline_token_hash_lookup_count": token_dispatches,
            "candidate_token_hash_lookup_count": 0,
            "baseline_token_direct_index_count": 0,
            "candidate_token_direct_index_count": token_dispatches,
        },
        "hot_inheritance": {
            "baseline_repeated_descendant_edge_scan_count": legacy_descendant_scans,
            "candidate_repeated_descendant_edge_scan_count": 0,
            "baseline_slot_owner_operation_count": legacy_slot_operations,
            "candidate_slot_owner_operation_count": candidate_pages,
            "candidate_shared_hot_ancestor_parent_lookup_count": (
                hot_pages + hierarchy_edges
            ),
            "candidate_indexed_hot_ancestor_count": hierarchy_edges,
        },
        "acceptance": {
            "sample_order": "alternating",
            "percentile_method": "nearest_rank",
            "direct_token_p95_maximum_legacy_ratio": 0.50,
            "indexed_hot_p95_maximum_legacy_ratio": 0.25,
            "release_timing_pending": True,
        },
        "invariants": {
            "callsite_generation_rejection_preserved": True,
            "callsite_dense_ordinal_order_preserved": True,
            "hot_inheritance_legacy_oracle_parity": True,
            "duplicate_slot_first_owner_wins_preserved": True,
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
    parser.add_argument("--reflected-fields", type=int, default=4_096)
    parser.add_argument("--token-rounds", type=int, default=32)
    parser.add_argument("--candidate-pages", type=int, default=2_048)
    parser.add_argument("--hot-pages", type=int, default=64)
    parser.add_argument("--hierarchy-edges", type=int, default=4_096)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.reflected_fields,
        args.token_rounds,
        args.candidate_pages,
        args.hot_pages,
        args.hierarchy_edges,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
