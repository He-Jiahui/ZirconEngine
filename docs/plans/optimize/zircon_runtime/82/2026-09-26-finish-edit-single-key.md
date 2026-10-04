---
title: Runtime82 Finish Edit Single Key
category: zircon_runtime
report_id: Runtime82-finish-edit-single-key-2026-09-26
date: 2026-09-26
baseline_head: 01a9f062e203e0115c388f9dcabaf00fcc7c9d2c
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime82 finish edit single key

## Scope

`UiTextDocumentSession::finish_edit` used to construct a separate owned
`UiTextDocumentBindingKey` for binding insertion, synchronization-error removal,
and changed-edit history. Because `UiTreeId` owns a `String`, each key
construction cloned and allocated that tree ID. The function now constructs one
key, borrows it for error/history removal, and moves it into the binding map.
Only the changed edit that updates a history entry clones the key once more.
No public contract or document/store authority changes.

## Correctness and performance boundary

- Unchanged commits still refresh the binding and remove a synchronization
  error, while leaving history intact even when the supplied history policy is
  `Barrier`. Changed `Record` commits still append to the owner history; changed
  `Barrier` commits still clear it. Focused tests cover all three cases and
  document revision/source epoch continuity.
- Deterministic tree-ID allocation count in this function falls from two to
  one for unchanged commits, three to one for changed barriers, and three to
  two for changed history updates. This counts key cloning in `finish_edit`,
  not allocations in document editing or the complete input transaction.
- `RUNTIME82_FINISH_EDIT_SINGLE_KEY_BENCH_V1` keeps the exact previous
  `finish_edit` implementation in the test module. It compares old and new
  binding/error/history output before timing a changed barrier on a 150-byte
  tree ID. The Release run uses four warmup pairs, 17 alternating sample pairs,
  and 16,384 calls per sample. The target is optimized P95 at most 90% of the
  old P95 on the same machine and build profile. This is a pending target, not
  an observed result.

## Grouped validation manifest

Coalesce with other Runtime edits in one managed Windows validation lane:

1. `cargo check -p zircon_runtime --lib` once for the grouped package snapshot.
2. Include `cargo test -p zircon_runtime --lib finish_edit_` in the grouped
   focused Runtime test batch; also retain existing document-session/history
   tests in the same batch.
3. Group `cargo test --release -p zircon_runtime --lib finish_edit_single_key_release_p95 -- --ignored --nocapture`
   with other Release performance evidence.
4. `rustfmt --check --edition 2021` on the two source files and scoped
   `git -c core.autocrlf=false diff --check` on the source and plan records.

Rustfmt and scoped whitespace checks passed locally. No Cargo test or Release
measurement was run for this slice. Runtime82's broader retained document,
secure text, product session, and scale qualification gates remain open.
