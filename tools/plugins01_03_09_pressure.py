"""Deterministic work model for Plugins01, Plugins03, and Plugins09."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/plugins01_03_09_pressure.py",
    "tools/zircon-validation-plugins01-03-09-batch.ps1",
    "zircon_plugins/plugin_sdk/src/native.rs",
    "zircon_plugins/plugin_sdk/src/native/tests.rs",
    "zircon_plugins/native_window_hosting/editor/src/capability.rs",
    "zircon_plugins/native_window_hosting/editor/src/lib.rs",
    "zircon_plugins/native_window_hosting/editor/src/plugin.rs",
    "zircon_plugins/native_window_hosting/editor/src/tests.rs",
    "zircon_plugins/particles/runtime/src/service.rs",
    "zircon_plugins/particles/runtime/src/tests/snapshot.rs",
)
BASELINE_GIT_REVISION = "ca3ac3cc6ad218d04a5cd469447cea2452441321"
HEAD_BASELINE_GIT_BLOBS = {
    "zircon_plugins/plugin_sdk/src/native.rs": (
        "3a147cc61eaf62f7ffe3ecd941469bbe40b65a68"
    ),
    "zircon_plugins/plugin_sdk/src/native/tests.rs": (
        "742c33d7c58317845beb55665763774a716cab77"
    ),
    "zircon_plugins/native_window_hosting/editor/src/capability.rs": (
        "c7e2176a3b105152aac41db330536e597be68da9"
    ),
    "zircon_plugins/native_window_hosting/editor/src/lib.rs": (
        "3f6ce5e6ccf9d39d0cf15b70e3bb70447e3884bb"
    ),
    "zircon_plugins/native_window_hosting/editor/src/plugin.rs": (
        "25c1f412ea4aebead57d5d6def2f4f073c461eac"
    ),
    "zircon_plugins/native_window_hosting/editor/src/tests.rs": (
        "fe89b9cec6ae966ec80dd9a482c2ac695e28692c"
    ),
    "zircon_plugins/particles/runtime/src/service.rs": (
        "4ce627f3b5a05c20686aeca2c20a5298f3401dde"
    ),
    "zircon_plugins/particles/runtime/src/tests/snapshot.rs": (
        "52097e055e5f8af870529140f70cb0c724f72e05"
    ),
}


def run(
    registrations: int = 1_000,
    legacy_contributions_per_registration: int = 8,
    snapshot_iterations: int = 128,
    snapshot_sprites: int = 4_096,
    snapshot_diagnostics: int = 256,
) -> dict[str, object]:
    for name, value in (
        ("registrations", registrations),
        (
            "legacy_contributions_per_registration",
            legacy_contributions_per_registration,
        ),
        ("snapshot_iterations", snapshot_iterations),
        ("snapshot_sprites", snapshot_sprites),
        ("snapshot_diagnostics", snapshot_diagnostics),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    legacy_contributions = registrations * legacy_contributions_per_registration
    legacy_payload_clones = snapshot_iterations * (
        snapshot_sprites + snapshot_diagnostics
    )
    return {
        "schema": "zircon.plugins.sdk_window_particles_pressure.v1",
        "source_binding": source_binding(),
        "interpretation": {
            "evidence_kind": "deterministic_contract_and_hot_path_work_model",
            "implementation_evidence": False,
            "product_timing": False,
            "included": (
                "native static layout guards and allocations, phantom extension "
                "contributions and template resolutions, and particle payload clones"
            ),
            "excluded": (
                "compiler code generation, allocator constants, cache locality, lock "
                "contention, wall-clock duration, renderer work, and frame latency"
            ),
            "dynamic_acceptance_pending": (
                "one managed cross-package release batch; Plugins09 uses 21 paired "
                "samples and nearest-rank P95"
            ),
        },
        "inputs": {
            "registrations": registrations,
            "legacy_contributions_per_registration": (
                legacy_contributions_per_registration
            ),
            "snapshot_iterations": snapshot_iterations,
            "snapshot_sprites": snapshot_sprites,
            "snapshot_diagnostics": snapshot_diagnostics,
            "snapshot_sample_pairs": 21,
        },
        "sealed_native_static": {
            "baseline_blanket_sync_impl_count": 1,
            "candidate_blanket_sync_impl_count": 0,
            "candidate_audited_carrier_type_count": 5,
            "candidate_layout_overhead_bytes": 0,
            "candidate_runtime_guard_branch_count": 0,
            "candidate_runtime_allocation_count": 0,
        },
        "phantom_authoring": {
            "baseline_contribution_count": legacy_contributions,
            "candidate_contribution_count": 0,
            "baseline_missing_template_resolution_count": registrations,
            "candidate_missing_template_resolution_count": 0,
        },
        "particle_snapshot": {
            "baseline_large_payload_element_clone_count": legacy_payload_clones,
            "candidate_large_payload_element_clone_count": 0,
            "candidate_shared_handle_clone_count": snapshot_iterations * 2,
            "diagnostic_retention_limit": snapshot_diagnostics,
            "diagnostic_page_limit": 64,
        },
        "acceptance": {
            "native_static_zero_layout_overhead_required": True,
            "native_static_zero_runtime_allocations_required": True,
            "phantom_contributions_must_equal": 0,
            "phantom_template_resolutions_must_equal": 0,
            "particle_snapshot_p95_maximum_legacy_ratio": 0.25,
            "particle_release_timing_pending": True,
        },
        "invariants": {
            "native_static_public_layout_preserved": True,
            "native_static_interior_mutability_rejected": True,
            "native_window_package_and_capability_preserved": True,
            "particle_snapshot_payload_parity_preserved": True,
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
    parser.add_argument("--registrations", type=int, default=1_000)
    parser.add_argument(
        "--legacy-contributions-per-registration", type=int, default=8
    )
    parser.add_argument("--snapshot-iterations", type=int, default=128)
    parser.add_argument("--snapshot-sprites", type=int, default=4_096)
    parser.add_argument("--snapshot-diagnostics", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.registrations,
        args.legacy_contributions_per_registration,
        args.snapshot_iterations,
        args.snapshot_sprites,
        args.snapshot_diagnostics,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
