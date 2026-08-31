---
title: Hub05 panic-safe read-fast focus refresh gate
category: zircon_hub
report_id: Hub05-focus-refresh-panic-safe-read-fast-gate-2026-08-31
date: 2026-08-31
session_id: root-hub05-focus-refresh-gate-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Hub05 Panic-Safe Read-Fast Focus Refresh Gate

## Scope

This slice addresses `ZHUB-HOST-P1-16` and `HHOST-G12` for the existing focused-window
recent-project refresh worker. It does not claim the parent plan's broader worker registry,
join, cancellation, shutdown, or stale-window generation work.

## Implementation

- `FocusRefreshGate::try_enter` rejects an already-pending refresh with one Acquire load.
  Only the first entrant executes compare-exchange and clones the shared pending state.
- `FocusRefreshPermit` owns admission through refresh and event publication. Its `Drop`
  clears pending with Release ordering on normal return and panic unwind.
- `HubCommandState` no longer manipulates the atomic flag directly. The worker closure owns
  the permit, keeping admission and release policy in one module.

## TDD Evidence

The source contract was written first and failed because the gate module did not exist.
After implementation:

- `python -m unittest tools.tests.test_hub05_focus_refresh_gate_performance_contract -v`
  passes 4/4.
- Rust behavior tests cover duplicate rejection until permit drop and successful re-entry
  after a simulated worker panic caught with `catch_unwind`.
- Scoped `rustfmt +1.94.1 --edition 2021 --check` and `git diff --check` pass locally.

## Performance Contract

The ignored Windows-native release benchmark compares the actual read-fast gate with the
previous `AtomicBool::swap(true, AcqRel)` rejection path. It runs 2,000,000 rejected focus
events per sample over 21 alternating pairs, records raw nanosecond samples, and requires
both P50 and P95 to improve by at least 30%.

Deterministic rejected-path work is reduced from one atomic write/RMW per duplicate event to
zero: 2,000,000 writes become 0 per sample, a 100% reduction. Exact managed P50/P95 values
remain pending and must be included in the terminal optimization record and WeCom message.

## Validation And Integration

- Direct Cargo was not used.
- The exact asynchronous Windows release command is:
  `cargo +1.94.1 test --manifest-path zircon_hub/Cargo.toml --lib --locked --release --jobs 1 -- hub05_ --include-ignored --nocapture --test-threads=1`.
- Coordinator validation, independent review, integration commit, and terminal WeCom delivery
  remain pending.
