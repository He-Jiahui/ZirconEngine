#!/usr/bin/env python3
"""Model transient analog-control ownership for the current source candidate.

This is deterministic source-contract evidence, not compiled behavior or product timing.
"""

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMPLEMENTATION_SOURCES = (
    "zircon_runtime/src/ui/surface/input/analog.rs",
    "zircon_runtime/src/ui/surface/input/analog_navigation.rs",
    "zircon_runtime/src/ui/surface/input/state/analog.rs",
)
REFERENCE_SOURCES = (
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/NavigationConfig.cpp",
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


def pressure_report(analog_events: int, canonical_control_bytes: int) -> dict[str, object]:
    if analog_events <= 0:
        raise ValueError("analog_events must be positive")
    if canonical_control_bytes <= 0:
        raise ValueError("canonical_control_bytes must be positive")

    prior_allocations = analog_events * 2
    prior_bytes = analog_events * canonical_control_bytes * 2
    return {
        "evidence_kind": "deterministic_source_bound_pressure_model",
        "implementation_evidence": False,
        "implementation_source_contract": True,
        "is_product_timing": False,
        "source_binding": {
            "implementation": source_bindings(IMPLEMENTATION_SOURCES),
            "primary_reference": source_bindings(REFERENCE_SOURCES),
        },
        "reference_contract": {
            "retained_key": (
                "Unreal FNavigationConfig uses FAnalogNavigationKey(FKey, direction) "
                "instead of rebuilding a text key for axis classification"
            ),
        },
        "inputs": {
            "analog_events": analog_events,
            "canonical_control_bytes": canonical_control_bytes,
        },
        "prior_owned_string_baseline": {
            "event_control_clone_allocations": analog_events,
            "normalized_control_allocations": analog_events,
            "transient_string_allocations": prior_allocations,
            "minimum_control_bytes_copied": prior_bytes,
        },
        "candidate_borrowed_canonical_control": {
            "event_control_clone_allocations": 0,
            "normalized_control_allocations": 0,
            "transient_string_allocations": 0,
            "minimum_control_bytes_copied": 0,
            "retained_value_scalar_reads": analog_events,
        },
        "eliminated_or_avoided": {
            "transient_string_allocations": prior_allocations,
            "minimum_control_bytes_copied": prior_bytes,
        },
        "retained_work": [
            "one retained analog-control map lookup",
            "canonical control classification",
            "threshold and repeat-state evaluation",
            "navigation dispatch when the repeat gate opens",
        ],
        "excluded_from_model": [
            "repeat-state key formatting for active navigation",
            "non-canonical control normalization ownership",
            "CPU and allocator timing",
            "map lookup and navigation callback cost",
            "managed Rust behavior tests and product input latency",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--analog-events", type=int, default=100_000)
    parser.add_argument("--canonical-control-bytes", type=int, default=12)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(args.analog_events, args.canonical_control_bytes)
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
