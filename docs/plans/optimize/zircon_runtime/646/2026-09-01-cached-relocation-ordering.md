---
title: Runtime646 Cached Relocation Ordering
category: zircon_runtime
report_id: Runtime646-cached-relocation-ordering-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime646 Cached Relocation Ordering

Relocation preflight canonical ordering now sorts by locator once, then sorts only equal-locator
groups with cached UUID string keys. Unique locators therefore avoid UUID formatting entirely, and
duplicate locators format each UUID once instead of once per comparator call. The resulting order is
identical to the former locator-first, UUID-string tie-breaker.

The ignored Windows Release benchmark emits `RUNTIME646_CACHED_RELOCATION_ORDER_BENCH_V1` over 17
alternating sample pairs with 32,768 assets sharing one locator, forcing the UUID tie-breaker. It
verifies identical ordered UUIDs, then compares repeated comparator formatting with cached-key
sorting. The gate requires cached P95 to be at most 80% of comparator P95.

No direct Cargo validation was run. The coordinator owns combined Runtime646/Editor646 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime646 is prepared with Editor646 under the shared `optimization_batch_jg_` prefix. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
