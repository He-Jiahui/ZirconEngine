---
title: Runtime616 Adaptive Shader Contract Index
category: zircon_runtime
report_id: Runtime616-adaptive-shader-contract-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime616 Adaptive Shader Contract Index

Material/shader contract validation now builds borrowed property and texture-name hash indexes only
when both candidate and lookup counts can amortize construction. Smaller contracts retain the
allocation-free linear path. Diagnostic iteration order, first matching property semantics, and
standard material aliases remain unchanged.

Focused coverage locks the 8 x 8 indexing boundary and the no-index result for empty queries. The
ignored Windows Release benchmark emits
`RUNTIME616_ADAPTIVE_SHADER_CONTRACT_INDEX_BENCH_V2` over 17 alternating sample pairs with 4,096
schema entries and 4,096 reverse-order queries. Its large-contract gate requires indexed P95 to be
at most 30% of nested scans; the record also exposes the small 8 x 4 workload with zero index
allocations.

No direct Cargo validation was run. The coordinator owns batched compilation, focused regressions,
ignored Release performance evidence, integration, push, and one-shot WeCom publication.
