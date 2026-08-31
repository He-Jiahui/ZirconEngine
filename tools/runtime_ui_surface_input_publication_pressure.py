#!/usr/bin/env python3
"""Model cross-surface UI input publication work.

This is a deterministic algorithm-pressure model, not measured product timing.
It separates the legacy all-event RuntimeUiSurfaceSet fanout from the current
retained surface-directory/focus/navigation-owner cutover.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import subprocess


CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
        (
            "focused_surfaces: BTreeSet<usize>",
            "focused_surface: Option<usize>",
            "navigation_surface: Option<usize>",
            "input_publication: RuntimeUiInputPublication",
            "if matches!(&event, UiInputEvent::MouseMotion(_))",
            "ui.surface_set.input.unrouted_reject_count",
            "input_requires_focus_owner(&event)",
            "input_requires_navigation_owner(&event)",
            "ui.surface_set.input.focus_direct_route_count",
            "ui.surface_set.input.navigation_direct_route_count",
            "fn refresh_input_owners_from_publication(&mut self)",
            "self.publish_input_authority(viewport_size)",
            ".query(viewport_size, point, previous_point)",
            "RuntimeUiInputQueryAdmission::Published(query)",
            "RuntimeUiInputQueryAdmission::Unpublished",
            "RuntimeUiInputQueryAdmission::Rejected(reason)",
            "ui.surface_set.input.invalid_pointer_reject_count",
            "ui.surface_set.input.publication_unavailable_fallback_count",
            "query.hit_test_query()",
            ".candidate_surface(query, candidate_offset)",
            "dispatch_input_to_surface(surface_index, root_size, event, false)",
            "focus_before != focus_after",
            "for surface_index in (0..self.surfaces.len()).rev()",
            "runtime_surface.rebuild_dirty(root_size)?",
            "event.as_ref().cloned()",
            "synchronize_text_document_owners(&mut self.surface);",
        ),
    ),
    (
        "zircon_runtime/src/dynamic_api/session/runtime_ui/input_publication.rs",
        (
            "pub(super) struct RuntimeUiInputPublication",
            "pub(super) enum RuntimeUiInputQueryAdmission",
            "Published(RuntimeUiInputQuery)",
            "Rejected(RuntimeUiInputQueryRejectReason)",
            "surface_hit_generations: Vec<u64>",
            "cells: Vec<Vec<u32>>",
            "cell_visit_stamps: Vec<u32>",
            "next_cell_visit_stamp: u32",
            "surface_footprints: Vec<Vec<u32>>",
            "std::mem::take(&mut self.surface_footprints[surface_index])",
            "pub(super) fn publish(",
            "pub(super) fn query(",
            "pub(super) fn candidate_surface(",
            "fn visit_bounded_cells(",
            "physical_point: UiPoint",
            "virtual_pointer: Option<UiVirtualPointerPosition>",
            "map_pointer_axis(",
        ),
    ),
    (
        "zircon_runtime/src/dynamic_api/session/events.rs",
        (
            "match self.dispatch_runtime_ui_event(|metadata|",
            "UiInputEvent::MouseMotion(UiMouseMotionInputEvent",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/input/mouse_motion.rs",
        (
            "UiDispatchReply::unhandled()",
            "result.diagnostics.route_policy = UiInputRoutePolicy::Unrouted;",
        ),
    ),
    (
        "zircon_runtime/src/ui/dispatch/input_manager/manager.rs",
        (
            "pub fn dispatch_input_event(",
            "pub(crate) fn dispatch_input_event_with_query(",
            "pointer_query: Option<UiHitTestQuery>",
            "self.synchronize_text_document_owners(surface);",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/input/dispatch.rs",
        (
            "pointer_query: Option<UiHitTestQuery>",
            "dispatch_pointer_input(",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/input/pointer.rs",
        (
            "pointer_query: Option<UiHitTestQuery>",
            "dispatch_pointer_event_with_query_and_modifiers(",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/surface/event_routing.rs",
        (
            "pub(crate) fn dispatch_pointer_event_with_query_and_modifiers(",
            "query: UiHitTestQuery",
            "self.route_pointer_event_with_details(",
            "pub(crate) fn has_navigation_candidate(&self)",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/navigation_index.rs",
        (
            "pub(super) fn has_navigation_candidate(&self)",
            "!self.spatial_all.is_empty()",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/input/window_pump.rs",
        (
            "dispatch_input_event(",
            "input,\n            None,\n            None,",
        ),
    ),
)


class SourceContractError(RuntimeError):
    """Raised when the pressure model no longer describes current source."""


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


def _validate_current_fanout_contract(relative_path: str, source: str) -> None:
    if not relative_path.endswith("dynamic_api/session/runtime_ui.rs"):
        return
    reverse_loop = "for surface_index in (0..self.surfaces.len()).rev()"
    if source.count(reverse_loop) < 2:
        raise SourceContractError(
            "generic and uncaptured-pointer reverse Surface fanouts are no longer both present"
        )
    raw_motion_fast_path = "if matches!(&event, UiInputEvent::MouseMotion(_))"
    if source.index(raw_motion_fast_path) > source.index(reverse_loop):
        raise SourceContractError(
            "raw MouseMotion rejection no longer precedes generic Surface fanout"
        )
    if source.index("input_requires_focus_owner(&event)") > source.index(reverse_loop):
        raise SourceContractError(
            "focused input direct route no longer precedes generic Surface fanout"
        )
    if source.index("input_requires_navigation_owner(&event)") > source.index(reverse_loop):
        raise SourceContractError(
            "navigation/analog direct route no longer precedes generic Surface fanout"
        )
    pointer_query = ".query(viewport_size, point, previous_point)"
    if source.index(pointer_query) > source.rindex(reverse_loop):
        raise SourceContractError(
            "pointer publication query no longer precedes legacy fallback fanout"
        )
    rejected_admission = "RuntimeUiInputQueryAdmission::Rejected(reason)"
    if source.index(rejected_admission) > source.rindex(reverse_loop):
        raise SourceContractError(
            "invalid pointer rejection no longer precedes legacy fallback fanout"
        )


def source_binding_report(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    sources = []
    relative_paths = []
    source_set = hashlib.sha256()
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
        _validate_current_fanout_contract(relative_path, source)
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


def validate_output_path(path: Path) -> None:
    if path.drive.upper() == "C:":
        raise ValueError("profile artifacts must not be written to C:")


def pressure_report(
    surface_count: int = 64,
    candidate_surface_count: int = 2,
    pointer_events: int = 100_000,
    focused_events: int = 100_000,
    navigation_events: int = 100_000,
    unrouted_events: int = 100_000,
    dirty_surface_count: int | None = None,
    nodes_per_dirty_surface: int = 10_000,
    viewport_width: int = 1_920,
    viewport_height: int = 1_080,
    cell_size: int = 64,
    occupied_cells_per_surface: int | None = None,
) -> dict[str, object]:
    positive = {
        "surface_count": surface_count,
        "pointer_events": pointer_events,
        "focused_events": focused_events,
        "navigation_events": navigation_events,
        "unrouted_events": unrouted_events,
        "nodes_per_dirty_surface": nodes_per_dirty_surface,
        "viewport_width": viewport_width,
        "viewport_height": viewport_height,
        "cell_size": cell_size,
    }
    for name, value in positive.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if dirty_surface_count is None:
        dirty_surface_count = surface_count
    if not 0 <= candidate_surface_count <= surface_count:
        raise ValueError("candidate_surface_count must be within surface_count")
    if not 0 <= dirty_surface_count <= surface_count:
        raise ValueError("dirty_surface_count must be within surface_count")

    columns = math.ceil(viewport_width / cell_size)
    rows = math.ceil(viewport_height / cell_size)
    cell_count = columns * rows
    if occupied_cells_per_surface is None:
        occupied_cells_per_surface = cell_count
    if not 0 <= occupied_cells_per_surface <= cell_count:
        raise ValueError("occupied_cells_per_surface must be within the viewport grid")

    baseline_pointer_dispatches = surface_count * pointer_events
    baseline_focused_dispatches = surface_count * focused_events
    baseline_unrouted_dispatches = surface_count * unrouted_events
    baseline_navigation_dispatches = surface_count * navigation_events
    baseline_dispatches = (
        baseline_pointer_dispatches
        + baseline_focused_dispatches
        + baseline_navigation_dispatches
        + baseline_unrouted_dispatches
    )
    baseline_event_clones = (surface_count - 1) * (
        pointer_events + focused_events + navigation_events + unrouted_events
    )

    current_pointer_dispatches = candidate_surface_count * pointer_events
    current_focused_dispatches = focused_events
    current_navigation_dispatches = navigation_events
    current_dispatches = (
        current_pointer_dispatches
        + current_focused_dispatches
        + current_navigation_dispatches
    )
    current_event_clones = max(candidate_surface_count - 1, 0) * pointer_events

    published_pointer_dispatches = candidate_surface_count * pointer_events
    published_focused_dispatches = focused_events
    published_navigation_dispatches = navigation_events
    published_dispatches = (
        published_pointer_dispatches
        + published_focused_dispatches
        + published_navigation_dispatches
    )
    published_event_clones = max(candidate_surface_count - 1, 0) * pointer_events

    current_first_event_node_scale_work = (
        dirty_surface_count * nodes_per_dirty_surface
    )
    published_pre_input_node_scale_work = current_first_event_node_scale_work

    membership_count = surface_count * occupied_cells_per_surface
    u32_bytes = 4
    # Payload estimate includes cell memberships and the reverse per-surface
    # footprints needed for local patching. Arc/container overhead is excluded.
    publication_payload_bytes = (
        u32_bytes * (cell_count + 1) + 2 * u32_bytes * membership_count
    )

    return {
        "schema": "zircon.runtime.ui_surface_input_publication_pressure.v11",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            **positive,
            "candidate_surface_count": candidate_surface_count,
            "dirty_surface_count": dirty_surface_count,
            "occupied_cells_per_surface": occupied_cells_per_surface,
        },
        "legacy_reverse_surface_fanout_baseline": {
            "pointer_surface_dispatches": baseline_pointer_dispatches,
            "focused_surface_dispatches": baseline_focused_dispatches,
            "navigation_surface_dispatches": baseline_navigation_dispatches,
            "unrouted_mouse_motion_surface_dispatches": baseline_unrouted_dispatches,
            "combined_surface_dispatches": baseline_dispatches,
            "event_payload_clones": baseline_event_clones,
            "event_path_rebuild_probes": baseline_dispatches,
            "text_owner_sync_calls": 2 * baseline_dispatches,
            "unrouted_diagnostic_note_allocations": 2 * baseline_unrouted_dispatches,
            "dirty_first_event_node_scale_work": current_first_event_node_scale_work,
            "pointer_complexity": "O(S + sum(candidate cell work) + dirty rebuild work)",
            "focused_complexity": "O(S + dirty rebuild work)",
            "unrouted_complexity": "O(S + dirty rebuild work)",
        },
        "current_input_publication_cutover": {
            "pointer_directory_queries": pointer_events,
            "pointer_surface_dispatches": current_pointer_dispatches,
            "focused_owner_surface_dispatches": current_focused_dispatches,
            "navigation_owner_surface_dispatches": current_navigation_dispatches,
            "unrouted_mouse_motion_surface_dispatches": 0,
            "unrouted_session_counter_updates": unrouted_events,
            "combined_surface_dispatches": current_dispatches,
            "event_payload_clones": current_event_clones,
            "event_path_rebuild_probes": 0,
            "text_owner_sync_calls": current_dispatches,
            "unrouted_diagnostic_note_allocations": 0,
            "dirty_first_event_node_scale_work": 0,
            "pre_input_publication_node_scale_work": published_pre_input_node_scale_work,
            "pointer_complexity": "O(1 + C + sum(candidate cell work))",
            "focused_complexity": "O(1 + focused route depth)",
            "navigation_complexity": "O(1 + navigation route depth)",
            "unrouted_complexity": "O(1)",
        },
        "retained_input_publication": {
            "pointer_directory_queries": pointer_events,
            "pointer_surface_dispatches": published_pointer_dispatches,
            "focused_owner_surface_dispatches": published_focused_dispatches,
            "navigation_owner_surface_dispatches": published_navigation_dispatches,
            "unrouted_mouse_motion_surface_dispatches": 0,
            "combined_surface_dispatches": published_dispatches,
            "event_payload_clones": published_event_clones,
            "event_path_rebuild_probes": 0,
            "text_owner_sync_calls": published_dispatches,
            "unrouted_session_counter_updates": unrouted_events,
            "dirty_first_event_node_scale_work": 0,
            "pre_input_publication_node_scale_work": published_pre_input_node_scale_work,
            "pointer_complexity": "O(1 + C + sum(candidate cell work))",
            "focused_complexity": "O(1 + focused route depth)",
            "navigation_complexity": "O(1 + navigation route depth)",
            "unrouted_complexity": "O(1)",
        },
        "publication_memory_payload_estimate": {
            "columns": columns,
            "rows": rows,
            "cell_count": cell_count,
            "surface_cell_membership_count": membership_count,
            "u32_payload_bytes": publication_payload_bytes,
            "complexity": "O(U + M), without duplicating hit entries or route nodes",
        },
        "publication_patch_scratch": {
            "scenario": "warm hit-generation patch with same-or-smaller Surface footprints",
            "cell_count": cell_count,
            "removed_per_patch_occupancy_allocations": dirty_surface_count,
            "removed_per_patch_occupancy_bytes": dirty_surface_count * cell_count,
            "removed_per_patch_footprint_allocations": dirty_surface_count,
            "current_warm_occupancy_allocations": 0,
            "current_warm_footprint_allocations": 0,
            "retained_stamp_scratch_bytes": u32_bytes * cell_count,
            "removed_footprint_sort_invocations": dirty_surface_count,
            "removed_footprint_sort_input_items": membership_count,
            "current_footprint_sort_invocations": 0,
            "removed_per_entry_cell_vector_allocations": current_first_event_node_scale_work,
            "current_per_entry_cell_vector_allocations": 0,
            "overflow_policy": "clear the retained stamp array once after u32 wrap",
            "is_product_timing": False,
        },
        "fallback_admission_policy": {
            "unpublished_pointer_policy": "typed cold reverse-fanout compatibility fallback",
            "invalid_pointer_policy": "O(1) reject before Surface dispatch",
            "invalid_pointer_surface_dispatches": 0,
            "invalid_pointer_event_path_rebuild_probes": 0,
        },
        "delta": {
            "implemented_avoided_surface_dispatches": baseline_dispatches
            - current_dispatches,
            "implemented_avoided_event_payload_clones": baseline_event_clones
            - current_event_clones,
            "implemented_avoided_event_path_rebuild_probes": baseline_dispatches,
            "implemented_avoided_text_owner_sync_calls": 2 * baseline_dispatches
            - current_dispatches,
            "remaining_surface_dispatches_to_remove": current_dispatches
            - published_dispatches,
            "remaining_event_payload_clones_to_remove": current_event_clones
            - published_event_clones,
            "remaining_event_path_rebuild_probes_to_remove": 0,
            "remaining_text_owner_sync_calls_to_remove": current_dispatches
            - published_dispatches,
            "baseline_to_target_surface_dispatch_ratio": baseline_dispatches
            / published_dispatches,
            "current_to_target_surface_dispatch_ratio": current_dispatches
            / published_dispatches,
            "dirty_node_scale_work_shifted_before_input": current_first_event_node_scale_work,
        },
        "interpretation": {
            "included": "legacy all-event reverse fanout; current raw-motion O(1) rejection, direct focused and navigation/analog owner routing, incremental cell-to-surface pointer publication, retained cell-visit stamps, warm footprint-buffer reuse, removal of per-entry cell-vector allocation and the consumer-independent footprint sort, resize-time affine virtual-pointer lookup against the last published viewport, and typed unpublished-versus-invalid pointer admission; clone/rebuild/synchronization call counts visible in current source; true pointer candidate fallthrough; and node-scale dirty work shifted to a pre-input publication boundary",
            "excluded": "CPU time, allocator latency, RSS, cache behavior, route depth constants, exact layout/render rebuild cost, Arc/container overhead, concurrent publication, transparent hit semantics inside each candidate surface, and measured Editor latency",
            "required_product_evidence": "current-source CPU/RSS plus pointer and focused-input p50/p95/p99 with 1/4/16/64 surfaces, candidate counts, event allocations, event-path rebuild count, and publication patch counts",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--surface-count", type=int, default=64)
    parser.add_argument("--candidate-surface-count", type=int, default=2)
    parser.add_argument("--pointer-events", type=int, default=100_000)
    parser.add_argument("--focused-events", type=int, default=100_000)
    parser.add_argument("--navigation-events", type=int, default=100_000)
    parser.add_argument("--unrouted-events", type=int, default=100_000)
    parser.add_argument("--dirty-surface-count", type=int)
    parser.add_argument("--nodes-per-dirty-surface", type=int, default=10_000)
    parser.add_argument("--viewport-width", type=int, default=1_920)
    parser.add_argument("--viewport-height", type=int, default=1_080)
    parser.add_argument("--cell-size", type=int, default=64)
    parser.add_argument("--occupied-cells-per-surface", type=int)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = pressure_report(
        surface_count=args.surface_count,
        candidate_surface_count=args.candidate_surface_count,
        pointer_events=args.pointer_events,
        focused_events=args.focused_events,
        navigation_events=args.navigation_events,
        unrouted_events=args.unrouted_events,
        dirty_surface_count=args.dirty_surface_count,
        nodes_per_dirty_surface=args.nodes_per_dirty_surface,
        viewport_width=args.viewport_width,
        viewport_height=args.viewport_height,
        cell_size=args.cell_size,
        occupied_cells_per_surface=args.occupied_cells_per_surface,
    )
    result["source_binding"] = source_binding_report(Path(__file__).resolve().parents[1])
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        validate_output_path(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
