---
title: Editor11 Stable Console Tail Identity
category: zircon_editor
report_id: Editor11-stable-console-tail-identity-2026-09-09
date: 2026-09-09
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor11 Stable Console Tail Identity

## Scope

This slice removes repeated record cloning and temporary filter-tree construction from a stable
Console tail projection. It preserves the existing bounded tail size, filter semantics, sequence
order, incremental projection, and full snapshot API. It does not claim the parent plan's
asynchronous logging ingress, durable journal, cursor/query, or persistence milestones.

## Implementation

`EditorLogStore::snapshot_tail_if_changed` computes a compact identity from the first and last
matching sequences plus the bounded match count while holding the existing store lock. The common
unfiltered Console filter derives that identity directly from the contiguous `VecDeque` window;
other filters retain the bounded reverse scan. When the identity is unchanged, it returns no record
vector, so `ActivityLogConsoleProjection` reuses its existing immutable line generation without
cloning the tail. A changed identity falls back to the existing reverse bounded materialization and
preserves chronological order; nonmatching appends do not invalidate the visible tail. The
projection now constructs single-source filters directly from the existing channel bitmask, so a
stable frame does not allocate a one-element `BTreeSet`.

## Deterministic work model

For 2,048 retained records, a 256-record Console window, and 10,000 stable polls, the retired
projection materializes 2,560,000 `LogRecord` values and inspects 2,560,000 matching entries. The
unfiltered identity path materializes 256 records once, performs constant-time boundary probes on
the stable polls, and materializes zero additional records: 99.99% fewer stable-poll record
materializations and no per-poll tail scan. Source-specific polls also avoid one temporary filter
tree allocation. Other filters remain bounded by the retained journal when sparse; this model
excludes CPU, allocator, lock-wait, RSS, and input-to-present timing.

The ignored Rust release gate emits `EDITOR_LOG_STABLE_TAIL_IDENTITY_BENCH_V1` and requires a
coordinator-managed Windows run before any elapsed-time claim is accepted.

## Validation

- The new store regression covers initial materialization, unchanged identity reuse, append-driven
  invalidation, nonmatching appends, window-size invalidation, and chronological tail order. The
  logging filter regression verifies direct and set-based channel construction are equivalent.
- A batched static contract run passed `27/27`, including Console materialization, Editor10,
  Runtime03, Runtime438, and the existing diagnostic contracts; selected Rustfmt and scoped diff
  checks passed.
- The merged Runtime/Editor performance-contract batch passed `1723/1723` (`Runtime 1143/1143`,
  `Editor 580/580`).
- Managed Cargo, unit execution, and release CPU/allocation/RSS/frame p50/p95/p99 remain pending
  because the external `E:/Git/zr_vm` checkout is dirty. No coordinator state was queried or
  waited on.

## Current source fingerprints

- `zircon_editor/src/core/logging/store.rs`: `CEDE777889B80DD138CE92C9FCB0D263518990DA818D1734A75E36D695B8279E`
- `zircon_editor/src/core/logging/filter.rs`: `13B892CB3E75206FDCCB4FFF9A820E061BCEC28819957AB6583AFC5E2CFFC714`
- `zircon_editor/src/core/logging/mod.rs`: `0240483464A714CFB1973FC11F3EC08AB4DA0F03207AE5C79A4A3CFFA262A0CB`
- `zircon_editor/src/core/logging/service.rs`: `34AA6BF5E2D8EB61307D6538AF6A5E997EC5F26326D04DFF92DCFD2EC3894717`
- `zircon_editor/src/core/logging/tests.rs`: `7EA43469FBB4E929C335AD2B37154A9F52589E5D1DD0D13D5B82F74F13B27B25`
- `zircon_editor/src/ui/workbench/activity_log_console_projection.rs`: `F05DE7F1AB77C1CC772D515DCF777878C0A3B45DB3E1C50BAD1F139EC55E6AF9`
