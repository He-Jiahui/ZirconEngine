#!/usr/bin/env python3
"""Model exact-node publication for externally authored UI geometry.

This is a deterministic operation-count model, not product timing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs",
        (
            "pub struct UiPersistentSequence",
            "UI_PERSISTENT_SEQUENCE_SEGMENT_SIZE: usize = 64",
            "UI_PERSISTENT_SEQUENCE_DIRECTORY_FANOUT: usize = 32",
            "pub fn get_mut_with_stats",
            "UiPersistentSequenceCowStats",
        ),
    ),
    (
        "zircon_runtime_interface/src/ui/surface/arranged.rs",
        (
            "pub roots: UiPersistentSequence<UiNodeId>",
            "pub nodes: UiPersistentSequence<UiArrangedNode>",
            "pub draw_order: UiPersistentSequence<UiNodeId>",
        ),
    ),
    (
        "zircon_runtime_interface/src/ui/surface/hit.rs",
        (
            "pub entries: UiPersistentSequence<UiHitTestEntry>",
            "pub cells: UiPersistentSequence<UiHitTestCell>",
            "pub struct UiHitTestCellEntries",
            "shared: Option<Arc<Vec<usize>>>",
        ),
    ),
    (
        "zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs",
        (
            "observed_topology_generation",
            "changed_node_ids",
            "surface_ordering_fallback_count",
            "publish_authored_geometry",
        ),
    ),
    (
        "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs",
        (
            "observed_topology_generation",
            "changed_node_ids",
            "publish_authored_geometry",
            "surface_geometry_local_publication_count",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/rebuild/authored_geometry.rs",
        (
            "pub enum UiAuthoredGeometryPublication",
            "pub enum UiAuthoredGeometryFallbackReason",
            "pub fn publish_authored_geometry",
            "patch_arranged_tree_geometry",
            "patch_arranged_geometry",
            "patch_geometry",
            "patch_projected_hit_test_strict",
            "hit_grid_regrid_count",
            "root_size_local_publication_count",
            "publish_surface_frame_after_rebuild",
        ),
    ),
    (
        "zircon_runtime/src/ui/tree/hit_test.rs",
        (
            "pub(crate) fn hit_grid_capacity_bounds",
            "HIT_GRID_CELL_SIZE",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/rebuild.rs",
        ("pub fn rebuild_authored_frames", "pub fn rebuild(&mut self)"),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs",
        ("pub fn rebuild_dirty", "patch_arranged_tree_geometry"),
    ),
    (
        "zircon_runtime/src/ui/surface/arranged.rs",
        (
            "pub(crate) fn patch_arranged_tree_geometry",
            "affected_node_ids",
            "get_mut_with_stats",
            "ui.arranged.persistent_cow_segment_clone_count",
        ),
    ),
    (
        "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs",
        (
            "fn patch_arranged_geometry",
            "entry_cells",
            "get_mut_with_stats",
            "ui.hit_grid.persistent_entry_segment_clone_count",
            "ui.hit_grid.persistent_cell_membership_clone_count",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/render/cache.rs",
        ("pub fn patch_geometry", "geometry_patchable_node_ids"),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
        (
            "mark_surface_frame_rebuild_dirty",
            "publish_surface_frame_after_rebuild",
            "self.arranged_tree.clone()",
            "authoritative_grid(&self.hit_test.grid)",
            "ui.surface_frame.arranged_node_clone_count",
            "ui.surface_frame.hit_entry_clone_count",
            "ui.surface_frame.arranged_segment_share_count",
            "ui.surface_frame.hit_cell_segment_share_count",
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
    node_count: int = 529,
    frame_patch_count: int = 1_000,
    topology_change_count: int = 10,
    changed_nodes_per_frame_patch: int = 1,
    resize_event_count: int = 120,
    resize_regrid_count: int = 2,
) -> dict[str, object]:
    if node_count <= 0:
        raise ValueError("node_count must be positive")
    if frame_patch_count < 0:
        raise ValueError("frame_patch_count must not be negative")
    if topology_change_count < 0:
        raise ValueError("topology_change_count must not be negative")
    if not 0 < changed_nodes_per_frame_patch <= node_count:
        raise ValueError("changed_nodes_per_frame_patch must be within the surface")
    if resize_event_count < 0:
        raise ValueError("resize_event_count must not be negative")
    if not 0 <= resize_regrid_count <= resize_event_count:
        raise ValueError("resize_regrid_count must be within resize_event_count")

    rebuild_request_count = frame_patch_count + topology_change_count
    current_domain_visits = rebuild_request_count * node_count
    topology_domain_visits = topology_change_count * node_count
    local_domain_visits = frame_patch_count * changed_nodes_per_frame_patch
    target_domain_visits = topology_domain_visits + local_domain_visits
    avoided_domain_visits = current_domain_visits - target_domain_visits
    segment_size = 64
    local_segment_item_copy_upper_bound = frame_patch_count * min(
        node_count, changed_nodes_per_frame_patch * segment_size
    )
    persistent_item_copy_upper_bound = (
        topology_domain_visits + local_segment_item_copy_upper_bound
    )
    resize_current_domain_visits = resize_event_count * node_count
    resize_local_domain_visits = (
        resize_regrid_count * node_count
        + (resize_event_count - resize_regrid_count) * changed_nodes_per_frame_patch
    )

    return {
        "schema": "zircon.runtime.ui.authored_geometry_delta_pressure.v4",
        "evidence_kind": "deterministic_operation_count_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "node_count": node_count,
            "frame_patch_count": frame_patch_count,
            "topology_change_count": topology_change_count,
            "changed_nodes_per_frame_patch": changed_nodes_per_frame_patch,
            "rebuild_request_count": rebuild_request_count,
            "persistent_segment_size": segment_size,
            "resize_event_count": resize_event_count,
            "resize_regrid_count": resize_regrid_count,
        },
        "current_authored_frame_publication": {
            "full_pipeline_rebuild_count": rebuild_request_count,
            "arranged_node_visit_count": current_domain_visits,
            "hit_node_visit_count": current_domain_visits,
            "render_node_visit_count": current_domain_visits,
            "topology_validation_node_visit_count": current_domain_visits,
            "published_arranged_node_clone_count": current_domain_visits,
            "published_hit_entry_clone_count": current_domain_visits,
            "internal_complexity": "O((F + T) * N) in each arranged/hit/render domain",
            "published_snapshot_complexity": "O((F + T) * (N + E + C)) for arranged nodes, hit entries, and hit-cell references",
        },
        "runtime_exact_geometry_publication": {
            "full_pipeline_rebuild_count": topology_change_count,
            "exact_geometry_patch_count": frame_patch_count,
            "arranged_node_visit_count": target_domain_visits,
            "hit_node_visit_count": target_domain_visits,
            "render_node_visit_count": target_domain_visits,
            "topology_validation_node_visit_count": current_domain_visits,
            "published_arranged_node_clone_count": 0,
            "published_hit_entry_clone_count": 0,
            "persistent_arranged_item_copy_upper_bound": persistent_item_copy_upper_bound,
            "persistent_hit_entry_item_copy_upper_bound": persistent_item_copy_upper_bound,
            "internal_complexity": "O(T * N + F * K) in mutable arranged/hit/render authority",
            "published_snapshot_complexity": "O(1) root sharing at publication; producer mutation is bounded by touched 64-item leaves, directory paths, and hit-cell leaves",
        },
        "persistent_published_domain_end_state": {
            "full_pipeline_rebuild_count": topology_change_count,
            "exact_geometry_patch_count": frame_patch_count,
            "arranged_logical_update_count": target_domain_visits,
            "hit_entry_logical_update_count": target_domain_visits,
            "stable_frame_full_arranged_clone_count": 0,
            "stable_frame_full_hit_clone_count": 0,
            "arranged_item_copy_upper_bound": persistent_item_copy_upper_bound,
            "hit_entry_item_copy_upper_bound": persistent_item_copy_upper_bound,
            "complexity": "O(T * N + F * (K + S + H)) where S is copied persistent segments and H is touched hit-cell membership",
        },
        "editor_typed_delta_end_state": {
            "full_pipeline_rebuild_count": topology_change_count,
            "exact_geometry_patch_count": frame_patch_count,
            "topology_validation_node_visit_count": topology_domain_visits,
            "geometry_identity_check_count": local_domain_visits,
            "complexity": "O(T * N + F * K) for classification and each publication domain",
        },
        "window_resize_capacity_envelope": {
            "current_full_pipeline_rebuild_count": resize_event_count,
            "current_hit_arranged_node_visit_count": resize_current_domain_visits,
            "candidate_hit_capacity_regrid_count": resize_regrid_count,
            "candidate_hit_node_visit_count": resize_local_domain_visits,
            "avoided_hit_node_visit_count": resize_current_domain_visits
            - resize_local_domain_visits,
            "complexity": "O(R * K + G * N) where R is resize events, K is changed geometry, and G is capacity-boundary regrids",
        },
        "delta": {
            "avoided_arranged_node_visit_count": avoided_domain_visits,
            "avoided_hit_node_visit_count": avoided_domain_visits,
            "avoided_render_node_visit_count": avoided_domain_visits,
            "avoided_internal_domain_node_visit_count": avoided_domain_visits * 3,
            "runtime_only_avoided_published_arranged_node_clone_count": current_domain_visits,
            "runtime_only_avoided_published_hit_entry_clone_count": current_domain_visits,
        },
        "typed_fallbacks": (
            "missing_node_or_arranged_index",
            "topology_generation_changed",
            "clip_descendant_expansion_failed",
        "hit_grid_capacity_regrid",
            "render_command_not_geometry_patchable",
            "projected_hit_or_navigation_patch_failed",
        ),
        "interpretation": {
            "included": "mutable arranged, hit, and render node visits; topology validation; exact changed-node identity checks; topology fallbacks; zero-copy publication roots; and conservative 64-item persistent leaf-copy bounds",
            "excluded": "actual CPU time, exact hit-cell reference cardinality, render command cardinality, allocation bytes, GPU work, present latency, and driver residency",
            "required_product_evidence": "typed geometry/full fallback counters, affected-node and hit-cell counts, patched render ranges, arranged/hit snapshot clone counts, persistent segment clone counts, published domain generations, and input-to-present p50/p95/p99",
        },
    }


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must be written to D:, E:, or F:")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--node-count", type=int, default=529)
    parser.add_argument("--frame-patch-count", type=int, default=1_000)
    parser.add_argument("--topology-change-count", type=int, default=10)
    parser.add_argument("--changed-nodes-per-frame-patch", type=int, default=1)
    parser.add_argument("--resize-event-count", type=int, default=120)
    parser.add_argument("--resize-regrid-count", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        node_count=args.node_count,
        frame_patch_count=args.frame_patch_count,
        topology_change_count=args.topology_change_count,
        changed_nodes_per_frame_patch=args.changed_nodes_per_frame_patch,
        resize_event_count=args.resize_event_count,
        resize_regrid_count=args.resize_regrid_count,
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
