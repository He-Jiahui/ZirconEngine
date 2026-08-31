#!/usr/bin/env python3
"""Model eager pointer diagnostics versus source-bound summary/full routing modes.

This is deterministic source-contract evidence, not compiled behavior or product timing.
"""

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
IMPLEMENTATION_SOURCES = (
    "zircon_runtime_interface/src/ui/dispatch/input/result.rs",
    "zircon_runtime_interface/src/ui/surface/pointer/route.rs",
    "zircon_runtime/src/ui/surface/input/diagnostics_budget.rs",
    "zircon_runtime/src/ui/surface/input/dispatch.rs",
    "zircon_runtime/src/ui/surface/input/pointer.rs",
    "zircon_runtime/src/ui/surface/input/window_pump.rs",
    "zircon_runtime/src/ui/surface/input/mouse_motion.rs",
    "zircon_runtime/src/ui/surface/input/text_pointer.rs",
    "zircon_runtime/src/ui/surface/input/rich_link.rs",
    "zircon_runtime/src/ui/surface/input/route_policy.rs",
    "zircon_runtime/src/ui/dispatch/input_manager/manager.rs",
    "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
    "zircon_editor/src/ui/retained_host/shell_pointer/bridge.rs",
)
REFERENCE_SOURCES = (
    "dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h",
    "dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp",
)
MAX_ROUTE_NODES_PER_PATH = 128
MAX_ROUTE_STEPS = 256
MAX_NOTES = 32
MAX_POPUP_ENTRIES = 16
MAX_STRING_BYTES = 8 * 1024


def source_bindings(paths: tuple[str, ...]) -> list[dict[str, object]]:
    bindings = []
    for relative_path in paths:
        path = ROOT / relative_path
        payload = path.read_bytes()
        bindings.append(
            {
                "path": relative_path,
                "bytes": len(payload),
                "sha256": hashlib.sha256(payload).hexdigest().upper(),
            }
        )
    return bindings


def pressure_report(
    pointer_events: int,
    bubble_path_nodes: int,
    focus_path_nodes: int,
    popup_stack_entries: int,
    default_action_handlers: int,
) -> dict[str, object]:
    values = {
        "pointer_events": pointer_events,
        "bubble_path_nodes": bubble_path_nodes,
        "focus_path_nodes": focus_path_nodes,
        "popup_stack_entries": popup_stack_entries,
        "default_action_handlers": default_action_handlers,
    }
    for name, value in values.items():
        if value < 0 or (name == "pointer_events" and value == 0):
            requirement = "positive" if name == "pointer_events" else "non-negative"
            raise ValueError(f"{name} must be {requirement}")

    preview_nodes = bubble_path_nodes
    direct_step_nodes = 1
    eager_diagnostic_identity_copies_per_event = (
        preview_nodes
        + focus_path_nodes
        + popup_stack_entries
        + direct_step_nodes
    )
    eager_diagnostic_vector_allocations_per_event = (
        int(preview_nodes > 0)
        + int(focus_path_nodes > 0)
        + int(popup_stack_entries > 0)
        + direct_step_nodes
    )
    full_preview_nodes = min(preview_nodes, MAX_ROUTE_NODES_PER_PATH)
    full_bubble_nodes = min(bubble_path_nodes, MAX_ROUTE_NODES_PER_PATH)
    full_focus_nodes = min(focus_path_nodes, MAX_ROUTE_NODES_PER_PATH)
    full_popup_entries = min(popup_stack_entries, MAX_POPUP_ENTRIES)
    full_identity_copies_per_event = (
        full_preview_nodes
        + full_bubble_nodes
        + full_focus_nodes
        + full_popup_entries
        + direct_step_nodes
    )
    full_vector_allocations_per_event = (
        int(full_preview_nodes > 0)
        + int(full_bubble_nodes > 0)
        + int(full_focus_nodes > 0)
        + int(full_popup_entries > 0)
        + direct_step_nodes
    )

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
            "canonical_path": "Unreal FWidgetPath is root-to-leaf (leafmost last)",
            "capture_separation": (
                "Unreal FEventRouter policies retain routing path and widgets-under-cursor "
                "as distinct authorities"
            ),
        },
        "inputs": values,
        "events_per_modeled_path": pointer_events,
        "modeled_event_paths": [
            "window-pump normalized pointer move",
            "raw high-precision mouse motion",
        ],
        "diagnostic_limits": {
            "route_nodes_per_path": MAX_ROUTE_NODES_PER_PATH,
            "route_steps": MAX_ROUTE_STEPS,
            "notes": MAX_NOTES,
            "popup_entries": MAX_POPUP_ENTRIES,
            "combined_string_bytes": MAX_STRING_BYTES,
        },
        "prior_eager_diagnostics_baseline": {
            "diagnostic_identity_copies": (
                eager_diagnostic_identity_copies_per_event * pointer_events
            ),
            "diagnostic_vector_allocations": (
                eager_diagnostic_vector_allocations_per_event * pointer_events
            ),
            "default_action_handler_probes": (
                default_action_handlers * pointer_events
            ),
            "window_pump_diagnostic_string_allocations": 3 * pointer_events,
            "raw_mouse_motion_diagnostic_string_allocations": pointer_events,
        },
        "candidate_product_summary": {
            "diagnostic_identity_copies": 0,
            "diagnostic_vector_allocations": 0,
            "window_pump_diagnostic_string_allocations": 0,
            "raw_mouse_motion_diagnostic_string_allocations": 0,
            "diagnostic_budget_presence_checks": 16 * pointer_events,
            "behavioral_path_ownership_transfers": pointer_events,
            "ordinary_dispatch_path_allocations": 0,
            "default_action_dispatch_mask_branches": pointer_events,
            "route_trace_materialization": "disabled",
            "product_selection": [
                "dynamic runtime UI surface manager",
                "editor shell drag and resize pointer dispatch",
            ],
        },
        "candidate_explicit_full_capture": {
            "diagnostic_identity_copies": full_identity_copies_per_event
            * pointer_events,
            "diagnostic_vector_allocations": full_vector_allocations_per_event
            * pointer_events,
            "behavioral_path_ownership_transfers": pointer_events,
            "route_trace_materialization": "explicit Full capture only",
            "truncation_receipt": "records dropped nodes, steps, notes, popup rows, and bytes",
        },
        "eliminated_or_avoided": {
            "diagnostic_identity_copies": (
                eager_diagnostic_identity_copies_per_event * pointer_events
            ),
            "diagnostic_vector_allocations": (
                eager_diagnostic_vector_allocations_per_event * pointer_events
            ),
            "unrelated_default_action_handler_probes": (
                max(default_action_handlers - 1, 0) * pointer_events
            ),
            "window_pump_diagnostic_string_allocations": 3 * pointer_events,
            "raw_mouse_motion_diagnostic_string_allocations": pointer_events,
        },
        "retained_work": [
            "one published hit-grid query",
            "one owned physical root-to-leaf path",
            "an additional dispatch path only for capture or redirect",
            "behavioral bubble/direct dispatch over borrowed path views",
            "same-target hover equality",
            "matching default-action handler",
            "eight O(1) diagnostic budget presence checks",
            "effects, component events, binding reports, and damage publication",
        ],
        "excluded_from_model": [
            "CPU and allocator timing",
            "hit-grid cell density",
            "widget callback cost",
            "capture and redirect frequency",
            "render, present, and OS event-loop work",
            "managed Rust behavior tests and executable profiling",
        ],
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pointer-events", type=int, default=100_000)
    parser.add_argument("--bubble-path-nodes", type=int, default=12)
    parser.add_argument("--focus-path-nodes", type=int, default=12)
    parser.add_argument("--popup-stack-entries", type=int, default=3)
    parser.add_argument("--default-action-handlers", type=int, default=5)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = pressure_report(
        args.pointer_events,
        args.bubble_path_nodes,
        args.focus_path_nodes,
        args.popup_stack_entries,
        args.default_action_handlers,
    )
    payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        with open(args.output, "w", encoding="utf-8", newline="\n") as output:
            output.write(payload)
    else:
        print(payload, end="")


if __name__ == "__main__":
    main()
