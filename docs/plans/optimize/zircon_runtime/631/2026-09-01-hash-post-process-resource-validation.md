---
title: Runtime631 Hash Post-Process Resource Validation
category: zircon_runtime
report_id: Runtime631-hash-post-process-resource-validation-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime631 Hash Post-Process Resource Validation

Post-process stack validation now tracks available and produced resource names with hash sets. The
previous local ordered sets paid tree insertion and lookup cost even though their iteration order
was never observed. Produced membership reserves the total declared output count before traversal.

Node traversal remains governed by the existing stable topological order, so missing-input and
duplicate-output error precedence is unchanged. The dependency graph continues to use ordered
integer sets where deterministic traversal is part of the graph contract.

The ignored Windows Release benchmark emits
`RUNTIME631_HASH_POST_PROCESS_RESOURCE_MEMBERSHIP_BENCH_V1` over 17 alternating sample pairs with
65,536 borrowed resource identities. The gate requires hash-membership P95 to be at most 40% of
ordered-tree P95.

No direct Cargo validation was run. The coordinator owns combined Runtime631/Editor631 Windows
Release compilation, prefixed regression tests, ignored benchmarks, measured record finalization,
manifest-only commit/push, and one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime631 is prepared with Editor631 under request
`runtime631-editor631-post-process-consumer-membership-performance-20260901iu-v1`. Receipt,
validation ticket, measured P95, pushed SHA, and notification result are recorded only after
coordinator completion.
