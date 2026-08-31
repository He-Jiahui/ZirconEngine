"""Model stable-frame Runtime UI screen-space planner work."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMPLEMENTATION_SOURCES = (
    "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs",
    "zircon_runtime/src/ui/surface/surface/frame_publication.rs",
    "zircon_runtime/src/core/framework/render/ui_submission.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
)
PRIMARY_REFERENCE_SOURCES = (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp",
)
SECONDARY_REFERENCE_SOURCES = (
    "dev/slint/internal/core/partial_renderer.rs",
)


def source_bindings(paths: tuple[str, ...]) -> list[dict[str, object]]:
    bindings = []
    for relative_path in paths:
        payload = (ROOT / relative_path).read_bytes()
        bindings.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest().upper(),
            }
        )
    return bindings


def run(
    frame_count: int = 4_096,
    plan_build_count: int = 64,
    commands_per_submission: int = 32_768,
    text_batches_per_submission: int = 4_096,
    modeled_text_bytes_per_batch: int = 48,
    vertices_per_plan: int = 196_608,
    modeled_vertex_bytes: int = 24,
    surface_count: int = 64,
    changed_surface_count: int = 1,
    background_dependent_suffix_count: int = 63,
    draws_per_plan: int = 32_768,
    post_text_draws_per_plan: int = 4_096,
    image_batches_per_plan: int = 1_024,
    command_segment_size: int = 64,
    changed_command_segment_count: int = 1,
) -> dict[str, object]:
    positive_inputs = {
        "frame_count": frame_count,
        "plan_build_count": plan_build_count,
        "commands_per_submission": commands_per_submission,
        "text_batches_per_submission": text_batches_per_submission,
        "modeled_text_bytes_per_batch": modeled_text_bytes_per_batch,
        "vertices_per_plan": vertices_per_plan,
        "modeled_vertex_bytes": modeled_vertex_bytes,
        "surface_count": surface_count,
        "changed_surface_count": changed_surface_count,
        "draws_per_plan": draws_per_plan,
        "post_text_draws_per_plan": post_text_draws_per_plan,
        "image_batches_per_plan": image_batches_per_plan,
        "command_segment_size": command_segment_size,
        "changed_command_segment_count": changed_command_segment_count,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    positive_inputs["background_dependent_suffix_count"] = (
        background_dependent_suffix_count
    )
    if plan_build_count > frame_count:
        raise ValueError("plan_build_count must not exceed frame_count")
    if commands_per_submission % surface_count != 0:
        raise ValueError("commands_per_submission must be divisible by surface_count")
    if changed_surface_count > surface_count:
        raise ValueError("changed_surface_count must not exceed surface_count")
    if background_dependent_suffix_count < 0:
        raise ValueError("background_dependent_suffix_count must be non-negative")
    if changed_surface_count + background_dependent_suffix_count > surface_count:
        raise ValueError(
            "changed_surface_count plus background_dependent_suffix_count must not exceed surface_count"
        )

    commands_per_surface = commands_per_submission // surface_count
    if commands_per_surface % command_segment_size != 0:
        raise ValueError("commands_per_surface must be divisible by command_segment_size")
    command_segments_per_surface = commands_per_surface // command_segment_size
    changed_command_segment_capacity = changed_surface_count * command_segments_per_surface
    if changed_command_segment_count > changed_command_segment_capacity:
        raise ValueError(
            "changed_command_segment_count exceeds the changed-surface leaf capacity"
        )
    if vertices_per_plan % commands_per_submission != 0:
        raise ValueError("vertices_per_plan must be divisible by commands_per_submission")

    retired_command_visits = frame_count * commands_per_submission
    retained_command_visits = plan_build_count * commands_per_submission
    changed_generation_count = plan_build_count - 1
    segmented_neutral_command_visits = commands_per_submission + (
        changed_generation_count * changed_surface_count * commands_per_surface
    )
    segmented_background_suffix_command_visits = commands_per_submission + (
        changed_generation_count
        * (changed_surface_count + background_dependent_suffix_count)
        * commands_per_surface
    )
    segmented_cache_hit_count = (
        changed_generation_count * (surface_count - changed_surface_count)
    )
    command_segment_count = surface_count * command_segments_per_surface
    command_leaf_visits = commands_per_submission + (
        changed_generation_count
        * changed_command_segment_count
        * command_segment_size
    )
    command_leaf_cache_hit_count = changed_generation_count * (
        command_segment_count - changed_command_segment_count
    )
    flat_composition_payload_clone_count = plan_build_count * (
        vertices_per_plan
        + draws_per_plan
        + post_text_draws_per_plan
        + text_batches_per_submission
        + image_batches_per_plan
    )
    segmented_composition_payload_clone_count = 0
    retained_render_segment_reference_count = plan_build_count * surface_count
    vertices_per_surface = vertices_per_plan // surface_count
    segmented_neutral_vertex_hash_pass_count = surface_count + (
        changed_generation_count * changed_surface_count
    )
    segmented_neutral_vertex_hash_input_bytes = (
        vertices_per_plan
        + changed_generation_count * changed_surface_count * vertices_per_surface
    ) * modeled_vertex_bytes
    vertices_per_command = vertices_per_plan // commands_per_submission
    command_leaf_vertex_hash_pass_count = command_segment_count + (
        changed_generation_count * changed_command_segment_count
    )
    command_leaf_vertex_hash_input_bytes = (
        vertices_per_plan
        + changed_generation_count
        * changed_command_segment_count
        * command_segment_size
        * vertices_per_command
    ) * modeled_vertex_bytes
    segmented_background_vertex_hash_pass_count = surface_count + (
        changed_generation_count
        * (changed_surface_count + background_dependent_suffix_count)
    )
    segmented_background_vertex_hash_input_bytes = (
        vertices_per_plan
        + changed_generation_count
        * (changed_surface_count + background_dependent_suffix_count)
        * vertices_per_surface
    ) * modeled_vertex_bytes
    retired_planner_text_clone_bytes = (
        frame_count * text_batches_per_submission * modeled_text_bytes_per_batch
    )
    retained_planner_text_clone_bytes = (
        plan_build_count
        * text_batches_per_submission
        * modeled_text_bytes_per_batch
    )
    retired_vertex_hash_bytes = frame_count * vertices_per_plan * modeled_vertex_bytes
    retained_vertex_hash_bytes = (
        plan_build_count * vertices_per_plan * modeled_vertex_bytes
    )
    cache_hit_count = frame_count - plan_build_count
    return {
        "schema": "zircon.runtime.ui_screen_space_plan_cache_pressure.v6",
        "source_binding": {
            "implementation": source_bindings(IMPLEMENTATION_SOURCES),
            "primary_reference": source_bindings(PRIMARY_REFERENCE_SOURCES),
            "secondary_reference": source_bindings(SECONDARY_REFERENCE_SOURCES),
        },
        "inputs": positive_inputs,
        "retired_per_frame_planning": {
            "plan_build_count": frame_count,
            "command_visits": retired_command_visits,
            "paint_element_fill_calls": retired_command_visits,
            "modeled_planner_text_payload_clone_bytes": retired_planner_text_clone_bytes,
            "vertex_hash_pass_count": frame_count,
            "modeled_vertex_hash_input_bytes": retired_vertex_hash_bytes,
            "renderer_text_prepare_report_snapshot_clone_count": frame_count,
        },
        "retained_generation_planning": {
            "plan_build_count": plan_build_count,
            "cache_hit_count": cache_hit_count,
            "cache_identity_checks": frame_count,
            "command_visits": retained_command_visits,
            "paint_element_fill_calls": retained_command_visits,
            "modeled_planner_text_payload_clone_bytes": retained_planner_text_clone_bytes,
            "vertex_hash_pass_count": plan_build_count,
            "modeled_vertex_hash_input_bytes": retained_vertex_hash_bytes,
            "renderer_text_prepare_report_snapshot_clone_count": 0,
        },
        "retained_segment_planning_non_background_change": {
            "initial_full_command_visits": commands_per_submission,
            "changed_generation_count": changed_generation_count,
            "changed_surface_count_per_generation": changed_surface_count,
            "segment_cache_hit_count": segmented_cache_hit_count,
            "command_visits": segmented_neutral_command_visits,
            "paint_element_fill_calls": segmented_neutral_command_visits,
            "retained_render_segment_reference_count": retained_render_segment_reference_count,
            "flat_composition_payload_clone_count": flat_composition_payload_clone_count,
            "composition_payload_clone_count": segmented_composition_payload_clone_count,
            "vertex_hash_pass_count": segmented_neutral_vertex_hash_pass_count,
            "modeled_vertex_hash_input_bytes": segmented_neutral_vertex_hash_input_bytes,
            "modeled_vertex_upload_bytes_upper_bound": segmented_neutral_vertex_hash_input_bytes,
        },
        "retained_segment_planning_background_suffix_change": {
            "background_dependent_suffix_count": background_dependent_suffix_count,
            "command_visits": segmented_background_suffix_command_visits,
            "paint_element_fill_calls": segmented_background_suffix_command_visits,
            "retained_render_segment_reference_count": retained_render_segment_reference_count,
            "flat_composition_payload_clone_count": flat_composition_payload_clone_count,
            "composition_payload_clone_count": segmented_composition_payload_clone_count,
            "vertex_hash_pass_count": segmented_background_vertex_hash_pass_count,
            "modeled_vertex_hash_input_bytes": segmented_background_vertex_hash_input_bytes,
            "modeled_vertex_upload_bytes_upper_bound": segmented_background_vertex_hash_input_bytes,
        },
        "retained_command_leaf_planning_non_background_change": {
            "initial_full_command_visits": commands_per_submission,
            "changed_generation_count": changed_generation_count,
            "command_segment_count": command_segment_count,
            "changed_command_segment_count_per_generation": changed_command_segment_count,
            "command_leaf_cache_hit_count": command_leaf_cache_hit_count,
            "command_visits": command_leaf_visits,
            "paint_element_fill_calls": command_leaf_visits,
            "retained_render_segment_reference_count": plan_build_count
            * command_segment_count,
            "composition_payload_clone_count": 0,
            "vertex_hash_pass_count": command_leaf_vertex_hash_pass_count,
            "modeled_vertex_hash_input_bytes": command_leaf_vertex_hash_input_bytes,
            "modeled_vertex_upload_bytes_upper_bound": command_leaf_vertex_hash_input_bytes,
        },
        "delta": {
            "avoided_plan_builds": cache_hit_count,
            "avoided_command_visits": retired_command_visits
            - retained_command_visits,
            "avoided_paint_element_fill_calls": retired_command_visits
            - retained_command_visits,
            "avoided_modeled_planner_text_payload_clone_bytes": retired_planner_text_clone_bytes
            - retained_planner_text_clone_bytes,
            "command_visit_reduction_ratio": retired_command_visits
            / retained_command_visits,
            "avoided_vertex_hash_passes": cache_hit_count,
            "avoided_modeled_vertex_hash_input_bytes": retired_vertex_hash_bytes
            - retained_vertex_hash_bytes,
            "vertex_hash_input_reduction_ratio": retired_vertex_hash_bytes
            / retained_vertex_hash_bytes,
            "avoided_renderer_text_prepare_report_snapshot_clones": frame_count,
            "segment_non_background_avoided_command_visits_vs_whole_plan_cache": retained_command_visits
            - segmented_neutral_command_visits,
            "segment_non_background_command_visit_reduction_ratio_vs_whole_plan_cache": retained_command_visits
            / segmented_neutral_command_visits,
            "segment_consumer_avoided_composition_payload_clones": flat_composition_payload_clone_count
            - segmented_composition_payload_clone_count,
            "command_leaf_non_background_avoided_command_visits_vs_surface_segment": segmented_neutral_command_visits
            - command_leaf_visits,
            "command_leaf_non_background_command_visit_reduction_ratio_vs_surface_segment": segmented_neutral_command_visits
            / command_leaf_visits,
            "command_leaf_non_background_avoided_vertex_hash_input_bytes_vs_surface_segment": segmented_neutral_vertex_hash_input_bytes
            - command_leaf_vertex_hash_input_bytes,
            "command_leaf_non_background_vertex_hash_input_reduction_ratio_vs_surface_segment": segmented_neutral_vertex_hash_input_bytes
            / command_leaf_vertex_hash_input_bytes,
            "segment_non_background_avoided_vertex_hash_input_bytes_vs_whole_plan_cache": retained_vertex_hash_bytes
            - segmented_neutral_vertex_hash_input_bytes,
            "segment_non_background_vertex_hash_input_reduction_ratio_vs_whole_plan_cache": retained_vertex_hash_bytes
            / segmented_neutral_vertex_hash_input_bytes,
        },
        "interpretation": {
            "included": "screen-space plan builds, whole-plan, surface-segment, and persistent command-leaf visits, paint-element fill calls, prefix-dependent command-leaf cache hits, retired flat-composition payload clones, retained segment references, cache identity checks, explicitly modeled source-text bytes materialized by the planner into cached text batches, explicitly modeled segment-local vertex bytes visited by BLAKE3 payload hashing, a conservative changed-leaf vertex upload upper bound, and renderer-owned text prepare report snapshot clone operations",
            "excluded": "per-frame downstream text-system batch clones and routing, text shaping and atlas work after planning, consumer-requested text prepare report clones, payload bytes represented by each composition item, image preparation, actual GPU writes skipped by equal segment payload hashes, GPU buffer allocation and binding overhead, GPU command encoding, draw calls, world rendering, allocator latency, Arc atomic latency, actual command/text/vertex distributions, measured CPU time, and measured RSS",
            "scope": "deterministic stable-frame and changed-segment work model; unchanged background-effect deltas preserve suffix cache generations, while the background suffix scenario models conservative downstream invalidation; segment-local uploads are an upper bound rather than measured queue traffic",
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frame-count", type=int, default=4_096)
    parser.add_argument("--plan-build-count", type=int, default=64)
    parser.add_argument("--commands-per-submission", type=int, default=32_768)
    parser.add_argument("--text-batches-per-submission", type=int, default=4_096)
    parser.add_argument("--modeled-text-bytes-per-batch", type=int, default=48)
    parser.add_argument("--vertices-per-plan", type=int, default=196_608)
    parser.add_argument("--modeled-vertex-bytes", type=int, default=24)
    parser.add_argument("--surface-count", type=int, default=64)
    parser.add_argument("--changed-surface-count", type=int, default=1)
    parser.add_argument("--background-dependent-suffix-count", type=int, default=63)
    parser.add_argument("--draws-per-plan", type=int, default=32_768)
    parser.add_argument("--post-text-draws-per-plan", type=int, default=4_096)
    parser.add_argument("--image-batches-per-plan", type=int, default=1_024)
    parser.add_argument("--command-segment-size", type=int, default=64)
    parser.add_argument("--changed-command-segment-count", type=int, default=1)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.frame_count,
        args.plan_build_count,
        args.commands_per_submission,
        args.text_batches_per_submission,
        args.modeled_text_bytes_per_batch,
        args.vertices_per_plan,
        args.modeled_vertex_bytes,
        args.surface_count,
        args.changed_surface_count,
        args.background_dependent_suffix_count,
        args.draws_per_plan,
        args.post_text_draws_per_plan,
        args.image_batches_per_plan,
        args.command_segment_size,
        args.changed_command_segment_count,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
