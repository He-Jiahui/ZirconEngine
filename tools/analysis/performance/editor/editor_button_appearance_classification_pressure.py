#!/usr/bin/env python3
"""Model paint-time string classification versus published button appearance.

This is a deterministic algorithm-pressure model, not measured product timing.
"""

import argparse
import json
from collections.abc import Sequence


KIND_NEEDLES = (
    "danger",
    "delete",
    "trash",
    "primary",
    "filled",
    "accent",
    "tertiary",
    "text",
)
GLYPH_NEEDLES = (
    "delete",
    "trash",
    "danger",
    "dropdown",
    "drop-down",
    "menu",
    "icon",
    "add",
    "plus",
)
BUTTON_IDENTITY_FIELD_COUNT = 6


def _validate_inputs(
    candidates_per_repaint: int,
    repaint_count: int,
    field_lengths: Sequence[int],
) -> None:
    if candidates_per_repaint <= 0:
        raise ValueError("candidates_per_repaint must be positive")
    if repaint_count <= 0:
        raise ValueError("repaint_count must be positive")
    if len(field_lengths) != BUTTON_IDENTITY_FIELD_COUNT:
        raise ValueError(
            f"field_lengths must contain {BUTTON_IDENTITY_FIELD_COUNT} values"
        )
    if any(length < 0 for length in field_lengths):
        raise ValueError("field lengths must be non-negative")


def _substring_work_per_candidate(
    field_lengths: Sequence[int], needles: Sequence[str]
) -> tuple[int, int]:
    windows = 0
    upper_bound_byte_comparisons = 0
    for needle in needles:
        needle_length = len(needle.encode("ascii"))
        for field_length in field_lengths:
            window_count = max(0, field_length - needle_length + 1)
            windows += window_count
            upper_bound_byte_comparisons += window_count * needle_length
    return windows, upper_bound_byte_comparisons


def pressure_report(
    candidates_per_repaint: int,
    repaint_count: int,
    field_lengths: Sequence[int],
) -> dict[str, object]:
    _validate_inputs(candidates_per_repaint, repaint_count, field_lengths)
    candidate_visits = candidates_per_repaint * repaint_count
    needles = KIND_NEEDLES + GLYPH_NEEDLES
    windows_per_candidate, byte_comparisons_per_candidate = (
        _substring_work_per_candidate(field_lengths, needles)
    )
    field_needle_scans_per_candidate = len(field_lengths) * len(needles)

    return {
        "inputs": {
            "candidates_per_repaint": candidates_per_repaint,
            "repaint_count": repaint_count,
            "paint_candidate_visits": candidate_visits,
            "field_lengths": list(field_lengths),
            "identity_field_count": len(field_lengths),
            "kind_needle_count": len(KIND_NEEDLES),
            "glyph_needle_count": len(GLYPH_NEEDLES),
        },
        "current_string_classification": {
            "identity_array_projections": candidate_visits * 2,
            "field_needle_scans": candidate_visits
            * field_needle_scans_per_candidate,
            "substring_windows": candidate_visits * windows_per_candidate,
            "upper_bound_byte_comparisons": candidate_visits
            * byte_comparisons_per_candidate,
        },
        "published_typed_appearance": {
            "identity_array_projections": 0,
            "field_needle_scans": 0,
            "substring_windows": 0,
            "upper_bound_byte_comparisons": 0,
            "typed_field_reads": candidate_visits * 2,
        },
        "avoided": {
            "identity_array_projections": candidate_visits * 2,
            "field_needle_scans": candidate_visits
            * field_needle_scans_per_candidate,
            "substring_windows": candidate_visits * windows_per_candidate,
            "upper_bound_byte_comparisons": candidate_visits
            * byte_comparisons_per_candidate,
        },
        "allocation_reduction_claim": 0,
        "retained_work": [
            "typed button eligibility branch",
            "button visual-kind state selection",
            "glyph state selection",
            "text measurement and command emission",
        ],
    }


def pressure_suite(
    scenarios: Sequence[tuple[int, int]], field_lengths: Sequence[int]
) -> dict[str, object]:
    if not scenarios:
        raise ValueError("scenarios must not be empty")
    return {
        "evidence_kind": "deterministic_algorithm_pressure_model",
        "classification_case": "worst_case_nonmatching_identity",
        "is_product_timing": False,
        "scenarios": [
            pressure_report(candidates, repaints, field_lengths)
            for candidates, repaints in scenarios
        ],
        "interpretation_limits": [
            "models only button_kind and button_glyph identity scans",
            "byte comparisons are a worst-case upper bound",
            "does not model early matches or CPU cache effects",
            "does not model layout, text shaping, command filtering, GPU work, or present",
            "current classification is borrowed and this model claims no allocation reduction",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--scenario",
        action="append",
        nargs=2,
        type=int,
        metavar=("CANDIDATES", "REPAINTS"),
        default=None,
    )
    parser.add_argument(
        "--field-lengths",
        nargs=BUTTON_IDENTITY_FIELD_COUNT,
        type=int,
        default=[32, 24, 24, 9, 16, 8],
    )
    parser.add_argument("--output")
    args = parser.parse_args()
    scenarios = (
        [tuple(scenario) for scenario in args.scenario]
        if args.scenario
        else [(1, 10_000), (32, 10_000), (512, 1_000)]
    )
    report = pressure_suite(scenarios, args.field_lengths)
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
