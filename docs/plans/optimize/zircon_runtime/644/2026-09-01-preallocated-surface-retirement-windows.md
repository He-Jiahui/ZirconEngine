---
title: Runtime644 Preallocated Surface Retirement Windows
category: zircon_runtime
report_id: Runtime644-preallocated-surface-retirement-windows-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime644 Preallocated Surface Retirement Windows

Surface-lease all-window retirement now reserves the unique-window set once from the exact
`entries.len()` upper bound. It removes per-window membership probes and incremental reservation
while preserving deterministic sorting, duplicate suppression, and `CapacityExhausted` behavior.

The source regression checks the bounded reservation and single insertion path. The ignored
Windows Release benchmark emits `RUNTIME644_PREALLOCATED_SURFACE_RETIREMENT_WINDOWS_BENCH_V1`
over 17 alternating sample pairs with 65,536 entries. The gate requires preallocated P95 to be at
most 85% of the unreserved collection P95.

No direct Cargo validation was run. The coordinator owns aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, optimization closeout, and WeCom outcome are
recorded only after coordinator completion.
