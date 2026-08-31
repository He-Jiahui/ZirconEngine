"""Model metadata work avoided by the transient product paint path."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(command_count: int = 32_768, text_command_count: int = 8_192) -> dict[str, object]:
    if command_count <= 0:
        raise ValueError("command_count must be positive")
    if text_command_count < 0:
        raise ValueError("text_command_count must not be negative")
    if text_command_count > command_count:
        raise ValueError("text_command_count must not exceed command_count")

    return {
        "schema": "zircon.runtime.ui_transient_paint_metadata_pressure.v1",
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "is_product_timing": False,
        "inputs": {
            "command_count": command_count,
            "text_command_count": text_command_count,
        },
        "legacy_cached_paint_path": {
            "stable_json_generation_calls": command_count,
            "debug_label_format_calls": command_count,
        },
        "transient_product_path": {
            "stable_json_generation_calls": text_command_count,
            "debug_label_format_calls": 0,
            "generation_policy": "text_batches_only",
        },
        "delta": {
            "avoided_stable_json_generation_calls": command_count - text_command_count,
            "avoided_debug_label_format_calls": command_count,
        },
        "excluded_from_model": [
            "paint payload construction",
            "text shaping and glyph preparation",
            "vertex generation and GPU uploads",
            "allocator latency and RSS",
            "CPU, GPU, and frame timing",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--command-count", type=int, default=32_768)
    parser.add_argument("--text-command-count", type=int, default=8_192)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(args.command_count, args.text_command_count)
    payload = json.dumps(result, indent=2, sort_keys=True)
    if args.output is not None:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload + "\n", encoding="utf-8")
    print(payload)


if __name__ == "__main__":
    main()
