#!/usr/bin/env python3
"""Model duplicate Runtime Diagnostics surface construction pressure.

This is a deterministic operation-count model. It deliberately does not claim
CPU time, allocator latency, or product frame latency.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/runtime_diagnostics.rs",
    "zircon_editor/src/ui/retained_host/ui/apply_presentation/pane_conversion.rs",
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs",
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/surface.rs",
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/node.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)

SOURCE_GUARDS = {
    SOURCE_PATHS[0]: (
        "pane.body_surface_frame.as_ref()",
        "debug_surface_frame(surface_frame)",
        "RuntimeDebugReflectorNodeWriter::new",
        "writer.finish()",
    ),
    SOURCE_PATHS[1]: (
        "rebuild_pane_template_hit_artifacts(",
        "refresh_runtime_diagnostics_debug_reflector_from_body_surface(",
    ),
    SOURCE_PATHS[2]: (
        "build_template_surface_frame",
        "pane.body_surface_frame =",
        'pane.kind.as_str() == "RuntimeDiagnostics"',
    ),
    SOURCE_PATHS[3]: (
        "include_non_dispatchable",
        "include_non_dispatchable || is_dispatchable(node)",
        "template_surface_tree_node(row, node, is_dispatchable(node))",
    ),
    SOURCE_PATHS[4]: (
        "dispatchable: bool",
        "UiInputPolicy::Ignore",
        "clickable: dispatchable",
    ),
    SOURCE_PATHS[5]: (
        "FSlateInvalidationRoot",
        "FastPath",
    ),
}


def git_output(root: Path, *args: str) -> str:
    return subprocess.check_output(("git", *args), cwd=root, text=True).strip()


def source_binding(root: Path) -> dict[str, Any]:
    files: list[dict[str, str]] = []
    for relative in SOURCE_PATHS:
        path = root / relative
        data = path.read_bytes()
        files.append(
            {
                "path": relative,
                "sha256": hashlib.sha256(data).hexdigest().upper(),
            }
        )
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
            if not isinstance(guard, str):
                continue
            if guard not in source:
                raise AssertionError(f"missing source guard {guard!r} in {relative}")
    production = (root / SOURCE_PATHS[0]).read_text(encoding="utf-8").split(
        "#[cfg(test)]", 1
    )[0]
    for forbidden in ("UiSurface::new(", "surface.rebuild()", "BTreeMap<"):
        if forbidden in production:
            raise AssertionError(f"duplicate surface construction remains: {forbidden}")
    live_projection = production.split(
        "fn runtime_debug_reflector_nodes_from_model", 1
    )[1].split("fn runtime_debug_reflector_nodes_from_parts", 1)[0]
    for forbidden in ("section_display_lines()", "collect::<Vec<_>>()", "node_labels"):
        if forbidden in live_projection:
            raise AssertionError(f"live reflector intermediate remains: {forbidden}")


def pressure_report(
    *,
    recompute_count: int = 200,
    row_count: int = 256,
    section_count: int = 9,
    section_line_count: int = 48,
) -> dict[str, Any]:
    if min(recompute_count, row_count, section_count, section_line_count) < 0:
        raise ValueError("counts must be non-negative")
    legacy_nodes = recompute_count * (row_count + 1)
    flattened_lines_per_refresh = row_count + section_count + section_line_count
    return {
        "schema": "zircon.editor.runtime_diagnostics_surface_pressure.v2",
        "inputs": {
            "recompute_count": recompute_count,
            "displayed_row_count": row_count,
            "section_count": section_count,
            "section_line_count": section_line_count,
        },
        "legacy_duplicate_surface": {
            "surface_build_count": recompute_count,
            "tree_node_materialization_count": legacy_nodes,
            "path_materialization_count": legacy_nodes,
            "attribute_map_materialization_count": recompute_count * row_count,
            "layout_rebuild_count": recompute_count,
        },
        "current_published_frame_read": {
            "published_frame_read_count": recompute_count,
            "surface_build_count": 0,
            "tree_node_materialization_count": 0,
            "path_materialization_count": 0,
            "attribute_map_materialization_count": 0,
            "layout_rebuild_count": 0,
        },
        "delta": {
            "avoided_surface_build_count": recompute_count,
            "avoided_tree_node_materialization_count": legacy_nodes,
            "avoided_path_materialization_count": legacy_nodes,
            "avoided_attribute_map_materialization_count": recompute_count * row_count,
            "avoided_layout_rebuild_count": recompute_count,
        },
        "live_reflector_projection": {
            "legacy_intermediate_vec_count": recompute_count * 2,
            "current_intermediate_vec_count": 0,
            "legacy_intermediate_owned_string_product_count": (
                recompute_count * flattened_lines_per_refresh
            ),
            "current_intermediate_owned_string_product_count": 0,
            "final_template_node_count_unchanged": True,
        },
        "interpretation": {
            "timing_claim": False,
            "authority": "debug_surface_frame reads the pane body_surface_frame already published by host_contract",
            "missing_frame_behavior": "fail_closed_without event-time reconstruction",
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
    parser.add_argument("--recompute-count", type=int, default=200)
    parser.add_argument("--row-count", type=int, default=256)
    parser.add_argument("--section-count", type=int, default=9)
    parser.add_argument("--section-line-count", type=int, default=48)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.recompute_count < 0 or args.row_count < 0:
        raise ValueError("counts must be non-negative")
    result = pressure_report(
        recompute_count=args.recompute_count,
        row_count=args.row_count,
        section_count=args.section_count,
        section_line_count=args.section_line_count,
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
