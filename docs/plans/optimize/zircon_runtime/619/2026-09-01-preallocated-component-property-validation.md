---
title: Runtime619 Preallocated Component Property Validation
category: zircon_runtime
report_id: Runtime619-preallocated-component-property-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime619 Preallocated Component Property Validation

Runtime extension component validation now reserves its borrowed property-name membership index
from the descriptor property count. The previous hash set grew incrementally even though the
maximum number of admitted property names was known before traversal.

Property field validation order, borrowed `&str` identities, and first-duplicate diagnostics remain
unchanged. Existing behavior coverage continues to lock the first duplicate error, while focused
source coverage rejects an unreserved production index.

The ignored Windows Release benchmark emits
`RUNTIME619_PREALLOCATED_COMPONENT_PROPERTY_BENCH_V1` over 17 alternating sample pairs with 32,768
unique long property names. The gate requires preallocated membership P95 to be at most 85% of
unreserved membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime619 is prepared with Editor619 under request
`runtime619-editor619-component-entry-owner-performance-20260901ii-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
