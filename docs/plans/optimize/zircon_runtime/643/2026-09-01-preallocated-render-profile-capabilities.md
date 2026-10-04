---
title: Runtime643 Preallocated Render Profile Capabilities
category: zircon_runtime
report_id: Runtime643-preallocated-render-profile-capabilities-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime643 Preallocated Render Profile Capabilities

Render profile capability expansion now caches its four feature predicates and reserves the output
vector from the enabled static requirement tables. The bound is 0–14 entries and remains valid
when capabilities overlap; `push_unique` continues to preserve the former ordering and deduplication
semantics. Reusing the predicates means capacity planning adds no extra feature scans.

The regression checks the Solari upper bound and exact equality with the retired unreserved
projection. The ignored Windows Release benchmark emits
`RUNTIME643_PREALLOCATED_RENDER_PROFILE_CAPABILITIES_BENCH_V1` over 17 alternating sample pairs
with 65,536 expansions. The gate requires preallocated P95 to be at most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, optimization closeout, and WeCom outcome are
recorded only after coordinator completion.
