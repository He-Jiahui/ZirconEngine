#!/usr/bin/env python3
"""Source-bound pressure model for UI tree and hit-grid admission costs.

This tool reports deterministic algorithm work and allocation cardinality. It
does not report product timing, memory use, or interactive latency.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_CONTRACTS = (
    (
        "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
        (
            "paint_order_cursor: PaintOrderCursor",
            "self.paint_order_cursor.invalidate();",
            ".rebuild(self.nodes.values().map(|node| node.paint_order))",
            "self.next = next.saturating_add(1);",
        ),
    ),
    (
        "zircon_runtime/src/ui/tree/hit_test.rs",
        (
            "const HIT_GRID_MAX_AXIS_CELLS: u32 = 128;",
            "const HIT_GRID_MAX_ENTRY_CELL_COUNT: usize = 4_096;",
            "pub(crate) fn bounded_hit_grid_dimensions(",
            "pub(crate) fn bounded_cells_for_frame(",
            "pub(crate) fn frame_is_finite_positive(frame: UiFrame) -> bool",
            '"ui.hit_grid.adaptive_coarsening_count"',
            '.checked_mul(rows as usize)',
        ),
    ),
    (
        "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs",
        (
            "bounded_cells_for_frame, entry_sort_key",
            "bounded_cells_for_frame(",
        ),
    ),
    (
        "zircon_runtime/src/ui/tree/mod.rs",
        (
            "bounded_cells_for_frame, bounded_hit_grid_dimensions",
            "frame_is_finite_positive, hit_grid_capacity_bounds",
        ),
    ),
    (
        "zircon_runtime/src/ui/surface/frame_hit_test.rs",
        (
            "bounded_hit_grid_dimensions(bounds, &entries, cell_size)",
            "bounded_cells_for_frame(bounds, columns, rows, cell_size, entry.clip_frame)",
            "frame_is_finite_positive(frame)",
            '.checked_mul(rows as usize)',
        ),
    ),
    (
        "tools/runtime_ui_tree_hit_grid_admission_pressure.py",
        ("zircon.runtime.ui_tree_hit_grid_admission_pressure.v1",),
    ),
)


class SourceContractError(RuntimeError):
    """Raised when current source no longer matches the modeled algorithm."""


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


def source_binding_report(repo_root: Path = ROOT) -> dict[str, object]:
    repo_root = repo_root.resolve()
    sources = []
    relative_paths = []
    source_set = hashlib.sha256()
    source_text = {}
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
        source_text[relative_path] = source
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

    projected_source = source_text[
        "zircon_runtime/src/ui/surface/frame_hit_test.rs"
    ]
    if "fn projected_cells_for_frame(" in projected_source:
        raise SourceContractError(
            "projected hit grid restored a duplicate cell-mapping implementation"
        )
    dirty_output = _git_output(
        repo_root, "status", "--porcelain=v1", "--", *relative_paths
    )
    dirty_entries = [] if not dirty_output else dirty_output.splitlines()
    return {
        "ready": True,
        "git_revision": _git_output(repo_root, "rev-parse", "HEAD"),
        "critical_sources_dirty": bool(dirty_entries),
        "critical_source_dirty_entry_count": len(dirty_entries),
        "source_set_sha256": source_set.hexdigest().upper(),
        "critical_sources": sources,
        "contracts": {
            "monotonic_paint_order_cursor": True,
            "shared_bounded_grid_helpers": True,
            "projected_grid_reuses_shared_helpers": True,
            "non_finite_membership_rejected": True,
            "no_duplicate_projected_cell_mapper": True,
        },
    }


def pressure_report(
    node_count: int = 10_000,
    huge_extent: float = 1_000_000.0,
    cell_size: float = 64.0,
    max_axis_cells: int = 128,
    max_entry_cell_count: int = 4_096,
) -> dict[str, object]:
    if node_count <= 0:
        raise ValueError("node_count must be positive")
    for name, value in (("huge_extent", huge_extent), ("cell_size", cell_size)):
        if not math.isfinite(value) or value <= 0:
            raise ValueError(f"{name} must be finite and positive")
    if max_axis_cells <= 0:
        raise ValueError("max_axis_cells must be positive")
    if max_entry_cell_count <= 0:
        raise ValueError("max_entry_cell_count must be positive")

    legacy_columns = math.ceil(huge_extent / cell_size)
    legacy_rows = legacy_columns
    legacy_cell_count = legacy_columns * legacy_rows
    requested_cell_size = max(cell_size, huge_extent / max_axis_cells)
    bounded_columns = min(
        max_axis_cells, max(1, math.ceil(huge_extent / requested_cell_size))
    )
    bounded_rows = bounded_columns
    pre_fallback_entry_cell_count = bounded_columns * bounded_rows
    adaptive_coarsening = pre_fallback_entry_cell_count > max_entry_cell_count
    if adaptive_coarsening:
        coarsened_cell_size = requested_cell_size * 2.0
        projected_columns = min(
            max_axis_cells, max(1, math.ceil(huge_extent / coarsened_cell_size))
        )
        projected_rows = projected_columns
    else:
        coarsened_cell_size = requested_cell_size
        projected_columns = bounded_columns
        projected_rows = bounded_rows
    projected_cell_count = projected_columns * projected_rows

    return {
        "schema": "zircon.runtime.ui_tree_hit_grid_admission_pressure.v1",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "node_count": node_count,
            "huge_extent": huge_extent,
            "cell_size": cell_size,
            "max_axis_cells": max_axis_cells,
            "max_entry_cell_count": max_entry_cell_count,
        },
        "paint_order_admission": {
            "legacy_rescan_node_visits": node_count * (node_count - 1) // 2,
            "cursor_sequential_rebuild_node_visits": 0,
            "cursor_invalidated_rebuild_node_visits": node_count,
            "cursor_invalidated_followup_insert_rebuild_node_visits": 0,
        },
        "hit_grid_admission": {
            "legacy_columns": legacy_columns,
            "legacy_rows": legacy_rows,
            "legacy_cell_count": legacy_cell_count,
            "bounded_max_cell_count": max_axis_cells * max_axis_cells,
            "pre_fallback_entry_cell_count": pre_fallback_entry_cell_count,
            "adaptive_coarsening": adaptive_coarsening,
            "coarsened_cell_size": coarsened_cell_size,
            "huge_entry_projected_columns": projected_columns,
            "huge_entry_projected_rows": projected_rows,
            "huge_entry_projected_cell_count": projected_cell_count,
            "wide_entry_membership_count": projected_cell_count,
            "global_grid_collapsed": projected_columns == 1 and projected_rows == 1,
            "legacy_to_candidate_cell_count_ratio": (
                legacy_cell_count / projected_cell_count
            ),
            "non_finite_entry_cell_memberships": 0,
            "full_bounds_entry_candidate_contribution_per_query": 1,
        },
        "invariants": {
            "sequential_insert_admission_is_constant_time_after_map_insert": True,
            "base_and_projected_grids_share_dimension_policy": True,
            "grid_backing_cell_count_is_bounded": True,
            "invalid_geometry_cannot_enter_cell_membership": True,
            "adaptive_coarsening_preserves_spatial_partition": True,
        },
        "reference_engine_findings": {
            "unreal_slate": {
                "source": "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp",
                "finding": (
                    "FHittestGrid sizes cells from the window hit-test area, clamps widget "
                    "paint geometry into that grid, and updates cell membership during paint"
                ),
            },
            "fyrox": {
                "source": "dev/Fyrox/fyrox-ui/src/lib.rs",
                "finding": (
                    "recursive picking rejects subtrees whose clip bounds do not intersect "
                    "the screen before descending"
                ),
            },
        },
        "interpretation": {
            "included": (
                "node-visit cardinality for paint-order admission and hit-grid backing-cell "
                "cardinality for one pathological square entry"
            ),
            "excluded": (
                "cell payload bytes, allocator overhead, per-cell membership bytes, cache "
                "locality, event routing work, and wall-clock latency"
            ),
            "dynamic_acceptance_pending": (
                "managed Rust regressions, allocation/RSS counters, coarsened-cell candidate "
                "counts, and product input latency under resize/popup pressure"
            ),
        },
    }


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("profile artifacts must use an absolute D:, E:, or F: path")
    return path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--node-count", type=int, default=10_000)
    parser.add_argument("--huge-extent", type=float, default=1_000_000.0)
    parser.add_argument("--cell-size", type=float, default=64.0)
    parser.add_argument("--max-axis-cells", type=int, default=128)
    parser.add_argument("--max-entry-cell-count", type=int, default=4_096)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output = validate_output_path(args.output)
    report = pressure_report(
        node_count=args.node_count,
        huge_extent=args.huge_extent,
        cell_size=args.cell_size,
        max_axis_cells=args.max_axis_cells,
        max_entry_cell_count=args.max_entry_cell_count,
    )
    report["source_binding"] = source_binding_report(ROOT)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
