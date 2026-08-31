---
title: Editor13 Atomic Clone-free Drawer Rebind
category: zircon_editor
report_id: Editor13-atomic-clone-free-drawer-rebind-2026-08-26
date: 2026-08-26
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Atomic Clone-free Drawer Rebind

## Scope

This slice makes window-registry drawer rebinding side-effect free on invalid targets and removes
the full `DrawerViewInstance` clone/reinsert cycle from valid rebinds. It preserves drawer bucket
deduplication, selection transfer, dock position, missing-drawer diagnostics, and public APIs.

## Implementation

`bind_drawer` previously removed the drawer from its old window and mutated its owner before asking
`register_drawer_view` to validate the target. A missing or non-drawer-capable target therefore
returned an error after partially corrupting the registry. The same path cloned the complete drawer,
including its title, only to replace the existing map value.

The optimized path validates both the drawer and target window first, then enters an infallible
commit section that updates old and new window buckets plus the existing drawer value in place. The
regression snapshots the complete registry and proves an invalid target produces no state change.

## Performance Contract

| Evidence for 64 alternating rebinds with a 64 KiB title | Retired path | Optimized gate |
| --- | ---: | ---: |
| Full drawer-instance clones | 64 | 0 |
| Cloned title bytes | 4,194,304 | 0 |
| Invalid-target mutation | partial old-binding removal | zero state change |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 60% of retired P95 |

The benchmark emits `EDITOR13_CLONE_FREE_DRAWER_REBIND_BENCH_V1` with title/rebind counts,
structural clone bytes, paired P50/P95 timings, and raw samples for coordinator-owned WeCom
reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and Editor13 source-structure gates are required
before submission. One managed Editor13 Cargo invocation filtered by `editor13_window_registry_`
covers the atomic regression and ignored release benchmark together with borrowed view-instance
indexing. Dynamic P95 evidence, integration SHA, and automatic WeCom delivery remain
coordinator-owned and pending.

## Remaining Parent-plan Work

Editor13 still requires a transactional layout authority, versioned workspace bundle, bounded
restore admission, unknown-provider placeholders, durable profile storage, monitor-aware placement,
and native host reconciliation. This focused registry transaction does not claim those milestones
complete.
