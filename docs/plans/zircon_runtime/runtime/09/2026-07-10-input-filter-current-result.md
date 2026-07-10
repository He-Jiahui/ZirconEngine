# Runtime 09 input filter current result

Date: 2026-07-10

Status: in progress

## Executable baseline

The available default-feature runtime test binary selected 455 `input` tests: 429 passed, 25 failed, 1 ignored, and 6983 were filtered out in 63.78 seconds. This binary predates the current source tree and is evidence for triage, not a fresh acceptance gate.

## Current-source reconciliation

Twelve failures were stale current-owner or numbered-output guards and are now reconciled:

- Runtime12 input-stack owner and inventory guards: covered by current standalone `input_stack` 11/11;
- input naming guards for mouse-wheel line delta, DOM keycode, and winit runtime-input baseline: 3/3;
- scene-world visibility owner: 1/1;
- prepare-input provider guard: 1/1 from the Runtime07 current-source reconciliation;
- Runtime09 navigation/pointer/pointer-capture/route-authority guards: covered by current standalone `legacy_renames` 11/11.

The remaining 13 old-binary failures are UI behavior tests owned by the active UI/Text lanes: text-input validation, shared-input diagnostics, multi-pointer capture, repeated analog route, capture/popup trace, missing-node routing, popup focus, tooltip, MUI word delete, multiline enter, pointer drag, shift pointer selection, and triple-click selection. This runtime reconciliation does not change those production owners or waive their failures.

Status anchor: `runtime_09_input_filter_old_binary_429_passed_25_failed_1_ignored_12_current_guards_reconciled_13_ui_behavior_pending`.
