---
title: Editor07 Play Output Drain-all Capacity
category: zircon_editor
report_id: Editor07-play-output-drain-all-capacity-2026-09-15
date: 2026-09-15
related_to:
  - docs/plans/optimize/zircon_editor/07/2026-08-26-play-output-streaming-decode.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor07 · Play output drain-all capacity

## Scope

`PlayOutputPump::drain_all` already drains a bounded channel after its reader
threads join, but it started the final diagnostics vector at zero capacity.
The channel length and the optional deferred line are known at that point, and
the budget reporter has a fixed five-message upper bound. This slice removes
geometric growth for a full queue drain without changing line order, byte
budget release, truncation text, or bounded output semantics.

## Implementation

- Add `drain_all_capacity`, which combines the joined receiver length, the
  deferred-line slot, and the fixed budget-diagnostic bound with saturating
  arithmetic.
- Reserve that bound before the deferred line and channel entries are
  appended, while retaining a zero-capacity fast path for an entirely empty
  finish.
- Reuse the same named five-message diagnostic bound for the limited drain so
  the per-poll result cannot grow for its budget summary either.
- Keep the existing queue/byte/line limits and counter-reset behavior intact.

## Regression and performance contract

The lower module
`zircon_editor/src/core/play/process_backend/output/drain_all_capacity_tests.rs`
checks the capacity bound, deferred-first ordering, and a paired ignored
Release benchmark emitting `EDITOR07_PLAY_OUTPUT_DRAIN_ALL_CAPACITY_BENCH_V1`.
The benchmark reports P50/P95 samples and the deterministic geometric-growth
model; it does not turn synthetic allocation timing into a product claim.

The Python source contract is
`tools/tests/test_editor_play_output_drain_capacity_performance_contract.py`.

## Local receipt

The focused source contract passes `3/3`; the merged recent Runtime/Editor
hot-path batch (including this contract, Editor763, Runtime782–785, and the
adjacent capacity guards) passes `15/15` in one process. Non-recursive Rustfmt,
scoped `git diff --check`, wiki navigation, referenced-path, and trailing-space
guards pass. The current cross-surface non-tooling batch remains the prior
`550`-module/`1966`-test receipt (Editor764 is a new focused contract and is
queued for the next merged run); these are local source/model checks only.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/play/process_backend/output.rs` | `ED745C84BD89F86E0855D2C95061F7921A416CF0E83DBAD6843FD82A3CE763AD` |
| `zircon_editor/src/core/play/process_backend/output/drain_all_capacity_tests.rs` | `72BA2870CBAEB06CD1B715125A55068A8C1DB3360E90D0EA803431FC566D8635` |
| `tools/tests/test_editor_play_output_drain_capacity_performance_contract.py` | `DE02BDC6E658C922D5EABDB74FB3CAEBA457E54B467D85D227C91B7855949F87` |

## Validation boundary

The source contract and Rustfmt checks are local evidence. The owner-attributed
managed Windows Cargo/Release lane must still verify compile/test parity,
allocator behavior, and Play-output p50/p95/p99 latency. Tooling production
changes remain deferred for the later Rust migration.
