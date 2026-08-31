"""Deterministic work model for table column resize pointer input."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def run(
    column_count: int = 256,
    pointer_move_count: int = 2_000,
    column_metadata_entry_count: int = 8,
    compatibility_flush_count: int = 1,
    scalar_operations_per_move: int = 3,
) -> dict[str, int | float | str]:
    for name, value in (
        ("column_count", column_count),
        ("pointer_move_count", pointer_move_count),
        ("column_metadata_entry_count", column_metadata_entry_count),
        ("scalar_operations_per_move", scalar_operations_per_move),
    ):
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if compatibility_flush_count < 0:
        raise ValueError("compatibility_flush_count must be non-negative")
    if compatibility_flush_count > pointer_move_count:
        raise ValueError("compatibility_flush_count cannot exceed pointer_move_count")

    # The current path performs at least three complete conversions for each aggregate:
    # metadata -> UiValue, UiValue -> metadata, and metadata -> previous binding value.
    legacy_width_map_entry_visits = pointer_move_count * 3 * column_count
    legacy_column_array_entry_visits = (
        pointer_move_count
        * 3
        * column_count
        * (column_metadata_entry_count + 1)
    )
    legacy_column_match_checks = pointer_move_count * ((column_count + 1) // 2)
    legacy_property_transactions = pointer_move_count * 2
    legacy_combined_work_units = (
        legacy_width_map_entry_visits
        + legacy_column_array_entry_visits
        + legacy_column_match_checks
        + legacy_property_transactions
    )

    target_schema_build_entry_visits = column_count * (column_metadata_entry_count + 1)
    target_scalar_work_units = pointer_move_count * scalar_operations_per_move
    target_property_transactions = pointer_move_count
    target_compatibility_projection_entry_visits = compatibility_flush_count * (
        column_count + column_count * (column_metadata_entry_count + 1)
    )
    target_combined_work_units = (
        target_schema_build_entry_visits
        + target_scalar_work_units
        + target_property_transactions
        + target_compatibility_projection_entry_visits
    )

    return {
        "schema_version": 1,
        "interpretation": (
            "deterministic aggregate-entry/scalar-operation model; not CPU, "
            "allocation, layout, render, or latency evidence"
        ),
        "column_count": column_count,
        "pointer_move_count": pointer_move_count,
        "column_metadata_entry_count": column_metadata_entry_count,
        "compatibility_flush_count": compatibility_flush_count,
        "scalar_operations_per_move": scalar_operations_per_move,
        "legacy_width_map_entry_visits": legacy_width_map_entry_visits,
        "legacy_column_array_entry_visits": legacy_column_array_entry_visits,
        "legacy_column_match_checks": legacy_column_match_checks,
        "legacy_property_transactions": legacy_property_transactions,
        "legacy_combined_work_units": legacy_combined_work_units,
        "target_schema_build_entry_visits": target_schema_build_entry_visits,
        "target_scalar_work_units": target_scalar_work_units,
        "target_property_transactions": target_property_transactions,
        "target_compatibility_projection_entry_visits": (
            target_compatibility_projection_entry_visits
        ),
        "target_combined_work_units": target_combined_work_units,
        "eliminated_work_units": legacy_combined_work_units - target_combined_work_units,
        "work_reduction_ratio": legacy_combined_work_units / target_combined_work_units,
        "target_input_state_complexity": "O(1) scalar work per pointer move",
        "target_geometry_complexity": "O(A), A = actually affected visible geometry",
        "target_compatibility_complexity": (
            "O(C * F) per cadence/release flush, never per raw pointer move"
        ),
    }


def write_result(path: Path, result: dict[str, int | float | str]) -> None:
    resolved = path.resolve()
    if resolved.drive.upper() == "C:":
        raise ValueError("pressure artifacts must not be written to C:")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    resolved.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--column-count", type=int, default=256)
    parser.add_argument("--pointer-move-count", type=int, default=2_000)
    parser.add_argument("--column-metadata-entry-count", type=int, default=8)
    parser.add_argument("--compatibility-flush-count", type=int, default=1)
    parser.add_argument("--scalar-operations-per-move", type=int, default=3)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run(
        column_count=args.column_count,
        pointer_move_count=args.pointer_move_count,
        column_metadata_entry_count=args.column_metadata_entry_count,
        compatibility_flush_count=args.compatibility_flush_count,
        scalar_operations_per_move=args.scalar_operations_per_move,
    )
    if args.output is not None:
        write_result(args.output, result)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
