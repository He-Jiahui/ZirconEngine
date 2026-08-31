#!/usr/bin/env python3
"""Model retained Viewport Toolbar pointer-surface frame patches.

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
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs",
        (
            "fn try_patch_retained_surface_frames",
            "rebuild_authored_frames(",
            "fn rebuild_surface_from_scratch",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/handle_click.rs",
        ("sync_clicked_control", "self.rebuild_surface();"),
    ),
    (
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/sync_surface_frame.rs",
        ("applied_surface_frames", "self.rebuild_surface();"),
    ),
    (
        "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
        (
            "pub fn get_mut(&mut self, node_id: &UiNodeId)",
            "self.mutation_node_ids.insert(*node_id);",
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
    surface_count: int = 16,
    controls_per_surface: int = 32,
    frame_patch_count: int = 1_000,
    topology_change_count: int = 10,
) -> dict[str, object]:
    for name, value in {
        "surface_count": surface_count,
        "controls_per_surface": controls_per_surface,
    }.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    for name, value in {
        "frame_patch_count": frame_patch_count,
        "topology_change_count": topology_change_count,
    }.items():
        if value < 0:
            raise ValueError(f"{name} must not be negative")

    control_count = surface_count * controls_per_surface
    node_count = 1 + surface_count + control_count
    rebuild_request_count = frame_patch_count + topology_change_count

    current_node_allocations = rebuild_request_count * node_count
    current_route_materializations = rebuild_request_count * control_count
    target_node_allocations = topology_change_count * node_count
    target_route_materializations = topology_change_count * control_count

    return {
        "schema": "zircon.editor.viewport_toolbar_pointer_surface_reuse_pressure.v1",
        "evidence_kind": "deterministic_operation_and_allocation_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "surface_count": surface_count,
            "controls_per_surface": controls_per_surface,
            "control_count": control_count,
            "node_count": node_count,
            "frame_patch_count": frame_patch_count,
            "topology_change_count": topology_change_count,
            "rebuild_request_count": rebuild_request_count,
        },
        "current_full_reconstruction": {
            "surface_object_reconstruction_count": rebuild_request_count,
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "full_pipeline_node_visit_pressure": rebuild_request_count * node_count,
            "node_allocation_count": current_node_allocations,
            "route_materialization_count": current_route_materializations,
            "dispatcher_registration_count": current_route_materializations,
            "complexity": "O((F + T) * (S + C)) construction plus authored-frame full pipeline work",
        },
        "retained_surface_frame_patch": {
            "surface_object_reconstruction_count": topology_change_count,
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "retained_frame_patch_count": frame_patch_count,
            "topology_validation_node_visit_count": rebuild_request_count * node_count,
            "dirty_node_patch_count": frame_patch_count,
            "full_pipeline_node_visit_pressure": rebuild_request_count * node_count,
            "node_allocation_count": target_node_allocations,
            "route_materialization_count": target_route_materializations,
            "dispatcher_registration_count": target_route_materializations,
            "complexity": "O((F + T) * (S + C)) identity validation and authored-frame publication + O(F) frame writes + O(T * (S + C)) reconstruction",
        },
        "delta": {
            "avoided_surface_object_reconstruction_count": frame_patch_count,
            "avoided_full_pipeline_rebuild_count": 0,
            "avoided_node_allocation_count": current_node_allocations
            - target_node_allocations,
            "avoided_route_materialization_count": current_route_materializations
            - target_route_materializations,
            "avoided_dispatcher_registration_count": current_route_materializations
            - target_route_materializations,
        },
        "residual_cost": {
            "authored_frame_full_pipeline_rebuild_count": rebuild_request_count,
            "full_pipeline_node_visit_pressure": rebuild_request_count * node_count,
            "requires_runtime_geometry_patch_api": True,
            "reason": "UiSurface exposes local geometry patching only through layout-owned rebuild_dirty; this bridge owns already-computed frames and must keep authored-frame publication semantics",
        },
        "interpretation": {
            "included": "root/surface/control node construction, route materialization, dispatcher registration, topology validation, changed-frame dirty patches, and explicit topology fallbacks",
            "excluded": "actual CPU time, allocator RSS, event dispatch, hit-test query time, rendering, GPU work, and product latency",
            "required_product_evidence": "toolbar rebuild request kind, retained patch/full fallback counters, changed node count, authored-frame arranged/hit/render visits, allocation bytes, and click input-to-present p50/p95/p99",
        },
    }


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must be written to D:, E:, or F:")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--surface-count", type=int, default=16)
    parser.add_argument("--controls-per-surface", type=int, default=32)
    parser.add_argument("--frame-patch-count", type=int, default=1_000)
    parser.add_argument("--topology-change-count", type=int, default=10)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        surface_count=args.surface_count,
        controls_per_surface=args.controls_per_surface,
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
