---
title: Editor03 Adaptive Renderable Owner Dedup
category: zircon_editor
report_id: Editor03-adaptive-renderable-owner-dedup-2026-08-27
date: 2026-08-27
session_id: root-editor03-adaptive-renderable-owner-dedup-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor03 Adaptive Renderable Owner Dedup

## Scope

This slice closes the duplicate-candidate part of P1-24 when one selectable owner contributes
non-adjacent render meshes. It does not claim aggregate owner bounds, renderer-owned selectable
records, occlusion-aware box selection, or a spatial picking index are complete.

## Implementation

`renderable_candidates` retains the existing adjacent-owner fast path for the normal grouped render
extract. It now detects the first decreasing owner ID and only then builds a `HashSet` from the
candidates already emitted. Once the sequence has demonstrated that it is not grouped by owner,
the lazy set rejects later non-adjacent duplicates while preserving first-candidate order and
representative geometry.

This avoids an unconditional hash allocation and hash lookup on grouped input. Focused Rust
regressions lock both properties: grouped owners leave the fallback set absent, while an
interleaved sequence produces one candidate per owner.

## Performance Evidence

Windows-native release modeling uses 8,192 owners, eight primitives per owner, eight downstream
selection queries, 21 alternating samples, three inner runs per sample, median inner-run selection,
and `QueryThreadCycleTime`. Baseline and optimized checksums both equal `6240534528`.

| Input | Candidate rows | Allocations | Requested bytes | P50 cycles | P95 cycles |
| --- | ---: | ---: | ---: | ---: | ---: |
| Grouped baseline | 8,192 | 9 | 1,704,064 | 3,700,248 | 3,972,697 |
| Grouped optimized | 8,192 | 9 | 1,704,064 | 3,736,661 | 4,531,332 |
| Interleaved baseline | 65,536 | 9 | 9,961,600 | 80,198,144 | 107,264,100 |
| Interleaved optimized | 8,192 | 10 | 1,851,536 | 7,977,654 | 8,876,445 |

Grouped input keeps allocation count and requested bytes unchanged; measured P50/P95 overhead in
the latest run was 1.0%/14.1%. Interleaved input reduces candidate rows by 87.5%, requested bytes by
81.4%, and P50/P95 cycles by 90.1%/91.7%. Its one additional allocation is the lazy owner index.

## Validation

- The four-test Python source contract passes locally.
- Rustfmt 1.94.1 and scoped `git diff --check` pass locally; diff-check reports only the existing
  LF-to-CRLF checkout warning.
- The independent release model passes the grouped-input no-allocation-regression and interleaved
  reduction targets locally.
- Two focused Rust regressions and the repeated release model remain pending in one
  coordinator-managed validation ticket. No direct Cargo command was run.

## Remaining Parent-plan Work

P1-24 still requires renderer-owned aggregate per-selectable bounds and generation-stable owner
mapping. P1-23 still requires a visibility-aware spatial query rather than an O(N) rectangle scan.
