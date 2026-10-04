#!/usr/bin/env python3
"""Model editor menu interaction projection pressure.

This is a deterministic worst-case access/materialization count, not product timing.
"""

import argparse
import json
from pathlib import Path, PureWindowsPath


def pressure_report(
    row_count: int,
    hover_transitions: int,
    keyboard_events: int,
    submenu_transitions: int,
    open_depth: int,
    menu_button_count: int,
) -> dict[str, object]:
    values = {
        "row_count": row_count,
        "hover_transitions": hover_transitions,
        "keyboard_events": keyboard_events,
        "submenu_transitions": submenu_transitions,
        "open_depth": open_depth,
        "menu_button_count": menu_button_count,
    }
    for name, value in values.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")

    hover_row_comparisons = hover_transitions * row_count
    hover_row_materializations = hover_transitions * row_count
    keyboard_source_row_reads = keyboard_events * row_count
    keyboard_row_materializations = keyboard_events * row_count
    keyboard_active_row_scans = keyboard_events * row_count
    rebuilt_surface_nodes = submenu_transitions * (
        menu_button_count + open_depth + 2
    )
    current_units = (
        hover_row_comparisons
        + hover_row_materializations
        + keyboard_source_row_reads
        + keyboard_row_materializations
        + keyboard_active_row_scans
        + rebuilt_surface_nodes
    )

    publication_source_row_reads = row_count
    publication_row_materializations = row_count
    hover_state_delta_writes = hover_transitions
    keyboard_index_steps = keyboard_events
    popup_layer_patch_visits = submenu_transitions * open_depth
    target_units = (
        publication_source_row_reads
        + publication_row_materializations
        + hover_state_delta_writes
        + keyboard_index_steps
        + popup_layer_patch_visits
    )

    return {
        "schema": "zircon.editor.menu_interaction_projection_pressure.v1",
        "evidence_kind": "deterministic_worst_case_access_model",
        "is_product_timing": False,
        "inputs": values,
        "modeled_case": (
            "every hover changes row, keyboard navigation resolves the terminal eligible "
            "row, and every submenu transition changes the open stack"
        ),
        "current_event_owned_projection": {
            "hover_row_comparisons": hover_row_comparisons,
            "hover_row_materializations": hover_row_materializations,
            "keyboard_source_row_reads": keyboard_source_row_reads,
            "keyboard_row_materializations": keyboard_row_materializations,
            "keyboard_active_row_scans": keyboard_active_row_scans,
            "rebuilt_surface_nodes": rebuilt_surface_nodes,
            "operation_units": current_units,
        },
        "published_interaction_index": {
            "publication_source_row_reads": publication_source_row_reads,
            "publication_row_materializations": publication_row_materializations,
            "hover_state_delta_writes": hover_state_delta_writes,
            "keyboard_index_steps": keyboard_index_steps,
            "popup_layer_patch_visits": popup_layer_patch_visits,
            "operation_units": target_units,
        },
        "comparison": {
            "operation_reduction_ratio": current_units / target_units,
            "event_time_row_materialization_reduction": (
                hover_row_materializations + keyboard_row_materializations
            ),
            "target_event_time_row_materializations": 0,
        },
        "target_contract": [
            "menu content rows are immutable and published once per content generation",
            "pointer, keyboard, paint, and submenu projection share stable row keys",
            "hover, focus, and pressed state are sparse interaction deltas",
            "next and previous navigation use published eligible-row adjacency",
            "submenu stack changes patch popup layers without rebuilding row content",
            "event callbacks never materialize the published row vector",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and latency timing",
            "text shaping, paint, GPU submission, and native event-loop scheduling",
            "typeahead character comparison cost",
            "unchanged hover events and early active-row matches",
            "hashing, Arc/ModelRc reference increments, and string length constants",
        ],
    }


def pressure_suite() -> dict[str, object]:
    return {
        "schema": "zircon.editor.menu_interaction_projection_pressure_suite.v1",
        "evidence_kind": "deterministic_worst_case_access_model",
        "is_product_timing": False,
        "scenarios": {
            str(row_count): pressure_report(
                row_count=row_count,
                hover_transitions=1_000,
                keyboard_events=1_000,
                submenu_transitions=100,
                open_depth=4,
                menu_button_count=7,
            )
            for row_count in (20, 200, 10_000)
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
