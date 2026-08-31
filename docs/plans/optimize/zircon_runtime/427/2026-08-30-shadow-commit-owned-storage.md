---
title: Runtime427 Shadow Commit Owned Storage
category: zircon_runtime
report_id: Runtime427-shadow-commit-owned-storage-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime427 Shadow Commit Owned Storage

Glyph-atlas bitmap shadow commit merging now adopts an incoming patch vector when the destination
has no reusable allocation and otherwise appends it directly. Owned zero-initialization sets are
merged with `BTreeSet::append`, preserving patch order and set semantics.

The previous path consumed the incoming containers through `extend`. An empty destination therefore
allocated patch storage and moved every patch instead of taking the already-owned allocation.

The ignored Windows Release benchmark emits
`RUNTIME427_SHADOW_COMMIT_OWNED_PATCH_STORAGE_BENCH_V1` over 17 alternating paired samples, each
merging 2,048 owned buffers of 4,096 entries, requiring
`optimized_p95_ns <= legacy_p95_ns * 0.70`.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and one-shot
WeCom publication after measured evidence passes the declared gate.

## Current batched validation handoff (2026-08-30)

Runtime427 is prepared with Editor355 under request
`runtime427-editor355-performance-batch-20260830ds-v1`. Receipt, validation ticket, measured p95,
pushed SHA, and notification result are recorded only after coordinator completion.
