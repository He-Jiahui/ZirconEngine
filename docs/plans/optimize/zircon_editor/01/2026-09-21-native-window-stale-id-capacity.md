---
related_code:
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
plan_sources:
  - user: 2026-09-21 optimize Runtime and Editor hot paths and record completion
tests:
  - zircon_editor/src/ui/retained_host/app/native_windows/store.rs
  - tools/tests/test_editor874_native_window_stale_identity_capacity_performance_contract.py
doc_type: milestone-detail
title: Editor874 native-window stale-ID capacity
category: zircon_editor
report_id: Editor874-native-window-stale-id-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor874 - native-window stale-ID capacity

## Scope

Every native presenter target synchronization filtered the ordered window map
into a `Vec<MainPageId>` through an iterator whose filtered lower bound is zero.
A dense retirement could therefore geometrically grow the temporary stale-ID
vector even though the current window count is an authoritative upper bound.

## Optimization

- A dedicated collector walks the existing BTree keys once and preserves their
  deterministic retirement order.
- On the first stale match it reserves the complete current-window upper bound,
  then clones only stale identities.
- No-match and empty stores retain a zero-capacity vector, avoiding a new
  allocation on the common stable-topology path.
- Window removal, generation cleanup, row cleanup, hide error propagation, and
  target application order are unchanged.

## TDD and local evidence

The Editor874 source contract was observed RED with two failures and two
missing-helper errors, then GREEN at `4/4`. The adjacent Editor873/native
projection batch passes `38/38` in `0.032s`. The current non-Tooling
performance/contract loader passes `4204/4204` tests across `990` files in
`277.584s`, with zero load errors, failures, errors, or skips. Fixture-emitted
Cargo command lines are not managed compile evidence. Exact Rustfmt passes.

Lower semantics cover ordered alternating retirements and the all-retained
zero-capacity path. The ignored
`EDITOR874_NATIVE_WINDOW_STALE_ID_CAPACITY_BENCH_V1` marker exercises 65,536
queries over 128 windows with 64 stale identities per query; the deterministic
model changes temporary-vector growth events from five to zero.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/retained_host/app/native_windows/store.rs` | `F858DB8CD839D566B33C6F31BA02EF1A21C4032135A0D8E87357F091F7C1C6AD` |
| `tools/tests/test_editor874_native_window_stale_identity_capacity_performance_contract.py` | `E199151EEFB761C53901EE6FB73A62506C8A0CD7BAB28DE8A2D913D0F0648D37` |

## Acceptance boundary

Keep pending until managed current-source Windows compilation, Release marker,
allocator, and native-window topology-sync p50/p95/p99 receipts pass.
