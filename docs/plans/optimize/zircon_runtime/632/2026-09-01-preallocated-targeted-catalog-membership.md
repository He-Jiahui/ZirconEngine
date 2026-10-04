---
title: Runtime632 Preallocated Targeted Catalog Membership
category: zircon_runtime
report_id: Runtime632-preallocated-targeted-catalog-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime632 Preallocated Targeted Catalog Membership

Targeted project catalog publication now reserves its updated and touched asset-ID sets from the
input iterators' size hints. Updated membership uses the update bound, while touched membership
uses the saturated sum of update and removal bounds. The previous sets both grew from zero.

Unknown iterator upper bounds safely fall back to their lower bounds. Removal traversal, labeled
resource filtering, duplicate update suppression, shard mutation, and delta sorting remain
unchanged.

The ignored Windows Release benchmark emits
`RUNTIME632_PREALLOCATED_TARGETED_CATALOG_MEMBERSHIP_BENCH_V1` over 17 alternating sample pairs
with 32,768 updates and 32,768 removals. The gate requires preallocated P95 to be at most 80% of
the unreserved path P95.

No direct Cargo validation was run. The coordinator owns combined Runtime632/Editor632 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime632 is prepared with Editor632 under request
`runtime632-editor632-catalog-suggestion-membership-performance-20260901iv-v1`. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
