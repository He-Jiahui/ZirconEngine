#!/usr/bin/env python3
"""Model geometry-frame extraction avoided by render/input-only refreshes.

This is a deterministic work-count model, not product timing.
"""

import argparse
import json
from pathlib import Path, PureWindowsPath


def pressure_report(refresh_count: int, control_count: int) -> dict[str, object]:
    if refresh_count <= 0:
        raise ValueError("refresh_count must be positive")
    if control_count <= 0:
        raise ValueError("control_count must be positive")

    old_visits = refresh_count * control_count
    return {
        "schema": "zircon.editor.ui_frame_extraction_cache_pressure.v1",
        "evidence_kind": "deterministic_pipeline_work_count",
        "is_product_timing": False,
        "inputs": {
            "render_input_only_refresh_count": refresh_count,
            "geometry_control_count": control_count,
        },
        "current_unconditional_extraction": {
            "refresh_count": refresh_count,
            "frame_extract_count": refresh_count,
            "frame_control_visits": old_visits,
        },
        "cached_geometry_extraction": {
            "refresh_count": refresh_count,
            "frame_extract_count": 0,
            "frame_control_visits": 0,
            "frames_reused_count": refresh_count,
        },
        "comparison": {
            "avoided_frame_control_visits": old_visits,
            "frame_control_visit_reduction_ratio": float(old_visits),
        },
        "target_contract": [
            "geometry frames are extracted on layout recomputation",
            "render/input-only state refreshes reuse the last geometry frame snapshot",
            "host projection semantic and geometry patches remain incremental",
            "a layout recomputation invalidates and replaces the cached frame snapshot",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and input-to-present timing",
            "surface rebuild and host projection work",
            "native paint, GPU submission, and present blocking",
        ],
    }


def pressure_suite() -> dict[str, object]:
    return {
        "schema": "zircon.editor.ui_frame_extraction_cache_pressure_suite.v1",
        "evidence_kind": "deterministic_pipeline_work_count",
        "is_product_timing": False,
        "scenarios": {
            str(control_count): pressure_report(1_000, control_count)
            for control_count in (64, 256, 1_024)
        },
    }


def validate_output_path(output: str) -> Path:
    path = Path(output).resolve()
    if PureWindowsPath(str(path)).drive.upper() == "C:":
        raise ValueError("performance artifacts must not be written to the C drive")
    return path


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    args = parser.parse_args()
    payload = json.dumps(pressure_suite(), indent=2, sort_keys=True) + "\n"
    if args.output:
        output_path = validate_output_path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(payload, encoding="utf-8", newline="\n")
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
