---
title: Editor610 Borrowed Discovery Membership
category: zircon_editor
report_id: Editor610-borrowed-discovery-membership-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor610 Borrowed Discovery Membership

Editor plugin discovery now keeps the local catalog-manifest snapshot alive and builds a
preallocated `HashSet<&str>` over its package IDs. The previous membership index cloned every
package ID into an owned `BTreeSet`, even though discovery iteration and the result `BTreeMap`
already define diagnostic precedence and canonical output order.

Known-package checks are now amortized constant time and allocate no package-ID copies inside the
membership index. Unknown-package rejection still precedes insertion, duplicate discovery still
rejects the second occurrence, and successful output remains sorted by package ID. Focused behavior
coverage locks all three cases, while a source guard rejects the owned ID clone and ordered set.

The ignored Windows Release benchmark emits `EDITOR610_BORROWED_DISCOVERY_MEMBERSHIP_BENCH_V1`
over 17 alternating sample pairs with 16,384 package IDs of 249 bytes. Modeled temporary package-ID
clones in the membership index fall from 16,384 to zero. The gate requires borrowed hash-membership
P95 to be at most 25% of owned ordered-membership P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Editor610 is prepared with Runtime610 under request
`runtime610-editor610-delta-discovery-performance-20260901ia-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
