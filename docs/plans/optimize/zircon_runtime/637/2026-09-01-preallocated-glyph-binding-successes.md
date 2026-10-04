---
title: Runtime637 Preallocated Glyph Binding Successes
category: zircon_runtime
report_id: Runtime637-preallocated-glyph-binding-successes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime637 Preallocated Glyph Binding Successes

Glyph atlas texture-upload binding plans now reserve the success vector from the request count.
Each request produces at most one binding, while failures remain unreserved so malformed batches do
not pay for two full request-sized vectors.

Request order, first failing reason, byte-range validation, and failure reporting remain unchanged.
The regression exercises the real valid binding plan and verifies that all request successes fit the
initial capacity.

The ignored Windows Release benchmark emits
`RUNTIME637_PREALLOCATED_GLYPH_BINDING_SUCCESSES_BENCH_V1` over 17 alternating sample pairs and
65,536 success projections. The gate requires preallocated P95 to be at most 85% of unreserved P95.

No direct Cargo validation was run. The coordinator owns the aggregate Runtime/Editor regression and
performance validation. Measured P95, commit, push, and WeCom outcome are recorded only after
coordinator completion.
