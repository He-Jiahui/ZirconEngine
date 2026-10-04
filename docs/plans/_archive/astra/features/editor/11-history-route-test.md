status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md

# History route and operation-group contract

## Current source and repair

The history hash-index test fixture already modeled logical route capture and
activation but omitted route retirement after `EditContext` made lifecycle
cleanup explicit. It now implements the same no-op retirement used by the
other transaction fixtures, preserving the test's isolated history-store
scope while satisfying the shared contract.

`EditorTransactionEngine::execute_operation` also contained a stale call to
the never-defined `scope::ensure_single_gateway_history`. Route capture and
activation are already owned by `begin_transaction`, including
`HistoryContextId::PlaySession`; keeping the stale call made the transaction
module fail name resolution before that shared route authority could run. The
operation-group regression now executes a grouped Play-history command and
asserts that the retained context is activated for the requested Play world.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M28 | Complete the history hash-index `EditContext` fixture lifecycle | implemented_pending_validation | Static diff check pending; combined Editor batch will validate with M27 |
| M29 | Remove the stale operation-group route helper call and retain Play-history route coverage | implemented_pending_validation | Scoped `rustfmt` and diff checks are clean; a stable combined Runtime/Editor source-contract batch passed `30/30`. A focused 25-test Editor03 contract ticket is queued, while the wider batch isolated two pre-existing Editor04 text-anchor errors. The managed Editor compile dispatch is blocked before ticket issuance by the dirty external `E:/Git/zr_vm` worktree, so no compile or Release performance result is claimed. |

## Coordinator dispatch log

- Session registration was accepted under
  `astra-editor03-operation-group-compile-repair-20260910`; its initial
  request `79697fd1203943dda7086548ef305b1b` had no terminal response within
  the client timeout, so it was not queried. The later successful lease claim
  proves that the Session was registered.
- Lease request `3dcd61a0d64c410fbec7b92f703308b9` acquired the operation
  source, its regression test, and this record. Attribution request
  `33e127523d714a55bcdef0f08445a31b` recorded their current hashes.
- Compile dispatch request
  `astra-editor03-operation-group-compile-20260910-r1` targeted
  `cargo +1.94.1 test -p zircon_editor --lib --locked
  operation_group_uses_the_play_history_route`. The coordinator rejected it
  during immutable external-source preflight because `E:/Git/zr_vm` is dirty;
  it did not issue a ticket. No coordinator status was queried.
- The non-Cargo Editor03 source-contract batch passed locally as `25/25` and
  was submitted as ticket `4c44a906df934a4e8f27b874a2163dd5` with immutable
  source-manifest hash
  `3d23cbf30e16ed7a067629b69bb6d7f54d751df15360ecb1136ee60db22eb3c6`.
  It is queued; this record intentionally does not treat submission as a
  terminal validation result or as Rust compilation evidence.

This record covers transaction route and test-support contract repair only. It
does not claim the broader Editor compile or performance gates are complete.

## Follow-up: modular controller ownership (2026-09-12)

The Editor04 history-context contract had two source slices that still read
`core/play/controller.rs` after terminal detach and backend retirement moved to
the dedicated `controller/runtime_ownership.rs` module. The test now reads that
owner module for both slices while continuing to read the root controller for
the negative ownership assertions. No production behavior changed.

The pre-change focused contract run was RED with two `IndexError` failures;
after the path-only repair `python -m unittest -q
tools.tests.test_editor04_play_history_context_contract` passed all 18 tests.
`py_compile`, scoped `git diff --check`, and the source diff were also clean
(test hash `d1529a5ed418d150ed71884aa224f2800d3dc36d`). Managed Cargo,
Windows-product, and visual gates remain pending; the external
`E:/Git/zr_vm` worktree is still dirty. Session
`astra-editor04-history-test-route-20260912` remains
`implemented_pending_validation` until those gates can run.
