#!/usr/bin/env python3
"""Source-bound work-count model for retained per-parent Taffy products.

This tool models structural operations. It does not report product timing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PureWindowsPath
import subprocess
from typing import Iterable


SCHEMA = "zircon.runtime.ui_taffy_parent_product_pressure.v2"
SOURCE_GUARDS = {
    "zircon_runtime/src/ui/layout/pass/slot.rs": (
        "layout_order_generation: u64",
        "ordered_children_by_parent: BTreeMap<UiNodeId, UiOrderedChildren>",
        "pub(super) fn synchronize_ordered_children",
        "pub(super) fn ordered_children_for_container",
        "fn rebuild_ordered_children",
        "entries.sort_unstable_by_key",
        "assert!(Arc::ptr_eq(&first, &stable));",
    ),
    "zircon_runtime/src/ui/layout/taffy_bridge/compute.rs": (
        "pub(crate) fn begin_children",
        "self.clear();",
        "self.taffy.clear();",
        ".new_leaf(",
        ".new_with_children(",
        ".compute_layout(",
    ),
    "zircon_runtime/src/ui/layout/pass/taffy_arrange.rs": (
        "scratch.bridge.begin_children(container);",
        "scratch.layout_children.iter().copied()",
        "compute_taffy_child_frames(container, frame, &mut scratch.bridge)",
        "scratch.bridge.child_frames().iter().copied()",
    ),
    "zircon_runtime/src/ui/layout/pass/incremental.rs": (
        "fn propagated_layout_root(",
        "parent.container.is_auto_layout_container()",
        "fn layout_dependency_paths(",
    ),
    "zircon_runtime/src/ui/surface/surface/rebuild/report.rs": (
        "pub layout_taffy_tree_build_count: u64",
        "pub layout_taffy_tree_node_build_count: u64",
        "pub layout_measure_probe_node_count: usize",
        "pub layout_arrange_probe_node_count: usize",
    ),
}


def validate_source_texts(source_texts: dict[str, str]) -> dict[str, object]:
    blockers: list[dict[str, object]] = []
    for relative_path, tokens in SOURCE_GUARDS.items():
        source = source_texts.get(relative_path)
        if source is None:
            blockers.append(
                {"code": "missing_critical_source", "relative_path": relative_path}
            )
            continue
        for token in tokens:
            if token not in source:
                blockers.append(
                    {
                        "code": "source_contract_changed",
                        "relative_path": relative_path,
                        "missing_token": token,
                    }
                )
    return {"ready": not blockers, "blockers": blockers}


def source_binding(repo_root: Path) -> dict[str, object]:
    root = repo_root.resolve()
    source_texts: dict[str, str] = {}
    fingerprints: list[dict[str, object]] = []
    for relative_path in SOURCE_GUARDS:
        path = root / relative_path
        if not path.is_file():
            continue
        payload = path.read_bytes()
        source_texts[relative_path] = payload.decode("utf-8")
        fingerprints.append(
            {
                "relative_path": relative_path,
                "sha256": hashlib.sha256(payload).hexdigest().upper(),
                "byte_length": len(payload),
            }
        )

    guard = validate_source_texts(source_texts)
    digest = hashlib.sha256()
    for source in sorted(fingerprints, key=lambda item: str(item["relative_path"])):
        digest.update(str(source["relative_path"]).encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(source["sha256"]).encode("ascii"))
        digest.update(b"\n")

    revision = _git_output(root, ("rev-parse", "HEAD"))
    dirty_paths = _git_lines(
        root,
        ("status", "--porcelain", "--", *tuple(SOURCE_GUARDS)),
    )
    return {
        "repository_root": str(root),
        "git_revision": revision,
        "critical_sources_dirty": bool(dirty_paths),
        "critical_source_status": dirty_paths,
        "critical_source_set_sha256": digest.hexdigest().upper(),
        "critical_source_files": fingerprints,
        "source_guard": guard,
    }


def parent_work(
    *,
    event_count: int,
    visited_parent_count: int,
    children_per_parent: int,
    changed_children_per_parent: int,
    parent_style_changes_per_parent: int = 0,
) -> dict[str, object]:
    values = {
        "event_count": event_count,
        "visited_parent_count": visited_parent_count,
        "children_per_parent": children_per_parent,
        "changed_children_per_parent": changed_children_per_parent,
        "parent_style_changes_per_parent": parent_style_changes_per_parent,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("pressure inputs must be non-negative")
    if event_count == 0 or visited_parent_count == 0:
        raise ValueError("event_count and visited_parent_count must be positive")
    if changed_children_per_parent > children_per_parent:
        raise ValueError("changed children cannot exceed children per parent")

    parent_visits = event_count * visited_parent_count
    child_visits = parent_visits * children_per_parent
    changed_child_visits = parent_visits * changed_children_per_parent
    parent_style_updates = parent_visits * parent_style_changes_per_parent
    current_node_creates = parent_visits * (children_per_parent + 1)
    return {
        "inputs": values,
        "current_scratch_rebuild": {
            "parent_product_visit_count": parent_visits,
            "ordered_child_index_lookup_count": parent_visits,
            "ordered_child_sort_count": 0,
            "ordered_child_sort_item_count": 0,
            "topology_build_count": parent_visits,
            "taffy_node_create_count": current_node_creates,
            "child_contract_visit_count": child_visits,
            "taffy_compute_count": parent_visits,
            "child_layout_read_count": child_visits,
        },
        "retained_topology_conservative": {
            "parent_product_lookup_count": parent_visits,
            "parent_product_hit_count": parent_visits,
            "topology_build_count": 0,
            "taffy_node_create_count": 0,
            "child_contract_visit_count": child_visits,
            "child_style_update_count": changed_child_visits,
            "parent_style_update_count": parent_style_updates,
            "taffy_compute_count": parent_visits,
            "child_layout_read_count": child_visits,
        },
        "retained_delta_patch": {
            "parent_product_lookup_count": parent_visits,
            "parent_product_hit_count": parent_visits,
            "topology_build_count": 0,
            "taffy_node_create_count": 0,
            "child_contract_visit_count": changed_child_visits,
            "child_style_update_count": changed_child_visits,
            "parent_style_update_count": parent_style_updates,
            "taffy_compute_count": parent_visits,
            "child_layout_read_count": child_visits,
        },
        "comparison": {
            "avoided_topology_build_count": parent_visits,
            "avoided_taffy_node_create_count": current_node_creates,
            "conservative_compute_count_reduction": 0,
            "conservative_child_layout_read_count_reduction": 0,
            "delta_patch_avoided_child_contract_visit_count": (
                child_visits - changed_child_visits
            ),
        },
    }


def pressure_suite(event_count: int = 1_000) -> dict[str, object]:
    if event_count <= 0:
        raise ValueError("event_count must be positive")
    scenarios = {
        "wide_parent_single_child_change": parent_work(
            event_count=event_count,
            visited_parent_count=1,
            children_per_parent=1_024,
            changed_children_per_parent=1,
        ),
        "nested_auto_layout_leaf_change": parent_work(
            event_count=event_count,
            visited_parent_count=8,
            children_per_parent=8,
            changed_children_per_parent=1,
        ),
        "independent_forest_single_parent_change": {
            **parent_work(
                event_count=event_count,
                visited_parent_count=1,
                children_per_parent=64,
                changed_children_per_parent=1,
            ),
            "unrelated_parent_count": 10_000,
            "unrelated_parent_visit_count": 0,
        },
        "window_resize_all_visible_parents": parent_work(
            event_count=120,
            visited_parent_count=100,
            children_per_parent=16,
            changed_children_per_parent=0,
            parent_style_changes_per_parent=1,
        ),
    }
    total_avoided_nodes = sum(
        int(scenario["comparison"]["avoided_taffy_node_create_count"])
        for scenario in scenarios.values()
    )
    return {
        "schema": SCHEMA,
        "evidence_kind": "deterministic_source_guarded_work_count",
        "is_product_timing": False,
        "scenarios": scenarios,
        "aggregate": {
            "modeled_scenario_count": len(scenarios),
            "avoided_taffy_node_create_count": total_avoided_nodes,
        },
        "invariants": [
            "stable layout-order generations reuse retained ordered-child slices without sorting",
            "retained topology does not reduce required solve work by assumption",
            "retained topology does not reduce child layout reads by assumption",
            "unrelated parents are not visited by a local independent-parent change",
            "topology reuse starts after a successful warm product build",
        ],
        "excluded_from_model": [
            "CPU p50, p95, and p99",
            "allocator operations and allocated bytes",
            "resident memory and retained Taffy node byte size",
            "input-to-present latency and GPU work",
            "Taffy internal compute-cache hit rate",
        ],
        "required_product_counters": [
            "parent_product_lookup_hit_count",
            "parent_product_lookup_miss_count",
            "topology_create_count",
            "topology_reconcile_count",
            "topology_remove_count",
            "child_style_update_count",
            "parent_style_update_count",
            "taffy_compute_count",
            "taffy_compute_reuse_count",
            "child_layout_read_count",
            "live_parent_product_high_water_count",
            "live_taffy_node_high_water_count",
            "layout_allocation_count",
            "layout_allocation_bytes",
        ],
    }


def build_report(repo_root: Path, event_count: int = 1_000) -> dict[str, object]:
    report = pressure_suite(event_count)
    binding = source_binding(repo_root)
    report["source_binding"] = binding
    report["ready"] = bool(binding["source_guard"]["ready"])
    report["tool_binding"] = _tool_binding()
    return report


def validate_output_path(output: str | Path) -> Path:
    path = Path(output).resolve()
    drive = PureWindowsPath(str(path)).drive.upper()
    if drive not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written below D:, E:, or F:")
    return path


def _git_output(root: Path, args: Iterable[str]) -> str | None:
    completed = subprocess.run(
        ["git", *args],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
        encoding="utf-8",
    )
    value = completed.stdout.strip()
    return value if completed.returncode == 0 and value else None


def _git_lines(root: Path, args: Iterable[str]) -> list[str]:
    value = _git_output(root, args)
    return value.splitlines() if value is not None else []


def _tool_binding() -> dict[str, object]:
    path = Path(__file__).resolve()
    payload = path.read_bytes()
    return {
        "path": str(path),
        "sha256": hashlib.sha256(payload).hexdigest().upper(),
        "byte_length": len(payload),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
    )
    parser.add_argument("--events", type=int, default=1_000)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    output_path = validate_output_path(args.output)
    report = build_report(args.repo_root, args.events)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
        newline="\n",
    )
    print(json.dumps({"ready": report["ready"], "output": str(output_path)}))
    return 0 if report["ready"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
