---
title: Runtime82 Physical Key Edit Command Routing
category: zircon_runtime
report_id: Runtime82-physical-key-edit-command-routing-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: product_gate_pending
---

# Runtime82 physical key edit command routing

## Failure and repair

The shared SelectAll predicate accepted an empty logical key. For a pressed
Ctrl+Left event with `logical_key = ""` and `key_code = 37`, the logical branch
therefore returned selection `0..text.len()` before the physical fallback could
move to the previous word. Ctrl+Backspace and other physical-only primary
commands took the same incorrect branch. Public `UiKeyboardInputEvent` permits
this representation; the dispatcher neither rejects nor supplies a logical key.

The logical SelectAll match now accepts only `a` or `A`. The existing physical
`65 | 97` branch explicitly supplies `a` after checking the key code. All other
physical codes continue through the existing navigation/deletion mapping.
Modifier guards, repeated events, secure line navigation, document transactions,
and history ownership are unchanged. The prior CR-UI-SURFACE-0005 comment is
replaced with the repaired routing invariant; unrelated preexisting comments
are preserved.

## Reference and regression evidence

Local Unreal `Slate/Private/Framework/Commands/GenericCommands.cpp:25` binds
SelectAll to Ctrl+A. `Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp`
handles Left at lines 1094-1113 and Ctrl+Backspace at 1276-1295 as their own
commands, including the secure-text line policy. Zircon retains its current
physical-code mapping; the reference supports keeping command identity distinct
from the presence or absence of a logical string.

The new `runtime82_physical_key_edit_commands` integration target contains eight
real `UiSurface::dispatch_input_event_with_manager` regressions:

- Ctrl+Left/Right, including repeated events, move to a word boundary without
  selecting the whole document.
- Ctrl+Shift+Left extends the selection only to that word boundary.
- Super+Left/Right retains line navigation.
- Ctrl+Backspace/Delete commits exactly one `6..10` deletion in `alpha beta`,
  then Undo restores the text with the same document identity and next revision.
- Logical `a`/`A`, physical `65`/`97`, and Super+A still select the whole body.
- An unknown physical primary key leaves text, caret, and selection unchanged.
- Secure Ctrl+Left/Backspace retains line navigation/deletion and redaction.
- A released physical command changes neither text nor selection.

Tests were prepared before the production repair. No Cargo run was performed
during this slice; execution belongs to the next grouped managed Windows batch
with `--features ui --test runtime82_physical_key_edit_commands`. Static control
flow proves that the old blank-key predicate takes SelectAll in these cases;
this is not a claimed failing or passing test execution.

The change adds no scan or allocation: command classification remains constant
work. It repairs a correctness prerequisite for Runtime82 acceptance and makes
no latency improvement claim. Native edit-to-present, RSS/allocation, and
matched Unreal performance gates remain open.

## Source ownership

The four exact source, test, and record paths are outside the sealed Batch M
manifest. Their preimage bytes and Git diffs are retained under
`.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-n-runtime82-physical-key-preimage.json`.
The owning Session is `astra-optimize-20260926-batch-a`; exact-path lease
receipt is `9a5b77a4a28b4a4ba2c81bc12a02d5fb`. The optimize and Astra record
maintenance authorizations are `24eea766d6f04257a2a482ae194ce289` and
`27a47f89cff2444fb06bf5c6882ee2a5`. Original failed requests and successful
bounded recovery receipts remain beside the preimage record.

Both Rust files pass scoped rustfmt checks. Reversing only the two command
predicate edits and repaired invariant comment exactly restores the original
source bytes, including all unrelated preexisting changes.
