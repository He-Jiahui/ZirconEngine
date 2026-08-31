---
record_kind: architecture_and_performance_research_plan
status: measurement_contract_owner_collection_observability_repair_implementation_complete_secondary_review_complete_coordinator_atomic_staging_required_managed_validation_pending
created_at: 2026-08-09
owner_plan: docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
related_code:
  - zircon_runtime/src/text/cache
  - zircon_runtime/src/text/layout_session.rs
  - zircon_runtime/src/text/parallel
  - zircon_runtime/src/text/native_bitmap_atlas
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/measure_cache/retained_document.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/text_prewarm.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/graphics/runtime/render_framework/frame_profiler.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/ShapedTextCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateTextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateTextBlockLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Test/TextLayoutTest_LazyGeneration.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/{FontCache,SlateSdfGenerator}.cpp
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/bevy/crates/bevy_text/src/{pipeline,font_atlas_set}.rs
  - dev/Fyrox/fyrox-ui/src/formatted_text.rs
  - dev/slint/internal/core/textlayout.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - docs/plans/zircon_runtime/text/09-threading-caching-and-performance.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
---

# Text Pipeline Performance Architecture and Profiling Plan

## Status and Scope

This is a research and implementation plan, not an acceptance record. It was produced from a
current-source review on 2026-08-09. No Cargo, WGPU, screenshot, CPU-time, GPU-time, or power
measurement is claimed here. The required real text framebuffer proof remains coordinator-owned
and may only be written to `docs/tests/runtime/text`, never to `target`.

The immediate goal is to make the existing MVP text path measurable end to end, then improve the
stage proven to dominate a representative trace. It is intentionally not a proposal to replace
the text system, add a second cache hierarchy, or add threads before a trace warrants them.

## Current-Source Findings

The current architecture is already materially more mature than the historical Text09 problem
statement. The following items are present in production source and must be preserved.

| Stage | Current owner and invariant | Performance consequence |
| --- | --- | --- |
| Shaping | `SharedTextLayoutSession` stores canonical `Arc<ShapedGlyphRun>` values. `SharedTextLayoutService` projects neutral DTOs only at the framework boundary; stable font generation retries are bounded to two attempts. | Internal UI layout avoids a glyph-by-glyph DTO round trip and cannot spin indefinitely during a font reload. |
| Text caches | `IndexedTextCache` provides stable slots and linked LRU. Measure, layout, and shaped-run caches retain exact-text collision guards, width validity, byte caps, and direction aliases. | Steady lookup/touch/eviction is bucket-local or constant work; lowering capacity or accepting hash-only hits is not an optimization option. |
| Shape prewarm | `text_prewarm.rs` gathers applicable commands once, uses the shared process compute pool, and `shape_pool.rs` deduplicates exact pending misses before one join. Plain horizontal requests stay source-isomorphic; normal rich/vertical requests project canonical visible hard lines with the base style, while inline rich requests reuse resolved layout spans under the same canonical hard-line/run-cap boundary. Batches below eight jobs remain inline. | The known historical double-prewarm and fixed extra pool policies are not present. A trace must decide whether remaining prewarm materialization or the actual shaper is expensive. |
| Surface invalidation | `UiSurface::rebuild_dirty` returns before layout, arrange, hit-grid, render extract, and text-frame work when no dirty domain is present. Dirty text/style/layout changes use incremental layout and node-level render patching when their ownership invariants hold; `UiSurface::rebuild` is the explicit full surface-projection rebuild route. | A forced `rebuild()` trace is a worst-case arranged-tree, hit-grid, and render-extract workload, not a layout-recomputation proof or a steady retained-frame measurement. The two paths must be captured and reported separately. |
| Owner document layout | Owner commands retain `TextDocumentKey` plus `UiTextViewport`. The parsed Plain owner and hard-line index reject same-key/different-source aliases; hard-line stable hits share and pointer-match the compiled source `Arc`. `UiTextMeasureCache::resolve_or_shape` bypasses persistent layout reuse only for a strict Plain/HorizontalTb/None/Clip viewport subset. | Hard-line lookup no longer trusts revision alone, but render extraction still materializes a fresh `String`, forcing parsed-source exact qualification. The controlled trace must separate source materialization/qualification from visible-line layout before selecting an optimization. |
| Native raster | `TextRasterWorkerPool` has bounded request and completion queues. Source-cache work shares `Arc<[u8]>` font data by face epoch and drains under item/byte budgets. Queue pressure follows placeholder/defer paths. | Increasing worker count or queue capacity cannot be the first response; it risks hiding backpressure and increasing latency or power. |
| Atlas and draw | Native source cache, persistent slots, page shadow, dirty upload, and instance rendering expose cache, byte, upload, and draw counters. A continuous same-contract atlas run is one draw with one 68-byte instance per glyph. | CPU quad expansion, full-atlas upload, and per-glyph draw submission are already excluded by deterministic contracts. |
| Render observation | `FrameProfiler` already joins CPU submit timing, WGPU timestamp results, pass metrics, and UI `RenderStats`; `ScreenSpaceUiTextPrepareReport` projects raster/cache/upload health. | The generic render profiler must be reused rather than creating a competing timer or benchmark framework. |

The source-level M0 observation boundaries cover caller-thread owner request collection,
overlap admission, command collection, shaping, prewarm, layout, and GPU preparation. The two
owner scans have fixed `ui_text.extract` stages so their p95 cost cannot be attributed to shaping.
The 2026-08-26 review also found that `UiNodeVisualData::resolve` and `UiRenderCommand` do not yet
share a pointer-stable source snapshot with `text_layout_revision`. Correct same-key qualification
is implemented at both retained consumers, but the parsed owner still performs an exact source scan.
The first controlled capture must therefore report this extraction/qualification cost; a future
source-owner rewrite is selected only if that evidence dominates, and never by adding a per-frame
whole-document hash.
The remaining gap before an actual bottleneck can be named is a controlled Windows capture:
isolated helper timings cannot attribute queue delay, WGPU work, or power for a real UI frame. The
existing owner route resolves after the one post-collection
`prewarm_render_command_text(...)` pass, with source-isomorphic paragraphs for plain horizontal,
canonical visible hard lines with the base style for normal rich/vertical, and resolved layout
spans for inline rich.

Therefore no algorithmic bottleneck is currently confirmed. Calling shape, cache, raster, upload,
or GPU draw the bottleneck before the first controlled capture would be speculation.

## Reference-Engine Synthesis

Unreal is the primary architectural reference. Slate separates a shaped-text cache keyed by
range, scale, shaping context, and font identity (`ShapedTextCache.h`) from `FSlateFontCache`,
which owns glyph atlas lookup, deferred flushing, and per-frame cache update. Its SDF generator
uses a bounded reusable task pool with explicit `BUSY`, placeholder, completion-update, and
flush semantics (`SlateSdfGenerator.cpp`). Zircon already has the corresponding ownership split:
the remaining work is to expose and tune it with evidence, not merge these owners into the
renderer root.

Godot supplies the second engine boundary: its `TextServerAdvanced` owns a persistent shaped-text
object, then derives line breaking, caret, selection, and glyph access from that object. Its font
cache is keyed by physical size and retains per-glyph atlas placement. Zircon's
`Arc<ShapedGlyphRun>`, `TextDocumentKey`, physical `raster_scale`, and raster keys should remain
the equivalent immutable-shape and physical-raster boundaries. Zircon must not take Godot's
on-demand render-thread glyph rasterization path, because its worker/backpressure model is more
appropriate for a responsive retained UI.

Bevy provides the Rust/wgpu implementation check. `TextPipeline` constructs one computed layout,
then walks positioned glyphs using `FontAtlasKey` containing font identity, physical size,
variations, hinting, and smoothing. It lazily builds one scaler for a compatible run when a glyph
misses. This reinforces Zircon's physical-key and compatible Swash scaler batching contracts.
Slint independently reinforces the layout rule: shape once into a buffer, then use that buffer for
measurement, line breaking, elision, hit testing, and positioned-glyph iteration.

Slate's retained path supplies the primary invalidation comparison. `FSlateInvalidationRoot` keeps
separate pre-update, prepass, and post-update heaps, builds `FinalUpdateList`, and
`PaintFastPath` consumes only that list while preserving cached element data; it falls back to the
slow path only when the invalidation contract requires it. `FSlateTextBlockLayout` independently
tracks text snapshots and marshaller dirtiness, then calls `TextLayout->UpdateIfNeeded()` instead
of reconstructing content unconditionally. Its `TextLayoutTest_LazyGeneration.cpp` test proves
that lazy layout computes line estimates first and materializes visible line views on demand.
This is evidence for Zircon's existing `rebuild_dirty` and viewport-document ownership, not a
reason to import Slate proxy types or create another retained tree.

The Rust comparison narrows implementation choices. Bevy retains reusable section/text buffers,
stores the computed layout separately from `TextLayoutInfo`, and retains vector capacity while
reprojecting positioned glyphs. Fyrox's `FormattedText` likewise owns persistent lines and glyphs
and explicitly reuses the line allocation during measure. These designs support keeping Zircon's
canonical layout/cache owners and measuring materialization and traversal before adding a new
cache or thread boundary.

The derived Zircon rule is:

```
UiRenderCommand collection
  -> existing frame dedup / persistent layout cache
  -> immutable shared ShapedGlyphRun
  -> physical native or SDF raster cache
  -> persistent atlas slot/page owner
  -> WGPU instance draw and frame profile
```

Every arrow has one authoritative owner. A new facade cache, an alternate text renderer, or a
second shaping backend is prohibited. Cache invalidation stays at the existing document revision,
font generation/face epoch, physical raster key, and atlas generation boundaries.

## M0: Measurement Foundation

M0 is the first implementation milestone. It changes no layout or raster algorithm.

1. Retain fixed-name profiler scopes at existing boundaries only:
   `ui_text.extract`, `ui_text.prewarm`, `ui_text.layout_resolve`, `text.shape_batch`,
   `ui_text.prepare`, `ui_text.native_raster_plan`, `ui_text.atlas_upload`, and
   `ui_text.sdf_prepare`. `ui_text.extract` separately retains
   `owner_prewarm_request_collection`, `owner_prewarm_overlap_admission`, and
   `render_command_collection` so the two caller-thread scans cannot be attributed to shaping.
   Use `profile_scope!`, never dynamically generated per-node names.
2. At those boundaries, project existing reports through fixed-name `profile_counter!` values:
  shape-cache hit/miss and caller wait, layout-cache hit/miss, uncached document layout resolves,
  source/slot cache hit/miss,
   worker pending/deferred/failed, upload bytes/copies, placeholder count, instance count, and
   draw-command count. Do not duplicate cache state or add a per-frame heap-owned report solely
   for profiling.
3. Add a focused profiling-feature regression that activates a bounded recorder, exercises a
   mixed native/SDF UI extract, and asserts the fixed stage paths and critical counters appear.
   It must also demonstrate that the feature-disabled macro path does not evaluate dynamic
   payloads, matching the existing profiler contract.
4. Keep code in the owner files or a folder-backed `text/performance/` leaf only if a real
   aggregation responsibility appears. `text.rs`, `render.rs`, and `prepare_report.rs` remain
   orchestrators; production files stay below the 800-line review threshold and tests remain
   folder-backed.

M0 decision gate: collect the baseline below before changing an algorithm. The top CPU hotspot by
p95/total, together with its counter trajectory, selects exactly one M1 path. No cache or thread
"optimization" may bypass this gate.

## M0 Implementation Status (2026-08-09)

Status: `measurement_contract_forward_fix_implementation_complete /
secondary_review_complete / managed_validation_pending`.

Completed source work:

- `shape_paragraphs_with_cache` now records the fixed `text.shape_batch` CPU span plus request,
  hit/miss, deduplication, shaped/inserted, generation-deferred, inline/join, caller-wait,
  chunk-size, and worker-parallelism counters.
- UI text preparation records fixed projections from the existing `ScreenSpaceUiTextPrepareReport`:
  input/resolved batches, native source/slot cache behavior, worker health, visible placeholders,
  native upload bytes/copies/requeues/failures, and SDF batch/slot/vertex/draw work.
- Owner-level CPU spans now distinguish the UI text prepare, native preparation, native bitmap
  raster planning, bitmap atlas submission/upload, SDF atlas planning, and SDF renderer
  preparation. The counters and their call sites are feature-gated, so a build without the
  `profiling` feature retains no profiling function call or report projection work.
- Focused feature-gated regressions lock the shape span/counter contract and the UI report
  projection contract. They deliberately use the existing recorder rather than a new benchmark
  harness.
- UI extraction now retains owner-only document revision, viewport, and editable metadata in a
  sparse command-indexed sidecar. Commands collect first, then source prewarms once through the
  shared compute pool and owner layouts resolve afterward from the preserved inputs. Empty/invalid
  owner input retains its prior layout behavior; normal rich/vertical source uses canonical
  visible hard-line base-style requests, while inline rich source uses the shared resolved-span
  projection under the same hard-line/run-cap invariant as layout. The rich/vertical integration
  regression uses adjacent Markdown runs to lock actual layout cache consumption after prewarm.
- The remaining UI M0 stages now emit the caller-thread `ui_text.extract` scopes
  `owner_prewarm_request_collection`, `owner_prewarm_overlap_admission`, and
  `render_command_collection`, plus `ui_text.prewarm` and `ui_text.layout_resolve`. Fixed
  counters project collected command/owner counts,
  prewarm shape outcomes, existing layout-cache/frame-dedup reports, and the shaped-run cache
  hit/miss/lookup/insert delta produced by layout after prewarm. The delta uses scalar report
  snapshots rather than a second cache or per-frame report allocation. A profiling-feature
  `UiSurface` regression asserts the caller-thread collection/admission, command, prewarm, and
  layout stage paths on one calling frame without introducing a separate benchmark harness. An
  empty prewarm batch now publishes the same seven fixed counters with zero values, so a
  stable-frame capture does not silently omit the stage.
- The focused Render17 regression
  `ui_text_prepare_profiles_mixed_native_and_sdf_batches` exercises a real WGPU UI frame with
  both native and SDF text, then asserts the fixed preparation stage paths and cache/raster/atlas
  counters through the existing bounded recorder.
- A Windows-only ignored baseline now implements the first three surface measurement-matrix rows through
  named `forced-full-rebuild`, `retained-steady`, and `localized-text-dirty` `UiSurface` paths plus
  real WGPU submit. The paths cover 1, 100, 1,000, and 10,000 label nodes with 60 warm-up frames plus 300 measured
  frames in each of three repetitions. The capture retains 65,536 samples, verifies exact
  frame-index coverage for owner request collection, overlap admission, command collection, and
  the downstream Text stages, distinguishes repeated forced Text work from a clean retained frame,
  enforces stable raster/upload zero-work, and exports current plus resolved WGPU frame profiles
  beneath the coordinator-owned D/E/F `CARGO_TARGET_DIR`. This is raw profiler evidence, not
  framebuffer proof, and it has not yet been executed.
- A separate ignored layout-cache pressure baseline drives the actual non-owner persistent-cache
  route with unique frame keys while bounding the 10k row to 512 shaped-text identities. The
  1/100/1k rows require settled persistent hits with zero miss; the 10k row requires every
  measured scan to exceed the 2,048-entry capacity and report positive deterministic misses.
  This synthetic capacity trace is kept separate from the production owner route.
- Product proof, UI text, font, SDF, and profiler-trace test-fixture work roots now derive from
  the workspace `docs/tests/runtime/text` directory rather than `std::env::temp_dir()`. They
  remain unique and removable; the accepted framebuffer still uses its fixed atomic PNG output
  path. The local ignore rules exclude only these hidden work roots, never accepted visual
  evidence.

Second review (2026-08-10): the Text09 owner sidecar, fixed M0 profile stages, retained
document/viewport/editable inputs, empty-source behavior, and source-isomorphic rich/vertical
boundary were independently re-reviewed with no P0/P1/P2 finding. Scoped Rustfmt and diff checks
remain clean apart from existing CRLF notices; this review does not substitute for managed Cargo,
WGPU, profiler, power, or framebuffer evidence.

Second review addendum (2026-08-10): UI text, font, SDF, product framebuffer, and profiler-trace
test fixtures now use unique, removable workspace-local roots beneath `docs/tests/runtime/text`.
The existing execution tests assert their SDF/project/profile export roots, and a scoped source
scan finds no remaining `temp_dir()` call in the text, UI-text, product framebuffer, or profiler
regression paths. The accepted framebuffer PNG remains absent until the managed real-render proof
succeeds.

Second-review correction (2026-08-10): a proposed normal-rich prewarm split at Markdown source
run boundaries was rejected before acceptance. `wrap_source_runs_with_provider` assembles source
runs into candidate lines, and `resolve_line_widths_with_provider` measures the resulting complete
candidate text. Prewarming canonical hard lines with the base style therefore matches the
non-wrapped layout key without speculative per-run variants. The rich/vertical owner regression
uses adjacent `plain **bold**` Markdown runs and asserts the canonical request, deduplication, and
post-layout cache consumption contract.

Second-review completion (2026-08-10): the empty-command path now produces the default prewarm
report and routes it through the same profiling projection, so all seven fixed counters are
present with zero values instead of disappearing from a stable-frame trajectory. The focused test
uses the real `prewarm_render_command_text` call path and bounded recorder. Independent static
review found no P0/P1/P2 in this correction; managed `ui + profiling` Cargo execution remains
pending and no runtime-performance result is claimed.

Static-label baseline second review (2026-08-10): the first Windows measurement row explicitly
enables synchronous WGPU timestamp collection and requires all 300 measured generations to return
resolved GPU profiles. It also requires the atlas renderer's native instance and draw counters to
be positive in every measured frame, in addition to the exact fixed-span coverage and zero stable
raster/upload work gates. Independent review of the current source found no remaining P0/P1/P2;
the ignored baseline has not been run and supplies no accepted runtime metric yet.

Measurement-contract correction (2026-08-10): source review found that the static-label harness
called `UiSurface::rebuild()` for every warm-up and measured frame. That route deliberately
rebuilds the arranged tree, hit grid, and complete render extract, whereas the production retained
contract lives in `rebuild_dirty()` and returns with zero owner visits on an unchanged frame. A
single result from the former therefore cannot name a steady-state Text bottleneck. The ignored
Windows baseline now runs two named scenarios on the same label-count matrix:

- `forced-full-rebuild` preserves the existing full surface-projection rebuild workload and requires all Text
  CPU stages and counters on all 300 frames. Every row requires persistent layout-cache misses to
  remain zero and separately requires `uncached_document_resolves == label_count`, making the
  retained-document bypass explicit instead of interpreting two zero cache counters as no work.
  The 10k row keeps shaped text identities bounded to fit the 1024-entry shaped-run cache while
  preserving 10k document/frame identities;
- `retained-steady` calls `rebuild_dirty`, requires zero layout/arrange/hit/render owner visits and
  no Text extract/prewarm/layout/shape samples, while still requiring positive native WGPU text
  instances/draws, resolved GPU timestamps, and zero raster/upload work.
- `localized-text-dirty` alternates one label between the already-resident `L0000` and `L0001`
  identities before each `rebuild_dirty` call. It records existing `UiSurfaceRebuildReport`
  node-visit, render-command, and phase-microsecond values as fixed profiler counters, and requires
  one-or-fewer layout/arranged/hit/render outer visits plus one-or-fewer rebuilt command per frame.
  This is the Zircon counterpart to Slate's selective fast-update list: a full arranged/render
  fallback at 100/1k/10k is rejected as a structural regression rather than hidden by cache hits.
- `layout-cache-pressure` directly drives the real persistent layout-cache route without document
  or viewport ownership. It preserves the original 1/100/1k settled-hit and 10k capacity-miss
  contract instead of pretending the production Plain-owner bypass exercised that cache.

This correction changes only the profiler experiment and plan record. It does not select or land
an algorithm optimization, and no result is claimed until coordinator-managed execution produces
the three repetitions for all four baseline paths.

Viewport-routing correction (2026-08-11): request viewport metadata alone is not a cache-bypass
signal. `resolve_or_shape` first uses the same hard-line query as layout: only a strict
Plain/HorizontalTb/None/Clip subset uses retained parsed source and frame-local dedup without
entering the persistent layout cache. Complete viewports and vertical text reuse the persistent
layout cache. `ui_text.layout_resolve.uncached_document_resolves` is sampled from the actual
partial-resolve branch, not from request metadata; focused complete, vertical, and partial-owner
regressions keep the routing and telemetry aligned.

Forced static-label captures therefore have no uncached document resolves. Their 1/100/1k document
keys fit the 2,048-entry persistent layout cache and require settled hits with zero misses. The 10k
document-key row deliberately exceeds that capacity and requires deterministic cache pressure
while retaining 512 shaped-text identities; it is a cache-capacity observation, not evidence that
viewport virtualization is active. No M1 change is selected until the corresponding managed p95
trace identifies a bottleneck.

Viewport hot-path follow-up (2026-08-11): the M5 route now rejects a virtualized-document probe
before parse/index allocation when a strict Plain owner has neither a canonical hard-line separator
nor an over-64-KiB shaping-cap split. This is a semantics-preserving infrastructure repair, not an
M1 algorithm choice: multi-line and over-cap text still use the exact shared hard-line query, and
the persistent layout-cache lookup precedes that query for a complete cached viewport. Static
regressions cover the separator/cap predicate and prove a second complete-viewport frame adds no
hard-line-index lookup. No timing, power, or cache-rate result is claimed until the managed matrix
runs.

Worker-font residency hypothesis (2026-08-10): `FontDatabase::face_bytes` already owns the
canonical `Arc<[u8]>` for a resolved `FontFaceId`, including lazy materialization of a `fontdb`
source. The raster source-cache currently receives a backend font id and materializes a separate
worker snapshot from the glyphon slice once per active face epoch. M0 now records the snapshot's
resident bytes and backend-face count beside the existing copied-byte counter. If managed traces
show that this duplication is a material CPU or long-session memory cost, M1 must route the
backend id through the existing `FontDatabase` face mapping and clone that canonical `Arc`; it
must preserve the current Swash request identity/epoch contract and must not create another font
byte cache or speculative eviction policy. No change is selected until the counter trajectory and
CPU profile demonstrate that cost.

M0 observability forward-fix (2026-08-13): `ScaleFactorChanged` and subsequent physical resize
now travel through the runtime Winit translator and `UiSurface` input pump before `rebuild_dirty`
derives `render_extract.raster_scale`. The focused product-path fixture keeps one renderer across
the 1x-to-2x transition and font-generation change, asserting the updated metrics, physical
source-cache miss, and 2x extraction scale. The editor retained host is intentionally not part of
this chain because it owns no runtime `UiSurface`; its window presentation scale is not treated as
native text-cache evidence.

The native source-cache report now includes the actual cardinality of live persistent physical
`GlyphRasterKey` bindings. A successful worker completion derives and binds the key immediately
from the registered font database and cached image format, then the value is projected through the
screen-space text report, profiler, `RenderStats`, and product diagnostics. The direct source-cache
regression registers Fira Sans, completes a real glyph bitmap, and asserts one completed binding;
it does not use an input-string count as a cache-key substitute.

The ignored queue-pressure baseline uses 512 visible native glyph requests (64 glyph identities by
8 physical-pixel buckets), exceeding the production 256-request frame cap. Before its 300-frame
capture it repeatedly submits one pressure epoch until the actual `RenderStats` persistent-key
count reaches 512, bounded by 240 diagnostic frames. The measured window separately requires a
512 exact-miss peak, a 512 persistent-key peak, per-frame counter coverage, and observed frame-cap
defer or worker backpressure. This is measurement infrastructure only: no queue depth, cache size,
or raster algorithm was tuned. The `window::input` interface was also hard-cut into named context,
platform-event, normalization, kind, touch, and window-event owners; the public re-export paths
remain unchanged and the root is declaration-only.

Independent secondary review (2026-08-13): the runtime DPI chain, completion-time persistent-key
binding, bounded queue-pressure convergence, report/profile/diagnostics projections, and input
hard-cut found no P0/P1/P2. Scoped `rustfmt --check` and `git diff --check` passed. This is not
managed Cargo or WGPU validation and does not create, validate, or accept a framebuffer PNG.

Follow-up compile-forward-fix (2026-08-13): the sole complete
`ScreenSpaceUiTextRasterUploadReport` expectation now supplies and asserts a non-zero persistent
raster-key count from its source-cache fixture. A focused independent re-review found no
P0/P1/P2 in the report field visibility, production mapping, or explicit constructors. This is a
test-owner completeness repair only; it changes no raster policy, queue budget, cache lifetime,
or managed-validation state.

Not yet complete:

- No Cargo, WGPU, ETW, power, or wall-time command has been run in this session. Coordinator-owned
  Windows baseline capture of the forced-full-rebuild, retained-steady, localized-text-dirty, and layout-cache-pressure
  scenarios is required before an M1 algorithm change or any performance claim.
- The real framebuffer PNG remains absent; only a successful managed visual test may atomically
  create it under `docs/tests/runtime/text`. The fixture and writer test work roots are now also
  workspace-local under that directory, but this source-path correction is not screenshot
  evidence.
- Scoped `git diff --check`, focused `rustfmt --edition 2021 --check`, and a parse of the complete
  native bitmap atlas module tree pass. The former missing `frame/missing_raster.rs` child-module path was
  forward-fixed; shared renderer roots still have unrelated import-order drift and are not
  reformatted by this observability slice.
- The independent secondary review found no P0/P1. Its only final P2 was incomplete Rust 2021
  import formatting in the measurement-scope `extract.rs` and `render_profiling.rs` owners; both
  are forward-fixed and the complete M0 measurement file set now passes `rustfmt --edition 2021
  --check`. This source review remains distinct from managed Cargo, WGPU, profiler, power, and
  framebuffer acceptance.
- The post-review `localized-text-dirty` measurement owner initially retained its root's default
  `ContentDriven` layout boundary, so an invalidated label would have promoted the measured layout
  root to the whole tree. The source now sets `ParentDirected` on the `Free` root and has a direct
  `UiSurface` regression that validates one-node layout/arranged/hit/render traversal after a real
  `Text` invalidation. Fresh independent review found no P0/P1/P2. This remains a profiler-only
  experiment over the current `UiSurfaceRebuildReport`, not an invalidation or cache-policy change.

Second-review corrections (2026-08-09): CPU spans must end before their report counters are
written, because counter recording takes the global recorder and allocates snapshot names. The
M1 selection rule is therefore restricted to sibling leaf-stage spans, then confirmed by WPR/ETW;
the inclusive parent scope is diagnostic context rather than an algorithm ranking. A 300-frame
capture must explicitly set `max_counters` to at least `65_536` and export enough samples for all
observed stages (at least `300 * 19` UI report counters, plus `300 * 14` for each observed shape
batch). The recorder intentionally evicts oldest samples at capacity, so a default-capacity
snapshot is not valid evidence for the full matrix.

## Evidence-Selected M1 Paths

| Trace outcome | Allowed follow-up | Explicitly rejected shortcut |
| --- | --- | --- |
| `text.shape_batch` dominates with high unique misses | Reduce request materialization or improve paragraph-window scheduling while preserving exact dedup and one join; cache hit paths must remain allocation-free. | Raising the parallel worker count or introducing a second shaped-run cache. |
| `ui_text.layout_resolve` dominates while shaped cache hits | Optimize layout-result/window reuse or line-break work using the existing layout cache and document revision boundary. | Reusing stale geometry across frame, viewport, writing-mode, or wrap-width changes. |
| `ui_text.native_raster_plan` dominates with queue pressure | Tune budget partition or compatible scaler batches after checking source-cache misses and completion backlog. | Caller-thread synchronous raster or unbounded request/completion queues. |
| `ui_text.atlas_upload` or UI GPU pass dominates | Reduce dirty regions/copies or batch fragmentation inside atlas/page owners; validate actual timestamps and pixel output. | Full-page uploads, reordering mixed storage, or hiding misses with a screenshot-only test. |
| Render-framework lock time dominates | Route a narrow render-framework contention repair through Render17, retaining text counters for attribution. | Mislabeling a lock or device wait as a text algorithm defect. |

Each selected M1 change requires an exact source-level complexity contract, a regression for the
previous failure, scoped static checks, and an independent second review before coordinator
validation. A later M2 may tune budgets/hysteresis only from measured queue and residency data;
it is not authorized by this report alone.

## Windows Measurement Matrix

All runtime measurements and raw artifacts are coordinator-owned. No profile, ETL, Cargo target,
or screenshot may use `C:`. Raw WPR/ETW, Perfetto, and benchmark material belongs under an
allocated `D:`, `E:`, or `F:` coordinator directory; the only successful visual evidence belongs
under `docs/tests/runtime/text` after the real framebuffer assertions pass.

| Scenario | Warm-up / measured frames | Required observations |
| --- | --- | --- |
| Forced full-rebuild labels: 1, 100, 1k, 10k nodes with bounded stable text identities | 60 / 300, three repetitions | Text CPU stage p50/p95/p99, command count, zero uncached document resolves, 1/100/1k settled persistent layout hits with zero misses, 10k deterministic document-key capacity misses, shape hit/miss counts, zero steady raster/upload work, and UI GPU time. This is a surface-projection rebuild stress result, not a layout-recomputation proof or steady-state cost. |
| Retained static labels: 1, 100, 1k, 10k nodes | 60 / 300, three repetitions | zero UI layout/arrange/hit/render owner visits, no Text extract/prewarm/layout/shape samples, positive native instances/draws, resolved UI GPU time, and equal-scene power baseline. |
| Persistent layout-cache pressure: 1, 100, 1k, 10k unique layout keys; 512 shaped-text identities at 10k | 60 / 300, three repetitions | 1/100/1k settled cache hits with zero miss; 10k cache entries capped at capacity with deterministic positive misses; no document/viewport bypass. This is a synthetic CPU capacity trace, not the production owner path. |
| Continuous text: 1, 100, 1k, 10k glyphs | 60 / 300, three repetitions | instance bytes/draws, atlas upload bytes, UI pass timestamps, CPU plan time. |
| Scrolling list with 10% row turnover | 60 / 300, three repetitions | changed-row versus total-row shaping, cache/slot reuse, queue/backlog, upload copies and bytes. |
| Multilingual/rich/vertical | 60 / 300, three repetitions | fallback/defer counters, shaped/layout correctness, route stability, actual framebuffer pixels. |
| DPI 1x -> 2x and font-generation change | settled two frames plus 300 stable frames | exact physical raster miss at change, no stale cache hit, steady-state restoration and WGPU pixels. |
| Cache and queue pressure | 60 / 300, three repetitions | eviction/invalidation, budget rejection, bounded shutdown, no caller-thread raster or unbounded backlog. |

The tool chain is available on this Windows machine: the built-in Zircon profiler exports
hotspot/p95 summaries and Perfetto; WGPU timestamps feed `RenderFrameProfile`; `wpr.exe`,
`wpaexporter.exe`, `xperf.exe`, and `nvidia-smi.exe` are installed. Capture order is:

1. Zircon profiler and `RenderFrameProfile` first for stage attribution.
2. WPR/ETW only for a hotspot that survives the internal trace, to separate CPU execution,
   scheduling waits, allocation, and GPU queue behavior.
3. GPU power sampling only on the same fixed GPU, resolution, driver, power policy, and scenario;
   CPU/package energy requires a supported WPR energy source. Timestamp data alone is not power
   data.

The report must publish machine/driver/configuration, median-of-three p50/p95/p99, GPU timestamp
p95, cache/queue/upload counters, and energy if a valid sensor is available. It must never claim
that Zircon is "close to Unreal" from incomparable scenes or different hardware. A cross-engine
comparison needs the same workload, resolution, font set, quality settings, and hardware; until
then only Zircon before/after deltas are valid.

## Acceptance and Delivery

The deterministic MVP gates are independent of machine speed:

- a stable frame has zero avoidable shape miss, source-cache miss, placeholder, worker pending,
  and atlas upload work for an unchanged scene; the retained scenario additionally has no Text
  extract, prewarm, layout, or shape CPU stage because no owner was invalidated;
- a changed scroll window performs work proportional to entering/changed rows, not all rows;
- continuous glyph runs retain the existing one-instance-per-glyph and painter-order contracts;
- DPI/font changes invalidate exactly the physical or generation-sensitive cache boundary;
- real WGPU framebuffer assertions pass before a PNG is atomically written under
  `docs/tests/runtime/text`.

Time and energy gates are set from the measured baseline, not guessed in unit tests. The target is
to remove the selected hotspot from the top p95 contributor without moving equivalent work to an
unmeasured stage, increasing queue backlog, violating the deterministic gates, or materially
raising equal-workload power. The coordinator must then run the managed Windows Cargo/WGPU matrix,
attach quantitative before/after evidence, create the scoped milestone commit, and send one WeCom
summary containing the metrics and commit SHA.

## Current-Source Algorithm Re-audit (2026-08-10)

The current source was re-read across the canonical layout session, owner prewarm, shared shape
pool, and native source-cache boundaries, then compared again with Slate's shaped-text cache,
font-cache update/flush, and retained invalidation owners. The implemented Zircon flow still has
one canonical shape result, one frame-deduplicated prewarm join, one physical raster cache, and
one incremental surface rebuild route. No source evidence supports another cache, an additional
worker pool, or a renderer-root optimization.

Before the current test-owner extraction, `shape_pool.rs` was 794 lines and
`native_bitmap_atlas/source_cache.rs` was 798 lines. Both were at the 800-line soft guard, so a
new profiling, queue, or residency responsibility must first extract a named leaf owner rather
than extending either file. M1 therefore remains deliberately unselected until the managed
baseline identifies one top p95 leaf span and its matching cache/queue/GPU counter trajectory.
This re-audit records architecture evidence only; it claims no Cargo, WGPU, power, or framebuffer
result.

`shape_pool.rs` now uses its direct ownership-preserving split: the 374-line private test module
is hard-cut to `text/parallel/shape_pool/tests.rs` through an explicit path mount, retaining
parent-private access without a compatibility facade or behavior change. The parent production
owner is 418 lines after the structural mount. This is the only currently justified split;
`source_cache.rs` remains one cohesive cache owner until a new cache, worker-lifecycle, or
residency responsibility is actually selected by M1 evidence.

## Layout And Raster Boundary Re-audit (2026-08-10)

This direct audit covered `ScreenSpaceUiTextSystem`, resolved text batches, SDF CPU-frame reuse,
native bitmap-atlas handoff, and the render batch fidelity flags. It used Unreal Slate as the
primary reference (`FShapedTextCache`) and Fyrox `FormattedText` as the Rust-side stabilizer.
Slate keys a shaped sequence by exact text range, scale, shaping context, and font identity;
Fyrox retains line and glyph output as one mutable layout owner. Zircon's equivalent contracts
are already explicit: the shared layout session and glyph artifact own canonical geometry,
`ScreenSpaceUiTextBatch` carries its generation and raster scale, and the physical native/SDF
paths consume that resolved geometry rather than owning competing layout caches.

The native/SDF split is therefore a raster concern, not a second layout pipeline. Batches that
need SDF layout fidelity retain shaped artifacts and fail closed from unsafe native re-shaping;
local fallbacks preserve paired decoration metrics, and a whole-batch fallback invalidates the
CPU-frame snapshot before the next reuse. Glyphon remains a resilient raster fallback behind the
same font system while the bitmap atlas is unavailable or cannot represent a batch. Removing it,
adding a cache, or adding a worker pool is not an M1 candidate without a controlled M0 trace that
identifies the corresponding leaf-stage p95 and its cache, queue, or GPU counter trajectory.

The remaining evidence gap is empirical rather than architectural: coordinator-owned Windows
profiling must capture the M0 matrix, including WGPU timestamps and equal-workload power where a
valid sensor exists. Until then this report records the ownership boundary and rejects speculative
optimization; it makes no Cargo, WGPU, framebuffer, performance, or power claim.

## Direct Reference-Source Audit (2026-08-11)

This follow-up reads the reference owners themselves rather than relying on plan summaries. In
Unreal, `Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h` and its `.cpp`
implementation key shaped text by source range, scale, shaping context, and font identity; dirty
sequences are rejected before reuse. The measurement helpers reuse the complete shaped run first
and shape a narrower subrange only when that reuse cannot answer the request. In
`Engine/Source/Runtime/Slate/Private/Framework/Text/SlateTextLayout.cpp`, paint consumes the
layout pass's shaped/cached run blocks instead of introducing a second paint-time shaper. The
Rust-side comparison, `dev/Fyrox/fyrox-ui/src/formatted_text.rs`, keeps source, runs, retained
lines, and final glyphs in one `FormattedText` owner and reuses the line buffer during measure.

The current Zircon mapping remains deliberate: `SharedTextLayoutSession` is the canonical
shape/layout cache owner, glyph artifacts carry the resolved geometry across the raster boundary,
and `ScreenSpaceUiTextBatch` carries the route-sensitive generation and physical raster scale.
Native atlas and SDF paths therefore consume one resolved layout contract. This source comparison
does not select an M1 optimization and supplies no timing, power, Cargo, WGPU, or framebuffer
claim; the controlled M0 trace remains the prerequisite for choosing a measured leaf-stage
change.

## M2b Artifact Boundary Algorithm Re-audit (2026-08-24)

**Status:** `architecture_review_complete / measurement_plan_complete /
non_validation_correctness_implementation_complete / M1_unselected /
managed_validation_pending`.

This re-audit covers the path changed by P1-13 M2b rather than treating a typed failure repair as
evidence of a performance win:

`SharedTextLayoutSession::resolve_or_shape_outcome` -> `UiTextMeasureCache` ->
`layout_parsed_text_with_provider_and_viewport_outcome` ->
`build_resolved_text_*_glyph_artifact` -> render-command/atlas preparation.

The canonical session admits only `Ready(Arc<ShapedGlyphRun>)` into `ShapedRunCache`. The layout
owner now preserves the same disposition while attaching a glyph artifact: a stable renderer-only
case may be `Ready(None)`, but an artifact shaper failure or font-generation change remains
`Failed` or `Deferred` and never becomes `Ready(UiResolvedTextLayout)` for cache admission. This
is a correctness and ownership repair, not a new cache, retry loop, worker pool, layout pass, or
renderer-root shaper. The only added leaf owner,
`ui/text/layout_engine/artifact.rs`, makes the publication decision explicit and keeps the root
at 778 lines.

The 2026-08-26 source/range re-audit narrows `Ready(None)` further without selecting a performance
optimization. A source that cannot own the layout's absolute range, a line/run range outside that
layout owner, or a byte range that splits UTF-8 is an internal layout invariant failure and now
returns `Failed(LayoutFailed)`. Only a valid visual-only DTO remains eligible for `Ready(None)`.
This prevents invalid geometry from reaching cache publication and later becoming an untyped blank
renderer result. The focused tests live in the 116-line `glyph_artifact/tests/invariant_failures.rs`
child; formatter, whitespace, call-path, and file-budget checks are the current evidence. The
Plain resolved-glyph renderer receipt, managed Cargo, M0 trace, power capture, WGPU framebuffer,
and PNG were still open at that point, so no timing or acceptance conclusion followed from this
correctness repair.

The 2026-08-26 Plain MVP follow-up now makes that resolved-glyph renderer boundary typed without
adding a renderer shaper or another cache. Planning classifies each eligible command as artifact,
valid visual-only, source-isomorphic fallback, or rejected, and records missing/stale/incomplete
reasons separately. The report survives a rejection-only plan, flows through
`ScreenSpaceUiTextPrepareReport`, and publishes seven
`ui_text.resolved_glyph_artifact_route.*` counters. This gives M0 a direct way to distinguish a
canonical artifact hit from fallback pressure or lost output instead of inferring it from an empty
batch vector. The type and metric names intentionally say resolved glyph artifact; compiled-rich
paint remains a separate advanced route and must not be claimed by this MVP receipt.

Focused static regressions cover all four route dispositions, all three rejection reasons,
prepare-report transport, and profiler projection. Scoped formatter, whitespace, production
exception, call-site, and file-budget checks pass. Status remains
`non_validation_correctness_implementation_complete / M1_unselected /
managed_validation_pending`; compiled-rich parity, managed Cargo, controlled M0 samples, power,
real WGPU output, and the validated PNG remain open.

### Reference Evidence And Intentional Mapping

| Reference | Direct source evidence | Adopted constraint |
| --- | --- | --- |
| Unreal Slate | `Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h` keys a shaped sequence by source range, scale, shaping context, and font identity; `.../Private/Framework/Text/ShapedTextCache.cpp` rejects dirty cache values before reuse and shapes only on a miss. | Keep one exact canonical shape cache and reject stale/non-ready output before reuse. Do not introduce an artifact-local shaped cache. |
| Unreal Slate | `.../Private/Test/TextLayoutTest_LazyGeneration.cpp` distinguishes line models from lazily generated views and asserts view generation only for the needed window. | Preserve Zircon's resolved-layout versus glyph-artifact boundary; do not make final rendering reshape visual lines merely to fill a cache. |
| Bevy | `dev/bevy/crates/bevy_text/src/pipeline.rs` reuses a buffered section vector/string for one `TextPipeline::update_buffer` build, then transfers the laid-out glyph output to atlas lookup. | Retain the existing single source-to-layout-to-atlas direction. A future allocation change, if measured, belongs in the current artifact/local buffer owner rather than in the cache contract. |

Unreal's public cache API can return an empty shaped sequence when its font-cache pointer is
unavailable. Zircon intentionally diverges here: a known shaping failure is typed and cannot be
cached as empty geometry. This is required by Text09's cache-admission and M2b publication
contracts, and is stronger than copying Slate's API fallback behavior.

### Measurement Contract Before Any M1 Change

No bottleneck is confirmed. The coordinator must first run the managed Windows build and the
existing real-WGPU text scenarios, then capture three repetitions of each workload with 60 warmup
and 300 measured frames: retained static labels; forced 1/100/1k/10k label rebuilds; 10% turnover
scrolling; multilingual/rich/VerticalRl; and a font-generation/DPI transition. The capture must
record the existing profiler spans and counters rather than add a second timer:

- `text.layout/resolve_without_artifact`,
  `layout_pre_artifact_shaped_cache_hit_count`, and
  `layout_pre_artifact_shaped_cache_miss_count`;
- `text.artifact/build_resolved_text_glyph_artifact`, artifact line count, shaped-cache deltas,
  font-handle registration batch count, lock wait/hold time, and snapshot publications;
- `ui_text.extract`, `ui_text.prewarm`, `ui_text.layout_resolve`, `ui_text.prepare`, native/SDF
  source-cache and atlas-upload counters, `ui_text.resolved_glyph_artifact_route.*`,
  `RenderFrameProfile` CPU and WGPU timestamps;
- machine, driver, display scale, font corpus, resolution, power policy, p50/p95/p99, queue
  depth/placeholders, and equal-workload power only when a valid sensor exists.

| Measured signature | Permitted next investigation | Explicitly forbidden premature change |
| --- | --- | --- |
| Stable retained frames still enter layout/artifact spans with zero owner invalidation | Inspect dirty-owner and command-artifact reuse before changing cache keys. | Adding a second layout or artifact cache. |
| Layout p95 is high and shaped-cache misses rise with unchanged text/style/generation | Audit request-key construction, document revision, and viewport classification. | Hash-only lookup or width bucketing for wrapping. |
| Artifact p95 is high while pre-artifact shape hits stay high | Profile font-handle registration batch/lock counters and artifact line allocation within the existing owner. | Renderer-side reshaping or per-glyph global-lock retry. |
| GPU/prepare p95 or upload bytes dominate while CPU text spans are stable | Investigate atlas residency, page upload and route selection in the atlas owners. | Moving layout or shaping into the renderer root. |

The managed Cargo lane previously failed before a Rust/Cargo process started, so this session does
not retry or reinterpret it as a performance result. There is no p50/p95/RSS, WGPU, power, or
fresh PNG evidence. Any later product proof must reject policy-text images and write only its
validated real-rendered PNG under `docs/tests/runtime/text`, never under `target`.

## RTS-P1-046 Session-Owner Bypass Audit (2026-08-26)

**Status:** `architecture_review_complete / non_validation_implementation_complete /
static_checks_complete / performance_baseline_pending / managed_validation_pending`.

This audit identified and corrected the session-owner bypass in the Runtime UI operation paths.
Normal Runtime UI retains one `UiTextMeasureCache` per `UiSurface`; that owner contains the measure
cache, layout cache, retained Plain documents, hard-line index, and one `SharedTextLayoutSession`
under a common frame lifecycle. Standalone tree layout and extraction now create one bounded
operation-local cache/session for the complete operation. Full/incremental layout, render extraction,
secure-field layout, dialog-action measurement, and render-command artifact preparation receive that
owner instead of silently constructing a second production session.

The product call-site audit separates four cases:

- `UiSurface::{compute_layout,rebuild,rebuild_dirty}` already passes its retained cache and must
  remain the cross-frame product path;
- public standalone tree layout/extract helpers now own one bounded operation-local cache rather
  than one cache per leaf;
- secure fields and dialog action sizing now use the cache passed by surface extraction; overlap
  admission excludes component-owned text from the deferred no-cache collection phase;
- Editor projection/materialization now consumes the surface's published `render_extract` instead
  of re-extracting the tree. Editor retained paint still consumes public `layout_text`,
  `shape_text_line`, and `measure_text_size` one-shot helpers directly; that cross-crate owner
  migration remains open and must not be hidden behind a process-global mutex or cache.

The Unreal comparison supports the ownership correction rather than a new algorithm:
`Framework/Text/TextLayout.h` stores `FShapedTextCacheRef` on each `FLineModel`, uses line/layout
dirty flags, and exposes targeted shaping-cache flushes. `TextLayout.cpp` creates that cache with
the line model, supplies it through `FRunTextContext` during measure/layout, and regenerates it
only at the dirty boundary. Zircon therefore maps one surface/document owner to one retained
session and one standalone tree operation to one explicitly short-lived session; it does not add
a renderer cache, global registry, or second shaper.

The implementation slice is an infrastructure/correctness convergence, not a measured performance
optimization. Focused regressions count current-thread session construction and require one
construction for a multi-label standalone layout and one for a multi-label standalone extract.
Surface layout/extract/artifact preparation must add zero production session constructions; secure
field layout must publish through the surface layout cache; repeated identical dialog action
measurement must hit the same frame dedup owner. Static formatting, whitespace, call-site, and file
budget checks pass for this slice. Further changes to cache keys, storage, worker scheduling, or
Editor paint ownership remain blocked on the managed M0 trace.

The follow-up measurement must compare before/after session-construction count together with
`ui_text.extract`, layout measurement, `text.layout/resolve_without_artifact`, shaped-run
hit/miss, allocation samples, p50/p95/p99, and equal-workload power when a valid sensor exists.
The corpus is 1/100/1k/10k label layout and extract, secure fields, confirm dialogs, and the
existing retained static/10%-turnover scenarios. No time, RSS, energy, WGPU, or cross-engine
performance conclusion exists at this audit stage.

The follow-up owner hardening also removed the optional provider from the recursive Runtime layout
measure boundary. `measure_node`, `measure_node_incremental`, fixed-width leaf measurement, and
ordinary leaf measurement now require the operation's `UiTextMeasureCache`; the former `None`
arguments remain only in tests that were converted to explicit local owners. This closes a second
structural escape hatch below the public layout entry without changing the measurement algorithm.
The post-overlap render-command resolve phase now has the same required-owner contract; only the
parallel command collection phase may defer owner text until its explicit join.
Component text-field/dialog measurement and the extraction layout route now require the owner too;
the optional boundary exists only at overlap admission, where component-owned text is excluded.

## RTS-P1-046 Font-Generation Layout Invalidation Audit (2026-08-26)

**Status:** `architecture_review_complete / non_validation_implementation_complete /
static_checks_complete / managed_performance_baseline_pending / managed_validation_pending`.

The remaining glyph-artifact standalone rebuild is not an isolated cache miss. Current source loads
project font assets in graphics `resolve_text_batches` after UI layout and render extraction have
already published generation-qualified glyph artifacts. When that load advances the shared font
generation, `refresh_screen_space_text_batch_glyphs` calls
`rebuild_resolved_text_glyph_artifact_line`, which constructs a new `SharedTextLayoutSession` in
the renderer and reshapes one line. `presentation::rebuild_presentation_line` repeats the same
standalone-owner behavior for secure presentation lines. `UiSurface` cache keys include the font
generation, but `rebuild_dirty` does not observe a generation change before its clean-frame early
return, so removing the renderer repair without a layout-owner invalidation fence would leave a
retained surface permanently stale.

Reference evidence fixes the direction before implementation:

- Unreal Slate stores `FShapedTextCacheRef` on `FTextLayout::FLineModel`. When async font loading
  increments `GSlateLayoutGeneration`, `FTextLayout::UpdateIfNeeded` detects the generation change,
  dirties layout plus every line model's shaping/wrapping/view state, and runs `UpdateLayout` before
  the renderer consumes the line views. The renderer is not the recovery shaper.
- Bevy records layout output in `TextLayoutInfo`; unavailable fonts enter a next-frame reprocess
  queue and `update_text2d_layout` rebuilds the computed text buffer/layout before extraction and
  rendering. Font/text changes set the retained block's rerender state rather than reshaping inside
  the render pass.
- Fyrox keeps source, fonts, lines, and glyphs under `FormattedText`; its resource can wait for all
  fonts and cannot build successfully while required fonts are unavailable. This is a lower-feature
  Rust cross-check for keeping font readiness at the layout owner.

The approved infrastructure correction is therefore:

1. `UiSurface` retains the last observed text font generation. `rebuild_dirty` performs one atomic
   generation comparison before its clean-frame early return. A mismatch marks the surface text
   owner dirty and performs one complete layout/render-extract rebuild; ordinary steady frames keep
   the O(1) comparison and zero tree visits.
2. Graphics rejects an artifact batch whose generation becomes stale after batch planning. It
   records a low-cardinality post-layout rejection count and emits no glyphs from the retired
   artifact in that frame. The next surface frame repairs through the retained layout/session owner.
3. Delete the glyph-artifact and presentation standalone line rebuild APIs and the renderer's
   refreshed-line overlay. Do not replace them with a renderer cache, global session, per-line
   session injection, glyph-ID reinterpretation, or another artifact DTO.
4. Keep the already documented exact source-isomorphic Plain fallback separate. This slice neither
   expands that MVP fallback nor claims the Runtime81 M8 hard cut of every renderer fallback.

This is a correctness and lifecycle convergence, not a measured algorithm optimization. The
managed measurement stage must compare stable frames, one font-publication transition, and the
following repaired frame for 1/100/1k/10k labels. It must record session-construction count,
post-layout stale-artifact rejection count, layout/render owner visits, `ui_text.extract`,
`text.layout/resolve_without_artifact`, artifact-build and shaped-cache counters, allocation samples,
and p50/p95/p99 over 31 samples. Real WGPU work must retain the exact UI text pass and resolved GPU
timestamp; equal-workload power is recorded only when a valid sensor exists. No performance,
power, or cross-engine conclusion is valid until that managed evidence exists, and the real
framebuffer PNG belongs only under `docs/tests/runtime/text`, never under `target`.

Implementation result: `UiSurface` now retains `observed_text_font_generation`, performs its O(1)
generation comparison before the clean `rebuild_dirty` return, and runs one full retained-owner
layout/extract rebuild on mismatch. `resolve_text_batches` performs an O(batch-count) final
generation fence after font loading, drops retired artifact batches, and publishes the new
low-cardinality rejection counter. The renderer artifact transport is again only an immutable
`Arc<ResolvedTextGlyphArtifact>` plus line index/generation; the refreshed-line overlay and both
standalone line rebuild APIs were deleted. Atlas/CPU/SDF cache regressions now model recovery as a
new artifact Arc published by the text owner.

Static evidence is rustfmt success, `git diff --check` success, zero production matches for the
deleted rebuild APIs/overlay, and focused source/behavior contracts. A Windows managed lib-test
attempt for filter `font_generation` stopped before Cargo: `cargo.acquire` submission was accepted
but the coordinator returned `command_post_timeout` without a terminal result. Per the active
execution rule this plan does not poll or retry that request. Therefore no Cargo pass, timing,
allocation, WGPU, power, PNG, acceptance, commit, or WeCom result is claimed.

Static closeout also repaired the tests that still encoded the deleted line-rebuild path. The
virtual-glyph contract now republishes the entire artifact after an explicit font-generation
advance, the render structure convention checks the source-isomorphic/native routes and forbids
the retired refresh helper, and the prepare-report fixture includes the new rejection counter.
The removed API/field literal scan is now `0`; this does not substitute for managed compilation.

The Editor paint owner was re-audited but not changed. Its direct Runtime layout and shape helpers
sit behind the existing 2,048-entry generation-aware retained paint cache and therefore execute on
cache misses, while the extra measurement call is restricted to the terminal Host fallback with no
run or shaped payload. The existing plan's M0 gate still applies: collect cache-hit/miss, absolute
origin/size churn, session construction, allocation, and p50/p95/p99 data before changing the
global mutex owner, cache coordinates, or fallback policy. This audit is not evidence that the
Editor path is or is not the measured bottleneck.

## Rich Layout Repeated-Shaping Hypothesis (2026-08-26)

**Status:** `architecture_review_complete / instrumentation_implemented /
static_checks_complete / structural_optimization_not_started / managed_profile_pending`.

The cluster/public-caret review exposed a likely ownership conflict, not yet a measured bottleneck.
Plain UI layout can attach a process-local `ResolvedTextGlyphArtifact` directly to its resolved
layout. Rich layout uses the same opaque artifact slot for the compiled rich document, while its
current pipeline constructs a `RichAdvanceIndex` for line ranges/layout and UI item-advance
projection can shape styled segments again. A composite artifact or paragraph model might remove
work, but changing that boundary before attribution could merge parsed-document, glyph, source-map,
and layout lifetimes incorrectly.

The next managed profile must therefore add low-cardinality phase/counter evidence without changing
the shaping algorithm:

1. Count shape requests by `plain_layout`, `rich_range_index`, `rich_layout`, and
   `ui_rich_item_projection`, together with shaped-cache hit/miss and total shaped source bytes.
2. Record aggregate phase spans for rich compile/parse, advance-index construction, line layout,
   UI item projection, glyph-artifact publication, and renderer preparation. Do not emit per-run or
   per-glyph trace records.
3. Capture cold/warm 31-sample p50/p95/p99 and allocation/RSS for 1/100/1k/10k styled runs using the
   same text, fonts, fallback generation, width, viewport, build, device, and power policy. Include
   a plain source-equivalent control and the existing real WordSmart BBCode workload.
4. Accept a structural migration only if duplicate backend/cache-miss shaping or repeated projection
   materially dominates after warm-up. If confirmed, design an Unreal-aligned document-revision-owned
   line model that carries parsed runs, cluster/source geometry, break candidates, and shaped cache;
   viewport/UI/renderer consumers receive views of that artifact rather than rebuilding it.

No shape-call reduction, latency, allocation, power, cross-engine parity, or optimal-complexity claim
is made here. The real framebuffer proof remains the existing WGPU exporter, and any new PNG belongs
only under `docs/tests/runtime/text`, never under a Cargo target directory.

Instrumentation implementation result: horizontal rich UI layout now has exactly three aggregate
scopes: `rich_range_index`, `rich_layout_materialization`, and `ui_rich_item_projection`. A dedicated
`rich_layout/profile.rs` provider adapter counts shape requests and input bytes for each phase, then
publishes two counters once when the phase ends. Non-profiling builds retain only direct provider
forwarding; there is no global/TLS registry, per-run span, per-glyph counter, cache-policy change, or
shape-result copy.

A profiling-feature contract uses a real styled BBCode Glyph-wrap fixture and requires one span plus
one sample per counter for each phase, preventing trace cardinality from scaling with styled runs.
Rustfmt, scoped whitespace, counter-owner, and file-budget checks pass; the main rich layout owner is
404 lines and its profile child is 140 lines. Managed compilation and the 31-sample workload have not
run, so the repeated-shaping hypothesis is now measurable but remains unconfirmed. No structural
optimization is authorized until those numbers identify the dominant phase and cache-miss behavior.

## Source Lifetime and Range-Lease Hypothesis (2026-08-26)

**Status:** `source_lifetime_architecture_research_complete /
unreal_external_text_owner_confirmed / source_materialization_and_batch_owner_instrumentation_implemented /
algorithm_unchanged / static_checks_complete / managed_profile_pending`.

Current source stores one exact `Arc<str>` in every `ShapedGlyphRun`. A parallel request can lend an
exact paragraph Arc to the run, but `horizontal_paragraphs` first copies each hard-line slice into a
distinct Arc; synchronous requests without an exact owner allocate when the shaped artifact is
finalized. Cache collision guards and measurement/artifact consumers require exact source content,
so replacing the field with a full-document Arc without a validated range contract would break
coordinates and could charge the same owner once per cache entry.

The local Unreal reference makes the intended boundary concrete. `FSlateTextShaper` borrows external
text plus start/length, while `FShapedGlyphSequence` retains only its source range, glyph source
indices, and reverse source-index map. Its allocated-size report excludes text storage. Zircon should
therefore converge on a revisioned immutable document snapshot and source-range lease if measurements
justify the migration, while preserving Zircon's typed Unicode/failure receipts and cache accounting.

Instrumentation now exposes two complementary views without changing shaping behavior:

1. `text_shape_source_materialization_count`, owner reuse, allocation count, and allocation bytes at
   the shaped-artifact boundary. A hybrid attempt can expose two materializations explicitly.
2. `text.shape_batch.source_lease_count`, unique source owner count, leased source bytes, and unique
   owner bytes before duplicate/cache admission. This distinguishes repeated logical leases from
   distinct Arc allocations without publishing pointer values or source text.
3. Run cold/warm synchronous and parallel workloads for 1/100/1k/10k hard lines, a stable document,
   one-line edits, duplicate paragraphs, mixed scripts, and a controlled horizontal hybrid failure.
   Use 31 samples with identical build/font/generation/width/viewport/device/power policy.
4. Record shaped-cache hit/miss/current/peak bytes, source counters, allocation/RSS, p50/p95/p99, and
   equal-workload power only with a valid sensor. Compare leased bytes to unique-owner bytes and
   materialized allocation bytes to total cache/source residency; do not infer from one ratio alone.

If source copying or residency materially dominates, M1 introduces an immutable snapshot plus
validated local range and absolute source origin, with wire serialization retaining the current
exact source slice. M2 moves parallel requests, shaped runs, and cache collision checks to that lease
and adds a cache-owned unique-owner residency registry with exact insert/update/evict/clear accounting.
`Arc::strong_count`, per-entry full-document charging, hash-only equality, or renderer-owned source
reconstruction are prohibited. Per-glyph SoA/cluster-table work requires a separate size/density/access
profile and is not bundled into this lifetime migration.

No managed run has executed, so source lifetime remains an instrumented hypothesis. No allocation,
latency, RSS, power, optimality, or Unreal-parity improvement is claimed, and the lease hard cut is not
authorized yet.

## Paragraph and Layout Lifetime Audit (2026-08-26)

**Status:** `paragraph_lifetime_architecture_review_complete /
duplicate_analysis_instrumentation_deferred / algorithm_unchanged / static_checks_complete /
managed_profile_pending`.

Current source already retains `SharedTextLayoutSession` in the UI measure owner and separates
shaped-run, hard-line, and layout/measure caches. The remaining hypothesis is duplicate
`LineBreakOpportunityMap`/hard-line materialization across direct and Cosmic fallback, plus separate
rich advance-index, physical-line, and viewport projections. A cache hit does not rebuild Bidi/script
analysis, and keyed plain viewport reuse already shares the source owner.

The next measurement must cover plain/rich and direct-success/partial-fallback/terminal paths at
cold/warm 1/100/1k/10k hard lines, stable scroll, and one-line edits, with 31 samples. Record analysis
construction count, hard-line/line-break bytes, shaped-cache hit/miss, layout DTO current/peak,
allocation/RSS/p50/p95/p99, and valid-sensor power. A retained document-revision paragraph artifact is
allowed only if duplicate analysis is a dominant measured share; its first scope is source snapshot,
Bidi/script/line-break analysis, hard-line index, and dirty-range dependencies. Source leases, glyph
SoA, renderer artifacts, and cache policy are explicitly separate migrations.

No production algorithm or shape-call behavior changed in this audit. Managed Cargo, counters,
profiles, RSS/power, WGPU, and PNG remain pending.

## Cache Hash and Artifact Digest Identity Review (2026-08-26)

**Status:** `ephemeral_cache_hash_type_implemented / stable_artifact_digest_type_implemented /
default_hasher_isolated / sdf_v1_bytes_unchanged / algorithm_unchanged / static_checks_complete /
managed_validation_pending`.

The audit found no evidence that cache hashing is the measured Runtime Text bottleneck. Shaped,
parallel pending, rich, measure/layout, physical-line, and document-revision hashes only select
process-local buckets; full keys and exact source still qualify reuse. Replacing them with per-request
BLAKE3 would retain `O(n)` hashing while making retained document viewport work `O(document bytes)`
instead of `O(1)` owner+revision identity.

The engineering defect was semantic: bare `u64` fields named content/source hash could escape into a
future artifact or replay contract. Runtime caches now use non-serializable `EphemeralCacheHash`, with
`DefaultHasher` isolated in one builder. Persisted `.zsdf` generation/offline identities use
`StableContentDigest`; existing BLAKE3 variation/source bytes, v1 header, artifact path, and checksum
remain byte-for-byte defined by the current codec. New persisted consumers must add their own explicit
format/domain version rather than reusing a runtime cache hash.

Local Unreal uses `FCachedShapedTextKey::GetTypeHash` only for `TMap` and compares the full key, while
its persistent systems use separate deterministic hash owners. Zircon therefore follows the same
separation without importing Unreal's text-lifetime assumptions. This type-only migration has no
intended timing, allocation, cache-hit, shape-call, or power effect; its size tests require wrappers to
remain equal to `u64` and `[u8;32]`.

Rustfmt, scoped whitespace, sole hasher-owner, bare production hash-field, and digest propagation
checks are complete. Managed Cargo, shaped/measure collision regressions, SDF encode/decode golden,
31-sample timing/RSS/power, and WGPU/PNG remain pending. No optimization or bottleneck-removal claim is
authorized from this slice.

## Text Layout Diagnostic Catalog Boundary (2026-08-26)

**Status:** `diagnostic_code_catalog_implemented / backend_neutral_boundary_preserved /
focused_behavior_tests_complete / managed_validation_pending`.

Every `TextLayoutError` variant now has one allocation-free stable diagnostic code and one stable
localization catalog key. This is a contract correction rather than a performance optimization:
telemetry and Editor consumers can classify errors without parsing English `Display` output, while
the core framework remains independent of Runtime Text implementation receipts. No cache, shaping,
layout, scheduling, or rendering behavior changed. Managed Cargo and integration evidence remain
pending.

## UI Shaper Facade Hard Cut (2026-08-26)

**Status:** `empty_ui_shaper_stack_removed / sole_shared_adapter_preserved /
source_guard_updated / static_checks_complete / managed_validation_pending`.

Current-source review found no performance policy in `UiTextShaperStack`: it was a one-member
forwarder over `UiSharedTextShaper`. The wrapper is removed and all entrypoints call the sole adapter
directly. This is an ownership correction with no intended change to shaping calls, allocation,
cache, layout, renderer, timing, or power; managed validation remains pending.

## Serializable DTO and Renderer Batch Residency (2026-08-26)

**Status:** `layout_dto_and_renderer_batch_residency_receipts_implemented /
intermediate_paint_copy_open / algorithm_unchanged / static_checks_complete /
managed_profile_pending`.

The serializable layout DTO and the internal renderer batch are separate ownership domains. Existing
layout-cache accounting covers line/run text and advances; the prepare report now records final
native/SDF batch count, UTF-8 text bytes, and glyph-advance bytes after Auto routing. The receipt is a
lower bound, records no raw text, and adds no shape or route decision. Profile plain/rich and
artifact/visual/fallback paths before deciding whether internal leases should replace copies while
the versioned interface continues to materialize owned strings.

## Owner-Local Runtime Budget Matrix (2026-08-27)

**Status:** `owner_local_budget_snapshots_implemented /
runtime_budget_profile_projection_implemented / page_shadow_residency_receipt_implemented /
algorithm_defaults_unchanged / static_checks_complete / managed_profile_pending`.

Do not compare only raw defaults. Capture each effective budget with its usage and pressure receipt:
boundary context with max reshaped window/correction steps; tatweel limits with requested/probe/
candidate-byte/safety/accepted values; hard-line max entries/bytes with resident/eviction/oversized
bypass; SDF in-flight/source/completion limits with backlog/backpressure; and bitmap page-shadow
resident/max bytes with admission rejections. Correlate these across plain/rich, direct/fallback,
cold/warm, 1/100/1k/10k lines, scroll/edit, and 31 paired samples.

The local Unreal reference supports owner-local cache controls and memory statistics, not a shared
knob object spanning shaping correctness and renderer residency. No limit changes are authorized
until profile data identifies sustained pressure and rules out lifetime/duplicate-work causes. This
slice has no latency, RSS, power, optimality, or bottleneck-removal claim.

## Stable Hard-Line Model Foundation (2026-08-27)

**Status:** `separator_aware_stable_line_owner_implemented /
edit_local_reanalysis_implemented / full_document_hard_line_rebuild_removed_from_edit /
grapheme_index_ascii_splice_static_implemented / unicode_fallback_rebuild_preserved /
product_session_unwired / managed_validation_pending`.

The revisioned piece document now retains source hard-line models independently from wrapped visual
lines. Each model owns a stable document-scoped ID plus content/separator byte lengths. An edit
materializes only the affected line envelope with one separator context line on both sides, prepares
the complete model splice before mutation, preserves exact prefix/suffix IDs, keeps the left affected
ID on merge, and allocates revision-qualified IDs for additional split lines. The edit receipt carries
old/new reanalyzed line ordinal ranges.

This removes full-document hard-line reconstruction from the edit path and removes hard lines from
the revision-bound grapheme index so there is one line authority. It is not the complete incremental
layout result: ordered models currently use `Vec`; non-ASCII, Unicode-sensitive and CRLF edits still
rebuild grapheme boundaries from a complete snapshot; and UI/service/layout sessions do not consume
the IDs. Profile beginning/middle/end edits across 1/100/1k/10k lines before choosing a rope/tree
sequence or changing reflow/cache policy.

## No-Wrap Clip Width Semantics Review (2026-08-27)

**Status:** `current_overflow_alignment_matches_unreal / algorithm_unchanged /
line_view_slot_contract_open / managed_validation_pending`.

The natural width, clamped placement extent, renderer origin and interaction geometry call graph was
reviewed before changing overflow alignment. Unreal computes line display offset from
`max(DrawWidth, ViewSize)`, so an overwide line receives zero extra center/right justification space.
Zircon's current overwide origin is equivalent and its natural advances continue into the clip. The
suspected MVP alignment defect is therefore not reproduced and no algorithm changed.

Unreal still has a clearer contract because natural `FLineView.Size` and display offset are separate;
Zircon combines placement slot, content origin and full-frame hit candidacy in one line frame. That
remains a coordinated interface/serde/native/SDF/hit/caret/selection/IME migration, not an isolated
alignment patch.

## Surface Text Layout Revision Exhaustion Hard Cut (2026-08-27)

**Status:** `surface_text_revision_wrap_removed / exhausted_identity_fail_closed /
uncacheable_layout_fallback_preserved / retained_key_call_sites_converged /
static_checks_complete / managed_validation_pending`.

The serialized surface layout cache no longer wraps its text revision and republishes changed source
under an earlier `(node, revision)` key. `u64::MAX` is an unpublishable exhaustion sentinel; both
surface key-construction sites use the checked accessor, and pending prewarm metadata carries an
explicit optional key. Exhaustion disables retained document reuse only: ordinary exact-source
layout, editable state and the unretained viewport request remain available.

This is an identity correctness repair, not a latency optimization or the Runtime82 document
authority. Managed serde/integration fault injection, Cargo, profile/power and WGPU/PNG remain open.

## Retained Document Typed No-Op Edit (2026-08-27)

**Status:** `typed_noop_edit_outcome_implemented /
allocation_free_piece_range_equality_implemented /
revision_index_history_churn_suppressed / product_gateway_unwired /
static_checks_complete / managed_validation_pending`.

An exact replacement no longer publishes a false document change. `replace` returns typed
`Unchanged/Changed`, and an allocation-free comparison walks only the source range's original/
addition pieces before revision or storage mutation. An unchanged operation preserves the key,
chunks, pieces, stable hard-line IDs and revision-bound grapheme index, including at revision
`u64::MAX`. Checked piece coverage fails closed with `StorageInvariant`.

Unreal similarly compares editable text before rebuilding line models and only publishes undo/change
state for actual text changes. Zircon's real late-mismatch edits may pay an extra range-prefix compare;
measure that in the existing 1/100/1k/10k beginning/middle/end matrix before adding caller hints or
changing storage. Product authority/history/rebase, Cargo, profile/power and WGPU/PNG remain open.

## Dynamic Fallback UI Collection Identity (2026-08-30)

**Status:** `core_collection_injected_into_project_and_fallback_ui /
process_global_fallback_generation_removed / static_checks_complete /
managed_profile_and_product_validation_pending`.

The dynamic runtime previously had two font identities: project retained surfaces resolved
`FontCollectionService` from Core, while the HUD/menu fallback extract cache used a process-default
measure cache and global font-generation key. Session construction now resolves the Core service once
after module activation and injects the same `Arc` into both owners. The fallback key samples the
generation from its own retained layout session, so only its collection publication invalidates the
extract. Production has no default fallback-cache constructor. A focused Rust regression publishes a
real render-input generation on an independent collection and requires a new extract allocation; it is
written but remains unexecuted until Cargo can run.

This changes collection ownership and invalidation correctness, not the shaping/layout algorithm.
Before claiming performance or product completion, run project/no-project and two-Core cases through
the managed Windows build, publish one font generation, and record fallback extract rebuild count,
layout cache hit/miss, allocation/RSS, 31-sample p50/p95/p99, WGPU pixels and valid power data. No new
PNG or measured result is claimed by this static slice.

## Conservative Grapheme Index Incremental Splice (2026-08-30)

**Status:** `ascii_incremental_index_splice_static_implemented /
unicode_and_crlf_fallback_preserved / fixed_profile_counters_added /
managed_profile_pending`.

The preceding hard-line review identified a separate whole-document cost: every changed revision
invalidated `TextDocumentSourceIndex`, and its next query rebuilt all grapheme boundaries from a
flattened snapshot. The first structural correction is deliberately limited to edits whose cached
endpoints and one-boundary context on both sides are ASCII and contain no CR/LF. The prepared edit
receipt retains the old boundary indexes; commit creates the replacement byte boundaries and shifts
the unchanged suffix by checked byte delta. Stale indexes, Unicode/combining/emoji/ZWJ/RI context,
CRLF, non-boundary ranges, and arithmetic failures still invalidate and use the existing complete
rebuild owner. The splice avoids rebuilding a complete snapshot, while current piece-backed local
context extraction still scans retained pieces; end-to-end edit complexity remains a managed
measurement question. No second segmentation authority or line model was introduced.

Four fixed counters expose successful incremental update count, input bytes, boundary count, and
duration alongside the existing rebuild counters. Source regressions cover ASCII suffix shifting,
empty insertion/deletion, and the required Unicode/CRLF fallback. This is source-level infrastructure only: run the managed
1/100/1k/10k-line beginning/middle/end matrix, exact Unicode corpus comparison, allocations/RSS,
power, and query p50/p95/p99 before widening the safety boundary or claiming a bottleneck removal.
