"""Model residual Runtime UI image/text dependency-product prepare work."""

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
    frame_count: int = 4_096,
    segment_count: int = 64,
    image_dependencies_per_segment: int = 4,
    binding_cache_entry_count: int = 512,
    text_dependencies_per_segment: int = 32,
    text_run_spans_per_segment: int = 8,
    delta_frame_count: int = 32,
    changed_segments_per_delta_frame: int = 1,
    resource_generation_frame_count: int = 4,
) -> dict[str, Any]:
    positive_inputs = {
        "frame_count": frame_count,
        "segment_count": segment_count,
        "image_dependencies_per_segment": image_dependencies_per_segment,
        "binding_cache_entry_count": binding_cache_entry_count,
        "text_dependencies_per_segment": text_dependencies_per_segment,
        "text_run_spans_per_segment": text_run_spans_per_segment,
        "changed_segments_per_delta_frame": changed_segments_per_delta_frame,
    }
    for name, value in positive_inputs.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if delta_frame_count <= 0:
        raise ValueError("delta_frame_count must be positive")
    if resource_generation_frame_count < 0:
        raise ValueError("resource_generation_frame_count must be non-negative")
    if delta_frame_count + resource_generation_frame_count > frame_count:
        raise ValueError("delta and resource-generation frames must fit in frame_count")
    if changed_segments_per_delta_frame > segment_count:
        raise ValueError("changed segments must not exceed segment_count")

    stable_frame_count = (
        frame_count - delta_frame_count - resource_generation_frame_count
    )
    image_dependencies_per_frame = segment_count * image_dependencies_per_segment
    text_dependencies_per_frame = segment_count * text_dependencies_per_segment
    text_run_spans_per_frame = segment_count * text_run_spans_per_segment
    persistent_directory_depth = max(1, math.ceil(math.log2(segment_count)))

    current_image_segment_visits = frame_count * segment_count
    current_image_dependency_checks = frame_count * image_dependencies_per_frame
    current_binding_retention_visits = frame_count * binding_cache_entry_count

    delta_segment_visits = delta_frame_count * changed_segments_per_delta_frame
    full_fallback_segment_visits = resource_generation_frame_count * segment_count
    target_image_segment_visits = delta_segment_visits + full_fallback_segment_visits
    delta_image_dependency_checks = (
        delta_frame_count
        * changed_segments_per_delta_frame
        * image_dependencies_per_segment
    )
    full_fallback_image_dependency_checks = (
        resource_generation_frame_count * image_dependencies_per_frame
    )
    target_image_dependency_checks = (
        delta_image_dependency_checks + full_fallback_image_dependency_checks
    )

    current_text_delta_dependency_segment_visits = delta_frame_count * segment_count
    current_text_delta_dependency_entry_visits = (
        delta_frame_count * text_dependencies_per_frame
    )
    current_text_delta_run_segment_visits = delta_frame_count * segment_count
    current_text_delta_run_entry_visits = delta_frame_count * text_run_spans_per_frame

    target_text_delta_dependency_segment_visits = (
        delta_frame_count * changed_segments_per_delta_frame
    )
    target_text_delta_dependency_entry_visits = (
        delta_frame_count
        * changed_segments_per_delta_frame
        * text_dependencies_per_segment
    )
    target_text_delta_run_directory_visits = delta_frame_count * (
        persistent_directory_depth + changed_segments_per_delta_frame
    )
    target_text_delta_run_entry_visits = (
        delta_frame_count
        * changed_segments_per_delta_frame
        * text_run_spans_per_segment
    )

    full_fallback_text_dependency_segment_visits = (
        resource_generation_frame_count * segment_count
    )
    full_fallback_text_dependency_entry_visits = (
        resource_generation_frame_count * text_dependencies_per_frame
    )
    full_fallback_text_run_entry_visits = (
        resource_generation_frame_count * text_run_spans_per_frame
    )

    return {
        "schema": "zircon.runtime.ui_render_dependency_product_pressure.v1",
        "inputs": {
            **positive_inputs,
            "delta_frame_count": delta_frame_count,
            "resource_generation_frame_count": resource_generation_frame_count,
            "stable_frame_count": stable_frame_count,
            "image_dependencies_per_frame": image_dependencies_per_frame,
            "text_dependencies_per_frame": text_dependencies_per_frame,
            "text_run_spans_per_frame": text_run_spans_per_frame,
            "persistent_directory_depth": persistent_directory_depth,
        },
        "current_image_prepare": {
            "segment_visits": current_image_segment_visits,
            "texture_dependency_checks": current_image_dependency_checks,
            "binding_lookups": current_image_dependency_checks,
            "binding_retention_entry_visits": current_binding_retention_visits,
        },
        "target_image_dependency_product": {
            "stable_frame_key_checks": frame_count,
            "delta_segment_visits": delta_segment_visits,
            "full_fallback_segment_visits": full_fallback_segment_visits,
            "segment_visits": target_image_segment_visits,
            "delta_texture_dependency_checks": delta_image_dependency_checks,
            "full_fallback_texture_dependency_checks": (
                full_fallback_image_dependency_checks
            ),
            "texture_dependency_checks": target_image_dependency_checks,
            "binding_lookups": target_image_dependency_checks,
            "binding_retention_entry_visits": 0,
        },
        "current_text_delta_composition": {
            "stable_frame_dependency_visits": 0,
            "delta_dependency_segment_visits": (
                current_text_delta_dependency_segment_visits
            ),
            "delta_dependency_entry_visits": current_text_delta_dependency_entry_visits,
            "delta_run_segment_visits": current_text_delta_run_segment_visits,
            "delta_run_entry_visits": current_text_delta_run_entry_visits,
        },
        "target_text_persistent_delta": {
            "stable_frame_dependency_visits": 0,
            "delta_dependency_segment_visits": (
                target_text_delta_dependency_segment_visits
            ),
            "delta_dependency_entry_visits": target_text_delta_dependency_entry_visits,
            "delta_run_directory_node_visits": target_text_delta_run_directory_visits,
            "delta_run_entry_visits": target_text_delta_run_entry_visits,
        },
        "typed_full_fallback": {
            "resource_generation_frame_count": resource_generation_frame_count,
            "image_segment_visits": full_fallback_segment_visits,
            "image_dependency_checks": full_fallback_image_dependency_checks,
            "text_dependency_segment_visits": (
                full_fallback_text_dependency_segment_visits
            ),
            "text_dependency_entry_visits": full_fallback_text_dependency_entry_visits,
            "text_run_entry_visits": full_fallback_text_run_entry_visits,
        },
        "delta": {
            "image_segment_visit_reduction_ratio": round(
                current_image_segment_visits / target_image_segment_visits, 6
            ),
            "image_dependency_check_reduction_ratio": round(
                current_image_dependency_checks / target_image_dependency_checks, 6
            ),
            "avoided_binding_retention_entry_visits": (
                current_binding_retention_visits
            ),
            "text_delta_dependency_entry_reduction_ratio": round(
                current_text_delta_dependency_entry_visits
                / target_text_delta_dependency_entry_visits,
                6,
            ),
            "text_delta_run_entry_reduction_ratio": round(
                current_text_delta_run_entry_visits
                / target_text_delta_run_entry_visits,
                6,
            ),
        },
        "interpretation": {
            "timing_claim": False,
            "included": (
                "stable/delta/resource-generation state partition, image segment and "
                "texture dependency visits, binding lookup and retention-map entry "
                "visits, text frame dependency composition, run-span composition, "
                "and persistent-directory depth"
            ),
            "excluded": (
                "actual CPU/GPU time, allocator latency, cache-line effects, hash-map "
                "constants, buffer writes, draw calls, upload bytes, RSS, asynchronous "
                "resource completion, and product input-to-present latency"
            ),
            "scope": (
                "deterministic current-source residual-operation model; resource "
                "generation changes are typed full fallbacks and are not hidden inside "
                "the changed-segment reduction"
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
        path = source_root / relative_path
        critical_sources.append(
            {"relative_path": relative_path, "sha256": _sha256(path)}
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
    parser.add_argument("--frame-count", type=int, default=4_096)
    parser.add_argument("--segment-count", type=int, default=64)
    parser.add_argument("--image-dependencies-per-segment", type=int, default=4)
    parser.add_argument("--binding-cache-entry-count", type=int, default=512)
    parser.add_argument("--text-dependencies-per-segment", type=int, default=32)
    parser.add_argument("--text-run-spans-per-segment", type=int, default=8)
    parser.add_argument("--delta-frame-count", type=int, default=32)
    parser.add_argument("--changed-segments-per-delta-frame", type=int, default=1)
    parser.add_argument("--resource-generation-frame-count", type=int, default=4)
    parser.add_argument("--source-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    result = run(
        frame_count=args.frame_count,
        segment_count=args.segment_count,
        image_dependencies_per_segment=args.image_dependencies_per_segment,
        binding_cache_entry_count=args.binding_cache_entry_count,
        text_dependencies_per_segment=args.text_dependencies_per_segment,
        text_run_spans_per_segment=args.text_run_spans_per_segment,
        delta_frame_count=args.delta_frame_count,
        changed_segments_per_delta_frame=args.changed_segments_per_delta_frame,
        resource_generation_frame_count=args.resource_generation_frame_count,
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
