"""Deterministic work model for native-pane scrollbar damage selection."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    pane_paint_count: int = 4_000,
    descriptors_per_pane: int = 4,
    intersecting_descriptors_per_paint: int = 1,
    metadata_generation_count: int = 2,
) -> dict[str, int | float | str]:
    for name, value in (
        ("pane_paint_count", pane_paint_count),
        ("descriptors_per_pane", descriptors_per_pane),
        ("metadata_generation_count", metadata_generation_count),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if descriptors_per_pane > 4:
        raise ValueError("descriptors_per_pane must not exceed inline capacity 4")
    if not 0 <= intersecting_descriptors_per_paint <= descriptors_per_pane:
        raise ValueError(
            "intersecting_descriptors_per_paint must be between zero and "
            "descriptors_per_pane"
        )

    legacy_metadata_lookups = pane_paint_count * descriptors_per_pane
    legacy_style_reads = pane_paint_count * descriptors_per_pane
    legacy_geometry_evaluations = pane_paint_count * descriptors_per_pane

    target_descriptor_publications = (
        metadata_generation_count * descriptors_per_pane
    )
    target_metadata_lookups = pane_paint_count
    target_damage_probes = pane_paint_count * descriptors_per_pane
    target_style_reads = pane_paint_count * intersecting_descriptors_per_paint
    target_geometry_evaluations = (
        pane_paint_count * intersecting_descriptors_per_paint
    )

    return {
        "schema_version": 1,
        "interpretation": (
            "deterministic metadata/style/geometry work model; not CPU, allocation, "
            "layout, render, GPU, or latency evidence"
        ),
        "pane_paint_count": pane_paint_count,
        "descriptors_per_pane": descriptors_per_pane,
        "intersecting_descriptors_per_paint": intersecting_descriptors_per_paint,
        "metadata_generation_count": metadata_generation_count,
        "legacy_metadata_lookups": legacy_metadata_lookups,
        "legacy_style_reads": legacy_style_reads,
        "legacy_geometry_evaluations": legacy_geometry_evaluations,
        "target_descriptor_publications": target_descriptor_publications,
        "target_descriptor_heap_allocations": 0,
        "target_descriptor_inline_capacity": 4,
        "target_metadata_lookups": target_metadata_lookups,
        "target_damage_probes": target_damage_probes,
        "target_style_reads": target_style_reads,
        "target_geometry_evaluations": target_geometry_evaluations,
        "metadata_lookup_reduction_ratio": (
            legacy_metadata_lookups / target_metadata_lookups
        ),
        "style_read_reduction_ratio": (
            legacy_style_reads / max(target_style_reads, 1)
        ),
        "geometry_evaluation_reduction_ratio": (
            legacy_geometry_evaluations / max(target_geometry_evaluations, 1)
        ),
        "target_publication_complexity": "O(K) per pane metadata generation; K <= 4",
        "target_paint_selection_complexity": "O(K) damage probes + O(I) preparation",
    }


def write_result(path: Path, result: dict[str, int | float | str]) -> None:
    resolved = path.resolve()
    if resolved.drive.upper() == "C:":
        raise ValueError("pressure artifacts must not be written to C:")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    resolved.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pane-paint-count", type=int, default=4_000)
    parser.add_argument("--descriptors-per-pane", type=int, default=4)
    parser.add_argument("--intersecting-descriptors-per-paint", type=int, default=1)
    parser.add_argument("--metadata-generation-count", type=int, default=2)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        pane_paint_count=args.pane_paint_count,
        descriptors_per_pane=args.descriptors_per_pane,
        intersecting_descriptors_per_paint=args.intersecting_descriptors_per_paint,
        metadata_generation_count=args.metadata_generation_count,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
