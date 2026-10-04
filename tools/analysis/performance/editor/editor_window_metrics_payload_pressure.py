"""Deterministic source-collection model for WindowMetrics pane-payload reuse."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    payload_source_count: int,
    metrics_reflow_count: int,
    content_patch_count: int,
) -> dict[str, int | float]:
    for name, value in (
        ("payload_source_count", payload_source_count),
        ("metrics_reflow_count", metrics_reflow_count),
        ("content_patch_count", content_patch_count),
    ):
        if value < 0:
            raise ValueError(f"{name} must be non-negative")
    if payload_source_count == 0:
        raise ValueError("payload_source_count must be positive")

    old_payload_source_collection_count = (
        payload_source_count * metrics_reflow_count
    )
    new_metrics_payload_source_collection_count = 0
    new_content_refresh_source_collection_count = (
        payload_source_count * content_patch_count
    )
    new_total_payload_source_collection_count = (
        new_metrics_payload_source_collection_count
        + new_content_refresh_source_collection_count
    )
    eliminated_payload_source_collection_count = (
        old_payload_source_collection_count
        - new_total_payload_source_collection_count
    )

    return {
        "payload_source_count": payload_source_count,
        "metrics_reflow_count": metrics_reflow_count,
        "content_patch_count": content_patch_count,
        "old_payload_source_collection_count": old_payload_source_collection_count,
        "new_metrics_payload_source_collection_count": new_metrics_payload_source_collection_count,
        "new_content_refresh_source_collection_count": new_content_refresh_source_collection_count,
        "new_total_payload_source_collection_count": new_total_payload_source_collection_count,
        "eliminated_payload_source_collection_count": eliminated_payload_source_collection_count,
        "payload_source_collection_reduction_ratio": (
            old_payload_source_collection_count
            / max(new_total_payload_source_collection_count, 1)
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--payload-source-count", type=int, default=7)
    parser.add_argument("--metrics-reflow-count", type=int, default=4_096)
    parser.add_argument("--content-patch-count", type=int, default=16)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        args.payload_source_count,
        args.metrics_reflow_count,
        args.content_patch_count,
    )
    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(encoded + "\n", encoding="utf-8")
    print(encoded)


if __name__ == "__main__":
    main()
