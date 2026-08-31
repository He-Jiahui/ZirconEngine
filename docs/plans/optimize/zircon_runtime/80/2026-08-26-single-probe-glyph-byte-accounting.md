---
title: Runtime80 Single-probe Glyph Byte Accounting
category: zircon_runtime
report_id: Runtime80-single-probe-glyph-byte-accounting-2026-08-26
date: 2026-08-26
session_id: root-runtime59-diagnostics-retry-20260825
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime80 Single-probe Glyph Byte Accounting

## Scope

This slice removes the post-insert glyph-cache lookup used only to recover the just-inserted bitmap
length. It preserves replacement subtraction, saturating resident-byte accounting, measured-glyph
sidecar publication, LRU touch ordering, and all public runtime contracts.

## Implementation

`insert_baked_glyph` previously moved the glyph into `glyphs`, then hashed the complete
`SdfAtlasGlyphKey` again with `get` to read `bitmap.len()`. The optimized path captures the byte
count before moving the glyph and adds it directly after subtracting any replaced bitmap.

The regression replaces a three-byte bitmap with a five-byte bitmap and verifies one resident entry
and five resident bytes. The ignored release benchmark performs 256 repeated replacements using
keys with three 2 KiB text fields.

## Performance Contract

| Evidence per repeated glyph insertion | Retired path | Optimized gate |
| --- | ---: | ---: |
| Glyph-map hash probes | 2 | 1 |
| Post-insert key hash | 1 | 0 |
| Resident byte result | exact | exact |
| Alternating release benchmark | 21 paired samples | optimized P95 <= 90% of retired P95 |

The benchmark emits `RUNTIME80_SINGLE_PROBE_GLYPH_BYTE_ACCOUNTING_BENCH_V1` with insert/key/bitmap
counts, structural probe reductions, paired P50/P95 timings, and raw samples for coordinator-owned
WeCom reporting.

## Validation

Rust 1.94.1 `rustfmt --check`, scoped diff checks, and Runtime80 source-structure gates are required
before submission. This task is queued in the Runtime80 three-task asynchronous validation batch,
filtered by `runtime80_batch_`, together with borrowed protected-glyph set construction and
allocation-free primary-family matching. The batch runs four behavior tests, two ignored release
benchmarks, and one standalone family-match model. Dynamic P95 evidence, integration SHA, and
automatic WeCom delivery remain coordinator-owned and pending.

## Remaining Parent-plan Work

Runtime80 still requires font artifact cooking, session-owned collection service, generational
face/instance leases, typed glyph completeness, cross-cache pressure recovery, and product-scale
qualification. This hash-probe optimization does not claim those milestones complete.
