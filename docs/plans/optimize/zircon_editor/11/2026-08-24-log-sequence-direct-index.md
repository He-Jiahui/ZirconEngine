---
title: Editor11 Log Sequence Direct-index Optimization
category: zircon_editor
report_id: Editor11-log-sequence-direct-index-2026-08-24
date: 2026-08-24
session_id: root-editor11-log-lookup-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor11 Log Sequence Direct-index Optimization

## Scope

This slice closes `E-LOG-P1-26` and acceptance gate 12 for the current bounded in-memory log store:
retained record lookup no longer scans every record from the front of the journal. It does not claim
the parent plan's asynchronous ingress, durable journal, cursor/query, persistence, Console, or
process-wide routing milestones are complete.

## Implementation

The store already assigns contiguous, monotonically increasing sequences, evicts only from the
front, and clears the entire retained window without resetting `next_sequence`. `record(sequence)`
now subtracts the first retained sequence, converts the difference to a checked `usize`, and reads
that `VecDeque` offset directly. A final sequence equality check preserves a safe miss if future
store behavior ever introduces a gap.

Regression coverage includes lower and upper misses, front eviction, retained hits, clear-created
sequence gaps, and the first post-clear record. A source contract rejects reintroduction of
iterator/find scanning on this lookup path.

## Performance Evidence

| Evidence | Before | After / target | Structural change |
| --- | ---: | ---: | ---: |
| Tail lookup in a 1,000,000-record retained window, 100,000 calls | 100,000,000,000 sequence comparisons | 100,000 direct index probes; <= 500 ms | 99.9999% probe reduction; O(n) -> O(1) |

The ignored Windows-native release evidence prints `EDITOR_LOG_BENCH_V1` with the exact elapsed
nanoseconds and target. The comparison/probe counts are source-deterministic; elapsed time is
accepted only from coordinator terminal evidence.

## Validation

- Exact `rustfmt --check`, scoped `git diff --check`, and the source lookup contract: passed.
- Logging regression tests plus the release performance evidence: pending one coordinator-managed
  batch ticket using the `core::logging` filter and `--include-ignored`.
- No local Cargo lane is launched; this work is intentionally batched behind the existing Runtime
  and Editor validation queue.

## Remaining Parent-plan Work

The store still clones full filtered snapshots, and the logging service still performs synchronous
file I/O and event delivery on producer threads. These separate Editor11 P0/P1 items remain open.
