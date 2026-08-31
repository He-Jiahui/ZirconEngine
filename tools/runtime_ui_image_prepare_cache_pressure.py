"""Model Runtime UI image resolution and segment-retained prepare work."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    frame_count: int = 4_096,
    image_batches_per_frame: int = 1_024,
    unique_texture_count: int = 64,
    unresolved_unique_texture_count: int = 8,
    registry_record_count: int = 16_384,
    management_generation_count: int = 4,
    segment_count: int = 64,
    segment_plan_change_count: int = 32,
) -> dict[str, object]:
    positive_inputs = {
        "frame_count": frame_count,
        "image_batches_per_frame": image_batches_per_frame,
        "unique_texture_count": unique_texture_count,
        "registry_record_count": registry_record_count,
        "management_generation_count": management_generation_count,
        "segment_count": segment_count,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if unresolved_unique_texture_count < 0:
        raise ValueError("unresolved_unique_texture_count must be non-negative")
    if unresolved_unique_texture_count > unique_texture_count:
        raise ValueError(
            "unresolved_unique_texture_count must not exceed unique_texture_count"
        )
    if unique_texture_count > image_batches_per_frame:
        raise ValueError(
            "unique_texture_count must not exceed image_batches_per_frame"
        )
    if management_generation_count > frame_count:
        raise ValueError(
            "management_generation_count must not exceed frame_count"
        )
    if segment_plan_change_count < 0:
        raise ValueError("segment_plan_change_count must be non-negative")
    if segment_count > image_batches_per_frame:
        raise ValueError("segment_count must not exceed image_batches_per_frame")
    if image_batches_per_frame % segment_count != 0:
        raise ValueError("image_batches_per_frame must be divisible by segment_count")

    resolved_unique_texture_count = (
        unique_texture_count - unresolved_unique_texture_count
    )
    average_resolved_registry_probes = max(1, registry_record_count // 2)
    registry_record_visits_per_resolution_set = (
        resolved_unique_texture_count * average_resolved_registry_probes
        + unresolved_unique_texture_count * registry_record_count
    )
    image_texture_cache_lookups = frame_count * image_batches_per_frame
    per_frame_resolution_misses = frame_count * unique_texture_count
    generation_resolution_misses = management_generation_count * unique_texture_count
    per_frame_registry_visits = (
        frame_count * registry_record_visits_per_resolution_set
    )
    generation_registry_visits = (
        management_generation_count * registry_record_visits_per_resolution_set
    )
    image_batches_per_segment = image_batches_per_frame // segment_count
    full_frame_image_batch_visits = frame_count * image_batches_per_frame
    retained_segment_rebuild_count = segment_count + segment_plan_change_count
    retained_image_batch_visits = (
        retained_segment_rebuild_count * image_batches_per_segment
    )
    image_vertices_per_batch = 6
    image_vertex_size_bytes = 32
    full_frame_vertex_materializations = (
        full_frame_image_batch_visits * image_vertices_per_batch
    )
    retained_vertex_materializations = (
        retained_image_batch_visits * image_vertices_per_batch
    )
    unique_texture_dependency_checks = frame_count * unique_texture_count

    return {
        "schema": "zircon.runtime.ui_image_prepare_cache_pressure.v2",
        "inputs": {
            **positive_inputs,
            "unresolved_unique_texture_count": unresolved_unique_texture_count,
            "segment_plan_change_count": segment_plan_change_count,
        },
        "per_frame_resolution_cache": {
            "image_texture_cache_lookups": image_texture_cache_lookups,
            "resolution_cache_misses": per_frame_resolution_misses,
            "registry_record_visits": per_frame_registry_visits,
        },
        "generation_retained_resolution_cache": {
            "image_texture_cache_lookups": image_texture_cache_lookups,
            "resolution_cache_misses": generation_resolution_misses,
            "registry_record_visits": generation_registry_visits,
        },
        "full_frame_image_prepare": {
            "image_batch_visits": full_frame_image_batch_visits,
            "image_vertex_materializations": full_frame_vertex_materializations,
            "image_vertex_hash_input_bytes": (
                full_frame_vertex_materializations * image_vertex_size_bytes
            ),
            "image_texture_cache_lookups": image_texture_cache_lookups,
        },
        "segment_retained_image_prepare": {
            "segment_rebuilds": retained_segment_rebuild_count,
            "image_batch_visits": retained_image_batch_visits,
            "image_vertex_materializations": retained_vertex_materializations,
            "image_vertex_hash_input_bytes": (
                retained_vertex_materializations * image_vertex_size_bytes
            ),
            "unique_texture_dependency_checks": unique_texture_dependency_checks,
            "requested_identity_cache_lookups": generation_resolution_misses,
            "gpu_texture_map_lookups": unique_texture_dependency_checks,
        },
        "delta": {
            "avoided_resolution_cache_misses": (
                per_frame_resolution_misses - generation_resolution_misses
            ),
            "avoided_registry_record_visits": (
                per_frame_registry_visits - generation_registry_visits
            ),
            "registry_record_visit_reduction_ratio": (
                per_frame_registry_visits / generation_registry_visits
            ),
            "avoided_image_batch_visits": (
                full_frame_image_batch_visits - retained_image_batch_visits
            ),
            "image_batch_visit_reduction_ratio": round(
                full_frame_image_batch_visits / retained_image_batch_visits, 6
            ),
            "avoided_image_vertex_hash_input_bytes": (
                full_frame_vertex_materializations - retained_vertex_materializations
            )
            * image_vertex_size_bytes,
        },
        "interpretation": {
            "included": "requested-to-imported texture identity cache lookups, positive locator-derived registry scans, negative full-registry scans, management-generation invalidation, image batch visits, image vertex materialization, vertex hash input bytes, and stable-frame unique texture dependency checks",
            "excluded": "GPU buffer writes, bind-group lookup cost, texture-map lookup cost, SVG parsing inside the asset pipeline, additional per-segment vertex-buffer binds, actual CPU time, and allocator latency",
            "scope": "deterministic stable-frame model; image batches and globally unique texture dependencies are distributed evenly across segments, resolved identities use a half-registry average probe count, and unresolved identities conservatively scan the complete registry",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frame-count", type=int, default=4_096)
    parser.add_argument("--image-batches-per-frame", type=int, default=1_024)
    parser.add_argument("--unique-texture-count", type=int, default=64)
    parser.add_argument("--unresolved-unique-texture-count", type=int, default=8)
    parser.add_argument("--registry-record-count", type=int, default=16_384)
    parser.add_argument("--management-generation-count", type=int, default=4)
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--segment-plan-change-count", type=int, default=32)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.frame_count,
        args.image_batches_per_frame,
        args.unique_texture_count,
        args.unresolved_unique_texture_count,
        args.registry_record_count,
        args.management_generation_count,
        args.segment_count,
        args.segment_plan_change_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
