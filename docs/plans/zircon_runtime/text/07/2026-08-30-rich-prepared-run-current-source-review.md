# Rich prepared-run current-source review

Date: 2026-08-30

Status: `RRT-P1-034_paint_projection_profile_infrastructure_static_complete /
rich_renderer_style_admission_parity_static_complete /
resolved_run_visual_slice_congruence_static_complete /
RRT-P1-036_prepared_block_owner_cutover_designed_managed_baseline_pending`

## Scope

This review follows the current Runtime Text layout-to-paint-to-render path before changing the
resolved DTO, adding a cache, or moving string ownership. It supersedes the 2026-08-26 premise that
every non-inline rich paint run necessarily invokes the shaping backend again.

## Current-source correction

`ResolvedRichTextArtifact` already owns one process-local composite product:

- the immutable `CompiledRichText` metadata artifact;
- the immutable generation-bound `ResolvedTextGlyphArtifact`;
- the exact resolved line snapshot used for stale detection;
- a line/run directory that maps source and visual ranges to glyph slices and style-source ranges.

`rich_text_glyph_artifact_runs` walks layout runs and the directory together. A valid artifact route
passes the owned glyph slice to the renderer; it does not call renderer fallback shaping. Fallback is
limited to intentional `VisualOnly` lines or missing/stale/incomplete artifacts whose text is still
source-isomorphic. A rejected non-isomorphic route is not reshaped from a semantically different
string. Existing route counters distinguish artifact, visual-only, missing, stale, incomplete, and
fallback work.

The remaining RRT-P1-034/036 costs are different:

1. `UiResolvedTextLine` and `UiResolvedTextRun` own serializable `String` values.
2. `UiRenderCommand::text_paint` materializes another `UiTextPaintRun` vector and clones run text,
   color, font, and family strings.
3. The renderer uses the composite directory for glyph identity but resolves inline/style metadata
   from `CompiledRichText::run_for_range`, an `O(log runs)` checked lookup.

This is duplicated presentation residency and style projection, not repeated stable-run shaping.

## Unreal boundary

Local Unreal Slate keeps a shared full line string in `FSlateTextRun`, stores style and range on the
run, creates layout blocks that retain the run, and paints a shaped subsequence obtained from the
line-owned `FShapedTextCache`. It does not reconstruct a second independent run string to recover
style or glyph identity at paint time.

The corresponding Zircon end state remains one runtime-private prepared run/block product consumed
by local paint and rendering. The serializable `UiResolvedTextLayout` must remain a projection for
cross-boundary consumers rather than becoming the runtime's only glyph/style owner. This migration
cannot be implemented as an unmeasured `String` to `Arc<str>` substitution because serde, command
snapshot, remote/debug, cache residency, and retained-generation behavior share that contract.

## Existing evidence and missing profile

Current instrumentation already exposes:

- layout-cache heap estimates for serializable lines, runs, text, and advances;
- resolved renderer batch count, text bytes, and advance bytes;
- rich artifact-route and renderer-fallback request/source-byte counts;
- shaped-cache hit/miss and generation/stale receipts.

The missing evidence is the managed phase-local timing/allocation sample and stable repaint retention
result. The managed E-drive profile must use 1/100/1,000/10,000 styled runs across Latin,
CJK fallback, RTL/BiDi, font override, wrapping, inline objects, Native, and SDF routes. Record 60
warmup plus 300 measured frames for cold layout, first paint, and stable repaint, including p50/p95/
p99, allocation count/bytes, layout-cache bytes, renderer batch bytes, fallback/backend calls, RSS,
GPU timestamps, and a separate power trace.

## Implemented profiling slice

- `render/paint_projection.rs` brackets only real transient text-paint materialization with the fixed
  `text.paint_projection/materialize_transient_text_paint` scope. Non-text commands do not enter it.
- One saturating report records text/rich command counts, generated paint elements, layout lines/runs,
  paint runs, source/run text bytes, style-string bytes, and rich-only layout/paint run and text-byte
  counts. Twelve fixed counter names contain no source text, command identity, or dynamic label.
- Direct planning aggregates the complete call. Segment-cache planning counts only rebuilt segments;
  a complete cache hit explicitly publishes zero new projection work instead of replaying a cached
  report.
- The byte counters are materialized payload-length lower bounds, not allocator count/capacity claims.
  Actual allocation/RSS requires the managed trace described above.

The failing-first contract and complete reproducible Runtime Text static suite pass 52/52. Rustfmt
parses the new child and both call sites. Managed Rust execution and dynamic profile capture have not
run.

## 2026-08-31 resolved-presentation owner correction

The current-source review found a correctness consequence of the duplicate presentation owner. Rich
layout calls `resolve_rich_run_style`, which ignores non-finite/non-positive `font_size` and an empty
font family. `UiRenderCommand::text_paint` then projects only the base command presentation into every
`UiTextPaintRun`; the renderer reopens `CompiledRichText` and previously applied the raw overrides
without those admission rules. Layout geometry could therefore use the base font while fallback or
raster metadata used size `0.0` or an empty family.

The renderer-side compatibility boundary now applies the exact two layout admission predicates before
using those raw fields. A focused Rust regression constructs the same invalid overrides and requires
the already-laid-out base size, line height, and family to survive. The new static contract went RED
before the repair and is GREEN afterward. The adjacent projection admission now also prevents an empty
failed rich paint projection from falling through to generic plain layout batches; a present rich
layout must pass the existing run-cardinality check. Focused contracts are 6/6 and the complete Runtime
Text static suite is 94/94. The Rust
regression is source-present but has not run under managed Cargo, so this is static implementation
evidence rather than runtime acceptance.

The retained-layout audit also found that the compatibility paint projection trusted each run's text
independently from its visual range. It now publishes no paint DTO unless every nonempty run is contiguous,
equals the exact UTF-8-safe slice of `line.text`, and reaches the line visual end. Empty metadata runs remain
ignored and scalar-aligned style boundaries inside a grapheme remain legal. This adds one monotonic
`O(lines + runs)` admission pass, not another style/geometry cache; the focused contracts remain 6/6 and the
complete static suite remains 94/94. The two Rust regressions are source-present and unexecuted under managed
Cargo.

This repair is deliberately not the terminal architecture. Local Unreal Slate confirms the retained
owner boundary in `TextLayout.cpp`, `TextLayout.h`, `ILayoutBlock.h`, `ISlateRun.h`, and
`SlateTextLayout.cpp`: a line view retains layout blocks, every block retains its run and measured
geometry, and paint invokes that retained run or its registered renderer. Paint does not look up the
parser model again to reconstruct the run style.

## Decision gate and dependency order

The correctness owner cut and the residency optimization are separate gates. Removing parser-artifact
style reconstruction is required for single-owner behavior; changing serialized DTO/string residency
still requires the matched allocation/timing profile. Execute in this order:

1. Capture the existing 31-sample paint-projection baseline on the approved E-drive target. Record
   layout/paint run counts, parser style lookups, allocations/bytes, p50/p95/p99, RSS, and fallback
   requests for 1/100/1,000/10,000 styled runs.
2. Introduce one generation-bound runtime-private prepared block/run product in the layout owner. It
   must pair source/visual ranges and final block frame with the resolved presentation, typed inline
   object, canonical glyph slice, and virtual/source provenance used by layout.
3. Make local paint/render consume that prepared product directly. Keep `UiResolvedTextLayout` only as
   the serializable cross-boundary projection; it must not remain the runtime's sole style/glyph owner.
4. Remove renderer `CompiledRichText::run_for_range` style/inline reconstruction in the same hard cut.
   Do not add a second presentation cache, optional compatibility style field, or parallel parser route.
5. Preserve stale-generation failure, visual-only semantics, inline objects, BiDi, Native/SDF parity,
   hit-test/caret geometry, and artifact-route fallback receipts. A cardinality or generation mismatch
   rejects the command before materialization.
6. Re-run the matched profile. Only if DTO projection or retained strings are material may the
   serialized projection become lazy at explicit cross-boundary points; that performance change must
   remove the replaced eager owner in the same migration.

Managed Cargo/profile/power/WGPU/PNG evidence is still pending. No performance, allocation, power,
or Unreal parity claim is made.
