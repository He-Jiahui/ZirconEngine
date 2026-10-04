"""Model UI draw-buffer creation and upload work for retained GPU buffers."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    present_count: int = 4096,
    solid_bytes: int = 32_768,
    solid_instance_bytes: int = 4_096,
    image_bytes: int = 8_192,
) -> dict[str, object]:
    if present_count <= 0:
        raise ValueError("present_count must be positive")
    category_bytes = (solid_bytes, solid_instance_bytes, image_bytes)
    if any(byte_count < 0 for byte_count in category_bytes):
        raise ValueError("category byte counts must be non-negative")
    non_empty_categories = sum(byte_count > 0 for byte_count in category_bytes)
    bytes_per_present = sum(category_bytes)

    retired_creates = present_count * non_empty_categories
    retired_upload_bytes = present_count * bytes_per_present
    persistent_damage_creates = non_empty_categories
    versioned_upload_bytes = bytes_per_present
    return {
        "schema": "zircon.ui.draw_buffer_reuse_pressure.v1",
        "inputs": {
            "present_count": present_count,
            "solid_bytes": solid_bytes,
            "solid_instance_bytes": solid_instance_bytes,
            "image_bytes": image_bytes,
            "non_empty_categories": non_empty_categories,
            "bytes_per_present": bytes_per_present,
        },
        "retired_per_present_allocation": {
            "vertex_buffer_creates": retired_creates,
            "vertex_upload_bytes": retired_upload_bytes,
        },
        "persistent_unversioned_damage": {
            "vertex_buffer_creates": persistent_damage_creates,
            "vertex_upload_bytes": retired_upload_bytes,
        },
        "versioned_projection_reuse": {
            "vertex_buffer_creates": non_empty_categories,
            "vertex_upload_bytes": versioned_upload_bytes,
        },
        "delta": {
            "damage_avoided_buffer_creates": retired_creates
            - persistent_damage_creates,
            "damage_buffer_create_reduction_ratio": retired_creates
            / persistent_damage_creates
            if persistent_damage_creates
            else 0.0,
            "damage_avoided_upload_bytes": 0,
            "versioned_avoided_upload_bytes": retired_upload_bytes
            - versioned_upload_bytes,
            "versioned_upload_reduction_ratio": retired_upload_bytes
            / versioned_upload_bytes
            if versioned_upload_bytes
            else 0.0,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--present-count", type=int, default=4096)
    parser.add_argument("--solid-bytes", type=int, default=32_768)
    parser.add_argument("--solid-instance-bytes", type=int, default=4_096)
    parser.add_argument("--image-bytes", type=int, default=8_192)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        present_count=args.present_count,
        solid_bytes=args.solid_bytes,
        solid_instance_bytes=args.solid_instance_bytes,
        image_bytes=args.image_bytes,
    )
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
