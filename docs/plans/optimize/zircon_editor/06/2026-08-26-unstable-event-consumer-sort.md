---
title: Editor06 Unstable Event Consumer Sort
category: zircon_editor
report_id: Editor06-unstable-event-consumer-sort-2026-08-26
date: 2026-08-26
session_id: root-editor06-plugin-admission-borrows-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Unstable Event Consumer Sort

## Scope

The editor plugin descriptor canonicalizes event consumers by consumer ID every time a builder
consumer is appended. Equal consumer IDs do not carry insertion-order semantics in the manifest,
so the sort can use the allocation-free unstable sorter without changing the canonical order.

## Implementation

`EditorPluginDescriptor::with_event_consumer` now uses `sort_unstable_by` with the existing
consumer-ID comparator. The functional regression checks sorted output, while the release-only
benchmark compares the previous stable sort against the optimized path over 2,048 consumers.

## Performance Evidence

| Evidence | Before | After / target |
| --- | ---: | ---: |
| Stable sort calls per append | 1 | 0 |
| Consumer ordering | sorted by consumer ID | unchanged |
| Release p95 | dynamic evidence pending | <= 95% of legacy p95 |

The coordinator must publish `EDITOR06_UNSTABLE_EVENT_CONSUMER_SORT_BENCH_V1` with both p95
durations, sample/iteration/consumer counts, and the stable-sort reduction.

## Validation

Exact scoped rustfmt, diff checks, source contracts, and functional ordering tests are prepared.
The release benchmark is submitted together with the failed-stage cleanup benchmark in one Cargo
invocation; commit integration, terminal p95 values, and WeCom delivery remain coordinator-owned.
