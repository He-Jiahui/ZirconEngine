---
title: Runtime654 Preallocated Metadata Indexes
category: zircon_runtime
report_id: Runtime654-preallocated-metadata-indexes-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: retracted_after_review
validation_status: not_submitted
---

# Runtime654 Preallocated Metadata Indexes

This proposal was retracted during source review. The production implementation uses iterator
collection, whose standard `FromIterator` path already owns its allocation policy; comparing it to
a synthetic `HashMap::new()` insertion loop would not isolate a production improvement. No source
or test remains attached to this plan.

The ignored Windows Release benchmark emits `RUNTIME654_METADATA_INDEX_CAPACITY_BENCH_V1` over 21
alternating sample pairs, 64 batches per sample, and 4,096 entries per batch. The gate requires
reserved P95 to be at most 80% of unreserved P95.

No Cargo validation, performance measurement, commit, push, or WeCom publication is claimed for
this withdrawn candidate.
