---
title: Runtime622 Preallocated Interface Owner Dedup
category: zircon_runtime
report_id: Runtime622-preallocated-interface-owner-dedup-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime622 Preallocated Interface Owner Dedup

Runtime interface-owner projection now materializes the owner iterator once, reads its lower-bound
size hint, and reserves the deduplication hash set before insertion. Unique owners are still moved
to the output vector and sorted by raw module ID after membership collection.

Borrowed/copy-only owner admission, deduplication semantics, and deterministic sorted output remain
unchanged. Existing coverage locks unique sorted results, while focused source coverage locks the
size-hint capacity path.

The ignored Windows Release benchmark emits `RUNTIME622_PREALLOCATED_INTERFACE_OWNER_BENCH_V1`
over 17 alternating sample pairs with 32,768 unique module owners. The gate requires preallocated
membership P95 to be at most 85% of unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime622 is prepared with Editor622 under request
`runtime622-editor622-owner-reflector-performance-20260901il-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
