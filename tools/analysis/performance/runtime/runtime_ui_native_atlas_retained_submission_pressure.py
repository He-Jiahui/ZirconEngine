"""Model current and retained Runtime UI native-atlas submission work."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    frame_count: int = 4_096,
    text_batches_per_frame: int = 1_024,
    glyphs_per_batch: int = 24,
    unique_glyph_dependency_count: int = 2_048,
    segment_count: int = 64,
    font_generation_count: int = 4,
    segment_plan_change_count: int = 32,
    readiness_key_change_count: int = 256,
) -> dict[str, object]:
    positive_inputs = {
        "frame_count": frame_count,
        "text_batches_per_frame": text_batches_per_frame,
        "glyphs_per_batch": glyphs_per_batch,
        "unique_glyph_dependency_count": unique_glyph_dependency_count,
        "segment_count": segment_count,
        "font_generation_count": font_generation_count,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    non_negative_inputs = {
        "segment_plan_change_count": segment_plan_change_count,
        "readiness_key_change_count": readiness_key_change_count,
    }
    for name, value in non_negative_inputs.items():
        if value < 0:
            raise ValueError(f"{name} must be non-negative")
    if segment_count > text_batches_per_frame:
        raise ValueError("segment_count must not exceed text_batches_per_frame")
    if text_batches_per_frame % segment_count != 0:
        raise ValueError("text_batches_per_frame must be divisible by segment_count")
    if font_generation_count > frame_count:
        raise ValueError("font_generation_count must not exceed frame_count")

    glyph_instances_per_frame = text_batches_per_frame * glyphs_per_batch
    if unique_glyph_dependency_count > glyph_instances_per_frame:
        raise ValueError(
            "unique_glyph_dependency_count must not exceed glyph instances per frame"
        )
    if glyph_instances_per_frame % unique_glyph_dependency_count != 0:
        raise ValueError(
            "glyph instances per frame must be divisible by unique_glyph_dependency_count"
        )

    glyph_instances_per_segment = glyph_instances_per_frame // segment_count
    instances_per_dependency = (
        glyph_instances_per_frame // unique_glyph_dependency_count
    )
    current_readiness_discovery_visits = frame_count * glyph_instances_per_frame
    current_unique_readiness_lookups = frame_count * unique_glyph_dependency_count
    current_ordered_geometry_visits = frame_count * glyph_instances_per_frame
    current_total_work = (
        current_readiness_discovery_visits
        + current_unique_readiness_lookups
        + current_ordered_geometry_visits
    )

    segment_geometry_rebuilds = (
        segment_count * font_generation_count + segment_plan_change_count
    )
    retained_geometry_visits = (
        segment_geometry_rebuilds * glyph_instances_per_segment
    )
    retained_active_readiness_checks = (
        frame_count * unique_glyph_dependency_count
    )
    readiness_patch_instance_visits = (
        readiness_key_change_count * instances_per_dependency
    )
    retained_total_work = (
        retained_geometry_visits
        + retained_active_readiness_checks
        + readiness_patch_instance_visits
    )

    return {
        "schema": "zircon.runtime.ui_native_atlas_retained_submission_pressure.v1",
        "inputs": {
            **positive_inputs,
            **non_negative_inputs,
            "glyph_instances_per_frame": glyph_instances_per_frame,
            "glyph_instances_per_segment": glyph_instances_per_segment,
            "instances_per_dependency": instances_per_dependency,
        },
        "current_two_pass_native_prepare": {
            "readiness_discovery_glyph_visits": current_readiness_discovery_visits,
            "unique_readiness_cache_lookups": current_unique_readiness_lookups,
            "ordered_geometry_glyph_visits": current_ordered_geometry_visits,
            "total_modeled_work": current_total_work,
        },
        "retained_native_submission": {
            "segment_geometry_rebuilds": segment_geometry_rebuilds,
            "segment_geometry_rebuild_glyph_visits": retained_geometry_visits,
            "active_readiness_checks": retained_active_readiness_checks,
            "readiness_patch_instance_visits": readiness_patch_instance_visits,
            "total_modeled_work": retained_total_work,
        },
        "retained_index_shape": {
            "segment_product_count": segment_count,
            "unique_dependency_records": unique_glyph_dependency_count,
            "reverse_instance_index_entries": glyph_instances_per_frame,
        },
        "delta": {
            "avoided_modeled_work": current_total_work - retained_total_work,
            "modeled_work_reduction_ratio": round(
                current_total_work / retained_total_work, 6
            ),
            "avoided_ordered_geometry_visits": (
                current_ordered_geometry_visits
                - retained_geometry_visits
                - readiness_patch_instance_visits
            ),
            "readiness_lookup_reduction_ratio": 1.0,
        },
        "interpretation": {
            "included": "per-frame glyph visits needed to rediscover unique raster dependencies, unique readiness cache checks, ordered native geometry/source-image materialization, font-generation rebuilds, local segment-plan changes, and reverse-index patches for changed readiness keys",
            "excluded": "actual CPU and allocator latency, hash-map constants, worker raster time, atlas allocation and texture upload, GPU buffer writes, draw encoding, page binding, retry scheduling, RSS, and the concrete byte size of retained dependency/index records",
            "scope": "deterministic operation-count model; glyph instances are evenly distributed across segments and raster dependencies, every font generation rebuilds immutable segment geometry, each readiness update patches all instances of one dependency, and active readiness remains checked once per unique dependency per frame",
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
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--font-generation-count", type=int, default=4)
    parser.add_argument("--segment-plan-change-count", type=int, default=32)
    parser.add_argument("--readiness-key-change-count", type=int, default=256)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.frame_count,
        args.text_batches_per_frame,
        args.glyphs_per_batch,
        args.unique_glyph_dependency_count,
        args.segment_count,
        args.font_generation_count,
        args.segment_plan_change_count,
        args.readiness_key_change_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        _reject_c_drive(args.output)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
