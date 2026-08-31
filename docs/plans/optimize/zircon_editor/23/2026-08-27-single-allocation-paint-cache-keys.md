---
title: Editor23 Single-allocation Paint Cache Keys
category: zircon_editor
report_id: Editor23-single-allocation-paint-cache-keys-2026-08-27
date: 2026-08-27
session_id: root-editor23-precompiled-collection-type-traits-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Single-allocation Paint Cache Keys

## Scope

This slice removes intermediate and growth allocations from image-pixel and circular-progress
paint cache-key construction. It does not claim the parent plan's UI authoring, runtime atlas,
preview memory accounting, or large-document performance milestones are complete.

## Implementation

`image_pixels_cache_key` now computes the exact final byte length, allocates one `String`, and
writes the optional raster size and tint directly into it. The previous implementation allocated
separate size and tint strings before allocating the final key.

`circular_progress_image_key` now precomputes the prefix, decimal-size, separator, percentage, and
color field lengths before writing into one `String`. This removes the growth allocation observed
in the formatted baseline while preserving the existing lowercase, fixed-width hexadecimal wire
format. Focused Rust regressions lock both key formats and assert that the resulting string did not
grow beyond its planned capacity.

## Performance Evidence

Windows-native release modeling uses 65,536 rows, 21 alternating samples, three inner runs per
sample, median inner-run selection, and `QueryThreadCycleTime`. Checksums match between baseline
and optimized implementations.

| Key workload | Baseline allocations | Optimized allocations | Allocation reduction | P50 cycle reduction | P95 cycle reduction |
| --- | ---: | ---: | ---: | ---: | ---: |
| Image pixels | 262,144 | 65,536 | 75.0% | 60.5% | 59.6% |
| Circular progress | 131,072 | 65,536 | 50.0% | 29.7% | 27.7% |

Latest measured image cycles were `100601735 / 118660613` baseline P50/P95 and
`39777115 / 47984103` optimized. Latest circular-progress cycles were
`75885927 / 100486755` baseline and `53333732 / 72689686` optimized. Image checksum was
`8117641183536281240`; circular-progress checksum was `16616816648434303998`.

## Validation

- The four-test Python source contract passes locally.
- Rustfmt 1.94.1 and scoped `git diff --check` pass locally; diff-check reports only existing
  LF-to-CRLF checkout warnings.
- The independent release model passes both allocation and cycle targets locally.
- Focused Rust tests and the repeated release model remain pending in one coordinator-managed
  validation ticket. No direct Cargo command was run.

## Remaining Parent-plan Work

The visual-asset cache still needs unified memory accounting, hit/miss/eviction telemetry, and
runtime atlas evidence under the parent Editor23 and Runtime11C milestones.
