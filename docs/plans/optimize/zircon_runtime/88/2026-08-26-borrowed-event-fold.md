---
title: Runtime88 Borrowed Asset Event Fold
category: zircon_runtime
report_id: Runtime88-borrowed-asset-event-fold-2026-08-26
date: 2026-08-26
session_id: root-runtime88-borrowed-event-fold-20260826
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime88 borrowed asset event fold

## Scope

- Parent gaps: the allocation and burst-pressure portion of `WATCH88-P1-009`, `WATCH88-P1-010`, `WATCH88-P1-016`, and qualification gate `G44`.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: the batch watcher event fold, its focused Rust tests, the source/performance contract, and this record.
- This slice removes redundant URI clones from an in-memory batch only. Cross-batch algebra, rename lineage, filesystem truth reconciliation, source identity, committed generation publication, and complete Runtime88 qualification remain open.

## Change

- `AssetWatcher::fold_events` now iterates over borrowed events instead of cloning every `AssetWatchEvent` before folding.
- The borrowed fold updates an existing URI entry in place and clones the URI only when a new unique result entry needs ownership.
- Rename still owns the previous URI lineage, and the existing owned `fold_event` remains unchanged for streaming ingress that already transfers event ownership.
- Focused Rust tests cover a 4,096-event repeated Modify batch and preserve the existing rule that Added remains Added after later Modify events.

For 65,536 repeated Modify events targeting one asset, result-ownership URI clones fall from 65,536 to one, a 99.998% structural reduction. `AssetUri` owns a path string and an optional label string, so avoiding event clones also removes real allocator traffic.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime88_borrowed_event_fold_performance_contract -v` initially passed 2/6, failed three guards, and errored on the absent focused Rust tests.
- GREEN: the same command passes 6/6 after the borrowed batch fold and tests are added.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` completed for both owned Rust files.
- The standalone benchmark is compiled with `rustc +1.94.1 -O -C target-cpu=native` after importing the x64 MSVC environment; it does not use Cargo or the shared build lane.

The deterministic Rust model measures 31 alternating legacy/optimized sample pairs over 16,384 repeated Modify events per sample. Its URI model retains the production locator's owned path plus optional label shape, and allocation/reallocation calls are counted by a process-local global allocator.

| Metric | Clone every event | Borrow and clone unique result | Change |
|---|---:|---:|---:|
| P50 | 4.6130 ms | 1.3553 ms | -70.620% |
| P95 | 6.5853 ms | 1.7512 ms | -73.407% |
| allocations / 16,384-event batch | 32,769 | 3 | -99.991% |

These timings isolate in-memory folding for a repeated same-URI Modify burst. They do not claim notify backend, path mapping, filesystem truth, import, generation commit, subscriber delivery, or watch-to-visible latency.

## Async validation

One coordinator batch must run the six Python contracts, two focused Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, and the same optimized Rust performance model.

Acceptance requires both Rust tests and all six source contracts to pass, fold parity to remain exact, allocation reduction to remain at least 99%, and P50/P95 reductions to remain at least 50%. Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include managed P50/P95 and allocation reductions and label them as same-URI watcher-event-fold-only evidence.

## Recovery Batch 2026-08-31

- Ownership transfer apply: `c4f23d73cf8a4ead8b66a7f65c4216a2`.
- The two behavior tests now share the `runtime88_borrowed_event_fold_batch_` filter.
- `tools/runtime88_borrowed_event_fold_model.rs` restores the 16,384-event, 31-pair release model
  and allocation/P50/P95 gates described above.
- Managed batch script: `tools/zircon-validation-runtime88-borrowed-event-fold-batch.ps1`.
- Coordinator ticket: `pending_submission`; terminal managed evidence remains authoritative.
