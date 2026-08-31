"""Validate one-segment Runtime UI render dependency work from a profile timeline."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import re
from pathlib import Path
from typing import Any


PREPARED_FRAME_COUNT = "ui.screen_space_ui.prepared_frame_count"
INPUT_SEGMENT_COUNT = "ui.screen_space_ui.input_segment_count"
CHANGED_SEGMENT_COUNT = "ui.screen_space_ui.changed_segment_count"
CHANGED_SEGMENT_COMMAND_COUNT = "ui.screen_space_ui.changed_segment_command_count"
FULL_FALLBACK_COUNT = "ui.screen_space_ui.segment_delta_full_fallback_count"

LEGACY_PLAN_REUSE_COUNT = "ui.screen_space_ui_plan.segment_cache_hit_count"
PLAN_LEAF_COUNT = "ui.screen_space_ui_plan.command_leaf_count"
PLAN_LEAF_REUSE_COUNT = "ui.screen_space_ui_plan.command_leaf_cache_hit_count"
PLAN_LEAF_REBUILD_COUNT = "ui.screen_space_ui_plan.command_leaf_rebuild_count"
PLAN_COMMAND_VISIT_COUNT = "ui.screen_space_ui_plan.segment_command_visit_count"

IMAGE_REUSE_COUNT = "ui.screen_space_ui_image.segment_plan_reuse_count"
CHANGED_IMAGE_DEPENDENCY_COUNT = (
    "ui.screen_space_ui_image.changed_texture_dependency_count"
)
IMAGE_DEPENDENCY_CHECK_COUNT = (
    "ui.screen_space_ui_image.texture_dependency_check_count"
)
IMAGE_BINDING_LOOKUP_COUNT = "ui.screen_space_ui_image.binding_lookup_count"
IMAGE_BINDING_RETENTION_SCAN_COUNT = (
    "ui.screen_space_ui_image.binding_retention_scan_count"
)

TEXT_REUSE_COUNT = "ui_text.segment_cache.segment_product_reuse_count"
CHANGED_TEXT_BATCH_COUNT = "ui_text.segment_cache.changed_text_batch_count"
TEXT_BATCH_VISIT_COUNT = "ui_text.segment_cache.text_batch_visit_count"
CHANGED_GLYPH_COUNT = "ui_text.segment_cache.changed_glyph_count"
GLYPH_PROJECTION_COUNT = "ui_text.segment_cache.glyph_projection_count"
CHANGED_TEXT_DEPENDENCY_SEGMENT_COUNT = (
    "ui_text.segment_cache.changed_dependency_segment_count"
)
CHANGED_TEXT_DEPENDENCY_COUNT = "ui_text.segment_cache.changed_dependency_count"
TEXT_DEPENDENCY_SEGMENT_VISIT_COUNT = (
    "ui_text.segment_cache.frame_dependency_segment_visit_count"
)
TEXT_DEPENDENCY_ENTRY_VISIT_COUNT = (
    "ui_text.segment_cache.frame_dependency_entry_visit_count"
)

SURFACE_REUSE_COUNTERS = (IMAGE_REUSE_COUNT, TEXT_REUSE_COUNT)
PLAN_LEAF_COUNTERS = (
    LEGACY_PLAN_REUSE_COUNT,
    PLAN_LEAF_COUNT,
    PLAN_LEAF_REUSE_COUNT,
    PLAN_LEAF_REBUILD_COUNT,
)
REQUIRED_COUNTERS = (
    PREPARED_FRAME_COUNT,
    INPUT_SEGMENT_COUNT,
    CHANGED_SEGMENT_COUNT,
    CHANGED_SEGMENT_COMMAND_COUNT,
    FULL_FALLBACK_COUNT,
    *SURFACE_REUSE_COUNTERS,
    *PLAN_LEAF_COUNTERS,
    PLAN_COMMAND_VISIT_COUNT,
    CHANGED_IMAGE_DEPENDENCY_COUNT,
    IMAGE_DEPENDENCY_CHECK_COUNT,
    IMAGE_BINDING_LOOKUP_COUNT,
    IMAGE_BINDING_RETENTION_SCAN_COUNT,
    CHANGED_TEXT_BATCH_COUNT,
    TEXT_BATCH_VISIT_COUNT,
    CHANGED_GLYPH_COUNT,
    GLYPH_PROJECTION_COUNT,
    CHANGED_TEXT_DEPENDENCY_SEGMENT_COUNT,
    CHANGED_TEXT_DEPENDENCY_COUNT,
    TEXT_DEPENDENCY_SEGMENT_VISIT_COUNT,
    TEXT_DEPENDENCY_ENTRY_VISIT_COUNT,
)

CRITICAL_SOURCE_FILES = (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
)


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("evidence output must be written to D:, E:, or F:")
    return path


def validate_source_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    blockers: list[dict[str, Any]] = []
    if not isinstance(manifest, dict) or manifest.get("schema_version") != 2:
        blockers.append({"code": "invalid_source_manifest_schema"})
        return blockers

    if manifest.get("scenario") != "render_segment_delta":
        blockers.append(
            {
                "code": "invalid_source_manifest_scenario",
                "scenario": manifest.get("scenario"),
            }
        )

    options = manifest.get("capture", {}).get("options", {})
    if options.get("run_phase") != "measured":
        blockers.append(
            {"code": "capture_not_measured", "phase": options.get("run_phase")}
        )
    if (
        not isinstance(options.get("run_ordinal"), int)
        or options.get("run_ordinal", 0) <= 0
        or not isinstance(options.get("measured_run_count"), int)
        or options.get("measured_run_count", 0) <= 0
    ):
        blockers.append({"code": "invalid_capture_run_contract"})
    if options.get("warmup_complete") is not True:
        blockers.append({"code": "warmup_not_complete"})

    repository = manifest.get("repository", {})
    git = repository.get("git", {})
    revision = git.get("revision")
    if not isinstance(revision, str) or not re.fullmatch(
        r"[0-9a-fA-F]{40}", revision
    ):
        blockers.append({"code": "invalid_source_revision", "revision": revision})
    if not isinstance(git.get("dirty_paths"), list):
        blockers.append({"code": "invalid_dirty_paths"})

    files = repository.get("critical_source_files", [])
    if not isinstance(files, list):
        blockers.append({"code": "invalid_critical_source_files"})
    else:
        entries = {
            entry.get("relative_path"): entry
            for entry in files
            if isinstance(entry, dict) and isinstance(entry.get("relative_path"), str)
        }
        for source in CRITICAL_SOURCE_FILES:
            entry = entries.get(source)
            if entry is None:
                blockers.append({"code": "missing_critical_source", "path": source})
                continue
            source_hash = entry.get("sha256")
            if not isinstance(source_hash, str) or not re.fullmatch(
                r"[0-9a-fA-F]{64}", source_hash
            ):
                blockers.append(
                    {
                        "code": "invalid_critical_source_hash",
                        "path": source,
                        "sha256": source_hash,
                    }
                )
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
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            return None
        if not math.isfinite(value) or value < 0 or not float(value).is_integer():
            return None
        total += int(value)
    return total


def _append_exact_work_blocker(
    blockers: list[dict[str, Any]],
    code: str,
    expected: dict[str, int],
    totals: dict[str, int],
) -> None:
    observed = {name: totals[name] for name in expected}
    if observed != expected:
        blockers.append({"code": code, "expected": expected, "observed": observed})


def evaluate_render_dependency_delta_evidence(
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

    summary: dict[str, Any] = {
        "prepared_frames": totals.get(PREPARED_FRAME_COUNT),
        "input_surface_segments": totals.get(INPUT_SEGMENT_COUNT),
        "changed_surface_segments": totals.get(CHANGED_SEGMENT_COUNT),
        "expected_reused_surface_segments": None,
        "expected_reused_command_leaves": None,
    }

    if len(totals) == len(REQUIRED_COUNTERS):
        prepared_frames = totals[PREPARED_FRAME_COUNT]
        input_segments = totals[INPUT_SEGMENT_COUNT]
        changed_segments = totals[CHANGED_SEGMENT_COUNT]
        if prepared_frames <= 0 or input_segments <= 0 or changed_segments <= 0:
            blockers.append(
                {
                    "code": "missing_delta_activity",
                    "prepared_frames": prepared_frames,
                    "input_segments": input_segments,
                    "changed_segments": changed_segments,
                }
            )
        elif changed_segments > input_segments:
            blockers.append(
                {
                    "code": "changed_segment_count_exceeds_input",
                    "input_segments": input_segments,
                    "changed_segments": changed_segments,
                }
            )
        else:
            expected_reused_surface_segments = input_segments - changed_segments
            summary["expected_reused_surface_segments"] = expected_reused_surface_segments
            observed_reuse = {
                name: totals[name] for name in SURFACE_REUSE_COUNTERS
            }
            if any(
                value != expected_reused_surface_segments
                for value in observed_reuse.values()
            ):
                blockers.append(
                    {
                        "code": "surface_segment_reuse_conservation_failed",
                        "expected": expected_reused_surface_segments,
                        "observed": observed_reuse,
                    }
                )

        command_leaf_count = totals[PLAN_LEAF_COUNT]
        command_leaf_rebuild_count = totals[PLAN_LEAF_REBUILD_COUNT]
        if command_leaf_count <= 0 or not 0 < command_leaf_rebuild_count <= command_leaf_count:
            blockers.append(
                {
                    "code": "invalid_command_leaf_delta",
                    "command_leaf_count": command_leaf_count,
                    "command_leaf_rebuild_count": command_leaf_rebuild_count,
                }
            )
        else:
            expected_reused_command_leaves = (
                command_leaf_count - command_leaf_rebuild_count
            )
            summary["expected_reused_command_leaves"] = expected_reused_command_leaves
            observed_leaf_reuse = {
                LEGACY_PLAN_REUSE_COUNT: totals[LEGACY_PLAN_REUSE_COUNT],
                PLAN_LEAF_REUSE_COUNT: totals[PLAN_LEAF_REUSE_COUNT],
            }
            if any(
                value != expected_reused_command_leaves
                for value in observed_leaf_reuse.values()
            ):
                blockers.append(
                    {
                        "code": "command_leaf_reuse_conservation_failed",
                        "expected": expected_reused_command_leaves,
                        "observed": observed_leaf_reuse,
                    }
                )

        if totals[FULL_FALLBACK_COUNT] != 0:
            blockers.append(
                {
                    "code": "unexpected_full_fallback",
                    "count": totals[FULL_FALLBACK_COUNT],
                }
            )

        _append_exact_work_blocker(
            blockers,
            "plan_command_delta_failed",
            {PLAN_COMMAND_VISIT_COUNT: totals[CHANGED_SEGMENT_COMMAND_COUNT]},
            totals,
        )
        _append_exact_work_blocker(
            blockers,
            "image_dependency_delta_failed",
            {
                IMAGE_DEPENDENCY_CHECK_COUNT: totals[CHANGED_IMAGE_DEPENDENCY_COUNT],
                IMAGE_BINDING_LOOKUP_COUNT: totals[CHANGED_IMAGE_DEPENDENCY_COUNT],
            },
            totals,
        )
        if totals[IMAGE_BINDING_RETENTION_SCAN_COUNT] != 0:
            blockers.append(
                {
                    "code": "binding_global_scan_detected",
                    "count": totals[IMAGE_BINDING_RETENTION_SCAN_COUNT],
                }
            )
        _append_exact_work_blocker(
            blockers,
            "text_product_delta_failed",
            {
                TEXT_BATCH_VISIT_COUNT: totals[CHANGED_TEXT_BATCH_COUNT],
                GLYPH_PROJECTION_COUNT: totals[CHANGED_GLYPH_COUNT],
            },
            totals,
        )
        _append_exact_work_blocker(
            blockers,
            "text_dependency_delta_failed",
            {
                TEXT_DEPENDENCY_SEGMENT_VISIT_COUNT: totals[
                    CHANGED_TEXT_DEPENDENCY_SEGMENT_COUNT
                ],
                TEXT_DEPENDENCY_ENTRY_VISIT_COUNT: totals[
                    CHANGED_TEXT_DEPENDENCY_COUNT
                ],
            },
            totals,
        )

    return {
        "schema": "zircon.runtime.ui_render_dependency_delta_evidence.v2",
        "ready": not blockers,
        "summary": summary,
        "counter_totals": totals,
        "blockers": blockers,
        "claim": (
            "measured changed-segment frames; surface image/text reuse and command-leaf "
            "planner reuse must conserve independently, while render, binding, text, and "
            "dependency work equals only the explicitly published delta"
        ),
    }


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("profile_dir", type=Path)
    parser.add_argument("--timeline", type=Path)
    parser.add_argument("--source-manifest", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    profile_dir = args.profile_dir.resolve()
    timeline_path = (args.timeline or profile_dir / "timeline.json").resolve()
    source_manifest_path = (
        args.source_manifest or profile_dir / "source_manifest.json"
    ).resolve()
    output_path = validate_output_path(args.output.resolve())

    timeline = json.loads(timeline_path.read_text(encoding="utf-8"))
    result = evaluate_render_dependency_delta_evidence(timeline)
    if not source_manifest_path.is_file():
        result["blockers"].append(
            {"code": "missing_source_manifest", "path": str(source_manifest_path)}
        )
    else:
        manifest = json.loads(source_manifest_path.read_text(encoding="utf-8"))
        result["blockers"].extend(validate_source_manifest(manifest))
    result["ready"] = not result["blockers"]
    result["evidence"] = {
        "timeline_path": str(timeline_path),
        "timeline_sha256": _sha256(timeline_path),
        "source_manifest_path": str(source_manifest_path),
        "source_manifest_sha256": (
            _sha256(source_manifest_path) if source_manifest_path.is_file() else None
        ),
        "timing_claim": False,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    return 0 if result["ready"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
