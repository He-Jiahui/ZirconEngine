"""Deterministic work model for immutable UI surface-frame domain sharing."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[1]
CRITICAL_SOURCE_FILES = (
    "tools/ui_surface_frame_domain_sharing_pressure.py",
    "tools/ui-profile-counter-evidence.ps1",
    "zircon_runtime/src/ui/surface/arranged.rs",
    "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
    "zircon_runtime_interface/src/ui/surface/focus_state.rs",
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)


def run(
    arranged_node_count: int,
    render_command_count: int,
    hit_entry_count: int,
    hit_cell_entry_count: int,
    focus_node_count: int,
    pipeline_stage_count: int,
    window_only_update_count: int,
    render_only_update_count: int,
    changed_render_command_count: int = 1,
    render_segment_size: int = 64,
    directory_fanout: int = 32,
    owned_payload_bytes_per_command: int = 24,
    layout_only_update_count: int = 0,
    focus_path_depth: int = 8,
) -> dict[str, object]:
    for name, value in (
        ("arranged_node_count", arranged_node_count),
        ("render_command_count", render_command_count),
        ("hit_entry_count", hit_entry_count),
        ("hit_cell_entry_count", hit_cell_entry_count),
        ("focus_node_count", focus_node_count),
        ("pipeline_stage_count", pipeline_stage_count),
        ("window_only_update_count", window_only_update_count),
        ("render_only_update_count", render_only_update_count),
        ("changed_render_command_count", changed_render_command_count),
        ("render_segment_size", render_segment_size),
        ("owned_payload_bytes_per_command", owned_payload_bytes_per_command),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if layout_only_update_count < 0:
        raise ValueError("layout_only_update_count must not be negative")
    if focus_path_depth <= 0:
        raise ValueError("focus_path_depth must be positive")
    if directory_fanout <= 1:
        raise ValueError("directory_fanout must be greater than one")
    if changed_render_command_count > render_command_count:
        raise ValueError(
            "changed_render_command_count must not exceed render_command_count"
        )

    heavy_domain_size = (
        arranged_node_count
        + render_command_count
        + hit_entry_count
        + hit_cell_entry_count
        + focus_node_count
        + pipeline_stage_count
    )
    total_update_count = window_only_update_count + render_only_update_count
    old_all_domain_clone_work = heavy_domain_size * total_update_count

    new_window_only_element_clone_work = 0
    new_render_only_element_clone_work = render_command_count * render_only_update_count
    new_changed_domain_clone_work = (
        new_window_only_element_clone_work + new_render_only_element_clone_work
    )
    new_shared_domain_arc_clone_count = (
        6 * window_only_update_count + 5 * render_only_update_count
    )
    eliminated_element_clone_work = (
        old_all_domain_clone_work - new_changed_domain_clone_work
    )

    # A direct Arc/COW conversion still clones the whole flat command vector when the
    # previous frame retains the Arc and the next generation performs its first patch.
    current_render_publication_command_clone_work = (
        render_command_count * render_only_update_count
    )
    direct_arc_cow_command_clone_work = current_render_publication_command_clone_work

    render_segment_count = ceil_div(render_command_count, render_segment_size)
    changed_render_segment_count = min(
        render_segment_count,
        ceil_div(changed_render_command_count, render_segment_size),
    )
    persistent_directory_depth = persistent_directory_depth_for(
        render_segment_count, directory_fanout
    )
    commands_copied_per_segment_patch = min(
        render_command_count,
        changed_render_segment_count * render_segment_size,
    )
    persistent_segment_command_clone_work = (
        commands_copied_per_segment_patch * render_only_update_count
    )
    persistent_directory_node_clone_work = (
        changed_render_segment_count
        * persistent_directory_depth
        * render_only_update_count
    )
    current_render_publication_owned_payload_clone_bytes = (
        current_render_publication_command_clone_work
        * owned_payload_bytes_per_command
    )
    persistent_segment_owned_payload_clone_bytes = (
        persistent_segment_command_clone_work * owned_payload_bytes_per_command
    )
    historical_layout_focus_clone_work = (
        focus_node_count * layout_only_update_count
    )
    new_layout_focus_clone_work = 0
    focus_path_validation_node_visit_upper_bound = (
        focus_path_depth * layout_only_update_count
    )

    return {
        "schema": "zircon.runtime.ui_surface_frame_domain_sharing_pressure.v2",
        "source_binding": source_binding(),
        "interpretation": {
            "included": (
                "immutable frame-domain element copy work, persistent render segment copy "
                "work, layout-period focus clone work, and focus-path validation upper bounds"
            ),
            "excluded": "CPU timing, allocator latency, cache locality, RSS, and product frame latency",
            "timing_claim": False,
        },
        "arranged_node_count": arranged_node_count,
        "render_command_count": render_command_count,
        "hit_entry_count": hit_entry_count,
        "hit_cell_entry_count": hit_cell_entry_count,
        "focus_node_count": focus_node_count,
        "focus_path_depth": focus_path_depth,
        "pipeline_stage_count": pipeline_stage_count,
        "window_only_update_count": window_only_update_count,
        "render_only_update_count": render_only_update_count,
        "layout_only_update_count": layout_only_update_count,
        "changed_render_command_count": changed_render_command_count,
        "render_segment_size": render_segment_size,
        "render_segment_count": render_segment_count,
        "changed_render_segment_count": changed_render_segment_count,
        "directory_fanout": directory_fanout,
        "owned_payload_bytes_per_command": owned_payload_bytes_per_command,
        "old_all_domain_clone_work": old_all_domain_clone_work,
        "new_window_only_element_clone_work": new_window_only_element_clone_work,
        "new_render_only_element_clone_work": new_render_only_element_clone_work,
        "new_changed_domain_clone_work": new_changed_domain_clone_work,
        "new_shared_domain_arc_clone_count": new_shared_domain_arc_clone_count,
        "eliminated_element_clone_work": eliminated_element_clone_work,
        "element_clone_reduction_ratio": (
            old_all_domain_clone_work / new_changed_domain_clone_work
        ),
        "current_render_publication_command_clone_work": (
            current_render_publication_command_clone_work
        ),
        "direct_arc_cow_command_clone_work": direct_arc_cow_command_clone_work,
        "persistent_segment_command_clone_work": (
            persistent_segment_command_clone_work
        ),
        "persistent_directory_depth": persistent_directory_depth,
        "persistent_directory_node_clone_work": (
            persistent_directory_node_clone_work
        ),
        "persistent_publication_handle_clone_work": render_only_update_count,
        "current_render_publication_owned_payload_clone_bytes": (
            current_render_publication_owned_payload_clone_bytes
        ),
        "persistent_segment_owned_payload_clone_bytes": (
            persistent_segment_owned_payload_clone_bytes
        ),
        "persistent_segment_clone_reduction_ratio": (
            current_render_publication_command_clone_work
            / persistent_segment_command_clone_work
        ),
        "historical_layout_focus_clone_work": historical_layout_focus_clone_work,
        "new_layout_focus_clone_work": new_layout_focus_clone_work,
        "focus_path_validation_node_visit_upper_bound": (
            focus_path_validation_node_visit_upper_bound
        ),
        "eliminated_layout_focus_clone_work": (
            historical_layout_focus_clone_work - new_layout_focus_clone_work
        ),
        "layout_focus_clone_to_validation_ratio": (
            historical_layout_focus_clone_work
            / focus_path_validation_node_visit_upper_bound
            if focus_path_validation_node_visit_upper_bound > 0
            else 0.0
        ),
    }


def source_binding() -> dict[str, object]:
    source_sha256 = {
        relative_path: hashlib.sha256((ROOT / relative_path).read_bytes())
        .hexdigest()
        .upper()
        for relative_path in CRITICAL_SOURCE_FILES
    }
    manifest_payload = "\n".join(
        f"{path}:{source_sha256[path]}" for path in sorted(source_sha256)
    ).encode("utf-8")
    git_revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    return {
        "git_revision": git_revision,
        "critical_source_files": list(CRITICAL_SOURCE_FILES),
        "source_sha256": source_sha256,
        "source_manifest_sha256": hashlib.sha256(manifest_payload)
        .hexdigest()
        .upper(),
    }


def ceil_div(value: int, divisor: int) -> int:
    return (value + divisor - 1) // divisor


def persistent_directory_depth_for(segment_count: int, fanout: int) -> int:
    depth = 1
    capacity = fanout
    while capacity < segment_count:
        capacity *= fanout
        depth += 1
    return depth


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--arranged-node-count", type=int, default=32_768)
    parser.add_argument("--render-command-count", type=int, default=32_768)
    parser.add_argument("--hit-entry-count", type=int, default=16_384)
    parser.add_argument("--hit-cell-entry-count", type=int, default=65_536)
    parser.add_argument("--focus-node-count", type=int, default=1_024)
    parser.add_argument("--pipeline-stage-count", type=int, default=8)
    parser.add_argument("--window-only-update-count", type=int, default=4_096)
    parser.add_argument("--render-only-update-count", type=int, default=1_024)
    parser.add_argument("--changed-render-command-count", type=int, default=1)
    parser.add_argument("--render-segment-size", type=int, default=64)
    parser.add_argument("--directory-fanout", type=int, default=32)
    parser.add_argument("--owned-payload-bytes-per-command", type=int, default=24)
    parser.add_argument("--layout-only-update-count", type=int, default=4_096)
    parser.add_argument("--focus-path-depth", type=int, default=8)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.arranged_node_count,
        args.render_command_count,
        args.hit_entry_count,
        args.hit_cell_entry_count,
        args.focus_node_count,
        args.pipeline_stage_count,
        args.window_only_update_count,
        args.render_only_update_count,
        args.changed_render_command_count,
        args.render_segment_size,
        args.directory_fanout,
        args.owned_payload_bytes_per_command,
        args.layout_only_update_count,
        args.focus_path_depth,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
