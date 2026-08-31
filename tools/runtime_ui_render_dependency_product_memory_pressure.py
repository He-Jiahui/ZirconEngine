"""Model bounded retained memory for Runtime UI render dependency products."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import subprocess
from pathlib import Path
from typing import Any


CRITICAL_SOURCE_FILES = (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
)


def validate_output_path(path: Path) -> Path:
    if path.drive.upper() not in {"D:", "E:", "F:"}:
        raise ValueError("performance artifacts must be written to D:, E:, or F:")
    return path


def run(
    present_count: int = 1_000_000,
    segment_count: int = 64,
    image_dependencies_per_segment: int = 4,
    unique_image_binding_count: int = 64,
    image_batches_per_segment: int = 16,
    text_run_spans_per_segment: int = 8,
    text_glyph_instances_per_segment: int = 768,
    retained_generation_count: int = 3,
    max_retained_generation_count: int = 3,
    changed_segments_per_delta_generation: int = 1,
    metadata_budget_bytes: int = 8 * 1024 * 1024,
    binding_product_metadata_bytes: int = 128,
    weak_binding_map_entry_bytes: int = 32,
    dependency_arc_bytes: int = 8,
    segment_identity_bytes: int = 8,
    persistent_directory_node_bytes: int = 32,
    text_segment_leaf_arc_bytes: int = 8,
    text_run_span_bytes: int = 16,
    image_vertex_bytes: int = 32,
    image_vertices_per_batch: int = 6,
    text_glyph_record_bytes: int = 32,
) -> dict[str, Any]:
    positive_inputs = {
        "present_count": present_count,
        "segment_count": segment_count,
        "image_dependencies_per_segment": image_dependencies_per_segment,
        "unique_image_binding_count": unique_image_binding_count,
        "image_batches_per_segment": image_batches_per_segment,
        "text_run_spans_per_segment": text_run_spans_per_segment,
        "text_glyph_instances_per_segment": text_glyph_instances_per_segment,
        "retained_generation_count": retained_generation_count,
        "max_retained_generation_count": max_retained_generation_count,
        "changed_segments_per_delta_generation": (
            changed_segments_per_delta_generation
        ),
        "metadata_budget_bytes": metadata_budget_bytes,
        "binding_product_metadata_bytes": binding_product_metadata_bytes,
        "weak_binding_map_entry_bytes": weak_binding_map_entry_bytes,
        "dependency_arc_bytes": dependency_arc_bytes,
        "segment_identity_bytes": segment_identity_bytes,
        "persistent_directory_node_bytes": persistent_directory_node_bytes,
        "text_segment_leaf_arc_bytes": text_segment_leaf_arc_bytes,
        "text_run_span_bytes": text_run_span_bytes,
        "image_vertex_bytes": image_vertex_bytes,
        "image_vertices_per_batch": image_vertices_per_batch,
        "text_glyph_record_bytes": text_glyph_record_bytes,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if retained_generation_count > max_retained_generation_count:
        raise ValueError("retained generations exceed the explicit generation bound")
    if changed_segments_per_delta_generation > segment_count:
        raise ValueError("changed segments must not exceed segment_count")

    image_dependency_count = segment_count * image_dependencies_per_segment
    if unique_image_binding_count > image_dependency_count:
        raise ValueError("unique image bindings must not exceed image dependencies")

    image_vertex_payload_bytes_per_segment = (
        image_batches_per_segment * image_vertices_per_batch * image_vertex_bytes
    )
    text_glyph_payload_bytes_per_segment = (
        text_glyph_instances_per_segment * text_glyph_record_bytes
    )
    source_payload_bytes_per_segment = (
        image_vertex_payload_bytes_per_segment
        + text_glyph_payload_bytes_per_segment
    )
    image_vertex_payload_bytes = (
        segment_count * image_vertex_payload_bytes_per_segment
    )
    text_glyph_payload_bytes = (
        segment_count * text_glyph_payload_bytes_per_segment
    )
    total_source_payload_bytes = image_vertex_payload_bytes + text_glyph_payload_bytes

    global_binding_metadata_bytes = unique_image_binding_count * (
        binding_product_metadata_bytes + weak_binding_map_entry_bytes
    )
    directory_leaf_capacity = 1 << (segment_count - 1).bit_length()
    directory_node_count = directory_leaf_capacity * 2 - 1
    directory_depth = math.ceil(math.log2(segment_count))
    text_run_span_count = segment_count * text_run_spans_per_segment

    base_image_dependency_arc_bytes = image_dependency_count * dependency_arc_bytes
    base_frame_segment_identity_bytes = segment_count * segment_identity_bytes
    base_directory_bytes = directory_node_count * persistent_directory_node_bytes
    base_text_segment_leaf_bytes = segment_count * text_segment_leaf_arc_bytes
    base_text_run_span_bytes = text_run_span_count * text_run_span_bytes
    base_generation_metadata_bytes = (
        base_image_dependency_arc_bytes
        + base_frame_segment_identity_bytes
        + base_directory_bytes
        + base_text_segment_leaf_bytes
        + base_text_run_span_bytes
    )

    delta_directory_path_bytes = (
        changed_segments_per_delta_generation
        * (directory_depth + 1)
        * persistent_directory_node_bytes
    )
    delta_segment_identity_bytes = (
        changed_segments_per_delta_generation * segment_identity_bytes
    )
    delta_image_dependency_arc_bytes = (
        changed_segments_per_delta_generation
        * image_dependencies_per_segment
        * dependency_arc_bytes
    )
    delta_text_run_span_bytes = (
        changed_segments_per_delta_generation
        * text_run_spans_per_segment
        * text_run_span_bytes
    )
    delta_generation_metadata_bytes = (
        delta_directory_path_bytes
        + delta_segment_identity_bytes
        + delta_image_dependency_arc_bytes
        + delta_text_run_span_bytes
    )
    retained_delta_generation_count = retained_generation_count - 1
    delta_source_payload_bytes = (
        changed_segments_per_delta_generation * source_payload_bytes_per_segment
    )
    retained_delta_source_payload_bytes = (
        retained_delta_generation_count * delta_source_payload_bytes
    )
    target_retained_source_payload_bytes = (
        total_source_payload_bytes + retained_delta_source_payload_bytes
    )
    delta_generation_retained_bytes = (
        delta_source_payload_bytes + delta_generation_metadata_bytes
    )
    total_metadata_bytes = (
        global_binding_metadata_bytes
        + base_generation_metadata_bytes
        + retained_delta_generation_count * delta_generation_metadata_bytes
    )
    total_retained_bytes = target_retained_source_payload_bytes + total_metadata_bytes
    rejected_full_generation_payload_bytes = (
        total_source_payload_bytes * retained_generation_count
    )

    return {
        "schema": "zircon.runtime.ui_render_dependency_product_memory_pressure.v1",
        "inputs": {
            **positive_inputs,
            "image_dependency_count": image_dependency_count,
            "text_run_span_count": text_run_span_count,
            "directory_leaf_capacity": directory_leaf_capacity,
            "directory_node_count": directory_node_count,
            "directory_depth": directory_depth,
            "retained_delta_generation_count": retained_delta_generation_count,
        },
        "shared_source_payload": {
            "image_vertex_payload_bytes_per_segment": (
                image_vertex_payload_bytes_per_segment
            ),
            "text_glyph_payload_bytes_per_segment": (
                text_glyph_payload_bytes_per_segment
            ),
            "total_payload_bytes_per_segment": source_payload_bytes_per_segment,
            "image_vertex_payload_bytes": image_vertex_payload_bytes,
            "text_glyph_payload_bytes": text_glyph_payload_bytes,
            "total_payload_bytes": total_source_payload_bytes,
            "base_retained_copy_count": 1,
        },
        "target_dependency_product": {
            "global_binding_metadata_bytes": global_binding_metadata_bytes,
            "base_image_dependency_arc_bytes": base_image_dependency_arc_bytes,
            "base_frame_segment_identity_bytes": (
                base_frame_segment_identity_bytes
            ),
            "base_directory_bytes": base_directory_bytes,
            "base_text_segment_leaf_bytes": base_text_segment_leaf_bytes,
            "base_text_run_span_bytes": base_text_run_span_bytes,
            "base_generation_metadata_bytes": base_generation_metadata_bytes,
            "delta_directory_path_bytes": delta_directory_path_bytes,
            "delta_segment_identity_bytes": delta_segment_identity_bytes,
            "delta_image_dependency_arc_bytes": delta_image_dependency_arc_bytes,
            "delta_text_run_span_bytes": delta_text_run_span_bytes,
            "delta_generation_metadata_bytes": delta_generation_metadata_bytes,
            "delta_source_payload_bytes": delta_source_payload_bytes,
            "retained_delta_source_payload_bytes": (
                retained_delta_source_payload_bytes
            ),
            "target_retained_source_payload_bytes": (
                target_retained_source_payload_bytes
            ),
            "delta_generation_retained_bytes": delta_generation_retained_bytes,
            "total_metadata_bytes": total_metadata_bytes,
            "total_retained_bytes": total_retained_bytes,
            "metadata_budget_bytes": metadata_budget_bytes,
            "metadata_budget_headroom_bytes": metadata_budget_bytes
            - total_metadata_bytes,
            "within_metadata_budget": total_metadata_bytes <= metadata_budget_bytes,
            "retained_bytes_depend_on_present_count": False,
        },
        "rejected_full_generation_clone": {
            "retained_payload_bytes": rejected_full_generation_payload_bytes,
            "payload_copy_count": retained_generation_count,
        },
        "delta": {
            "avoided_payload_duplication_bytes": (
                rejected_full_generation_payload_bytes
                - target_retained_source_payload_bytes
            ),
            "metadata_over_source_payload_ratio": round(
                total_metadata_bytes / total_source_payload_bytes, 6
            ),
        },
        "interpretation": {
            "rss_measured": False,
            "gpu_resident_bytes_measured": False,
            "included": (
                "one base source vertex/glyph payload, live changed-segment payload "
                "versions, unique binding-product and weak-map metadata, base "
                "persistent directory, segment/dependency/run references, conservative "
                "changed-path copies, retained-generation bound, and explicit CPU "
                "metadata budget"
            ),
            "excluded": (
                "allocator and container capacity overhead, Rust enum padding, Arc "
                "control blocks, wgpu::BindGroup and driver allocations, texture "
                "pixels, GPU resident bytes, actual process RSS, fragmentation, and "
                "backend recovery transients"
            ),
            "required_product_evidence": (
                "current-source allocated metadata bytes, live and retired generation "
                "counts, binding-product count, eviction/retirement counts, process "
                "working/private bytes, GPU resource/resident bytes, and post-pressure "
                "quiescent recovery"
            ),
        },
    }


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest().upper()


def build_source_binding(source_root: Path) -> dict[str, Any]:
    revision = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=source_root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    critical_sources = []
    for relative_path in CRITICAL_SOURCE_FILES:
        source_path = source_root / relative_path
        critical_sources.append(
            {"relative_path": relative_path, "sha256": _sha256(source_path)}
        )
    dirty_lines = subprocess.run(
        ["git", "status", "--short", "--", *CRITICAL_SOURCE_FILES],
        cwd=source_root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    return {
        "git_revision": revision,
        "dirty_paths": [line[3:] for line in dirty_lines if len(line) > 3],
        "critical_source_files": critical_sources,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--present-count", type=int, default=1_000_000)
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--image-dependencies-per-segment", type=int, default=4)
    parser.add_argument("--unique-image-binding-count", type=int, default=64)
    parser.add_argument("--image-batches-per-segment", type=int, default=16)
    parser.add_argument("--text-run-spans-per-segment", type=int, default=8)
    parser.add_argument("--text-glyph-instances-per-segment", type=int, default=768)
    parser.add_argument("--retained-generation-count", type=int, default=3)
    parser.add_argument("--max-retained-generation-count", type=int, default=3)
    parser.add_argument("--changed-segments-per-delta-generation", type=int, default=1)
    parser.add_argument("--metadata-budget-bytes", type=int, default=8 * 1024 * 1024)
    parser.add_argument(
        "--source-root", type=Path, default=Path(__file__).resolve().parents[1]
    )
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        present_count=args.present_count,
        segment_count=args.segment_count,
        image_dependencies_per_segment=args.image_dependencies_per_segment,
        unique_image_binding_count=args.unique_image_binding_count,
        image_batches_per_segment=args.image_batches_per_segment,
        text_run_spans_per_segment=args.text_run_spans_per_segment,
        text_glyph_instances_per_segment=args.text_glyph_instances_per_segment,
        retained_generation_count=args.retained_generation_count,
        max_retained_generation_count=args.max_retained_generation_count,
        changed_segments_per_delta_generation=(
            args.changed_segments_per_delta_generation
        ),
        metadata_budget_bytes=args.metadata_budget_bytes,
    )
    source_root = args.source_root.resolve()
    result["source_binding"] = build_source_binding(source_root)
    result["source_binding"]["model_source_sha256"] = _sha256(Path(__file__).resolve())
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        output = validate_output_path(args.output.resolve())
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
