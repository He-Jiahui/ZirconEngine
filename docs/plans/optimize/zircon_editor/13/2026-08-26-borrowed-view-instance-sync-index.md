---
title: Editor13 Borrowed View-instance Sync Index
category: zircon_editor
report_id: Editor13-borrowed-view-instance-sync-index-2026-08-26
date: 2026-08-26
session_id: root-editor13-drawer-tab-dedup-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Borrowed View-instance Sync Index

## Scope

This slice removes owned view-instance ID copies from `EditorWindowRegistry::sync_from_layout`.
It preserves exact instance lookup, duplicate-ID last-writer behavior, missing-instance fallback,
drawer ordering, detached drawer projection, and all public Editor contracts.

## Implementation

The layout synchronization path previously built a `BTreeMap<ViewInstanceId, &ViewInstance>`,
cloning every string-backed ID before projecting drawers and floating windows. The optimized path
builds a `HashMap<&str, &ViewInstance>` over the stable input slice and performs average constant-
time lookups without copying any ID payload.

The identity regression proves each index result points to the original `ViewInstance`. The ignored
release benchmark compares the retired owned ordered index with the borrowed hash index at 16,384
instances and 256 bytes per ID.

## Performance Contract

| Evidence for 16,384 view instances | Retired path | Optimized gate |
| --- | ---: | ---: |
| Instance ID clones | 16,384 | 0 |
| Cloned instance ID bytes | 4,194,304 | 0 |
| Index structure | owned `BTreeMap` | borrowed `HashMap` |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 50% of retired P95 |

The benchmark emits `EDITOR13_BORROWED_INSTANCE_INDEX_BENCH_V1` with scale, ID bytes, index kinds,
structural clone counts, paired P50/P95 timings, and raw samples for coordinator-owned WeCom
reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and Editor13 source-structure gates are required
before submission. One managed Editor13 Cargo invocation filtered by `editor13_window_registry_`
covers this regression and ignored release benchmark together with atomic clone-free drawer
rebinding. Dynamic P95 evidence, integration SHA, and automatic WeCom delivery remain
coordinator-owned and pending.

## Remaining Parent-plan Work

Editor13 still requires a transactional layout authority, versioned workspace bundle, bounded
restore admission, unknown-provider placeholders, durable profile storage, monitor-aware placement,
and native host reconciliation. This focused sync optimization does not claim those milestones
complete.
