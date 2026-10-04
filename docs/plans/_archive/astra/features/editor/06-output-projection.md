---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_editor/269-editor-build-export-preset-pipeline-cook-pack-platform-bundle-publishing-resume-determinism-current-working-tree-review.md
  - docs/plans/astra/performance/01-bounded-hotpaths.md
---

# Incremental output projection

## Current source and change

Every wizard StageOutput delta cloned the complete panel snapshot, including
finished stages and their retained stdout/stderr. The consumer now mutates the
current output buffer and event header in place. Full snapshot events still
replace the snapshot. Cancellation precedence, terminal state, diagnostics,
coalesced count, output truncation and event ordering retain the previous behavior.

The owning method and directory stay unchanged. Unreal's
`FOutputLogTextLayoutMarshaller::SubmitPendingMessages` moves pending messages and
appends only the new layout range. Fyrox's `LogPanel::update` consumes individual
messages and bounds retained entries. These support incremental projection;
their UI trees and retention policies are not imported into this data-only owner.

## Validation

A retained before implementation checks exact ViewModel equality across empty,
small, 1k and 10k source history, 1,600 stdout/stderr updates, cancellation and all
terminal states. Pointer checks establish that unrelated retained allocations
survive deltas; they are not a claim of zero total allocation.

Release evidence alternates before/after for 101 samples after eight warmups.
Each sample applies 32 events, with seed/event setup outside timing. History
retention is capped at 512 lines per completed stage, including the 10k input.
Report p50/p95/p99; require no more than 5% small-case p95 regression and at least
20% p95 improvement for 1k/10k history. This does not qualify product frame time.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M19 | Apply wizard live-output deltas without cloning unrelated snapshots | implemented_pending_validation | Static equivalence review and 108-test runtime/editor contract batch pass (including core asset/index/export contracts); deterministic pressure model retained separately; combined M18-M20 managed release data pending because external `E:/Git/zr_vm` is dirty (latest `87c112d27f25e2a10e2c078d61ef638954d0f8eb`, 112 tracked/submodule + 88 untracked = 200) |
