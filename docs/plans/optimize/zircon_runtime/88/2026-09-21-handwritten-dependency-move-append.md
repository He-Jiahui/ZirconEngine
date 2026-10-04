---
title: Runtime Handwritten Dependency Move Append
category: zircon_runtime
report_id: Runtime864-handwritten-dependency-move-append-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime864 Handwritten Dependency Move Append

## Finding

Runtime88 had already replaced repeated `Vec::contains` scans with one borrowed
`HashSet`, but accepted dependency candidates were still cloned into a second
`Vec<AssetUri>`. The function owns the candidate vector, so each accepted URI's
owned locator strings could be moved directly after classification instead of
cloned and then discarded with the original candidate.

## Optimization

- Retain the existing borrowed membership index over current dependencies and
  the immutable candidate vector.
- Record acceptance in a compact `Vec<bool>` while counting exact additions.
- Drop the borrowed index, reserve the exact accepted count in the destination,
  and move accepted candidates through `into_iter()`.
- Preserve current dependencies, first-seen candidate order, duplicate
  rejection, URI identity, and per-entry ownership.

## TDD and deterministic evidence

The Runtime864 source/model contract was run before implementation and failed
`4/4`: no acceptance mask, move append, exact destination reserve, lower
regression, or Release marker existed. It now passes `4/4`.

For 4,096 distinct candidates, the old indexed path reserved 4,096 full
`AssetUri` slots in an additions vector and cloned 4,096 owned locators. The new
path performs zero candidate `AssetUri` clones and stores a 4,096-bit acceptance
mask before moving candidates. A lower regression verifies stable dedup/order
and that the accepted locator's path allocation retains its original pointer.
The ignored 101-pair Release marker
`RUNTIME864_HANDWRITTEN_DEPENDENCY_MOVE_APPEND_BENCH_V1` reports alternating
p50/p95/p99 samples and requires improved p95 over the retired clone-buffer
implementation.

## Local validation boundary

- Exact-file Rustfmt passes for the production owner and lower regression.
- The combined Runtime864/863, UI visitor, Runtime200/205, and Runtime87 batch
  passes `36/36` in `0.028s`, with zero failures or errors.
- The post-Runtime865 one-process non-Tooling loader passes `4216/4216` tests
  across `993` files in `458.125s`; the current record audit matches `100/100`
  hashes across `19` dated Runtime/Editor records.
- Runtime864 landed after the v6 source snapshot and was submitted with
  Runtime865 in the asynchronous v7 multi-task current-source batch; it was not
  submitted alone and v7 has not been polled.
- Local source/model evidence does not establish allocator counts or asset
  import/product p50/p95/p99 behavior. Tooling production remains deferred.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/registry/dependency_extractors/mod.rs` | `910C3824F1F1B8B4F88573F20408F4C11C911E1B3B735B27F629BB87AF535A2C` |
| `zircon_runtime/src/asset/registry/dependency_extractors/dedup_index_tests.rs` | `4B88EB371F67E1C7E773B414D39ED3C86CFCA44C8790930B3CE051F19573B4F6` |
| `tools/tests/test_runtime864_handwritten_dependency_move_append_performance_contract.py` | `FE5F86C43EB46A7C4973699D0A45EA1315BCCB90FC8781822B6059C21ED3F19B` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the next combined current-source Windows lane compiles Runtime, executes the
lower regression and ignored Release marker, and supplies allocator plus asset
import product p50/p95/p99 evidence. Do not infer product acceptance from the
deterministic allocation model.
