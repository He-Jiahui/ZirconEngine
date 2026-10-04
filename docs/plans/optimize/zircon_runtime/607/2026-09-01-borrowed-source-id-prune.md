---
title: Runtime607 Borrowed Source ID Membership
category: zircon_runtime
report_id: Runtime607-borrowed-source-id-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime607 Borrowed Source ID Membership

Shader prewarm manifest integrity validation and variant-source pruning now build `HashSet` indexes
of borrowed `ShaderVariantPrewarmSourceId` references. The previous paths cloned every 64-byte
content-addressed ID before duplicate checks or one retain pass. Source order, duplicate detection,
missing-source diagnostics, replacement behavior, and pruning results are unchanged. Replacement
still performs the one owned ID clone required to update the selected request; only temporary
membership ownership is removed.

A focused replacement test verifies that the old source is pruned while other referenced sources
and the replacement remain valid. A source guard covers both integrity validation and replacement
pruning. The ignored Windows Release benchmark emits
`RUNTIME607_BORROWED_SOURCE_ID_MEMBERSHIP_BENCH_V1` over 17 alternating sample pairs with 16,384
canonical source IDs. The modeled deep ID clones fall from 16,384 to zero, and the gate requires
borrowed-membership P95 to be at most 85% of owned-membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime607 is prepared with Editor607 under request
`runtime607-editor607-source-registry-performance-20260901hx-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
