"""Model flat-frame and segment-retained Runtime UI text prepare work."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    frame_count: int = 4_096,
    text_batches_per_frame: int = 1_024,
    glyphs_per_batch: int = 24,
    unique_glyph_dependency_count: int = 2_048,
    unique_font_asset_count: int = 8,
    font_generation_count: int = 4,
    segment_count: int = 64,
    segment_plan_change_count: int = 32,
    average_text_bytes_per_batch: int = 32,
) -> dict[str, object]:
    positive_inputs = {
        "frame_count": frame_count,
        "text_batches_per_frame": text_batches_per_frame,
        "glyphs_per_batch": glyphs_per_batch,
        "unique_glyph_dependency_count": unique_glyph_dependency_count,
        "unique_font_asset_count": unique_font_asset_count,
        "font_generation_count": font_generation_count,
        "segment_count": segment_count,
        "average_text_bytes_per_batch": average_text_bytes_per_batch,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if segment_plan_change_count < 0:
        raise ValueError("segment_plan_change_count must be non-negative")
    if segment_count > text_batches_per_frame:
        raise ValueError("segment_count must not exceed text_batches_per_frame")
    if text_batches_per_frame % segment_count != 0:
        raise ValueError("text_batches_per_frame must be divisible by segment_count")
    glyph_instances_per_frame = text_batches_per_frame * glyphs_per_batch
    if unique_glyph_dependency_count > glyph_instances_per_frame:
        raise ValueError(
            "unique_glyph_dependency_count must not exceed glyph instances per frame"
        )
    if font_generation_count > frame_count:
        raise ValueError("font_generation_count must not exceed frame_count")

    batches_per_segment = text_batches_per_frame // segment_count
    flat_batch_visits = frame_count * text_batches_per_frame
    flat_glyph_instance_projections = frame_count * glyph_instances_per_frame
    retained_segment_rebuilds = (
        segment_count * font_generation_count + segment_plan_change_count
    )
    retained_batch_visits = retained_segment_rebuilds * batches_per_segment
    retained_glyph_instance_projections = retained_batch_visits * glyphs_per_batch
    active_glyph_dependency_checks = frame_count * unique_glyph_dependency_count
    font_dependency_checks = frame_count * unique_font_asset_count
    retained_total_glyph_work = (
        retained_glyph_instance_projections + active_glyph_dependency_checks
    )

    return {
        "schema": "zircon.runtime.ui_text_prepare_cache_pressure.v1",
        "inputs": {
            **positive_inputs,
            "segment_plan_change_count": segment_plan_change_count,
        },
        "flat_frame_text_prepare": {
            "text_batch_visits": flat_batch_visits,
            "resolved_text_batch_clones": flat_batch_visits,
            "text_payload_copy_bytes": (
                flat_batch_visits * average_text_bytes_per_batch
            ),
            "glyph_instance_projections": flat_glyph_instance_projections,
            "font_asset_candidate_visits": flat_batch_visits,
        },
        "segment_retained_text_prepare": {
            "segment_rebuilds": retained_segment_rebuilds,
            "text_batch_visits": retained_batch_visits,
            "resolved_text_batch_materializations": retained_batch_visits,
            "text_payload_materialization_bytes": (
                retained_batch_visits * average_text_bytes_per_batch
            ),
            "glyph_instance_projections": retained_glyph_instance_projections,
            "active_glyph_dependency_checks": active_glyph_dependency_checks,
            "font_asset_dependency_checks": font_dependency_checks,
            "total_glyph_projection_and_dependency_work": retained_total_glyph_work,
        },
        "delta": {
            "avoided_text_batch_visits": flat_batch_visits - retained_batch_visits,
            "text_batch_visit_reduction_ratio": round(
                flat_batch_visits / retained_batch_visits, 6
            ),
            "avoided_text_payload_copy_bytes": (
                flat_batch_visits - retained_batch_visits
            )
            * average_text_bytes_per_batch,
            "glyph_work_reduction_ratio": round(
                flat_glyph_instance_projections / retained_total_glyph_work, 6
            ),
        },
        "interpretation": {
            "included": "flat resolved-batch cloning, text payload copies, glyph-instance projection, font dependency checks, font-generation invalidation, local segment-plan changes, and stable-frame unique glyph readiness checks",
            "excluded": "actual CPU and allocator latency, shaping and layout already retained upstream, atlas allocation/map constants, async raster worker time, SDF bake time, GPU buffer writes, draw calls, RSS, and per-segment buffer binding overhead",
            "scope": "deterministic renderer-prepare model; text batches are evenly distributed across segments, every font generation invalidates immutable segment products, and mutable atlas readiness remains a per-frame unique-dependency operation",
        },
    }


def _reject_c_drive(path: Path) -> None:
    if path.drive.casefold() == "c:":
        raise ValueError("performance artifacts must not be written to C drive")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frame-count", type=int, default=4_096)
    parser.add_argument("--text-batches-per-frame", type=int, default=1_024)
    parser.add_argument("--glyphs-per-batch", type=int, default=24)
    parser.add_argument("--unique-glyph-dependency-count", type=int, default=2_048)
    parser.add_argument("--unique-font-asset-count", type=int, default=8)
    parser.add_argument("--font-generation-count", type=int, default=4)
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--segment-plan-change-count", type=int, default=32)
    parser.add_argument("--average-text-bytes-per-batch", type=int, default=32)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.frame_count,
        args.text_batches_per_frame,
        args.glyphs_per_batch,
        args.unique_glyph_dependency_count,
        args.unique_font_asset_count,
        args.font_generation_count,
        args.segment_count,
        args.segment_plan_change_count,
        args.average_text_bytes_per_batch,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        _reject_c_drive(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
