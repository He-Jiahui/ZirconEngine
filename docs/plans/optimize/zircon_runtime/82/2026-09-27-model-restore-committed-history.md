---
title: Runtime82 Model Restore Committed History
category: zircon_runtime
report_id: Runtime82-model-restore-committed-history-2026-09-27
date: 2026-09-27
implementation_status: implemented_pending_validation
validation_status: managed_validation_pending
performance_status: product_gate_pending
---

# Runtime82 model restore committed history

## Failure and repair

After a committed Backspace changes `abcd` to `abc`, Preedit `X` projects
`abcX` while the retained document remains `abc`. ExplicitSetText or
ExplicitLoadText of `abc` takes the model transaction's text-equal branch.
Its ordinary property commit previously advanced the source epoch because
the visible body changed. The returned receipt still identified the old
document. The next synchronization then reopened a new document and lost
the preceding Undo history; reusing the receipt key could also conflict.

The branch already proves that projected text equals committed source. It
now prepares the property transaction without an edit intent and preserves
the committed source epoch. Display invalidation and composition cleanup
still occur. The returned document identity/revision remains usable. Real
text replacement still commits through the existing document transaction,
advances the epoch and revision, and installs its history barrier.

The support contract is shared with the accompanying
[IME source repair](2026-09-27-ime-committed-source-epoch.md). The owner is
`ui/dispatch/input_manager/bound_text_model_updates/transaction.rs`; no new
public API, serialization schema, or comparison of whole document contents
is introduced here.

## Regression and acceptance

Two real manager tests each cover ExplicitSetText and ExplicitLoadText:

- Restore the committed body after Preedit; assert inactive composition,
  unchanged epoch/key/revision, a receipt without a document edit, another
  successful key query, and Undo restoring the earlier `abcd` in that document.
- Replace with different model text; assert a real edit receipt and increased
  epoch/revision, then confirm Undo cannot cross the existing model barrier.

Tests were added before the branch repair. Execution is pending the grouped
managed Windows lib tests; no failing or passing Cargo result is claimed.
This repairs correctness needed by Runtime82 acceptance, with no separate
speedup claim against the incorrect behavior. `RTE-GATE-016/047`, native
edit-to-present, allocation/RSS, and Unreal comparison remain open.

## Source ownership

Four exact paths were acquired without conflict under Session
`astra-optimize-20260926-batch-a`, receipt
`4ceebeb982ab420ca67ae075176fe0a4`. Original source bytes and diffs are saved
in the Batch M bound-model preimage record. Preexisting comments and earlier
inactive-composition assertions are retained. Scoped record authorizations
are `66f0f335bb7b46f49245ac4bf1c3c416` and
`53eb10dcab4d4547a2d542411f3b5b87`.
