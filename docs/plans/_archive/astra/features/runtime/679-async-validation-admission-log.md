---
doc_type: validation-handoff
status: admission_deferred_owner_scope
session_id: 01a08220-bfec-7dd3-84d6-e7a59e1929b7
request_id: astra-runtime-text-batch-20260911-01
cargo_request_id: astra-runtime-text-batch-20260911-02
minimal_cargo_request_id: astra-runtime-text-core-min-cargo-20260911-04
static_request_id: astra-runtime-text-static-batch-20260911-03
ticket_id: 7684931d60fb4a02bcc7fb5c0eb61879
---

# Astra Runtime/Editor Asynchronous Validation Admission Log

## Submission

2026-09-11 first attempted one batched managed Cargo submission with the
current registered Session (`astra-runtime-text-batch-20260911-01`). The
request used the immutable Runtime text manifest and the locked command
`cargo test -p zircon_runtime --lib --locked`; it was rejected before ticket
creation by the ownership gate. After acquiring the six scoped leases and
recording attribution, a second Cargo submission
(`astra-runtime-text-batch-20260911-02`) was rejected by the external
`E:\\Git\\zr_vm` dirty-worktree gate.

To preserve a genuine non-blocking receipt for the work that can run without
that external Cargo closure, a third submission
(`astra-runtime-text-static-batch-20260911-03`) was accepted as ticket
`7684931d60fb4a02bcc7fb5c0eb61879`, status `queued`, with source manifest hash
`d45924f22858e4315ef80c881d047d4cd4feb2c977492ae31211a22324668148`. Its
command is the two-module Python Runtime text contract batch. The receipt was
recorded and the session continued without querying ticket status.

The six temporary admission leases were released immediately after the
submission, leaving the queued ticket as the only coordinator-side artifact.

## Admission result

The first response returned `validation_copy_overlay_not_owned` for all six
submitted paths. Lease and attribution then succeeded for the registered
Runtime81 text scope, but the second Cargo response returned
`validation_ticket_external_worktree_dirty` for `E:\\Git\\zr_vm`. The third
non-Cargo response is the only queued ticket in this log. Its eventual result
is intentionally left for the coordinator wakeup; this session does not poll
or continuously monitor it.

A minimal `core-min,text` Cargo retry (`astra-runtime-text-core-min-cargo-20260911-04`)
hit the same external dirty-worktree gate and was also rejected before ticket
creation. No further Cargo retries are planned in this turn.

## Local evidence carried forward

- The latest focused pointer/navigation/input/hit-query batch passed `132/132`
  in `0.143s`; the latest parallel Runtime + Editor performance-contract batch
  passed Runtime `1146/1146` in `4.114s` and Editor `581/581` in `1.051s`
  (`1727/1727` aggregate). A Runtime UI multi-contract batch subsequently
  passed `213/213` in `0.371s` after the control-index directory, hit-grid,
  test-import/API, and test-helper compile repairs; the related Runtime Text
  contract batch passed `143/143` in `1.159s`. The Runtime batch includes the
  repaired benchmark fixtures in records 690–691 and the accessibility test
  contracts in record 692.
  The latest UI DTO/assertion batch (record 693) also passed Runtime
  `1146/1146` in `5.074s`, Editor `581/581` in `0.954s`, and Runtime Text
  `143/143` in `0.971s`.
- Scoped Rustfmt and `git diff --check` passed for the touched Runtime/Editor
  production modules (Git emitted only existing line-ending notices).
- The final production-source Rustfmt check used `skip_children=true`. A
  recursive check additionally observed only a concurrent formatting diff in
  `zircon_editor/src/ui/control/service/activity_registry_hash_tests.rs`; it
  was not rewritten, and the parse-only batch still parsed it successfully.
- A local parse-only Rustfmt pass successfully parsed all `3,563` changed Rust
  files selected by the batch. This is syntax evidence only; it is not a Cargo
  compile, Rust test, or product-performance result.
- Runtime records 665–693 and Editor records 661–666 remain
  `implemented_pending_validation`; none claims managed Cargo or product
  p50/p95/p99 evidence.
- Runtime records 697–702 add layout-slot, segmented-submit, layout-report,
  Runtime200 route/hover, text-wrapping, and residual UI test-API repairs.
  Their batched local suites passed, but they likewise remain
  `implemented_pending_validation` pending owner-attributed managed validation.
- Runtime record 703 adds the remaining render-graph alias, segmented UI-plan,
  SDF report, vertical-text, and visibility benchmark test-API repairs. Its
  local evidence is included in the same batched suites; it likewise remains
  `implemented_pending_validation` pending owner-attributed managed validation.
- The latest batched local runs after those repairs passed Runtime `1146/1146`
  in `6.548s`, Editor `581/581` in `1.741s`, and Runtime Text `143/143` in
  `1.191s` (`1870/1870` aggregate); the focused Runtime set passed `58/58` in
  `0.606s`. These are contract-suite timings, not product p50/p95/p99
  measurements.
- The batched Runtime/Editor pressure-model suites passed `154/154` and
  `129/129` (`283/283` aggregate; `4.650s` and `4.149s`); these deterministic model checks likewise
  do not substitute for the pending Windows Release product metrics.

## Next owner action

An owner-attributed multi-task ticket must be submitted after the relevant
Session scope is available. That future ticket should combine the focused
Runtime navigation/input/layout regressions with the Editor chrome/asset
projection regressions and collect Windows Release allocation/time evidence.
Until then this log is a handoff, not a passed-validation receipt.
