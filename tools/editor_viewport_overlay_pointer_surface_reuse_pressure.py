#!/usr/bin/env python3
"""Model retained viewport-overlay pointer candidate authority reuse.

This is a deterministic operation/allocation pressure model, not product timing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs",
        (
            "fn try_patch_retained_surface",
            "fn retained_surface_topology_matches",
            "retained_candidate_count",
            "candidate_route_identity_changed",
            "fn publish_retained_candidates",
            "candidate_authority_map_reuse_count",
            "candidate_authority_map_rebuild_count",
            ".get(&candidate.node_id)",
            "current.route != candidate.candidate.route",
            "self.surface.release_pointer_capture()",
            'strip_prefix("editor.viewport.pointer/candidate_")',
            "fn rebuild_surface_from_scratch",
            "rebuild_authored_frames(",
            "viewport.pointer.surface_authority_reuse_count",
            "viewport.pointer.surface_authority_rebuild_count",
        ),
    ),
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/"
        "viewport_overlay_pointer_router_sync.rs",
        (
            "if self.layout == layout",
            "self.rebuild_surface();",
        ),
    ),
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/"
        "viewport_overlay_pointer_router.rs",
        ("retained_candidate_count",),
    ),
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/"
        "viewport_overlay_pointer_router_new.rs",
        ("retained_candidate_count: 0",),
    ),
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/"
        "viewport_overlay_pointer_router_clone.rs",
        ("clone.retained_candidate_count",),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/rebuild.rs",
        (
            "pub fn rebuild_authored_frames",
            "self.rebuild();",
        ),
    ),
)


class SourceContractError(RuntimeError):
    """Raised when current source no longer matches the modeled authority."""


def _sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest().upper()


def _git_output(repo_root: Path, *args: str) -> str | None:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return completed.stdout.strip()


def source_binding_report(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    sources = []
    source_set = hashlib.sha256()
    relative_paths = []
    for relative_path, required_tokens in CRITICAL_SOURCE_CONTRACTS:
        path = repo_root / relative_path
        try:
            payload = path.read_bytes()
        except OSError as error:
            raise SourceContractError(f"missing critical source: {relative_path}") from error
        source = payload.decode("utf-8")
        missing = [token for token in required_tokens if token not in source]
        if missing:
            raise SourceContractError(
                f"critical source contract changed: {relative_path}: {missing}"
            )
        if relative_path.endswith("overlay_router/rebuild_surface.rs"):
            route_guard_start = source.index("let candidate_route_identity_changed")
            route_guard_end = source.index("let mut changed_node_count", route_guard_start)
            route_guard = source[route_guard_start:route_guard_end]
            forbidden_route_allocations = [
                token for token in (".collect::<Vec", "Vec::", ".clone()")
                if token in route_guard
            ]
            forbidden_node_id_accessors = [
                token for token in (".raw()", "UiNodeId::raw") if token in source
            ]
            if forbidden_route_allocations or forbidden_node_id_accessors:
                raise SourceContractError(
                    "critical retained route/node identity contract changed: "
                    f"{relative_path}: route_allocations={forbidden_route_allocations}, "
                    f"node_id_accessors={forbidden_node_id_accessors}"
                )
        digest = _sha256(payload)
        sources.append(
            {
                "relative_path": relative_path,
                "sha256": digest,
                "byte_length": len(payload),
            }
        )
        relative_paths.append(relative_path)
        source_set.update(relative_path.encode("utf-8"))
        source_set.update(b"\0")
        source_set.update(digest.encode("ascii"))
        source_set.update(b"\n")

    dirty_output = _git_output(repo_root, "status", "--porcelain=v1", "--", *relative_paths)
    dirty_entries = [] if not dirty_output else dirty_output.splitlines()
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD"),
        "critical_sources_dirty": bool(dirty_entries),
        "critical_source_dirty_entry_count": len(dirty_entries),
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": sources,
    }


def pressure_report(
    candidate_count: int = 4_096,
    frame_patch_count: int = 1_000,
    topology_change_count: int = 10,
) -> dict[str, object]:
    if candidate_count <= 0:
        raise ValueError("candidate_count must be positive")
    for name, value in {
        "frame_patch_count": frame_patch_count,
        "topology_change_count": topology_change_count,
    }.items():
        if value < 0:
            raise ValueError(f"{name} must not be negative")

    node_count = candidate_count + 2
    rebuild_request_count = frame_patch_count + topology_change_count
    current_node_allocations = rebuild_request_count * node_count
    retained_node_allocations = topology_change_count * node_count

    return {
        "schema": "zircon.editor.viewport_overlay_pointer_surface_reuse_pressure.v1",
        "evidence_kind": "deterministic_operation_and_allocation_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "candidate_count": candidate_count,
            "node_count": node_count,
            "frame_patch_count": frame_patch_count,
            "topology_change_count": topology_change_count,
            "rebuild_request_count": rebuild_request_count,
        },
        "current_full_reconstruction": {
            "surface_object_reconstruction_count": rebuild_request_count,
            "node_allocation_count": current_node_allocations,
            "node_path_materialization_count": current_node_allocations,
            "candidate_projection_count": rebuild_request_count * candidate_count,
            "candidate_map_materialization_count": rebuild_request_count * candidate_count,
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "full_pipeline_node_visit_pressure_per_domain": rebuild_request_count
            * node_count,
            "complexity": "O((F + T) * C) projection, tree construction, and authored-frame full pipeline work",
        },
        "retained_candidate_authority": {
            "surface_object_reconstruction_count": topology_change_count,
            "node_allocation_count": retained_node_allocations,
            "node_path_materialization_count": retained_node_allocations,
            "candidate_projection_count": rebuild_request_count * candidate_count,
            "candidate_map_materialization_count": topology_change_count * candidate_count,
            "candidate_map_value_patch_count": frame_patch_count * candidate_count,
            "topology_validation_node_visit_count": rebuild_request_count * node_count,
            "retained_frame_patch_probe_count": frame_patch_count * node_count,
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "full_pipeline_node_visit_pressure_per_domain": rebuild_request_count
            * node_count,
            "complexity": "O((F + T) * C) projection and exact topology validation plus O(F * C) geometry patch probes and O(T * C) construction",
        },
        "delta": {
            "avoided_surface_object_reconstruction_count": frame_patch_count,
            "avoided_node_allocation_count": current_node_allocations
            - retained_node_allocations,
            "avoided_node_path_materialization_count": current_node_allocations
            - retained_node_allocations,
            "avoided_candidate_projection_count": 0,
            "avoided_candidate_map_materialization_count": frame_patch_count
            * candidate_count,
            "avoided_full_pipeline_rebuild_count": 0,
        },
        "residual_cost": {
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "full_pipeline_node_visit_pressure_per_domain": rebuild_request_count
            * node_count,
            "requires_runtime_geometry_patch_api": True,
            "reason": "UiSurface::rebuild_authored_frames still delegates to the full arranged/hit/render rebuild authority",
        },
        "interpretation": {
            "included": "precision candidate projection, retained candidate-map value publication, exact retained topology validation, candidate-route capture ownership validation, node/path construction, and authored-frame publication",
            "excluded": "actual CPU time, allocator RSS, pointer dispatch latency, renderer-visible spatial queries, rendering, GPU work, and product latency",
            "required_product_evidence": "candidate count, retained reuse/fallback counts, geometry patch count, authored-frame arranged/hit/render visits, allocation bytes, and pointer input-to-present p50/p95/p99",
        },
    }


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must be written to D:, E:, or F:")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--candidate-count", type=int, default=4_096)
    parser.add_argument("--frame-patch-count", type=int, default=1_000)
    parser.add_argument("--topology-change-count", type=int, default=10)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        candidate_count=args.candidate_count,
        frame_patch_count=args.frame_patch_count,
        topology_change_count=args.topology_change_count,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[1])
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        output = validate_output_path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
