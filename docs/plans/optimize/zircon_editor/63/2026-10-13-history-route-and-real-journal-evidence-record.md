---
title: Editor63 History Route and Real Journal Evidence
category: zircon_editor
report_id: Editor63-history-route-real-journal-2026-10-13
date: 2026-10-13
implementation_status: source_reviewed
validation_status: managed_validation_pending
---

# Editor63 History Route and Real Journal Evidence

## Scope

Retained undo and redo entries are tied to the world route under which they were recorded. A new transaction now compares the captured or inherited route with the retained history route before activation, transaction allocation, or command execution. Nested transactions keep their parent's route; a different route can start after the history is cleared or when its store is empty.

The performance work keeps the original flat V1 receipt contract and adds a separate V2 measurement over the real bounded dirty journal. V2 measures materialization only. It does not measure route admission or a sequential journal lifecycle.

## Source changes

- `zircon_editor/src/core/editing/engine/transaction/scope.rs` rejects mixed retained routes before activation.
- `zircon_editor/src/core/editing/engine/observation/decision/tests/cases.rs` adds activation counting and regressions for replacement-route rejection, same-route nesting, and replacement after clear or on an empty store.
- `zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs` moves test-only visit accounting out of the per-entry loop and counts the actual retained range after traversal.
- `zircon_editor/src/core/editing/engine/transaction/tests/dirty_batch_perf.rs` preserves V1 and adds the separate V2 producer and quantile regression.

## Performance contracts

V1 retains its historical flat `u64` model: 65,536 inputs, 8,192 globally unique values, and 8,192 sorted outputs per sample. It uses 17 samples, the legacy floor estimator `floor((n-1)*95/100)` (index 15 for 17 samples), and the original `optimized_estimate * 100 <= legacy_estimate * 60` assertion. V1 receipts retain their original meaning.

V2 prepares 16 independent real journal windows of 4,096 changes each. The generated input has 4,096 unique histories within each window and 8,192 globally; each sample produces 65,536 sorted values in total. The baseline applies `BTreeSet` to the same `HistoryContextId` values in the same windows. Only materialization is timed; journal setup, recording, expected-result checks, visit checks, and percentile calculation are outside the timers. The marker is `EDITOR63_BOUNDED_DIRTY_JOURNAL_MATERIALIZATION_BENCH_V2`.

V2 uses nearest-rank P95, `ceil(0.95*n)-1` (index 16 for 17 samples), with the independent gate `optimized_p95 * 100 <= legacy_p95 * 60`. Tests cover sample counts 1, 17, 20, and 100. The visit counter accounts for the actual retained range once after each traversal, outside the per-row loop.

Neither benchmark has been run locally. The V2 60% target is an acceptance threshold, not a measured result; V1 and V2 measure different workloads and their timings are not directly comparable.

## Validation and handoff

R3 source review and patch reconciliation passed. Static checks recorded in the evidence are `rustfmt --check --edition 2021 --config skip_children=true` and `git diff --check` on the scoped Rust files. Cargo tests and release benchmarks have not run locally. Functional and performance acceptance remain pending the root's merged Jenkins validation.

The exact R3 change guard passed with zero errors and one warning for the new ignored V2 release benchmark at line 184 (`test.weakened`). The warning requires running that exact benchmark filter during managed validation; the guard does not represent a test or performance pass. The batch08 file handoff was written with 15 command arms, but its JSON, Markdown, and prospective response paths were absent during one file-evidence reconciliation after independent repairs. The cause is unknown; see the [continuation log](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/continuation-log.md). The cumulative follow-up Editor regression arm must match all listed route and quantile cases, and its Release arm must match both ignored producers. Verify the named matches in the shared batch rather than creating a separate build for each filter. Actual queue, native, and performance receipts remain pending.

## Review artifacts

- Cumulative patch: `.codex/outbox/reports/optimize-20261013-01a13451/editor-core-r3.patch` (SHA-256 `8ed5acdc0c64bde0f7a6e78f2996ea813c2bd73d527878ae40d7ef8c12ddbf3c`). Use R3 instead of R1 or R2; the checkout already contains the patch.
- Evidence: `.codex/outbox/reports/optimize-20261013-01a13451/editor-core-r3-evidence.json` (SHA-256 `6bf8dd6b1a63c57cbde61e56e82b2a7a329d42c59c1679da26c6ac080765bad8`).
- Exact change guard: `.codex/outbox/reports/optimize-20261013-01a13451/editor-change-guard-r3.json` (SHA-256 `a4277512ee0e6fdfa6d6ed2d271ca27c0f5117b5a1e42285ee196102365e75be`; passed, 0 errors, 1 warning).

R1 and R2 remain historical evidence. Source hashes are unchanged from R2; R3 corrects the cumulative patch's newline filtering and does not alter source.
