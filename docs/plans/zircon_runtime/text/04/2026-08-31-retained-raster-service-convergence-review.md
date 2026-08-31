---
status: architecture_review_complete_measurement_and_hard_cut_pending
plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
failure: docs/plans/zircon_runtime/text/04/failure-2026-08-31-retained-swash-native-scale-bypasses-physical-raster.md
reviewed_at: 2026-08-31
---

# Retained raster service convergence review

## Scope and current decision

This review covers the retained Editor glyph raster path after correcting its physical ppem policy.
The current-source correction is necessary for product correctness, but it is not the terminal text
architecture. The terminal direction is one Runtime-owned font collection and glyph raster service,
matching Unreal's separation between Slate layout consumers and the shared font cache/renderer.

Do not optimize the Editor `HashMap` or expose Runtime's crate-private Swash types as public API.
First publish a backend-neutral Runtime receipt, cut the retained consumer to it, measure the shared
service, and only then change cache concurrency or residency policy.

## Current-source ownership audit

| Concern | Runtime owner | Retained Editor owner | Structural result |
|---|---|---|---|
| Font collection and revision | `text/font/shared.rs`, `FontCollectionService` | `paint_text/font.rs` creates a separate system `fontdb::Database` | Duplicate font authority |
| Exact shaped face | resolved glyph artifact and `UiTextGlyphArtifactRasterFace` | artifact snapshot copies bytes into `HostTextFont` | Exact identity is available, but rewrapped |
| Raster backend | `text/raster/swash/SwashRasterizer` | `paint_text/raster.rs` owns another `ScaleContext` | Duplicate Swash service |
| Bitmap cache | native source cache is generation-qualified and bounded | process `HashMap<GlyphRasterKey, CachedGlyphRaster>` is unbounded | Duplicate residency policy |
| Font metrics fallback | Runtime font metadata/layout owners | Editor `fontdue::Font` | Duplicate metric authority |
| Async work | Runtime raster pool and frame budgets | retained raster miss executes synchronously | Duplicate scheduling policy |

The retained artifact lane already carries source identity, font generation, collection/face handles,
collection index, variations, and shared bytes. That is sufficient provenance for a staged hard cut.
The remaining host fallback lane still resolves system fonts independently and can shape/raster a
different face from the Runtime collection. It must not survive the terminal migration.

## Reference engine findings

- Unreal `FSlateFontKey` includes font info, outline settings, and scale. `FShapedGlyphEntryKey`
  records the computed render size, and `ComputeFontPixelSize` converts the request to the physical
  pixel size consumed by glyph loading. `FSlateFontCache` owns shared glyph/font cache state; widgets
  do not create a separate FreeType/Slate raster cache.
- Bevy stores physical font size in `FontAtlasKey` and passes the same size to Swash. Atlas identity
  and raster input cannot disagree.
- Slint passes the window scale into layout and rasterizes the resulting physical `run.font_size()`;
  scale changes invalidate the applicable cache state.
- Fyrox exposes supersampling as a separate named policy. It does not make fallback-rasterizer
  identity select a hidden density multiplier.

These references support one physical ppem key and one shared cache owner. They do not support a
fixed 8x multiplier, a renderer-local font database, or public exposure of a third-party backend.

## Target Runtime contract

The public boundary must use Zircon types only. A minimal service request needs:

- exact font collection revision and instanced face identity;
- glyph ID and integer physical ppem bucket;
- normalized horizontal/vertical phase;
- hinting, smoothing, synthetic style, and variation identity;
- requested bitmap format.

The immutable receipt needs:

- request identity and font revision;
- bitmap format and shared `Arc<[u8]>` payload;
- width, height, bearing, and physical ppem;
- typed ready, missing-face, missing-glyph, invalid-request, budget-deferred, and stale-generation
  outcomes.

The service implementation remains under Runtime `text/raster` and may use Swash internally. The
interface must not expose `swash::Source`, `swash::Content`, `fontdb::ID`, `fontdue::Font`, borrowed
font bytes, or a renderer/WGPU atlas type. Native atlas and retained CPU painting consume the same
receipt through separate adapters.

## Hard-cut order

1. Keep the current retained path correct: one physical ppem bucket, Swash primary, straight RGBA,
   and explicit smoothing/phase identity. This is the current failure repair, not final convergence.
2. Add the backend-neutral Runtime request/receipt and a session-owned raster service. Initially feed
   it the exact collection/face identity already present in the resolved glyph artifact.
3. Cut retained artifact drawing to the Runtime receipt. Delete retained `ScaleContext`, raster
   `HashMap`, Fontdue fallback rasterization, and duplicated color-alpha normalization in the same
   migration. No facade or dual success route remains.
4. Make retained layout require the canonical Runtime glyph artifact for renderable text. Resolve UI,
   strong, and mono preferences through the injected `FontCollectionService`; then delete the Editor
   system `fontdb`, `fontdue`, embedded-font copy, host font-set cache, and runtime-artifact font cache.
5. Reuse the service from native bitmap atlas preparation so both consumers share generation
   invalidation and residency. Renderer atlas slots and WGPU uploads remain graphics owners; the
   raster service owns no WGPU resource.

Steps 3 and 4 are cross-owner hard cuts and must move callers, tests, Cargo dependencies, and docs in
one integration window. A bytes-only public adapter is allowed only as a private migration tool in
the same change; it is not an accepted terminal API.

## Measurement plan before cache optimization

Current retained observations are now available through fixed low-cardinality counters for cache
hit/miss, miss bitmap bytes, Swash/Fontdue route, duplicate miss publication, and current/peak
resident entries and bitmap bytes, plus a cache-miss span. Residency accounting is constant-time on
publication, does not scan the cache, and is absent from ordinary non-profiling builds. Before
redesigning the shared cache, add only the remaining measurements needed to distinguish:

- lookup mutex wait and hold time;
- duplicate concurrent raster work before publication for one key;
- raster CPU time by alpha/subpixel/color route;
- resident entry/byte current and peak values;
- generation invalidation, eviction, and deferred work counts.

Run Windows managed release and profiling lanes for 1/16/128/512 distinct glyphs, shared and distinct
faces, 13px at 100/125/150/200%, cold/warm caches, alpha/subpixel/COLR/bitmap-color glyphs, and one
generation replacement. Record 31 raw samples with p50/p95/p99, allocations, working-set delta,
CPU sampled stacks, GPU timestamps where applicable, and package power. The UI12 upward workload adds
1000 click, 1000 pointer move, and 200 resize/scale transitions.

Only measured bottlenecks authorize one of these changes:

- single-flight for duplicate misses;
- sharded lookup for lock contention;
- byte-bounded LRU for resident growth;
- asynchronous scheduling for miss latency.

No latency, power, Unreal parity, or optimality claim is valid until matched data exists.

## Validation and visual evidence

Lower gates must cover physical buckets 13/16/20/26, equivalent-bucket reuse, phase and smoothing
identity, generation replacement, Fontdue-route deletion after hard cut, COLR unpremultiply, embedded
bitmap straight RGBA, and zero-alpha RGB normalization.

The existing Runtime WGPU DPI gate proves a 1x-to-2x cache transition but writes no image. Product
acceptance still requires current-source real WGPU frames. The multilingual Runtime proof remains:

`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`

UI12 must additionally capture real retained Editor frames at 100/125/150/200% under
`docs/tests/runtime/text`. These must show the actual Workbench text with rounded rectangles and SVG
content in the same frame; strategy diagrams, HTML previews, offline bitmap enlargement, old images,
and anything under a Cargo target directory are not evidence.

## Current state

Architecture review and the pre-optimization measurement design are complete. The retained physical
ppem and color-alpha repairs plus profiling counters are source-implemented. Runtime service API,
consumer hard cut, dependency deletion, managed Cargo, performance/power sampling, WGPU frames, PNG
inspection, milestone commit, and WeCom notification remain pending.
