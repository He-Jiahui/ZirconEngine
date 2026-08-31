---
title: Runtime80 Borrowed Protected Glyph Set
category: zircon_runtime
report_id: Runtime80-borrowed-protected-glyph-set-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime80 Borrowed Protected Glyph Set

## Scope

This slice removes owned `SdfAtlasGlyphKey` copies from the visible-slot protection set built during
SDF glyph-cache budget enforcement. It preserves cache limits, protected-slot membership, oldest-
unprotected eviction order, CPU sidecar cleanup, resident-byte accounting, and public contracts.

## Implementation

`enforce_baked_glyph_budget` previously cloned every visible slot key into a temporary
`HashSet<SdfAtlasGlyphKey>`. Each key carries three optional `Arc<str>` fields, so a large visible
atlas incurred repeated atomic reference-count traffic and stored full key values solely for
membership tests. The optimized set stores `&SdfAtlasGlyphKey` references into the stable input
slice and performs the same content-based hash lookup without owning key payloads.

The identity regression proves each protected entry points to the original slot key. The ignored
release benchmark compares owned and borrowed protection sets at 16,384 slots.

## Performance Contract

| Evidence for 16,384 visible slots | Retired path | Optimized gate |
| --- | ---: | ---: |
| Protected key clones | 16,384 | 0 |
| Arc reference-count increments | 49,152 | 0 |
| Temporary set key storage | full `SdfAtlasGlyphKey` | one reference |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 75% of retired P95 |

The benchmark emits `RUNTIME80_BORROWED_PROTECTED_GLYPH_SET_BENCH_V1` with slot/key/Arc counts,
structural clone reductions, paired P50/P95 timings, and raw samples for coordinator-owned WeCom
reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and Runtime80 source-structure gates are required
before submission. This task is queued in the Runtime80 three-task asynchronous validation batch,
filtered by `runtime80_batch_`, together with single-probe glyph byte accounting and allocation-free
primary-family matching. The batch runs four behavior tests, two ignored release benchmarks, and
one standalone family-match model. Dynamic P95 evidence, integration SHA, and automatic WeCom
delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime80 still requires font artifact cooking, session-owned collection service, generational
face/instance leases, typed glyph completeness, cross-cache pressure recovery, and product-scale
qualification. This allocation optimization does not claim those milestones complete.
