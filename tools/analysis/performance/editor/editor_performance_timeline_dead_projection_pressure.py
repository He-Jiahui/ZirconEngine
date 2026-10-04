#!/usr/bin/env python3
"""Model removal of unused Performance Timeline host-row projections."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/host_contract/data/panes/performance_timeline.rs",
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/performance_timeline.rs",
    "zircon_editor/src/ui/retained_host/host_contract/data/panes/pane.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/selection.rs",
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs",
)

SOURCE_GUARDS = {
    SOURCE_PATHS[0]: ("pub(crate) struct PerformanceTimelinePaneData", "pub nodes:"),
    SOURCE_PATHS[1]: (
        "let mut nodes =",
        "nodes.extend(performance_timeline_nodes(native, &nodes, content_size));",
        "visible_row_range",
    ),
    SOURCE_PATHS[2]: (
        '"PerformanceTimeline" => Some(&self.performance_timeline.nodes)',
        "pub(crate) fn template_nodes(&self)",
    ),
    SOURCE_PATHS[3]: ("pane.template_nodes()",),
    SOURCE_PATHS[4]: ("pane.template_nodes()",),
}


def git_output(root: Path, *args: str) -> str:
    return subprocess.check_output(("git", *args), cwd=root, text=True).strip()


def source_binding(root: Path) -> dict[str, Any]:
    files = []
    for relative in SOURCE_PATHS:
        data = (root / relative).read_bytes()
        files.append({"path": relative, "sha256": hashlib.sha256(data).hexdigest().upper()})
    manifest = "\n".join(f"{item['path']}:{item['sha256']}" for item in files).encode()
    return {
        "head": git_output(root, "rev-parse", "HEAD"),
        "files": files,
        "manifest_sha256": hashlib.sha256(manifest).hexdigest().upper(),
    }


def validate_source_contract(root: Path) -> None:
    for relative, guards in SOURCE_GUARDS.items():
        source = (root / relative).read_text(encoding="utf-8")
        for guard in guards:
            if guard not in source:
                raise AssertionError(f"missing source guard {guard!r} in {relative}")
    contract = (root / SOURCE_PATHS[0]).read_text(encoding="utf-8")
    forbidden_contract_fields = (
        "PerformanceTimelineFrameRowData",
        "PerformanceTimelineSpanRowData",
        "PerformanceTimelineHotspotRowData",
        "PerformanceTimelineCaptureControlData",
        "pub frame_rows:",
        "pub span_rows:",
        "pub hotspot_rows:",
        "pub capture_controls:",
        "pub summary:",
        "pub session_label:",
        "pub output_label:",
    )
    for forbidden in forbidden_contract_fields:
        if forbidden in contract:
            raise AssertionError(f"dead host contract field remains: {forbidden}")
    projection = (root / SOURCE_PATHS[1]).read_text(encoding="utf-8")
    for forbidden in (
        "to_host_contract_frame_row",
        "to_host_contract_span_row",
        "to_host_contract_hotspot_row",
        "to_host_contract_capture_control",
        "map_model_rc",
    ):
        if forbidden in projection:
            raise AssertionError(f"dead row projection remains: {forbidden}")


def pressure_report(*, refresh_count: int = 200, logical_rows: int = 256) -> dict[str, Any]:
    if refresh_count < 0 or logical_rows < 0:
        raise ValueError("counts must be non-negative")
    return {
        "schema": "zircon.editor.performance_timeline_dead_projection_pressure.v1",
        "inputs": {"refresh_count": refresh_count, "logical_rows": logical_rows},
        "legacy_dead_projection": {
            "frame_row_mapping_count": refresh_count * logical_rows,
            "span_row_mapping_count": refresh_count * logical_rows,
            "hotspot_row_mapping_count": refresh_count * logical_rows,
            "capture_control_mapping_count": refresh_count * 4,
            "summary_string_clone_count": refresh_count * 3,
        },
        "current_nodes_only_projection": {
            "frame_row_mapping_count": 0,
            "span_row_mapping_count": 0,
            "hotspot_row_mapping_count": 0,
            "capture_control_mapping_count": 0,
            "summary_string_clone_count": 0,
            "visible_template_node_projection_preserved": True,
        },
        "delta": {
            "avoided_dead_row_mapping_count": refresh_count * logical_rows * 3,
            "avoided_dead_control_mapping_count": refresh_count * 4,
            "avoided_dead_summary_clone_count": refresh_count * 3,
        },
        "interpretation": {
            "timing_claim": False,
            "authority": "PerformanceTimelinePaneData.nodes is the only current paint/geometry/hit consumer",
            "retained": "visible-row virtualized template nodes and capture control action nodes",
            "excluded": "CPU time, allocator latency, GPU time, RSS, and measured UI latency",
        },
    }


def run(root: Path | None = None) -> dict[str, Any]:
    repo = root or Path(__file__).resolve().parents[4]
    result = pressure_report()
    result["source_binding"] = source_binding(repo)
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--refresh-count", type=int, default=200)
    parser.add_argument("--logical-rows", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = pressure_report(
        refresh_count=args.refresh_count,
        logical_rows=args.logical_rows,
    )
    result["source_binding"] = source_binding(Path(__file__).resolve().parents[4])
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is not None:
        if args.output.drive.upper() == "C:":
            raise ValueError("profile artifacts must not be written to C:")
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload, encoding="utf-8")
    print(payload, end="")


if __name__ == "__main__":
    main()
