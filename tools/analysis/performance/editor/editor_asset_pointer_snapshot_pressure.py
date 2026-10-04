"""Deterministic accounting for the retained asset pointer snapshot projection."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


def run(
    *,
    project_metadata_fields: int = 5,
    folder_tree_rows: int = 2_000,
    visible_folder_rows: int = 128,
    visible_asset_rows: int = 10_000,
    selection_payload_fields: int = 16,
    stable_publications: int = 1_000,
) -> dict[str, Any]:
    values = {
        "project_metadata_fields": project_metadata_fields,
        "folder_tree_rows": folder_tree_rows,
        "visible_folder_rows": visible_folder_rows,
        "visible_asset_rows": visible_asset_rows,
        "selection_payload_fields": selection_payload_fields,
        "stable_publications": stable_publications,
    }
    if any(value < 0 for value in values.values()):
        raise ValueError("pressure dimensions must be non-negative")
    if stable_publications == 0:
        raise ValueError("stable_publications must be positive")

    # The item generation itself is Arc-backed, so both paths only account for the fields that
    # would otherwise be traversed/cloned while publishing the pointer-owned snapshot.
    unrelated_rows = folder_tree_rows + visible_folder_rows
    legacy_units_per_publication = (
        project_metadata_fields
        + unrelated_rows
        + visible_asset_rows
        + selection_payload_fields
    )
    target_units_per_publication = visible_asset_rows + selection_payload_fields
    legacy_units = legacy_units_per_publication * stable_publications
    target_units = target_units_per_publication * stable_publications
    return {
        **values,
        "legacy_unrelated_row_units_per_publication": unrelated_rows,
        "legacy_clone_work_units": legacy_units,
        "target_projection_work_units": target_units,
        "removed_work_units": legacy_units - target_units,
        "work_reduction_ratio": legacy_units / target_units
        if target_units
        else None,
        "timing_claim": False,
    }


def write_result(path: Path, result: dict[str, Any]) -> None:
    resolved = path.resolve()
    if resolved.drive.rstrip(":").upper() == "C":
        raise ValueError("profile output must not be written to C:")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    resolved.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    result = run()
    if args.output:
        write_result(args.output, result)
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
