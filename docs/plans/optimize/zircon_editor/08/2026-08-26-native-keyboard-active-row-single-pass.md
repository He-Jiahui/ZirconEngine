---
title: Editor08 Native Keyboard Active Row Single Pass
category: zircon_editor
report_id: Editor08-native-keyboard-active-row-single-pass-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor08 Native Keyboard Active Row Single Pass

## Scope

This slice removes repeated visible-row walks when native keyboard routing chooses the active popup
row. Hovered interaction identity remains the highest priority regardless of row position, followed
by the first focused row, the first selected row, and index zero. Dispatch-kind mapping,
virtualization offsets, row ownership, and keyboard commands remain unchanged. It advances Editor08
keyboard/palette routing without claiming completion of command policy, remote automation,
accessibility, or product acceptance.

## Change

- Walk visible popup rows once.
- Return immediately for an exact non-empty interaction identity.
- Record only the first focused and selected rows during the same pass, then apply the legacy
  global focus-before-selection priority after identity matching is exhausted.
- Add no persistent index, allocation, cache, or cross-frame generation state.

## Deterministic Performance Evidence

| 4,096 visible rows, missing identity, selected final row | Before | After |
|---|---:|---:|
| Row visits | 12,288 | 4,096 |
| Full row walks | 3 | 1 |
| Temporary allocations | 0 | 0 |
| Priority/order changes | 0 | 0 |

Deterministic row visits fall by 66.6667%. The ignored release gate runs 17 alternating sample
pairs and emits `EDITOR08_ACTIVE_KEYBOARD_ROW_SINGLE_PASS_BENCH_V1`. Acceptance requires single-pass
selection P95 to be at least 20% below the legacy three-walk implementation. Exact Windows P50/P95
timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826bi_active_keyboard_row_single_pass_preserves_global_priority`
  covers identity, focused, selected, and default precedence.
- `optimization_batch_20260826bi_active_keyboard_row_single_pass_eliminates_repeated_row_walks`
  locks the 12,288-visit model and rejects iterator `position` rescans.
- `optimization_batch_20260826bi_active_keyboard_row_single_pass_p95` reports paired release
  P50/P95 samples and enforces the 20% P95 reduction gate.

## Remaining Parent-plan Work

Editor08 still owns command identity, context policy, keymap conflicts, menu/palette projection,
remote automation, accessibility routing, and product-scale input evidence. This slice only
converges active popup-row selection.
