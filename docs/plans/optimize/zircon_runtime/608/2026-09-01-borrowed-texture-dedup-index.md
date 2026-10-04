---
title: Runtime608 Borrowed Texture Dedup Index
category: zircon_runtime
report_id: Runtime608-borrowed-texture-dedup-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime608 Borrowed Texture Dedup Index

Material direct-reference collection now indexes texture identities in a preallocated `HashSet` of
borrowed keys. The previous collector projected each texture into an owned value and scanned the
growing output slice, producing quadratic equality work for materials with many unique texture
slots. The shader remains first, the parent remains last, and only texture entries participate in
deduplication.

The shared collector receives an explicit borrowed-key projection so both public projections keep
their original equality contract: full direct references deduplicate by UUID and locator, while
locator-only references deduplicate by locator. A focused behavior test covers two references with
different UUIDs and the same locator. The source guard requires a capacity-sized borrowed index and
rejects the former output-slice scan.

The ignored Windows Release benchmark emits
`RUNTIME608_BORROWED_TEXTURE_DEDUP_BENCH_V1` over 17 alternating sample pairs and 4,096 unique
texture references. The modeled worst-case equality probes fall from 8,386,560 to 4,096, and the
gate requires indexed P95 to be at most 10% of legacy P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime608 is prepared with Editor608 under request
`runtime608-editor608-texture-category-performance-20260901hy-v1`. Receipt, validation ticket,
measured P95, pushed SHA, and notification result are recorded only after coordinator completion.
