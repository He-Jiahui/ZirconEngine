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
| Owner document layout | Owner commands retain `TextDocumentKey` plus `UiTextViewport`. `UiTextMeasureCache::resolve_or_shape` bypasses persistent layout reuse only when the shared hard-line query selects a strict Plain/HorizontalTb/None/Clip viewport subset; complete and unsupported owner layouts use the persistent cache. | Actual partial-document resolves are reported separately from persistent cache hits/misses, so a profiler trace distinguishes virtualized geometry from normal cross-frame reuse. |
| Native raster | `TextRasterWorkerPool` has bounded request and completion queues. Source-cache work shares `Arc<[u8]>` font data by face epoch and drains under item/byte budgets. Queue pressure follows placeholder/defer paths. | Increasing worker count or queue capacity cannot be the first response; it risks hiding backpressure and increasing latency or power. |
| Atlas and draw | Native source cache, persistent slots, page shadow, dirty upload, and instance rendering expose cache, byte, upload, and draw counters. A continuous same-contract atlas run is one draw with one 68-byte instance per glyph. | CPU quad expansion, full-atlas upload, and per-glyph draw submission are already excluded by deterministic contracts. |
| Render observation | `FrameProfiler` already joins CPU submit timing, WGPU timestamp results, pass metrics, and UI `RenderStats`; `ScreenSpaceUiTextPrepareReport` projects raster/cache/upload health. | The generic render profiler must be reused rather than creating a competing timer or benchmark framework. |

The source-level M0 observation boundaries cover caller-thread owner request collection,
overlap admission, command collection, shaping, prewarm, layout, and GPU preparation. The two
owner scans have fixed `ui_text.extract` stages so their p95 cost cannot be attributed to shaping.
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
