"""Validate stable-frame Runtime UI render-segment work from a profile timeline."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from pathlib import Path
from typing import Any


FRAME_COUNT = "ui.screen_space_ui.frame_prepare_count"
SEGMENT_COUNT = "ui.screen_space_ui.input_segment_count"

FRAME_REUSE_COUNTERS = (
    "ui.screen_space_ui_plan.cache_hit_count",
    "ui.screen_space_ui_vertex.plan_reuse_count",
    "ui_text.segment_cache.frame_product_reuse_count",
)

SEGMENT_REUSE_COUNTERS = (
    "ui.screen_space_ui_image.segment_plan_reuse_count",
    "ui_text.segment_cache.segment_product_reuse_count",
)

PLAN_WORK_COUNTERS = (
    "ui.screen_space_ui_plan.build_count",
    "ui.screen_space_ui_plan.command_visit_count",
    "ui.screen_space_ui_plan.segment_command_visit_count",
    "ui.screen_space_ui_plan.composition_payload_clone_count",
)

COMMAND_LEAF_COUNT = "ui.screen_space_ui_plan.command_leaf_count"
COMMAND_LEAF_REUSE_COUNT = "ui.screen_space_ui_plan.command_leaf_cache_hit_count"
COMMAND_LEAF_REBUILD_COUNT = "ui.screen_space_ui_plan.command_leaf_rebuild_count"
COMMAND_LEAF_COUNTERS = (
    COMMAND_LEAF_COUNT,
    COMMAND_LEAF_REUSE_COUNT,
    COMMAND_LEAF_REBUILD_COUNT,
)

VERTEX_WORK_COUNTERS = (
    "ui.screen_space_ui_vertex.hash_count",
    "ui.screen_space_ui_vertex.hash_input_bytes",
    "ui.screen_space_ui_vertex.segment_write_count",
    "ui.screen_space_ui_vertex.segment_write_bytes",
    "ui.screen_space_ui_vertex.segment_buffer_allocation_count",
)

IMAGE_GEOMETRY_WORK_COUNTERS = (
    "ui.screen_space_ui_image.batch_visit_count",
)

DEPENDENCY_WORK_COUNTERS = (
    "ui.screen_space_ui_image.texture_dependency_check_count",
    "ui_text.segment_cache.font_dependency_segment_visit_count",
    "ui_text.segment_cache.font_dependency_asset_visit_count",
    "ui_text.segment_cache.font_asset_ensure_count",
)

TEXT_PRODUCT_WORK_COUNTERS = (
    "ui_text.segment_cache.text_batch_visit_count",
    "ui_text.segment_cache.glyph_projection_count",
    "ui_text.segment_cache.compatibility_batch_clone_count",
    "ui_text.segment_cache.compatibility_glyph_run_clone_count",
)

REQUIRED_COUNTERS = (
    FRAME_COUNT,
    SEGMENT_COUNT,
    *FRAME_REUSE_COUNTERS,
    *SEGMENT_REUSE_COUNTERS,
    *COMMAND_LEAF_COUNTERS,
    *PLAN_WORK_COUNTERS,
    *VERTEX_WORK_COUNTERS,
    *IMAGE_GEOMETRY_WORK_COUNTERS,
    *DEPENDENCY_WORK_COUNTERS,
    *TEXT_PRODUCT_WORK_COUNTERS,
)

REQUIRED_SOURCE_PATHS = (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
)


def validate_source_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    if not isinstance(manifest, dict) or manifest.get("schema_version") != 2:
        blockers.append({"code": "invalid_source_manifest_schema"})
        return blockers
    if manifest.get("scenario") != "render_segment_stable":
        blockers.append(
            {
                "code": "invalid_source_manifest_scenario",
                "scenario": manifest.get("scenario"),
            }
        )
    options = manifest.get("capture", {}).get("options", {})
    if (
        options.get("run_phase") != "measured"
        or not isinstance(options.get("run_ordinal"), int)
        or options.get("run_ordinal", 0) <= 0
        or not isinstance(options.get("measured_run_count"), int)
        or options.get("measured_run_count", 0) <= 0
    ):
        blockers.append({"code": "invalid_capture_contract"})
    git = manifest.get("repository", {}).get("git", {})
    if not isinstance(git.get("revision"), str) or not re.fullmatch(
        r"[0-9a-fA-F]{40}", git["revision"]
    ):
        blockers.append({"code": "invalid_source_revision"})
    files = manifest.get("repository", {}).get("critical_source_files", [])
    entries = {
        entry.get("relative_path")
        for entry in files
        if isinstance(entry, dict)
    }
    for path in REQUIRED_SOURCE_PATHS:
        if path not in entries:
            blockers.append({"code": "missing_critical_source", "path": path})
    return blockers


def _counter_values(timeline: dict[str, Any], name: str) -> list[Any]:
    counters = timeline.get("counters", [])
    if not isinstance(counters, list):
        return []
    return [
        counter.get("value")
        for counter in counters
        if isinstance(counter, dict) and counter.get("name") == name
    ]


def _counter_total(values: list[Any]) -> int | None:
    total = 0
    for value in values:
        if isinstance(value, bool):
            return None
        try:
            number = float(value)
        except (TypeError, ValueError, OverflowError):
            return None
        if not math.isfinite(number) or number < 0 or not number.is_integer():
            return None
        total += int(number)
    return total


def _nonzero_totals(totals: dict[str, int], names: tuple[str, ...]) -> dict[str, int]:
    return {name: totals[name] for name in names if totals.get(name, 0) != 0}


def evaluate_stable_render_segment_evidence(
    timeline: dict[str, Any],
) -> dict[str, Any]:
    blockers: list[dict[str, Any]] = []
    totals: dict[str, int] = {}
    for name in REQUIRED_COUNTERS:
        values = _counter_values(timeline, name)
        if not values:
            blockers.append({"code": "missing_counter", "counter": name})
            continue
        total = _counter_total(values)
        if total is None:
            blockers.append({"code": "invalid_counter_value", "counter": name})
            continue
        totals[name] = total

    conservation: dict[str, Any] = {
        "prepared_frames": totals.get(FRAME_COUNT),
        "input_segments": totals.get(SEGMENT_COUNT),
        "frame_reuse": {
            name: totals.get(name) for name in FRAME_REUSE_COUNTERS
        },
        "segment_reuse": {
            name: totals.get(name) for name in SEGMENT_REUSE_COUNTERS
        },
        "command_leaf_reuse": {
            name: totals.get(name) for name in COMMAND_LEAF_COUNTERS
        },
    }
    if len(totals) == len(REQUIRED_COUNTERS):
        prepared_frames = totals[FRAME_COUNT]
        input_segments = totals[SEGMENT_COUNT]
        if prepared_frames <= 0 or input_segments <= 0:
            blockers.append(
                {
                    "code": "missing_stable_frame_activity",
                    "prepared_frames": prepared_frames,
                    "input_segments": input_segments,
                }
            )

        mismatched_frames = {
            name: totals[name]
            for name in FRAME_REUSE_COUNTERS
            if totals[name] != prepared_frames
        }
        if mismatched_frames:
            blockers.append(
                {
                    "code": "stable_frame_reuse_conservation_failed",
                    "prepared_frames": prepared_frames,
                    "observed": mismatched_frames,
                }
            )

        mismatched_segments = {
            name: totals[name]
            for name in SEGMENT_REUSE_COUNTERS
            if totals[name] != input_segments
        }
        if mismatched_segments:
            blockers.append(
                {
                    "code": "stable_segment_reuse_conservation_failed",
                    "input_segments": input_segments,
                    "observed": mismatched_segments,
                }
            )

        command_leaf_count = totals[COMMAND_LEAF_COUNT]
        command_leaf_reuse_count = totals[COMMAND_LEAF_REUSE_COUNT]
        command_leaf_rebuild_count = totals[COMMAND_LEAF_REBUILD_COUNT]
        if (
            command_leaf_count <= 0
            or command_leaf_reuse_count != command_leaf_count
            or command_leaf_rebuild_count != 0
        ):
            blockers.append(
                {
                    "code": "stable_command_leaf_reuse_conservation_failed",
                    "expected": {
                        COMMAND_LEAF_REUSE_COUNT: command_leaf_count,
                        COMMAND_LEAF_REBUILD_COUNT: 0,
                    },
                    "observed": {
                        COMMAND_LEAF_COUNT: command_leaf_count,
                        COMMAND_LEAF_REUSE_COUNT: command_leaf_reuse_count,
                        COMMAND_LEAF_REBUILD_COUNT: command_leaf_rebuild_count,
                    },
                }
            )

        work_groups = (
            ("stable_frame_plan_work", PLAN_WORK_COUNTERS),
            ("stable_frame_vertex_work", VERTEX_WORK_COUNTERS),
            ("stable_frame_image_geometry_work", IMAGE_GEOMETRY_WORK_COUNTERS),
            ("stable_frame_dependency_work", DEPENDENCY_WORK_COUNTERS),
            ("stable_frame_text_product_work", TEXT_PRODUCT_WORK_COUNTERS),
        )
        for code, names in work_groups:
            nonzero = _nonzero_totals(totals, names)
            if nonzero:
                blockers.append({"code": code, "counters": nonzero})

    return {
        "schema": "zircon.runtime.ui_render_segment_stable_evidence.v2",
        "ready": not blockers,
        "blockers": blockers,
        "counter_totals": totals,
        "conservation": conservation,
        "scope": (
            "measured stable frames after warmup; every frame and segment must reuse its "
            "published surface and command-leaf render products with zero command, vertex, "
            "image dependency, font dependency, or text-product reconstruction work"
        ),
    }


def validate_output_path(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if resolved.drive.casefold() not in {"d:", "e:", "f:"}:
        raise ValueError("performance artifacts must be written under D:, E:, or F:")
    return resolved


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile-dir", required=True, type=Path)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    profile_dir = validate_output_path(args.profile_dir)
    timeline_path = profile_dir / "timeline.zrtrace.json"
    if not timeline_path.is_file():
        raise FileNotFoundError(f"timeline artifact is missing: {timeline_path}")
    timeline = json.loads(timeline_path.read_text(encoding="utf-8"))
    result = evaluate_stable_render_segment_evidence(timeline)
    source_manifest_path = (
        validate_output_path(args.source_manifest)
        if args.source_manifest is not None
        else profile_dir / "source_manifest.json"
    )
    if not source_manifest_path.is_file():
        result["blockers"].append(
            {
                "code": "missing_source_manifest",
                "path": str(source_manifest_path),
            }
        )
    else:
        source_manifest = json.loads(source_manifest_path.read_text(encoding="utf-8"))
        result["blockers"].extend(validate_source_manifest(source_manifest))
    result["ready"] = not result["blockers"]
    result["profile_binding"] = {
        "timeline_path": str(timeline_path),
        "timeline_sha256": _sha256(timeline_path),
        "tool_sha256": _sha256(Path(__file__)),
        "source_manifest_path": str(source_manifest_path),
        "source_manifest_sha256": (
            _sha256(source_manifest_path) if source_manifest_path.is_file() else None
        ),
    }
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(payload, end="")
    else:
        output_path = validate_output_path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8")
    return 0 if result["ready"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
