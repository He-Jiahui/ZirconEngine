---
title: Runtime615 Reused Chunk Hash Index
category: zircon_runtime
report_id: Runtime615-reused-chunk-hash-index-2026-09-01
date: 2026-09-01
session_id: root-runtime-editor-optimize-20260901-r6
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime615 Reused Chunk Hash Index

Zrpack manifest validation now retains owned chunk hashes in the duplicate-validation index and
reuses that same set for final asset/chunk coverage equality. The previous path built a third full
`HashSet` from the chunk-size map after already hashing every chunk once.

The four temporary validation sets now reserve their known input lengths. Duplicate asset-path and
chunk-hash errors retain first-occurrence precedence, sorted-window checks are unchanged, and the
canonical chunk-size map remains a `BTreeMap`. Existing behavior coverage still locks the first
duplicate error; focused source coverage rejects the redundant final set construction.

The ignored Windows Release benchmark emits `RUNTIME615_REUSED_CHUNK_HASH_INDEX_BENCH_V1` over 17
alternating sample pairs with 32,768 unique 32-byte hashes. It compares three temporary hash sets
against reuse of the validated chunk index with two sets. The gate requires reused-index P95 to be
at most 75% of legacy P95.

No direct Cargo validation was run. The coordinator owns combined Windows Release compilation,
prefixed regression tests, ignored benchmarks, record finalization, manifest-only commit/push, and
one-shot WeCom publication after both gates pass.

## Current Batched Validation Handoff (2026-09-01)

Runtime615 is prepared with Editor615 under request
`runtime615-editor615-chunk-locale-performance-20260901ie-v1`. Receipt, validation ticket, measured
P95, pushed SHA, and notification result are recorded only after coordinator completion.
