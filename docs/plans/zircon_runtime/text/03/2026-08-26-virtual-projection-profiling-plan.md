---
title: Text03 Virtual Projection Profiling Plan
status: implementation_in_progress_static_checked_not_measured
scope: zircon_runtime text artifact and UI layout
---

# Text03 Virtual Projection Profiling Plan

## Decision Record

This record precedes any cache-capacity, allocation, or power optimization for
virtual text. It records a current-source algorithm review, not a benchmark
result and not an acceptance claim.

The initial visual-run projection design would have located runs separately for
each shaped glyph. That topology can become `O(G * R)` for `G` glyphs and `R`
visual runs. It was rejected before measurement. The implemented final-LTR
ellipsis path uses monotonic run-to-cluster and glyph-to-cluster cursors, giving
`O(G + R)` traversal after shaping. It neither weakens `ShapedRunCacheKey` nor
aliases distinct source fragments.

Final-LTR lines whose complete visual runs are LTR retain the physical
visual-run fallback path when no typed sidecar can be captured. A source-
congruent Horizontal Plain virtual line now retains a private logical
`LogicalVirtualLineSequence` before UAX#9 materializes physical text. Layout
shapes one canonical logical fragment for its real metrics and grapheme
advances; artifact build projects that same current-generation fragment through
captured visual indices. RTL, Arabic tatweel, and mixed direction therefore
never re-shape physical text as local LTR input. Rich/VerticalRl,
non-isomorphic source runs, and rejected cross-anchor or direction-boundary
glyph clusters remain on the renderer fallback.

## Instrumented Topology

The capture must use the existing runtime counters:

| Counter | Expected meaning |
|---|---|
| `physical_line_fragment_initial_shape_request_count` | Canonical final source fragments created by Text03. |
| `artifact_build_retained_fragment_projection_count` | Source-congruent normal lines projected without an artifact shape request. |
| `artifact_build_fallback_shape_request_count` | Conservative paths still requesting artifact shaping. |
| `artifact_build_visual_projection_shape_request_count` | LTR virtual visual-line shapes, distinct from normal-fragment duplication. |
| `logical_virtual_fragment_shape_request_count` | Layout requests for canonical logical virtual fragments. |
| `artifact_build_retained_logical_virtual_fragment_projection_count` | Current-generation logical virtual fragments projected without an artifact shape request. |
| `artifact_build_logical_virtual_projection_shape_request_count` | Artifact fallback shapes only when the retained logical fragment is missing or generation-stale. |
| artifact shaped-cache hit/miss counters | Exact cache topology, not elapsed time. |
| font-handle registration batch/lock counters | Cost of renderer-handle projection. |

The virtual projection should be measured separately from normal source
fragments. A visual shape for a generated ellipsis is expected work; it must not
be counted as a duplicate normal-line shape.

`virtual_ellipsis_projects_the_retained_logical_fragment` is a profiling-feature
regression contract for the stable-generation Horizontal Plain fixture. It
requires one canonical logical-fragment layout request, one retained artifact
projection, and zero logical-virtual or generic artifact fallback shape
requests. It validates counter topology only; it is not a timing, allocation,
power, or renderer acceptance result.

## Matrix

For each named case, capture three repetitions after 60 warm-up frames and 300
measured frames:

| Case | Sizes | Required assertions |
|---|---|---|
| Plain Latin source-congruent lines | 1, 100, 1k, 10k labels | retained projection grows linearly; artifact fallback requests remain zero for matching lines. |
| LTR ellipsis virtual lines | 1, 100, 1k, 10k labels | one canonical layout request and one retained artifact projection per typed line; zero artifact virtual re-shapes at a stable generation and zero source-range drift. |
| CJK and fallback | 1, 100, 1k, 10k labels | no regression in fallback selection or font generation handling. |
| RTL and Arabic tatweel | 1, 100, 1k, 10k labels | logical sequence preserves contextual shaping and zero-width anchors; physical-LTR artifact admission remains forbidden. |
| Ligature and combining clusters | 1, 100, 1k, 10k labels | cluster source ownership and final advances remain exact. |

For every case record CPU layout and artifact p50/p95, resolved GPU time where
available, shaped-cache deltas, font-handle registration deltas, allocation
deltas, native/SDF raster work, and upload bytes. Use the same font generation,
source corpus, renderer route, and machine for before/after comparisons.

## Acceptance Boundaries

The CPU recorder can establish topology and locate regressions; it cannot by
itself establish power use or equivalence to another engine. Power needs the
separate platform-owner sampling window. No timing or power claim is permitted
until a managed validation run yields a durable receipt.

The product WGPU framebuffer is an independent functional gate. It must render
the actual scene, be pixel-inspected, and write only under
`docs/tests/runtime/text`. A strategy text image, historical PNG, or any image
under `target` is not evidence for this plan.

## Next Architectural Gate

The horizontal Plain sequence now owns a current-generation canonical fragment,
but the implementation remains unmeasured. Next extend the same typed
logical-fragment ownership to rich and VerticalRl without leaking it into
public UI DTOs or weakening source-fragment cache identity. Managed Cargo,
profiler samples, power data, and a real product WGPU framebuffer remain
required before any performance or acceptance claim.
