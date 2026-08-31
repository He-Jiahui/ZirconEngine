---
title: Runtime11C Page Shadow Hash Index
category: zircon_runtime
report_id: Runtime11C-page-shadow-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime11c-three-task-page-hash-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11C Page Shadow Hash Index

## Scope

This slice reduces lookup cost in bitmap glyph-atlas page-shadow storage and commit application.
It preserves generation filtering, the 32 MiB budget, byte accounting, patch semantics, and the
ordered zero-initialized-page admission that determines deterministic behavior under budget
pressure. It does not change GPU upload or atlas residency architecture.

## Change

- Store persistent page shadows in `HashMap<GlyphAtlasPageKey, _>`.
- Build resident-page and current-generation lookup tables as temporary hash maps.
- Keep accepted zero-initialized pages in `BTreeSet` so budgeted admission order is unchanged.
- Preserve every generation, size, bounds, and byte-length check.

## Deterministic Performance Evidence

| Representative index workload | Before | After |
|---|---:|---:|
| Persistent page membership | ordered O(log n) lookup | average O(1) hash lookup |
| Resident page projection | ordered map build/lookup | hash map build/lookup |
| Generation-retain projection | ordered map build/lookup | hash map build/lookup |
| Zero-init publication order | ascending page key | ascending page key, unchanged |
| Release workload | 32,768 pages / 262,144 lookups | same keys and lookups |

The ignored release gate alternates 17 ordered-index and hash-index lookup samples and emits
`RUNTIME11C_PAGE_SHADOW_HASH_INDEX_BENCH_V1`. Acceptance requires hash-index P95 to be at most 60%
of ordered-index P95. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `runtime11c_batch_hash_shadow_store_preserves_generation_filter`
  commits a zero page and proves that a later generation removes the stale shadow.
- `runtime11c_batch_page_shadow_uses_hash_indexes` requires all three
  membership-only hash maps while retaining the ordered zero-init set.
- `runtime11c_batch_page_shadow_hash_index_performance_evidence` verifies
  identical lookup results, emits both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

The managed `runtime11c_batch_` release gate seals this work together with the glyph allocator and
SDF page-owner slices in one Cargo invocation: three source contracts, nine Rust tests, and three
performance rows. Dynamic Windows marker values, commit attribution, and WeCom publication remain
pending the coordinator result.

## Remaining Parent-plan Work

Runtime11C still needs unified atlas/device generation ownership, device-loss recovery, DPI and
quality profiles, cross-route font/glyph identity, ordered GPU submission, VRAM-pressure policy,
and product pixel/performance qualification across game and Editor routes.
