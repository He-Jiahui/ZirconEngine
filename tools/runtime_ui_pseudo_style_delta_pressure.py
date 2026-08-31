#!/usr/bin/env python3
"""Model runtime pseudo-style recomputation against published property deltas.

This is a deterministic worst-case access/copy count, not product timing.
"""

import argparse
import json
from pathlib import Path, PureWindowsPath


def pressure_report(
    event_count: int,
    visited_nodes_per_event: int,
    affected_nodes_per_event: int,
    base_attribute_entries: int,
    style_override_entries: int,
    style_token_entries: int,
    candidate_rule_checks: int,
    changed_properties: int,
) -> dict[str, object]:
    values = {
        "event_count": event_count,
        "visited_nodes_per_event": visited_nodes_per_event,
        "affected_nodes_per_event": affected_nodes_per_event,
        "base_attribute_entries": base_attribute_entries,
        "style_override_entries": style_override_entries,
        "style_token_entries": style_token_entries,
        "candidate_rule_checks": candidate_rule_checks,
        "changed_properties": changed_properties,
    }
    for name, value in values.items():
        if value <= 0:
            raise ValueError(f"{name} must be positive")
    if affected_nodes_per_event > visited_nodes_per_event:
        raise ValueError(
            "affected_nodes_per_event must not exceed visited_nodes_per_event"
        )

    current_node_visits = event_count * visited_nodes_per_event
    full_map_entries_per_node = (
        base_attribute_entries
        + style_override_entries
        + style_token_entries
        + 2 * changed_properties
    )
    current_map_entry_copies = current_node_visits * full_map_entries_per_node
    current_rule_checks = current_node_visits * candidate_rule_checks
    current_units = (
        current_node_visits + current_map_entry_copies + current_rule_checks
    )

    target_dependency_visits = event_count * affected_nodes_per_event
    target_property_patches = target_dependency_visits * changed_properties
    target_units = target_dependency_visits + target_property_patches

    return {
        "schema": "zircon.runtime.ui_pseudo_style_delta_pressure.v1",
        "evidence_kind": "deterministic_worst_case_access_copy_model",
        "is_product_timing": False,
        "inputs": values,
        "modeled_case": (
            "every visited node has a retained style baseline and every candidate rule "
            "requires a selector check"
        ),
        "current_runtime_style_recompute": {
            "subtree_or_node_visits": current_node_visits,
            "full_map_entry_copies": current_map_entry_copies,
            "candidate_rule_checks": current_rule_checks,
            "operation_units": current_units,
        },
        "published_pseudo_style_delta": {
            "affected_dependency_visits": target_dependency_visits,
            "changed_property_patches": target_property_patches,
            "full_map_entry_copies": 0,
            "operation_units": target_units,
        },
        "comparison": {
            "operation_reduction_ratio": current_units / target_units,
            "avoided_full_map_entry_copies": current_map_entry_copies,
        },
        "target_contract": [
            "compile pseudo-state dependency edges at style publication",
            "self-state changes evaluate only the changed node's matching declarations",
            "ancestor-state changes visit indexed affected terminals, not the full subtree",
            "apply property deltas over immutable baselines without cloning complete maps",
            "dirty flags and damage are derived from changed property metadata",
            "event processing records zero full style-map materializations",
        ],
        "excluded_from_model": [
            "CPU, allocator, RSS, and latency timing",
            "BTreeMap/hash constants and actual TOML value byte sizes",
            "style publication/index construction and topology mutation cost",
            "layout, render extraction, frame publication, GPU, and present",
            "nodes without retained baseline attributes and selector early rejection",
        ],
    }


def pressure_suite() -> dict[str, object]:
    common = {
        "event_count": 1_000,
        "base_attribute_entries": 24,
        "style_override_entries": 8,
        "style_token_entries": 8,
        "candidate_rule_checks": 4,
        "changed_properties": 3,
    }
    return {
        "schema": "zircon.runtime.ui_pseudo_style_delta_pressure_suite.v1",
        "evidence_kind": "deterministic_worst_case_access_copy_model",
        "is_product_timing": False,
        "scenarios": {
            "self_hover": pressure_report(
                visited_nodes_per_event=1,
                affected_nodes_per_event=1,
                **common,
            ),
            "ancestor_sparse_descendants": pressure_report(
                visited_nodes_per_event=10_000,
                affected_nodes_per_event=64,
                **common,
            ),
            "ancestor_dense_descendants": pressure_report(
                visited_nodes_per_event=10_000,
                affected_nodes_per_event=10_000,
                **common,
            ),
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
