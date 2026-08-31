---
title: Runtime82 Inline Keyboard Edit Actions
category: zircon_runtime
report_id: Runtime82-inline-keyboard-edit-actions-2026-08-28
date: 2026-08-28
session_id: root-runtime82-two-task-keyboard-performance-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime82 Inline Keyboard Edit Actions

## Scope

This slice advances the allocation budget behind RTE-P1-014 and RTE-GATE-016 for keyboard edit
dispatch. It removes the temporary action-vector allocation from the logical-key and key-code
paths. It does not claim that the wider text document, shaping, layout, clipboard, or IME pipeline
is complete.

## Implementation

Keyboard mappings produce `KeyboardTextEditActions`, a fixed inline sequence containing one
required action and one optional action. Its consuming iterator chains the required action with the
optional second action without allocating. Single-key mappings preserve one action; word deletion
preserves the existing ordered `SetSelection` followed by `Backspace` or `Delete` sequence.

The downstream reducer already accepts `impl IntoIterator<Item = UiTextEditAction>`, so the owner
can change representation without modifying its caller. Three Rust regressions cover single-action
order, two-action word-delete order, and fused exhaustion behavior. The source contract rejects a
return to `Vec<UiTextEditAction>` or `vec!` construction in this owner.

## Performance Evidence

The managed release gate constructs 262,144 mixed production `UiTextEditAction` sequences per
sample, with every eighth sequence containing two actions. It uses 17 alternating legacy/inline
sample pairs after warmup and verifies an identical checksum. The acceptance threshold is zero
sequence-container allocations, identical output, and at least 50% lower P95.

| Metric | Before | After | Change |
| --- | ---: | ---: | ---: |
| Sequence-container allocations | 262,144 | 0 | -100% |
| Sequence-container allocated bytes | 4,718,592 | 0 | -100% |
| P50 per 262,144 sequences | 18,843,900 ns | 986,100 ns | -94.767% |
| P95 per 262,144 sequences | 37,523,200 ns | 1,420,400 ns | -96.215% |

Both independent model implementations retained checksum `34359934976`. A preceding run measured P50
`22,819,600 -> 1,039,200 ns` (-95.446%) and P95 `31,326,600 -> 2,305,100 ns` (-92.642%), with the
same allocation and checksum results. The model uses a compact action enum and therefore does not
overstate payload-byte savings for the production string-bearing enum; the production sequence
container still deterministically changes from one heap allocation to zero per handled key.

## Validation

- Source contract: 4/4 passed after a confirmed 0/4 initial state.
- `runtime82_batch_inline_single_action_preserves_order`,
  `runtime82_batch_inline_two_action_word_delete_preserves_order`, and
  `runtime82_batch_inline_action_iterator_stays_exhausted` lock behavior and iterator exhaustion.
- `runtime82_batch_inline_keyboard_edit_actions_p95` emits the production-enum allocation,
  payload-byte, checksum, P50, and P95 row.
- Exact Rust formatting, Python source contracts, and scoped `git diff --check` are required before
  coordinator submission.
- The managed `runtime82_batch_` release gate seals this work with keyboard payload byte-control
  scanning in one Cargo invocation: two source contracts, seven Rust tests, and two performance
  rows. No local Cargo lane is launched.
- Commit and WeCom publication remain pending independent review and managed validation.

## Remaining Parent-plan Work

Runtime82 still requires its product-scale text workload, storage-residency decision, broader
document/editing integration, and the remaining parent-plan gates. This local action-sequence result
does not close those requirements.
