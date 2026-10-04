"""Deterministic source-parse model for the bounded runtime SVG document cache."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    plan_count: int,
    unique_document_count: int,
    cache_capacity: int = 512,
) -> dict[str, int | float | bool | dict]:
    for name, value in (
        ("plan_count", plan_count),
        ("unique_document_count", unique_document_count),
        ("cache_capacity", cache_capacity),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if unique_document_count > cache_capacity:
        raise ValueError(
            "the stable-working-set model requires unique_document_count <= cache_capacity"
        )

    old_parse_count = plan_count * unique_document_count
    new_parse_count = unique_document_count
    return {
        "plan_count": plan_count,
        "unique_document_count": unique_document_count,
        "cache_capacity": cache_capacity,
        "stable_working_set_fits_cache": True,
        "old_parse_count": old_parse_count,
        "new_parse_count": new_parse_count,
        "eliminated_parse_count": old_parse_count - new_parse_count,
        "parse_reduction_ratio": old_parse_count / new_parse_count,
        "interpretation": {
            "included": [
                "repeated SVG document parsing across atlas plan builds",
                "bounded source-keyed document residency",
            ],
            "excluded": [
                "asset filesystem reads",
                "SVG rasterization",
                "GPU texture uploads",
                "CPU timing",
                "allocator RSS",
            ],
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan-count", type=int, default=1_000)
    parser.add_argument("--unique-document-count", type=int, default=64)
    parser.add_argument("--cache-capacity", type=int, default=512)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.plan_count,
        args.unique_document_count,
        args.cache_capacity,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
