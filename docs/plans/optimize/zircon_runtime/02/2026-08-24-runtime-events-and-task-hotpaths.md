---
title: Runtime02 Event and Task Hot-path Optimization
category: zircon_runtime
report_id: Runtime02-hotpaths-2026-08-24
date: 2026-08-24
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime02 Event and Task Hot-path Optimization

## Scope

This slice closes four bounded hot-path defects from the Runtime02 review without claiming the
parent execution-runtime and dynamic-library shutdown architecture is complete:

- concurrent event topic lookup no longer takes an exclusive registry lock;
- bulk disconnected-subscriber pruning no longer performs linear membership scans for every
  subscriber;
- empty and single-chunk `parallel_for` calls bypass unnecessary parallel iterator setup;
- task-pool reports now conserve and expose the physical worker count after per-pool minimums.

The separate subscriber teardown lock-scope work remains outside this integration candidate because
its source and regression owner belong to another active Session.

## Implementation

`EventBusState` now uses `RwLock<HashMap<...>>`: subscription and empty-topic removal retain the
write path, while publish lookup and diagnostics use concurrent reads. Bulk prune sorts one private
copy of the disconnected IDs and uses binary search, preserving the existing borrowed-slice API.

`parallel_for` returns immediately for empty input and directly installs one task when the input
fits one chunk. Multi-chunk input retains Rayon `par_chunks_mut` behavior.

Task-pool assignment first resolves the requested total, then raises it when the sum of the three
pool minimums requires more physical workers. Each greedy assignment is bounded by the remaining
future minimum and maximum capacity. The reported total therefore equals the sum of created I/O,
async-compute, and compute workers. Thread-budget tests were extracted to a dedicated module so the
parent test file remains below the repository size gate.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Parallel topic registry lookup, 8 x 50,000 | 400,000 exclusive acquisitions | 0 exclusive acquisitions; >= 250,000 lookup/s | 100% exclusive-lock reduction |
| Bulk prune, 4,096 subscribers | 8,390,656 membership probes | <= 53,248 probes | >= 99.3654% probe reduction |
| Empty `parallel_for`, 100,000 calls | 100,000 pool installs | 0 pool installs; <= 250 ms | 100% install reduction |
| Single-chunk `parallel_for`, 25,000 calls | 25,000 parallel iterators | 0 parallel iterators; <= 2 s | 100% iterator reduction |
| Default pools on 1/2-thread hosts | reported 1/2, created 3 | reported 3, created 3 | reporting error 200%/50% -> 0% |

Elapsed time and throughput in this table are accepted only from the Windows-native release
evidence ticket. Source-derived operation counts are deterministic but do not replace managed test
execution.

## Validation

- Exact `rustfmt --check` and scoped `git diff --check`: passed.
- Runtime behavior and release evidence batch: pending coordinator terminal evidence.
- Initial corrected-ownership ticket `5971174e38794e1087c67b7580b3f7ad` failed before Cargo because
  `.cargo` was requested as a dependency root but did not exist in pinned HEAD `858350a...`.
- PowerShell batch ticket `9f14f9084f6e46549c0a4c01d6328c72` materialized the repository overlay but
  failed before compilation because non-Cargo validation templates do not auto-discover the
  sibling `zr_vm` Git source.
- Replacement ticket `c17c7c2a95e743599ba8e360ec83247b` uses one direct release Cargo
  invocation with the `runtime02_` filter and `--include-ignored`. This preserves the complete
  Runtime02 regression/performance batch, enables coordinator-owned external source discovery, and
  avoids six repeated compilation launches. Terminal evidence remains pending and is not inferred
  from the queued receipt.

## Remaining Parent-plan Work

This slice does not close process/session task ownership, timer lifetime, typed task results,
bounded lossless event delivery, or dynamic-library unload safety. Those Runtime02 P0/P1 items
remain open and must not be inferred from this hot-path acceptance record.
