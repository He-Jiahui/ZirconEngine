---
title: Runtime91 Shader Definition Borrowed Hash Index
category: zircon_runtime
report_id: Runtime91-shader-definition-borrowed-hash-index-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime91 Shader Definition Borrowed Hash Index

## Scope

This slice reduces allocation and duplicate-check cost while shader readiness records are built for
asset management and render-product consumers. It supports Runtime91 readiness qualification but
does not claim to fix the parent plan's false-ready ABI, empty-entry, shader-kind, reflection,
pipeline-layout, compiler authority, cache, or hot-reload gaps.

## Change

- Preallocate a `HashSet<&str>` for shader definition duplicate detection.
- Borrow each trimmed name directly from the immutable shader definition.
- Allocate the normalized name only once for the readiness output instead of cloning a second copy
  into an ordered set.
- Preserve empty-name and trimmed duplicate diagnostics, original definition order, and output
  ownership.

## Deterministic Performance Evidence

| 32,768 long shader definitions | Before | After | Reduction |
|---|---:|---:|---:|
| Normalized-name string allocations | 65,536 | 32,768 | 50% removed |
| Normalized-name bytes copied | 2,621,440 | 1,310,720 | 50% removed |
| Ordered-tree admissions | 32,768 | 0 | 100% removed |
| Borrowed hash admissions | 0 | 32,768 | average O(1) index |
| Diagnostic/output order | source order | source order | unchanged |

The ignored release gate alternates 17 cloned-tree and borrowed-hash samples. It emits
`RUNTIME91_SHADER_DEFINE_BORROWED_HASH_INDEX_BENCH_V1`; acceptance requires optimized P95 to be at
most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826l_runtime91_borrowed_define_index_preserves_diagnostics` covers
  trimmed names, empty definitions, unique values, and duplicate diagnostics.
- `optimization_batch_20260826l_runtime91_define_index_borrows_normalized_names` requires exact
  capacity, borrowed trimmed keys, and zero normalized-name set clones.
- `optimization_batch_20260826l_runtime91_shader_define_borrowed_hash_performance_evidence` emits
  allocation counts, copied bytes, both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Runtime91 P0-5 remains open: readiness still needs non-empty compatible entry points, correct shader
kind/stage rules, qualified reflection and pipeline layout, and product ABI validation. Shared
compile scheduling, complete cache identity, reverse dependency reload, last-good publication,
failure policy, and full scale qualification also remain open.
