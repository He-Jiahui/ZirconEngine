---
title: Runtime645 Direct Resource ID Ordering
category: zircon_runtime
report_id: Runtime645-direct-resource-id-ordering-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime645 Direct Resource ID Ordering

Project resource reconciliation now compares `ResourceId` values directly when locators tie.
`ResourceId` already provides stable `Ord`, so formatting both UUIDs into temporary strings inside
the sort comparator was unnecessary. Locator-first ordering and deterministic ID tie-breaking are
preserved while comparator allocations are removed.

The ignored Windows Release benchmark emits `RUNTIME645_RESOURCE_IDENTITY_ORDER_BENCH_V1` over 17
alternating sample pairs with 32,768 identities sharing one locator, forcing every relevant
comparison through the ID tie-breaker. It verifies identical ordered IDs before comparing string
formatting with direct ID ordering. The gate requires direct-ID P95 to be at most 80% of
string-order P95.

No direct Cargo validation was run. The coordinator owns combined Runtime645/Editor645 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime645 is prepared with Editor645 under the shared `optimization_batch_jf_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
