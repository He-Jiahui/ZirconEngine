"""Deterministic work model for empty binding-target pointer events."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def model_pressure(
    event_count: int = 65_536,
    binding_id_bytes: int = 64,
    asset_id_bytes: int = 128,
) -> dict[str, object]:
    events = max(event_count, 0)
    binding_bytes = max(binding_id_bytes, 0)
    asset_bytes = max(asset_id_bytes, 0)
    temporary_identifier_bytes = events * (binding_bytes + asset_bytes)

    return {
        "schema": "zircon.ui.binding_target_passthrough_pressure.v1",
        "inputs": {
            "event_count": events,
            "binding_id_bytes": binding_bytes,
            "asset_id_bytes": asset_bytes,
        },
        "retired": {
            "timer_reads": events,
            "compiled_binding_lookups": 2 * events,
            "temporary_identifier_allocations": 2 * events,
            "temporary_identifier_bytes": temporary_identifier_bytes,
        },
        "passthrough": {
            "timer_reads": 0,
            "compiled_binding_lookups": events,
            "temporary_identifier_allocations": 0,
            "temporary_identifier_bytes": 0,
        },
        "delta": {
            "eliminated_timer_reads": events,
            "eliminated_compiled_binding_lookups": events,
            "eliminated_temporary_identifier_allocations": 2 * events,
            "eliminated_temporary_identifier_bytes": temporary_identifier_bytes,
        },
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--events", type=int, default=65_536)
    parser.add_argument("--binding-id-bytes", type=int, default=64)
    parser.add_argument("--asset-id-bytes", type=int, default=128)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    result = model_pressure(
        event_count=args.events,
        binding_id_bytes=args.binding_id_bytes,
        asset_id_bytes=args.asset_id_bytes,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, separators=(",", ":")))


if __name__ == "__main__":
    main()
