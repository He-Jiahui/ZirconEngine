---
related_code:
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/core/framework/input/button_input_state.rs
  - zircon_runtime/src/core/framework/input/cursor.rs
  - zircon_runtime/src/core/framework/input/input_button.rs
  - zircon_runtime/src/core/framework/input/input_action.rs
  - zircon_runtime/src/core/framework/input/input_action_context.rs
  - zircon_runtime/src/core/framework/input/input_binding.rs
  - zircon_runtime/src/core/framework/input/input_action_map.rs
  - zircon_runtime/src/core/framework/input/input_action_state.rs
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/core/framework/input/input_event_record.rs
  - zircon_runtime/src/core/framework/input/event_retention/mod.rs
  - zircon_runtime/src/core/framework/input/event_retention/queue_status.rs
  - zircon_runtime/src/core/framework/input/event_retention/recording_config.rs
  - zircon_runtime/src/core/framework/input/event_retention/recording_status.rs
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/core/framework/input/file_drag_drop.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/core/framework/input/input_snapshot.rs
  - zircon_runtime/src/core/framework/input/gamepad.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/core/framework/input/touch.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/input/prelude.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/runtime/action_evaluator/generation.rs
  - zircon_runtime/src/input/runtime/action_evaluator/workspace.rs
  - zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs
  - zircon_runtime/src/input/runtime/event_buffer/mod.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
  - zircon_runtime/src/input/runtime/event_buffer/recorder.rs
  - zircon_runtime/src/input/runtime/recording.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs
  - zircon_runtime/src/dynamic_api/session/events/gamepad.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - zircon_runtime/src/input/tests/input_manager/event_buffer.rs
  - zircon_runtime/src/input/tests/input_manager/touch_gamepad.rs
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
  - zircon_runtime/src/input/tests/input_manager/host_requests.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/pointer.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/enable.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/surrounding_text.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/entry_tree.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/framework/input/button_input_state.rs
  - zircon_runtime/src/core/framework/input/cursor.rs
  - zircon_runtime/src/core/framework/input/input_button.rs
  - zircon_runtime/src/core/framework/input/input_action.rs
  - zircon_runtime/src/core/framework/input/input_action_context.rs
  - zircon_runtime/src/core/framework/input/input_binding.rs
  - zircon_runtime/src/core/framework/input/input_action_map.rs
  - zircon_runtime/src/core/framework/input/input_action_state.rs
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/core/framework/input/input_event_record.rs
  - zircon_runtime/src/core/framework/input/event_retention/mod.rs
  - zircon_runtime/src/core/framework/input/event_retention/queue_status.rs
  - zircon_runtime/src/core/framework/input/event_retention/recording_config.rs
  - zircon_runtime/src/core/framework/input/event_retention/recording_status.rs
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/core/framework/input/file_drag_drop.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/input_manager.rs
  - zircon_runtime/src/core/framework/input/gamepad.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/core/framework/input/touch.rs
  - zircon_runtime/src/input/runtime/action_evaluator.rs
  - zircon_runtime/src/input/prelude.rs
  - zircon_runtime/src/input/runtime/action_evaluator/generation.rs
  - zircon_runtime/src/input/runtime/action_evaluator/workspace.rs
  - zircon_runtime/src/input/runtime/action_evaluator/frame_axis_index.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs
  - zircon_runtime/src/input/runtime/event_buffer/mod.rs
  - zircon_runtime/src/input/runtime/event_buffer/frame.rs
  - zircon_runtime/src/input/runtime/event_buffer/recorder.rs
  - zircon_runtime/src/input/runtime/recording.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_anchor_inventory.py
  - zircon_runtime_interface/src/runtime_api/mod.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/mod.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/pointer.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/enable.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/surrounding_text.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/cursor/request.rs
plan_sources:
  - user: 2026-05-16 Bevy-style platform/window/winit/gilrs/input parity plan
  - user: 2026-05-16 continue Bevy-style platform/window/input stable prelude completion
  - chat: ZirconEngine Bevy 式 Platform / Window / Input / Gilrs 完成度计划
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-17-input-event-growth-and-frequency.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/godot/core/input/input_map.cpp
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_input/src/touch.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_gilrs/src/lib.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
  - dev/bevy/crates/bevy_gilrs/src/converter.rs
tests:
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - zircon_runtime/src/input/tests/input_manager/event_buffer.rs
  - zircon_runtime/src/input/tests/input_manager/host_requests.rs
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/action_axis_transitions.rs
  - zircon_runtime/src/input/tests/gamepad_bridge.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
  - zircon_runtime/src/input/tests/input_manager/touch_gamepad.rs
  - zircon_runtime/src/tests/runtime_absorption/input_stack.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs::input_manager_accessors_recover_poisoned_state_lock
  - zircon_runtime/src/input/runtime/default_input_action_manager.rs::input_action_manager_accessors_recover_poisoned_evaluator_lock
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy.rs::runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state
  - zircon_runtime::tests::runtime_absorption::input_stack::runtime_12_input_stack_contracts_stay_documented_and_exported
  - zircon_runtime::tests::runtime_absorption::input_stack::runtime_12_action_mapping_keeps_ui_filtered_evaluation_path
  - zircon_runtime::tests::runtime_absorption::input_stack::runtime_12_gamepad_bridge_keeps_runtime_abi_path
  - zircon_runtime::tests::runtime_absorption::input_stack::runtime_12_input_stack_mirror_docs_match_structure_audit_counts
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json (2026-06-20 Runtime 12 input_stack_boundary targeted evidence: expected_runtime_module_count = 12, expected_framework_module_count = 20, expected_test_module_count = 7, public_surface_anchors = 26/26, runtime_12_guard_anchors = 5/5, behavior_test_anchor_count = 15, missing_doc_anchors = [], missing_test_anchors = [], missing_behavior_test_anchors = [], missing_cargo_gate_anchors = [], oversized_modules = [], mirror_docs_guard_present = true, risks = [])
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/input_stack_boundary.py direct import (2026-06-15 Runtime 12 gamepad event-owner drift sync: missing_gamepad_abi_anchors = [], risks = [])
  - cargo test -p zircon_runtime --lib input_snapshot_just_pressed_is_true_for_exactly_one_frame --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 12 M0.1 named anchor: pending after active HZB Cargo lane clears; source/rustfmt static checks passed)
  - cargo test -p zircon_runtime --lib frame_input_clears_after_level_tick_not_before --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 12 M0.1 named anchor: pending after active HZB Cargo lane clears; source/rustfmt static checks passed)
  - cargo test -p zircon_runtime --lib action_map_resolves_chords_and_reports_just_activated --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 12 M1 action-map named anchor: pending after active Cargo lane clears; source/rustfmt static checks passed)
  - cargo test -p zircon_runtime --lib replacing_action_map_rebuilds_bindings_automatically --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (current Runtime 12 action-map named anchor: managed Cargo validation pending)
  - cargo test -p zircon_runtime --lib gamepad_axis_binding_reports_continuous_action_value --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-input-axis-0617 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-17 Runtime 12 M1.4 action-axis value: implementation-first slice; focused Cargo deferred by request; rustfmt check, direct `input_stack_boundary_audit`, standalone input_stack 4/4, standalone plan_status 32/32, and direct runtime_plan_status boundary risks=[] passed)
  - cargo test -p zircon_runtime --lib gamepad_axis_action_reports_deadzone_transition_edges --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-input-axis-transition-0617 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-17 Runtime 12 M1.7 gamepad axis transition edges: implementation-first slice; focused Cargo deferred by request; rustfmt check, direct `input_stack_boundary_audit`, standalone input_stack 4/4, standalone plan_status 32/32, and direct runtime_plan_status boundary risks=[] passed)
  - cargo test -p zircon_runtime --lib input_config_builds_action_evaluator_from_serialized_action_map --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-input-config-0617 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-17 Runtime 12 M1.5 InputConfig action map source: implementation-first slice; focused Cargo deferred by request; rustfmt check, direct `input_stack_boundary_audit`, standalone input_stack 4/4, standalone plan_status 32/32, and direct runtime_plan_status boundary risks=[] passed)
  - cargo test -p zircon_runtime --lib input_action_manager_resolves_from_runtime_module_descriptor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-input-action-manager-0617 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-17 Runtime 12 M1.6 action-manager registration: implementation-first slice; focused Cargo deferred by request; rustfmt check, direct `input_stack_boundary_audit`, standalone input_stack 4/4, standalone plan_status 32/32, and direct runtime_plan_status boundary risks=[] passed)
  - cargo test -p zircon_runtime --lib gamepad_disconnect_clears_held_state_without_panic --locked --jobs 1 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-13 Runtime 12 M2 gamepad named anchor: pending after active Cargo lane clears; source/rustfmt static checks passed)
  - cargo test -p zircon_runtime --lib gamepad_host_bridge_uses_runtime_gamepad_abi_constructors --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-input-0617 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-17 Runtime 12 M2 gamepad source guard: source updated to include split event owner `session/events.rs`; rustfmt, direct `input_stack_boundary_audit`, and standalone input_stack 4/4 passed; focused Cargo timed out after 605s with no result and no residual cargo/rustc/rustdoc processes, broader gates pending)
  - cargo test -p zircon_runtime --lib cursor_host_requests_are_frame_local_and_drainable --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-12-cursor-host-0620 --message-format short --color never -- --nocapture --test-threads=1 (2026-06-20 Runtime 12 cursor host requests: source/static guards updated; package Cargo gate remains deferred with broader input/action_map/gamepad/app gates)
  - zircon_runtime/src/input/tests/boundary.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime/src/dynamic_api/tests/input_events.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/sources.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/entry_tree.rs
  - zircon_app/src/entry/tests/mod.rs
  - tools/tests/test_runtime_input_stack_audit.py
  - tests/acceptance/runtime-input-stack-audit-owner-sync.md
doc_type: module-detail
---

# Runtime Input State

Runtime 12 current child-owner inventory: the current `input_stack_boundary` source manifest contains `expected_runtime_module_count = 27`, `expected_framework_module_count = 44`, `expected_test_module_count = 7`, and `expected_guard_file_count = 6`. The runtime inventory includes the public `input/camera_controller/{free,orbit,pan}` implementations, while `ActionEvaluationGeneration`, `ActionEvaluationWorkspace`, and `FrameAxisIndex` remain dedicated evaluator child owners: generation owns immutable map-change-time lookup data, workspace owns reusable evaluator-local frame state, and the axis index owns reusable per-frame axis lookup. Each binding's axes use one `evaluate_binding_axes` pass while the evaluator filters caller-supplied UI-consumed axes before value and edge projection. The runtime prelude plus framework input-manager contract remain explicitly wired and inventoried. The 2026-07-18 M4 acceptance record remains historical evidence for the prior 18-module layout; it does not validate this later internal split. `runtime_12_input_stack_mirror_docs_match_structure_audit_counts` keeps this current owner inventory synchronized, while the protected parent plan and runtime index remain outside this business manifest and are not claimed as current mirror authorities.

Earlier accepted/deferred slice anchors remain discoverable as `input_frame_contract_static_passed_cargo_pending`, `arbitration_judgement_documented_static_passed`, `action_contract_static_passed_cargo_pending`, `action_evaluator_static_passed_cargo_pending`, `action_context_static_passed_cargo_pending`, `input_recording_replay_static_passed_cargo_deferred`, `gamepad_bridge_static_passed_cargo_pending`, and `runtime_12_input_stack_cargo_pending_gate_stays_explicit_until_input_validation`.

Runtime 12 managed Cargo gates remain pending: `cargo test -p zircon_runtime --lib input --locked -- --nocapture`, `cargo test -p zircon_runtime --lib action_map --locked -- --nocapture`, `cargo test -p zircon_runtime --lib gamepad --locked -- --nocapture`, and `cargo test -p zircon_app --locked`. Static source review does not replace any of these managed gates.

## Purpose

`zircon_runtime::core::framework::input` owns neutral runtime input contracts. M2 expands the old cursor/button/wheel snapshot into a Bevy-style state model without removing the legacy `InputSnapshot` fields consumed by existing runtime and app tests.

The design follows Bevy's input split:

- `ButtonInputState<T>` mirrors Bevy `ButtonInput<T>` semantics with durable `pressed` state plus per-frame `just_pressed` and `just_released` transitions.
- `InputEvent` remains the neutral submitted-event vocabulary, covering cursor position, cursor enter/leave, cursor host requests, file drag/drop, window status, mouse motion, Bevy-style wheel x/y/unit events, keyboard, IME composition/delete requests, outgoing IME host requests, focus loss, touch, gamepad connection, gamepad button, and gamepad axis events. Queue retention is an explicit manager policy rather than an implied unbounded event log.
- `InputFrameSnapshot` is the new full-frame state view for systems that need transitions, cursor-in-window state, current-frame cursor host requests, file drag/drop and window status events, active touches, connected gamepads, processed gamepad axes, current-frame gamepad axis transitions, processed analog button values, current-frame gamepad rumble requests, IME preedit/commit/delete-surrounding/host-request state, precise wheel events, and motion accumulators.
- `InputSnapshot` remains the compatibility view: cursor, pressed buttons, and scalar wheel accumulator.

## Runtime Manager Behavior

`DefaultInputManager::begin_frame()` clears transient button transitions plus wheel and mouse-motion accumulators, the reduced frame event queue, and current-frame host request queues. It does not release currently pressed buttons. Callers that do not use `begin_frame()` keep the previous behavior where wheel accumulates until inspected.

`InputManager::drain_events()` is a frame-transient read port, not a history store. Consecutive `CursorMoved` events collapse to the latest position and consecutive raw `MouseMotion` events collapse to their summed delta. Coalescing stops at every other event, so button, touch, keyboard, IME, window, file, wheel, and gamepad edges retain their submitted order. `InputEventQueueStatus` exposes current retained depth plus the frame's coalesced count. The simulated ten-minute 125/500/1000Hz pressure anchor proves each pointer-only interval stays at one queued event, while the mixed edge anchor proves coalescing never jumps across an edge.

Cursor events keep `cursor_position` and `cursor_inside_window` separate. `CursorMoved` updates position, while `CursorEntered` and `CursorLeft` only update whether the host cursor is inside the window. This mirrors Bevy's split between `CursorMoved`, `CursorEntered`, and `CursorLeft` messages in `dev/bevy/crates/bevy_window/src/event.rs`.

Cursor host requests are runtime-to-host messages. Runtime systems submit `InputEvent::CursorHostRequest` carrying `CursorHostRequest::SetVisible`, `SetGrabMode`, `SetHitTest`, or `SetPosition`; `InputFrameSnapshot::cursor_host_requests` exposes the current-frame view, and `InputManager::drain_cursor_host_requests()` is the one-shot handoff path used by the dynamic runtime host-request ABI. The queue is frame-local: draining clears only cursor host requests, while `begin_frame()` drops any undrained requests before the next frame. This lane is lower transport only; UI pointer capture, popup routing, focus, and hit testing authority still belong to Runtime 09.

File drag/drop events are current-frame messages. `FileDragDropEvent::Hovered` and `Dropped` carry a UTF-8 path string, and `Cancelled` records that the host drag operation left or was cancelled. `begin_frame()` clears `InputFrameSnapshot::file_drag_drop_events`; it does not store a durable hovered-file set because the asset/UI layer owns any longer-lived import workflow.

Window status events are current-frame messages. `WindowStatusEvent` mirrors Bevy window-event style messages for moved, occluded, theme changed, backend scale-factor changed, logical scale-factor changed, close requested, and destroyed notifications. `begin_frame()` clears `InputFrameSnapshot::window_status_events`; longer-lived window policy remains owned by the host/runtime profile layer.

Button events update `ButtonInputState<InputButton>`. Repeated press events do not produce repeated `just_pressed`, and repeated release events do not produce repeated `just_released`.

Mouse-wheel events now follow Bevy's `MouseWheel` / `AccumulatedMouseScroll` split from `dev/bevy/crates/bevy_input/src/mouse.rs`: `MouseWheelEvent` carries `MouseScrollUnit::Line` or `Pixel` plus horizontal and vertical deltas, while `InputFrameSnapshot::mouse_wheel_accumulator`, `mouse_wheel_unit`, and `mouse_wheel_events` preserve current-frame precise scroll data. `wheel_accumulator` remains as a compatibility scalar; pixel events use Zircon's `PIXEL_SCROLL_LINE_DELTA_SCALE` through `MouseWheelEvent::vertical_line_delta()` so older camera controls keep their previous vertical-scroll feel without retaining legacy API names.

Keyboard events update both physical and logical button views when the host provides a logical key. `FocusLost` releases every pressed button, clears touch, wheel, mouse-motion, disables IME composition and clears its preedit, and clears processed gamepad state while recording nonzero gamepad axes as transitions to zero so action mapping observes deactivation. It discards pending IME/cursor host requests and publishes one IME disable plus cursor ungrab request, so an inactive window cannot apply a stale composition enable or exclusive pointer capture. It retains connected gamepad identities because losing window focus is not a physical disconnect.

IME events update a separate composition state. `ImeEvent::Enabled` and `Disabled` track whether the host IME path is active. `Preedit` stores the current composing text plus optional byte cursor range, and an empty preedit clears composition. `Commit` clears preedit and records committed text in `InputFrameSnapshot::ime_commits` for the current frame. `DeleteSurrounding` records a before/after byte-count request in `InputFrameSnapshot::ime_delete_surrounding` for the current frame without mutating text directly. Outgoing `ImeHostRequest` values record enable/disable, cursor area, and surrounding-text requests in `InputFrameSnapshot::ime_host_requests`. `InputManager::drain_ime_host_requests()` is the one-shot host handoff path used by the dynamic runtime ABI; it clears only outgoing IME host requests and leaves the normal input event stream intact. `begin_frame()` also clears committed text, delete-surrounding requests, and outgoing host requests; it keeps IME enabled/preedit state.

Touch events keep a map of active touches. Started and moved phases update the active point; ended and cancelled phases remove it.

Gamepad connection events track connected gamepad ids. Disconnect clears that gamepad's axes, clears its analog button values, and releases its pressed gamepad buttons. This keeps stale physical-device state from surviving device removal; focus loss separately clears active input while preserving connected-device identity.

Gamepad button and axis values intentionally enter the runtime as raw host readings. `GamepadButtonAxisSettings` clamps analog button values into `[0.0, 1.0]`, applies a low zone of `0.05`, a high zone of `0.95`, and ignores processed changes below `0.01`. `GamepadButtonSettings` then applies Bevy-style digital hysteresis: a gamepad button presses at `0.75` and releases at `0.65`. `GamepadAxisSettings` applies an axis deadzone of `[-0.05, 0.05]`, livezone bounds of `[-1.0, 1.0]`, and a processed-value change threshold of `0.01`. When a processed axis value changes, `DefaultInputManager` records a `GamepadAxisTransition` for the current frame so action mapping can detect deadzone activation and deactivation edges without storing previous action state in gameplay code. These defaults mirror Bevy's split where the gilrs backend emits raw events and `bevy_input::gamepad::GamepadSettings` owns filtering.

Gamepad rumble requests are runtime-to-host requests. Runtime systems submit `InputEvent::GamepadRumbleRequest`; `InputFrameSnapshot::gamepad_rumble_requests` exposes the current-frame view, and `InputManager::drain_gamepad_rumble_requests()` is the one-shot handoff used by the dynamic runtime host-request ABI. The request intensity is clamped when converted to the stable ABI so invalid caller values cannot leak to a native backend.

Runtime 12 M2 adds named anchors for the gamepad bridge that already exists in the preview stack. `gamepad_disconnect_clears_held_state_without_panic` verifies that a disconnect clears the durable connected gamepad id, processed axis values, analog button values, and pressed gamepad buttons, while still surfacing a one-frame `just_released` transition. `gamepad_host_bridge_uses_runtime_gamepad_abi_constructors` is a source guard that locks the app-side gilrs path to `ZrRuntimeEventV1::gamepad_connection_with_ids`, `gamepad_button`, and `gamepad_axis`, and locks the dynamic-session event owner `zircon_runtime/src/dynamic_api/session/events.rs` to the matching `InputEvent::Gamepad*` reducers.

`runtime_absorption::input_stack::runtime_12_gamepad_bridge_keeps_runtime_abi_path` guards the gamepad contract, app-side ABI constructors, dynamic-session event reducers, and M2 bridge test anchors.

## Frame Input Contract

Runtime 12 M0.1 makes the frame-input lifecycle explicit. Platform and app host events enter through ABI constructors or direct runtime event submission, then `DefaultInputManager::submit_event(...)` reduces them into durable button state, frame-local transitions, accumulators, and current-frame event lists. Gameplay and UI-facing consumers should read `InputFrameSnapshot` when they need `just_pressed`, `just_released`, mouse motion, wheel deltas, touch, IME, file-drag, window-status, or gamepad state. `InputSnapshot` remains the compact compatibility view for cursor, pressed buttons, and scalar wheel data.

`DefaultInputManager::begin_frame()` is the transition-clear boundary for the next frame. It clears `just_pressed`, `just_released`, wheel/motion accumulators, current-frame message lists, and outgoing host request queues, but it does not release durable pressed buttons or persistent IME/gamepad state. This follows the same state split as Bevy `ButtonInput<T>`, where `just_pressed` and `just_released` are one-frame facts layered on top of a longer-lived pressed set.

The same boundary clears any undrained `drain_events()` data. Consumers that need deterministic history must opt into the separate recording lane; they must not treat a missed transient drain as persistent storage.

The dynamic runtime session has one important ordering rule: `RuntimeDynamicSession::tick_frame()` runs the loaded level before it calls `input_manager.begin_frame()`. Guard anchor: RuntimeDynamicSession::tick_frame() runs the loaded level before it calls `input_manager.begin_frame()`. Events submitted since the previous clear therefore remain visible to level systems for exactly one update, and are cleared only after that update has consumed the frame. `frame_input_clears_after_level_tick_not_before` is the source-order guard for that rule; `input_snapshot_just_pressed_is_true_for_exactly_one_frame` is the behavior anchor for the one-frame transition state.

Runtime 12 M0.2 settles the first arbitration boundary above this lower input state: UI surface/pointer capture/popup/focus 优先; UI surface hits, pointer capture, popup scope, and text/navigation focus go through the UI 09 `interaction_gate` / dispatch authority before gameplay actions. Gameplay action mapping consumes UI-unhandled input, or all input in headless/no-UI profiles. `DefaultInputManager` still does not make that decision; it only preserves the frame facts both consumers need.

`runtime_absorption::input_stack::runtime_12_input_stack_contracts_stay_documented_and_exported` guards this contract at the plan/doc/source boundary. It keeps the frame contract anchors, `DefaultInputManager` / `InputFrameSnapshot` public surface, and named M0.1 test anchors synchronized while the Cargo input filter remains pending.

Historical 2026-06-20 evidence from `input_stack_boundary.py` reported `expected_runtime_module_count = 12`, `expected_framework_module_count = 20`, `expected_test_module_count = 7`, `public_surface_anchors = 26/26`, `runtime_12_guard_anchors = 5/5`, `behavior_test_anchor_count = 15`, and empty missing/risk lists. Those dated counts are retained only as provenance; the unique current block at the top of this document is authoritative.

Historical 2026-06-21 inventory-split evidence recorded runtime/framework/test owner modules 12/20/7, public-surface anchors 26/26, Runtime 12 guard anchors 5/5, behavior-test anchors 15/15, and cursor host-request anchors 12/12. It also established `input_stack_source_inventory.py` as the module/line-budget owner and `input_stack_anchor_inventory.py` as the declaration and anchor owner. These dated counts do not describe current source.

## Action Mapping

Runtime 12 M1 adds the first data-driven gameplay action layer without removing the raw input read path. The shared contract lives in `zircon_runtime::core::framework::input`:

- `InputAction` names a gameplay action and optionally tags it with a context and display label. The id is a plain string so project files, dynamic API payloads, and tools can serialize the same key without Rust type identity.
- `InputActionContext` names an evaluable action layer such as `gameplay`, `menu`, or another project mode. It is serializable data with priority and enabled flags so projects can switch context sets without recompiling bindings.
- `InputBinding` maps one action id to one physical button chord, one or more gamepad axis bindings, or a button-gated axis binding. Button chords sort and deduplicate their buttons on construction, and axis bindings sort by gamepad, axis, and direction, which keeps serialized bindings deterministic and prevents repeated physical inputs from creating duplicate activation edges.
- `InputActionMap` stores the project-side context, action, and binding tables. It supports clearing and replacing bindings at runtime, so rebinding changes data instead of recompiling gameplay code.
- `InputActionState` is the frame result: durable `pressed` actions, one-frame `just_activated` actions, one-frame `just_deactivated` actions, and per-action continuous values readable through `InputActionState::value(...)`.

`zircon_runtime::input::runtime::InputActionEvaluator` owns the concrete evaluation step. It reads an `InputFrameSnapshot`, evaluates each explicit action against its bindings, reports a chord as active only when all chord buttons are currently pressed, and reports `just_activated` when an active binding contains at least one `just_pressed` button. It reports `just_deactivated` only when no binding for the action remains active and at least one binding button was released this frame.

The evaluator compiles a deterministic action-id-to-binding-index table when it receives an `InputActionMap`, and rebuilds that table only in `set_action_map(...)`. The count baseline for one binding per action is 100, 10,000, and 1,000,000 candidates under the former full scan at 10/100/1000 bindings; the indexed evaluator exposes 10, 100, and 1,000 candidates to the same evaluation loop. This is a count-based baseline, avoiding a new benchmark dependency while still proving the asymptotic owner changed from per-action full scans to stable configuration lookup.

The evaluator also accepts `evaluate_with_consumed_buttons(...)`, `evaluate_with_consumed_input(...)`, `evaluate_with_active_contexts(...)`, `evaluate_with_active_contexts_and_consumed_buttons(...)`, and `evaluate_with_active_contexts_and_consumed_input(...)`. That is the Runtime 12 M0.2/M1.3/M1.8 bridge: callers that run UI routing first pass any UI-consumed buttons and `GamepadAxisInput` values to the gameplay evaluator, callers with menu/gameplay modes pass active contexts, and those filtered bindings are ignored for the frame before axis values or axis transition edges are evaluated. Actions without a context remain global so host-level or always-on gameplay commands can keep working. Headless or no-UI callers use `evaluate(...)`, which treats the whole snapshot as unconsumed gameplay input and leaves context filtering disabled.

Runtime 12 M1.4 extends action mapping to processed gamepad axes without changing the raw input read path. `InputAxisBinding` targets `InputFrameSnapshot::gamepad_axes`, and `InputAxisDirection::{Full, Positive, Negative}` maps a signed axis into a signed full value or a one-sided strength. Button-only actions still report `1.0` while pressed. Axis actions report the dominant absolute axis value through `InputActionState::value(...)` and are considered pressed when that value is non-zero. Status anchor: `action_axis_value_static_passed_cargo_deferred`.

Runtime 12 M1.7 adds gamepad axis transition edges. `InputFrameSnapshot::gamepad_axis_transitions` stores the current frame's processed `GamepadAxisTransition` values, using the first value seen at frame start as `previous_value` and the final processed value as `value` when multiple raw events arrive in one frame. `InputActionEvaluator` uses those transitions to report axis-only `just_activated` when a binding moves from zero to non-zero after deadzone filtering, and `just_deactivated` when it returns to zero or the connected gamepad is removed. Button-gated axis bindings keep the same UI-consumed/context filtering as button bindings, then add axis activation and deactivation edges while the gate buttons are held. Status anchor: `action_axis_transition_static_passed_cargo_deferred`.

Runtime 12 M1.8 adds consumed-axis arbitration for the same UI-first contract. `GamepadAxisInput` is the neutral physical-axis key that UI 09 can pass after pointer capture, popup focus, menu navigation, or another UI route consumes a gamepad axis. The evaluator checks those caller-supplied keys inside the single `evaluate_binding_axes(...)` pass before value or transition projection, so a UI-consumed stick does not keep a gameplay action pressed, does not leak a continuous value, and does not report axis `just_activated` or `just_deactivated` in the same frame. `InputActionManager` and `DefaultInputActionManager` expose the same consumed-input path for runtime manager users. Status anchor: `action_axis_consumption_static_passed_cargo_deferred`.

Runtime 12 M1.5 makes `InputConfig` the module-level action-map data source. `InputConfig` serializes `enabled` plus `action_map`, preserves the previous default `enabled = false`, and exposes `with_enabled(...)`, `with_action_map(...)`, `effective_action_map()`, and `action_evaluator()` so runtime setup can build an `InputActionEvaluator` from deserialized configuration. Disabled configs resolve to an empty effective action map, so configured bindings cannot activate until the input action layer is enabled. It does not own UI routing, project asset persistence, or config-store loading; those later owners can feed the same `InputConfig` without bypassing the existing `InputActionMap` contract. Status anchor: `action_config_static_passed_cargo_deferred`.

Runtime 12 M1.6 registers action mapping through the runtime manager path instead of requiring callers to construct an evaluator by hand. `InputActionManager` is the framework contract, `DefaultInputActionManager` wraps `InputActionEvaluator`, `InputModule` registers `InputModule.Manager.InputActionManager` through `module_descriptor_with_config(InputConfig)`, and `core::manager::resolve_input_action_manager(...)` returns the manager handle from a `CoreRuntime`. The manager still evaluates caller-provided `InputFrameSnapshot` values with explicit active contexts and consumed buttons; UI routing remains owned by Runtime 09. Status anchor: `action_manager_registration_static_passed_cargo_deferred`.

`runtime_absorption::input_stack::runtime_12_action_mapping_keeps_ui_filtered_evaluation_path` guards the action-map contract shape, the `InputActionContext` contract, the axis-value, axis-transition, and consumed-axis contracts, the config data-source contract, the runtime action-manager registration path, the `evaluate_with_consumed_buttons(...)` / `evaluate_with_consumed_input(...)` and active-context evaluator paths, and the M1 action-map test anchors including `action_contexts_filter_gameplay_and_menu_maps_without_rebinding`, `gamepad_axis_binding_reports_continuous_action_value`, `consumed_gamepad_axis_does_not_activate_gameplay_action`, `gamepad_axis_action_reports_deadzone_transition_edges`, `input_config_builds_action_evaluator_from_serialized_action_map`, and `input_action_manager_resolves_from_runtime_module_descriptor`.

## Input Recording And Replay

`InputRecording` is the runtime-owned deterministic input capture container for headless replays, gameplay regression harnesses, and future tooling. It stores ordered `InputRecordingFrame` values, and each frame stores the drained `InputEventRecord` list for one runtime frame. `InputRecordingFrame::capture_from_manager(frame_index, input_manager)` drains `InputManager::drain_event_records()`, so recording does not bypass `DefaultInputManager::submit_event(...)`.

Recording is disabled by default. A capture owner must call `set_event_recording_config(InputEventRecordingConfig::enabled(capacity))` before submitting the events it wants to retain. Raw submitted events enter the recording lane before pointer coalescing, preserving deterministic input order up to the configured bound. When the bound is full the oldest record is discarded, `InputEventRecordingStatus::discarded_records` increases, and `retained_records` remains at or below capacity. `drain_event_records_with_status()` drains records and snapshots those counters under the same manager lock; `InputRecordingFrame` stores the cumulative discarded count, and `InputRecording::is_complete()` lets replay/tool owners reject an incomplete capture instead of silently treating it as lossless. Together, `InputEventQueueStatus::coalesced_events` and the recording status provide the required coalesce/drop diagnostics without adding an app-side cache. Disabling recording clears the retained queue and counters. System time and event cloning are skipped entirely while recording is disabled.

`InputReplayCursor::replay_next_frame(...)` replays one recorded frame by calling `InputManager::begin_frame()` and then submitting the recorded `InputEvent` values in their original frame order. The original `InputEventRecord.sequence` and `timestamp_millis` remain preserved in the recording for diagnostics and serialization, but replay deliberately re-enters through `submit_event(...)`; the destination manager owns fresh runtime sequence/timestamp metadata. `InputReplayFrameReport` returns the recorded frame index, event count, and the resulting `InputFrameSnapshot`, which lets headless tests or script tools read the same-frame button, axis, mouse, IME, and gamepad state after replay.

The bounded-recording cutover keeps editor UI capture controls, an asset-backed recording file format, and cross-process input streaming out of scope. Behavior anchors are `input_recording_captures_drainable_event_records_by_frame`, `input_recording_marks_a_bounded_capture_incomplete_after_discard`, `input_replay_restores_frame_snapshots_in_recorded_order`, `recording_is_opt_in_bounded_and_reports_discarded_raw_records`, `pointer_event_streams_are_frame_bounded_at_common_polling_rates`, `action_evaluator_indexes_10_100_1000_and_10000_bindings_once`, and `action_evaluator_records_generation_builds_and_distinct_projected_actions`; managed Cargo evidence is recorded by the Runtime12 numbered child output after the milestone testing stage.

## Compatibility

Existing callers can continue to call `snapshot()` and inspect `pressed_buttons`. New code should call `frame_snapshot()` when it needs Bevy-style transitions, mouse motion, touch, or gamepad state.

The stable runtime prelude now exposes the neutral input contracts, the default input manager, and the `InputModule` descriptor alongside the platform capability matrix. Runtime modules can therefore depend on Bevy-style input vocabulary without reaching through the concrete input module path.

## Event Log Harness

M6 adds a hardware-free log harness in `zircon_runtime/src/input/tests/input_manager.rs`: `input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad`. The test submits one mixed frame through `DefaultInputManager` only: window status, keyboard, cursor, raw mouse motion, mouse wheel, mouse button, touch, gamepad connection, gamepad button, and gamepad axis events all enter as `InputEvent` values.

The harness then builds its log from `InputFrameSnapshot`, not from the submitted fixture list. Window messages come from `window_status_events`; keyboard, mouse button, and gamepad button entries come from `ButtonInputState::just_pressed_inputs()`; mouse cursor, motion, and wheel entries come from the frame accumulators; touch entries come from `active_touches`; and gamepad connection/axis entries come from `connected_gamepads` and `gamepad_axes`. The same test drains `InputEventRecord` sequence numbers and checks they are contiguous from `1..=12`, so the example verifies both state reduction and append-only event recording on the normal runtime input manager path.

Runtime 15 M3 keeps the input manager test root folder-backed under `Runtime 15 M3 input manager test folder split` / `runtime_15_input_manager_tests_folder_split_static_passed_cargo_deferred`: `input/tests/input_manager.rs` now only owns shared imports and mounts `input/tests/input_manager/frame_state.rs`, `input/tests/input_manager/touch_gamepad.rs`, and `input/tests/input_manager/host_requests.rs`. `frame_state.rs` owns basic state, frame-clear, focus, and IME behavior; `touch_gamepad.rs` owns touch/gamepad state, the event-log harness, and gamepad filtering tests; `host_requests.rs` owns frame-local gamepad rumble and cursor host requests. `runtime_15_input_manager_tests_are_folder_backed` keeps those owners and docs/status anchors synchronized, and the 2026-07-01 follow-up repaired its status/date map reads to `expected_slices/{status,date}/runtime_15/m3_structure_support.rs`; broader input Cargo gates remain pending.

This is intentionally a test harness rather than a native desktop example binary. It gives CI the M6 example coverage without depending on a physical window, keyboard, mouse, touch device, or controller. Real winit/gilrs smoke testing remains optional because hardware availability cannot be a workspace gate.

## Runtime 15 M3 input runtime manager lock poison recovery

状态：`runtime_15_input_runtime_manager_lock_poison_recovery_static_passed_cargo_deferred`。

Runtime 15 M3 extends the E9/F2 poison-safe lock rule to the input runtime managers without changing the public `InputManager` or `InputActionManager` contracts. `zircon_runtime/src/input/runtime/default_input_manager.rs` now owns private `lock_state()` for `InputState`, and `begin_frame`, `submit_event`, snapshot, frame snapshot, and drain paths call that helper instead of direct lock unwrap. `zircon_runtime/src/input/runtime/default_input_action_manager.rs` now owns private `lock_evaluator()` for `InputActionEvaluator`, and action-map plus evaluation paths call that helper.

The module-local tests `input_manager_accessors_recover_poisoned_state_lock` and `input_action_manager_accessors_recover_poisoned_evaluator_lock` deliberately poison the input state and action evaluator locks, then verify submit/snapshot/drain/evaluate paths still recover. `structure_convention/lock_poison_policy.rs::runtime_15_input_runtime_manager_lock_poison_recovery_guard_covers_input_state` keeps `input/runtime/default_input_manager.rs`, `input/runtime/default_input_action_manager.rs`, this module doc, Runtime 15 status rows, and plan mirrors synchronized. Full input Cargo gates remain pending behind active external Cargo/Rust lanes.

## Runtime Preview Host Translation

M3 wires the runtime preview host through the existing `ZrRuntimeEventV1` ABI instead of importing `zircon_runtime::input` into `zircon_app`.

`zircon_app::entry::runtime_entry_app::ApplicationHandler` now translates winit window events as follows:

- `WindowEvent::PointerMoved` remains a pointer move for mouse-like sources, but touch sources become ABI touch moved events with the winit finger id.
- `WindowEvent::PointerEntered` and `PointerLeft` become ABI cursor boundary events, matching Bevy's `bevy_winit/src/state.rs` forwarding into `bevy_window::CursorEntered` and `CursorLeft`.
- Winit file drag events become ABI file drag/drop events. Bevy's checked-in `dev/bevy/crates/bevy_winit/src/state.rs` maps `WindowEvent::DroppedFile`, `HoveredFile`, and `HoveredFileCancelled` into `bevy_window::FileDragAndDrop`; Zircon's current winit 0.31 beta dependency names the same host capability `DragEntered`, `DragDropped`, and `DragLeft`. Zircon maps entered paths to `file_hovered`, dropped paths to `file_dropped`, and drag-left/cancelled to `file_drag_cancelled`.
- Winit window status events become ABI window status events. Bevy defines `WindowMoved`, `WindowOccluded`, `WindowThemeChanged`, `WindowBackendScaleFactorChanged`, `WindowScaleFactorChanged`, `WindowCloseRequested`, and `WindowDestroyed` in `dev/bevy/crates/bevy_window/src/event.rs`, then forwards host events from `dev/bevy/crates/bevy_winit/src/state.rs`. Zircon's local winit 0.31 beta names those host inputs `Moved`, `Occluded`, `ThemeChanged`, `ScaleFactorChanged`, `CloseRequested`, and `Destroyed`, and maps them to `ZrRuntimeEventV1::window_moved`, `window_occluded`, `window_theme_changed`, `window_backend_scale_factor_changed`, `window_scale_factor_changed`, `window_close_requested`, and `window_destroyed`.
- `WindowEvent::PointerButton` remains a mouse button for mouse-like sources, but touch button press/release becomes ABI touch started/ended.
- `WindowEvent::PointerLeft` for touch becomes ABI touch cancelled so runtime state can clear active touches when the platform cancels tracking without a release.
- `WindowEvent::KeyboardInput` becomes ABI keyboard pressed/released with a deterministic physical key code and optional text payload.
- `WindowEvent::MouseWheel` becomes ABI mouse-wheel data with x/y deltas and a Line/Pixel unit. When the preview host has a retained pointer position it emits `mouse_wheel_delta_at(...)` so UI-facing adapters can route by cursor point, and falls back to `mouse_wheel_delta(...)` when no point is known. This mirrors Bevy's `dev/bevy/crates/bevy_winit/src/state.rs` mapping of winit `MouseScrollDelta::LineDelta(x, y)` and `PixelDelta(p)` into `bevy_input::mouse::MouseWheel`.
- `WindowEvent::Ime` becomes ABI IME enabled/disabled/preedit/commit/delete-surrounding events. This follows Bevy's `bevy_winit` path, which forwards winit IME events into `bevy_window::Ime` messages instead of folding composition into keyboard input. Zircon keeps winit `DeleteSurrounding` as a neutral runtime event even though Bevy's checked-in window event enum does not currently expose that variant.
- `WindowEvent::Focused(false)` becomes ABI lifecycle background, which the runtime uses to clear keyboard state.
- `DeviceEvent::PointerMotion` becomes ABI raw mouse motion. This mirrors Bevy's `bevy_winit` path, where winit device motion is forwarded as `bevy_input::mouse::MouseMotion` instead of being treated as cursor position. Bevy's checked-in source still names the winit variant `DeviceEvent::MouseMotion`; Zircon's current winit 0.31 beta dependency exposes the same raw delta as `PointerMotion`.

`zircon_runtime::dynamic_api::session` consumes those ABI events into the richer runtime input state:

- Touch ABI events submit `InputEvent::Touch` and still drive the preview camera controller directly, without pretending touch is a mouse button in `InputFrameSnapshot`.
- Cursor entered/left ABI events submit `InputEvent::CursorEntered` and `InputEvent::CursorLeft`, updating `InputFrameSnapshot::cursor_inside_window` without changing the last known cursor position.
- File drag/drop ABI events submit `InputEvent::FileDragDrop`, appending `FileDragDropEvent` values to `InputFrameSnapshot::file_drag_drop_events` for the current frame.
- Window status ABI events submit `InputEvent::WindowStatus`, appending `WindowStatusEvent` values to `InputFrameSnapshot::window_status_events` for the current frame.
- Keyboard ABI events submit `InputEvent::KeyboardInput` so `DefaultInputManager` owns physical key state, text payload, and frame transitions. Text is not used as a logical key identity because text is usually absent on release events.
- IME ABI events submit `InputEvent::Ime` so composition and delete-surrounding requests are available to text widgets without being confused with physical key presses.
- Outgoing IME host requests are drained through `ZrRuntimeApiV2::drain_host_requests` as a JSON `ZrRuntimeHostRequestBatchV1`. `zircon_app::entry::runtime_library::RuntimeSession` decodes the batch, and the native preview host applies enable, disable, cursor-area, and surrounding-text requests to winit `Window::request_ime_update`. This follows Bevy's window-owned `ime_enabled` / `ime_position` configuration surface in `dev/bevy/crates/bevy_window/src/window.rs`, while using the richer local winit 0.31 `ImeRequest`, `ImeCapabilities`, `ImeRequestData`, and `ImeSurroundingText` API for native host application.
- Outgoing gamepad rumble requests are drained through the same optional host-request API as `ZrRuntimeHostRequestV1::GamepadRumble`. On desktop `gamepad-gilrs`, the native preview host now maps requests to gilrs force-feedback effects (`Strong`/`Weak`) and tracks active effect lifetimes so `Stop` requests, gamepad disconnects, and app shutdown clear playback handles deterministically. Missing gamepads, disconnected pads, unsupported force-feedback capability, and gilrs force-feedback channel failures are reported as host warnings; the ABI and runtime queue contract remain unchanged.
- Background and suspended lifecycle states submit `InputEvent::FocusLost`; low-memory notifications preserve input state.
- Mouse-motion ABI events submit `InputEvent::MouseMotion`, which is accumulated into `InputFrameSnapshot::mouse_motion_accumulator` and reset by `begin_frame()`. This follows Bevy's split between raw `MouseMotion` events and frame-local `AccumulatedMouseMotion`.
- Mouse-wheel ABI events with a Line/Pixel unit submit `InputEvent::MouseWheel`; legacy unit-less ABI events still submit `WheelScrolled` so older hosts keep working. Coordinate-present wheel events decode their real x/y deltas from `key_code`/`scan_code` bits and ignore `x`/`y` for input-state reduction because those fields carry the cursor point for UI hit routing. The dynamic session validates wheel x/y values as finite before appending precise current-frame wheel state.

## Raw Mouse Motion

M5 adds the first device-event input path. The runtime preview app now implements `ApplicationHandler::device_event` and forwards only raw pointer motion through `ZrRuntimeEventV1::mouse_motion`. Other device events remain ignored until the runtime has an explicit neutral contract for them.

This is intentionally separate from `WindowEvent::PointerMoved`: pointer movement reports logical cursor position, while device mouse motion reports raw physical delta. The distinction is the same one documented in `dev/bevy/crates/bevy_input/src/mouse.rs`, and it keeps future pointer-lock/high-precision camera controls from depending on cursor coordinates.

## Mouse Wheel

M14 adds Bevy-style wheel unit fidelity. Bevy defines `MouseScrollUnit`, `MouseWheel { unit, x, y }`, and `AccumulatedMouseScroll { unit, delta }` in `dev/bevy/crates/bevy_input/src/mouse.rs`, then forwards winit `LineDelta(x, y)` and `PixelDelta(p)` without dropping the horizontal axis in `dev/bevy/crates/bevy_winit/src/state.rs`.

Zircon mirrors that in the runtime input contract with `MouseScrollUnit` and `MouseWheelEvent`. The ABI keeps `ZrRuntimeEventV1::mouse_wheel(delta)` as vertical line compatibility, keeps `mouse_wheel_delta(unit, x, y)` as delta-only host input, and adds `mouse_wheel_delta_at(unit, point_x, point_y, delta_x, delta_y)` for Slate-style UI hit routing. `zircon_app` caches the last pointer position from pointer move/button events, uses `mouse_wheel_delta_at(...)` when known, and falls back to delta-only otherwise. Runtime input-state reduction recognizes the coordinate-present flag and decodes delta from `key_code`/`scan_code` bits so dynamic preview scroll/camera behavior continues using real wheel delta, not cursor coords. Runtime systems that still read `InputSnapshot::wheel_accumulator` get the same scalar view, while systems that read `InputFrameSnapshot` can inspect raw x/y deltas and the last frame unit.

Runtime 15 M2 input mouse-wheel line-delta naming hard cutover records `runtime_15_input_mouse_wheel_line_delta_naming_hard_cutover_static_passed_cargo_deferred`: the scalar compatibility view is now named as line-delta conversion, not as a legacy path. `zircon_runtime/src/core/framework/input/mouse_wheel.rs` owns `PIXEL_SCROLL_LINE_DELTA_SCALE`, the single pixel-to-line scale used by `MouseWheelEvent::vertical_line_delta()`, and both `DefaultInputManager` plus the dynamic preview session call that helper when feeding `wheel_accumulator` or camera scroll. `runtime_15_input_mouse_wheel_line_delta_uses_current_names` keeps this naming hard cutover guarded. No old helper alias is kept; precise `MouseWheelEvent { unit, x, y }` remains the richer runtime input source.

## Cursor Boundary

M8 adds the Bevy-style cursor boundary path. Bevy registers `CursorEntered` and `CursorLeft` messages in `dev/bevy/crates/bevy_window/src/lib.rs`, defines the event payloads in `dev/bevy/crates/bevy_window/src/event.rs`, and Zircon forwards current winit `WindowEvent::PointerEntered` / `PointerLeft` into the same cursor-boundary ABI concepts. Zircon mirrors that as a neutral bool in `InputFrameSnapshot::cursor_inside_window` plus two ABI constructors, `ZrRuntimeEventV1::cursor_entered` and `ZrRuntimeEventV1::cursor_left`.

## Cursor Host Requests

Runtime 12 adds the cursor-control request family with `CursorGrabMode`, `CursorPosition`, and `CursorHostRequest`. This mirrors the direction of the IME and gamepad rumble host-request lanes: runtime code records intent as current-frame data, `RuntimeDynamicSession::drain_host_requests()` serializes it as `ZrRuntimeHostRequestV1::Cursor`, and the native runtime-preview host applies it to the winit window. The host maps visible requests to `set_cursor_visible`, grab requests to `set_cursor_grab` with confined/locked fallback, hit-test requests to `set_cursor_hittest`, and finite position requests to `set_cursor_position`; missing or non-finite cursor payloads are logged and ignored. The behavior anchor is `cursor_host_requests_are_frame_local_and_drainable`, and the current status anchor is `cursor_host_request_static_passed_cargo_deferred`.

## File Drag And Drop

M9 adds the Bevy-style file drag/drop path. Bevy defines `bevy_window::FileDragAndDrop` in `dev/bevy/crates/bevy_window/src/event.rs` with dropped, hovered, and cancelled variants, registers it as a window event, and forwards host events from `dev/bevy/crates/bevy_winit/src/state.rs`. Zircon mirrors that with `FileDragDropEvent` in the runtime input contract and `ZrRuntimeEventV1::file_hovered`, `file_dropped`, and `file_drag_cancelled` in the stable runtime ABI.

The runtime preview host intentionally adapts to the local winit 0.31 beta API: `DragEntered { paths, .. }` emits one hovered event per path, `DragDropped { paths, .. }` emits one dropped event per path, and `DragLeft { .. }` emits a cancel event. `DragMoved` currently has only position in this winit version, so it is ignored until Zircon needs a drag-position contract. Runtime systems read the current-frame events from `InputFrameSnapshot::file_drag_drop_events`.

## Window Status Events

M11 adds the first non-input window status event family. Bevy keeps these in `bevy_window` instead of `bevy_input`: `WindowCloseRequested`, `WindowDestroyed`, `WindowOccluded`, `WindowMoved`, and `WindowThemeChanged` are defined in `dev/bevy/crates/bevy_window/src/event.rs` and forwarded from `dev/bevy/crates/bevy_winit/src/state.rs`.

Zircon mirrors the same host/runtime split with `WindowStatusEvent` and ABI constructors on `ZrRuntimeEventV1`. The runtime preview host forwards local winit `CloseRequested`, `Destroyed`, `Moved`, `Occluded`, and `ThemeChanged` into the dynamic runtime session, which reduces them into current-frame `InputFrameSnapshot::window_status_events`.

M12 adds Bevy's dedicated scale-factor event split. Bevy declares both `WindowBackendScaleFactorChanged` and `WindowScaleFactorChanged` in `dev/bevy/crates/bevy_window/src/event.rs`, and `dev/bevy/crates/bevy_winit/src/state.rs::react_to_scale_factor_change` always emits the backend notification while emitting the logical notification only when Bevy's window-resolution override policy allows it. Zircon does not yet expose a runtime scale-factor override setting, so the preview host forwards local winit `WindowEvent::ScaleFactorChanged` to both ABI constructors. The dynamic session validates that the scale factor is finite and positive before appending `WindowStatusEvent::BackendScaleFactorChanged` and `WindowStatusEvent::ScaleFactorChanged`. `ZrRuntimeViewportMetricsV1` and resize events still own actual surface size and framebuffer metrics; these status events are for systems that need to observe scale changes separately from resize state.

## IME Composition

M6 adds the Bevy-style IME window message path. Bevy defines `bevy_window::Ime` in `dev/bevy/crates/bevy_window/src/event.rs` with `Preedit`, `Commit`, `Enabled`, and `Disabled`, and `dev/bevy/crates/bevy_winit/src/state.rs` maps `WindowEvent::Ime` into those messages. Zircon mirrors that split with `ImeEvent` and `ImePreedit` in the runtime input contracts, while the preview host translates through `ZrRuntimeEventV1` instead of importing runtime implementation types.

M7 adds Zircon's explicit `ImeEvent::DeleteSurrounding` path for winit `Ime::DeleteSurrounding`. The app ABI uses `ZrRuntimeEventV1::ime_delete_surrounding` with `key_code` as `before_bytes` and `scan_code` as `after_bytes`, then `zircon_runtime::dynamic_api::session` reduces it into the runtime input manager. This is intentionally a request visible in the current frame, not a text mutation, because the text buffer owner lives in the UI/text-editing layer.

M13 adds the neutral outgoing host-request side for native IME control. Bevy keeps IME activation and candidate placement on `Window::ime_enabled` and `Window::ime_position` in `dev/bevy/crates/bevy_window/src/window.rs`, while winit 0.31 exposes richer request data for cursor area and surrounding text. Zircon mirrors that direction with `ImeHostRequest`: enable, disable, cursor area, and UTF-8 surrounding text are current-frame requests stored in `InputFrameSnapshot::ime_host_requests`.

The runtime ABI carries these requests on the IME event family with `ime_request_enable`, `ime_request_disable`, `ime_cursor_area`, and `ime_surrounding_text`. `zircon_runtime::dynamic_api::session` validates cursor areas as finite positive rectangles and validates surrounding-text cursor/anchor offsets as UTF-8 byte boundaries before submitting `InputEvent::ImeHostRequest`. The UI dispatch contract still owns higher-level widget intent as `UiInputMethodRequest`; this runtime input lane is the lower transport contract that the native host consumes through winit's IME request API.

M15 closes the native desktop preview loop with `ZrRuntimeApiV2::drain_host_requests`. The runtime drains outgoing IME requests into `ZrRuntimeHostRequestBatchV1` JSON, owns the returned byte buffer with the same free-callback pattern as frame/profile outputs, and leaves normal input events untouched. `zircon_app::entry::runtime_library::RuntimeSession` decodes the batch when the V2 capability is present, validates the DTO ABI version, and frees the buffer; it does not load an older runtime table. `RuntimeEntryApp::about_to_wait` applies the drained requests to the current winit window using `ImeRequest::Enable`, `ImeRequest::Update`, `ImeRequest::Disable`, `ImeCapabilities`, `ImeRequestData`, and `ImeSurroundingText`. This keeps Zircon aligned with Bevy's window-owned IME policy while using the richer local winit 0.31 API shape for cursor-area and surrounding-text updates.

The 2026-06-30 IM-M1 app host-request pump guard adds `runtime_entry_source_guards/host_requests.rs`. It locks the source order `tick_frame` -> `apply_runtime_host_requests` -> redraw, the drain owner loop over every `RuntimeSession::drain_host_requests()` item, routing of `ZrRuntimeHostRequestV1::Ime` to `apply_runtime_ime_host_request`, and the IME leaf mapping from `SetCursorArea` / `SetSurroundingText` to winit `ImeRequest::Update`. Scoped rustfmt and diff-check pass with only LF/CRLF warnings; focused `zircon_app` package execution timed out after 904s with no Rust diagnostics, so it is not counted as passing.

## Gilrs Runtime Preview Host Backend

M4 adds the first native gamepad backend in the runtime preview host. This follows Bevy's split where `bevy_gilrs::GilrsPlugin` owns gilrs startup/polling, then feeds neutral Bevy input events through `bevy_input::gamepad`.

`zircon_app::entry::runtime_entry_app::gamepad` is compiled behind `gamepad-gilrs`. It initializes `gilrs::Gilrs` with default filters disabled and manual state updates, matching Bevy's `GilrsBuilder::with_default_filters(false).set_update_state(false)` shape in `dev/bevy/crates/bevy_gilrs/src/lib.rs`. Existing connected gamepads are announced before polling, matching Bevy's startup connection pass in `gilrs_event_startup_system`.

Each winit wait cycle polls gilrs and translates events through the ABI instead of importing runtime input state into the app:

- `Connected` and `Disconnected` become `ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1`, carrying gamepad id, name, vendor id, and product id.
- `ButtonPressed`, `ButtonRepeated`, `ButtonReleased`, and `ButtonChanged` become `ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1`, carrying stable Zircon button codes and the analog value. `ButtonChanged` forwards the raw analog value without an app-side `value >= 0.5` threshold; runtime input state applies the Bevy-style button axis and digital hysteresis settings described above.
- `AxisChanged` becomes `ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1`, carrying stable Zircon axis codes and value.

`zircon_runtime::dynamic_api::session` keeps the ABI session entry, while `zircon_runtime/src/dynamic_api/session/events.rs` reduces those ABI events into `InputEvent::GamepadConnection`, `InputEvent::GamepadButton`, and `InputEvent::GamepadAxis`. The runtime keeps Bevy-style durable gamepad state in `InputFrameSnapshot`: connected gamepads, pressed gamepad buttons, per-frame transitions, processed analog button values, processed latest axis values, and current-frame `GamepadAxisTransition` edges. Disconnect still clears that gamepad's axes, analog button values, and pressed buttons, and produces zero-value axis transition edges for previously non-zero axes.

Current intentional gaps are browser Gamepad API support, additional non-mouse device events, and editor/native host convergence. Browser gamepad must remain a separate backend instead of being treated as a gilrs alias.

## 2026-07-10 current guard-owner validation

Plan sources are Runtime09, Runtime12, Runtime15, and Plan09 numbered output records. The current guard implementation files are `tests/runtime_absorption/input_stack/**`, `tests/runtime_absorption/naming_boundary/runtime_15_m2/{input,ui}/**`, `tests/runtime_absorption/ui_architecture/legacy_renames.rs`, and the scene-world production-budget guard. No input production behavior changed in this reconciliation.

The dated 2026-07-10 structure audit reported runtime/framework/test/guard counts 12/20/7/6, with empty missing/risk lists. Its input-stack 11/11, input naming 3/3, Runtime09 route/name 11/11, and scene-world visibility 1/1 results remain historical evidence only; current inventory and validation are recorded in the unique current block above and the Runtime 12 numbered output.

## 2026-08-27 Dynamic Input Adapter Ownership

The ABI event route is now folder-backed by input domain. `events/keyboard_ime.rs` owns keyboard and
IME payload conversion; `events/gamepad.rs` owns gamepad conversion plus UI navigation/analog mapping;
`events.rs` retains pointer/window/lifecycle coordination and the shared dispatch methods. This is a
physical owner split only: the same session input manager and UI surface receive the same current
events in the same order, and no queue, action-map, coalescing, threshold, UTF-8, or payload-budget
policy changed. Status:
`runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_static_passed_cargo_deferred`.
