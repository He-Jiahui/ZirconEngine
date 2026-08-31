---
title: Editor paint-text relative layout, glyph atlas and font publication performance review
date: 2026-08-23
module: zircon_editor retained-host host_contract/paint_text
priority: MVP-P0 editor text layout, recording and raster
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate shaped-text cache, text layout and font cache
---

# Goal

Make editor text work proportional to changed text/style/width and newly admitted glyphs, independent
of screen translation. One canonical Runtime shaped artifact must serve measurement, retained layout,
recording and raster submission; paint must not rescan system fonts, reshape equivalent lines, rebuild
absolute glyph vectors or retain unbounded per-generation bitmaps.

## Reviewed source

- Rust files: 37/37
- lines: 7,198
- bytes: 232,405
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `a3a86f08a27f0792b6dee63ae3e82dc4492470c958e56836bc19b6ec8da101d2`
- owning commit at review: `7762880fd1d8db3d3872888ba8377910177574af`

Scope: `zircon_editor/src/ui/retained_host/host_contract/paint_text.rs`,
`paint_text/**`, `paint_text_tests.rs` and `paint_text_tests/**`.

Supporting production paths traced/read: Runtime `layout_text`, shared layout sessions, resolved glyph
artifact generation/borrowing, exact artifact face snapshots, retained recording and command-stream
text conversion. Editor call-site search found 39 textual size/style draw matches and 103 textual
runtime-width measurement matches including definitions/tests.

## Correct foundations to retain

1. Clip rejection happens before layout. Empty/transparent text exits early, and record-only mode exits
   before CPU raster/blend.
2. Runtime layout owns Unicode shaping, wrapping and ellipsis. When a current resolved glyph artifact
   exists, the editor borrows its glyph lines, resolves exact face/instance pairs once per layout and
   does not call `shape_text_line` again.
3. Font and artifact identities are generation-qualified. Runtime artifact projection fails closed as
   a complete line/layout instead of mixing face-specific glyph IDs with host fallback IDs.
4. Raster bitmaps use `Arc<[u8]>`; keys separate font source, generation-derived cache key, glyph,
   size, phase, fallback scale and smoothing. Eight phase bins and pixel tests protect compact labels.
5. Layout/raster work already exposes artifact hit/miss, copied glyph and cache-miss profile markers.
   Screenshot/crop tests cover ellipsis, combining graphemes, subpixel stability and color glyphs.

## Structural findings

### P0: layout cache identity includes absolute screen position

`PaintTextLayoutCacheKey` stores `rect_x_bits` and `rect_y_bits`, because cached glyph vectors contain
absolute x/y. Identical text, width, font and scale at another row or after dock translation cannot
share layout. The existing ignored artifact benchmark explicitly varies x to force misses. Each probe
also allocates `text.to_string()` before taking the global cache mutex. At 2,048 entries the cache
clears every entry instead of evicting by recency/bytes, creating synchronized miss waves.

M2 splits content-relative shaping/layout from paint origin. The key retains text identity, width,
height only where wrapping/vertical policy requires it, font/style/generation/scale/smoothing, but not
x/y. Paint applies one origin transform, matching the existing Runtime artifact's relative glyph
positions and Unreal's block-location paint transform.

### P0: record-only text still builds the CPU-oriented complete layout projection

`draw_text_with_size_and_style_impl` needs the ellipsized/wrapped display text for recording, but first
builds `PaintTextLayout`, including positioned glyph vectors and artifact raster-font snapshots. It
returns before rasterization, so CPU glyph bitmap work is avoided, but a recording cache miss still
pays Runtime layout, artifact face resolution and editor glyph-vector projection that the GPU command
does not consume.

M2 makes the recorded command borrow/retain the canonical prepared Runtime shaped artifact plus
relative ranges/handles. Display text, measurement and GPU draw must come from that artifact without a
parallel editor glyph DTO.

### P0: fallback performs a second shaping pass and superlinear reconciliation

After `layout_text`, missing artifact coverage calls `shape_text_line` for every visual line, clones its
glyphs, and separately runs fontdue layout. Reconciliation repeatedly maps graphemes to glyphs with
`find`, scans all graphemes for each shaped glyph, rebuilds position vectors, and can call the same
mapping twice. For G glyphs and H graphemes the fallback contains `O(G*H)` stages; it is not the exact
artifact main path, but unsupported/stale/partial artifacts turn it into an editor hitch risk.

M3 first closes Runtime artifact coverage gaps. Any retained fallback that remains must use one
monotonic glyph/cluster cursor and one captured font snapshot, giving `O(G+H)` time and bounded
temporary storage. It must preserve RTL, ligatures, combining clusters and eight-phase parity.

### P0: font resolution performs filesystem and parser work lazily on the paint thread

The first draw after text-preference change constructs a new font database and calls
`load_system_fonts()`. Three roles then query faces; file-backed faces are read into new vectors,
fontdue parses them, and `host_text_font_cache_key` hashes every font byte. UI and strong roles can
repeat reads/hashes for the same source. The two-entry font-set cache bounds generations but does not
remove this main-thread I/O/parse stall.

M4 publishes a prepared font set from the font service/worker boundary. Source bytes and stable source
identity are deduplicated per file/collection, parsing is shared by role/instance, and presentation
generation changes only after all required faces are ready. Paint performs no database scan, file I/O,
full-font hashing or parser construction.

### P0: glyph raster cache is unbounded and serializes every CPU glyph lookup

The process-global raster cache is an unbounded `HashMap<GlyphRasterKey, CachedGlyphRaster>`.
Generations, sizes, eight phase bins, smoothing modes and artifact faces accumulate permanently. Every
CPU glyph, including a hit, takes the same mutex and clones the raster/bitmap Arc. The shaped glyph does
not retain a raster/atlas handle, so a line of G glyphs performs G global lookups/locks.

M5 introduces a byte/page-budgeted glyph atlas/resource owner, generation-qualified handles cached on
prepared glyphs, admission-only raster work and explicit owner/shards. CPU fallback storage is bounded
by bytes and LRU generation. A capacity guess or clear-all replacement is rejected until M0 records
real glyph distributions and residency.

### P1: per-glyph context reads repeat stable frame state

`draw_layout_glyphs` reads smoothing once, but raster lookup reads text preferences again for every
glyph. Host fallback also reacquires the host font snapshot for every glyph. For G host glyphs this is
`G+1` preference Arc clones and G font-set/snapshot lookups before cache locking. M1 captures smoothing
and the optional host font once per run and passes them into raster lookup. Exact artifact-only runs do
not capture a host font at all.

### P1: successful Swash rasterization copies its owned output buffer

Alpha, subpixel and color Swash results consume `Vec<u8>` through `take(...).collect()`, allocating and
copying a second vector even though the renderer already transferred ownership. M1 truncates and reuses
the owned vector for formats that retain the same bytes; grayscale conversion still creates its
different alpha representation.

### P1: measurement and drawing do not share one prepared text run

The retained host has many width-measure calls and separate draw calls. Measurement constructs Runtime
style values (including a font-family String) and invokes the Runtime measurement path; draw later
looks up/builds another positioned layout. M2 exposes prepared measurement from the same canonical
artifact used for display/paint and removes measure-then-layout duplication at controls.

### P1: cache contention and residency are not attributable

Layout, host font-set, artifact-font, raster and Swash-context locks have no wait/hold counters. Layout
cache owned-text bytes, clear waves, glyph bitmap bytes, stale generations, raster admissions and
font-scan/I/O/parse time are also absent. Existing artifact counters are valuable but cannot accept the
whole text system.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/ShapedTextCache.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/SlateTextRun.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/TextLayout.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp`

Unreal's shaped-text key contains text range, scale, shaping context and font, not screen position.
`FSlateTextRun::OnPaint` obtains a shaped subsequence and applies block location through paint geometry.
Measurement first reuses the full shaped run and only shapes a subrange when extraction cannot answer.
`FTextLayout::UpdateIfNeeded` is dirty/generation driven and can generate only visible line views.

`FSlateFontCache::GetShapedGlyphFontAtlasData` first checks atlas data cached on the shaped glyph, then
uses a shared glyph map, rasterizing/admitting only on a miss. The font cache has an owning thread,
atlas textures, deferred requests, conditional update/flush and explicit invalidation. Unreal also
tracks lazily loading font faces and bumps layout generation when they become ready.

The transferable rules are position-independent shaped identity, prepared run reuse, relative paint
transforms, glyph-owned resource handles, atlas-backed admission, explicit font publication and dirty/
visible layout. Zircon retains generation-safe Rust artifacts and exact fallback behavior; it must not
copy Unreal raw pointers or its global flush behavior literally.

## Target architecture

1. Runtime owns one immutable prepared text artifact per content/style/width/font generation. Glyphs
   are relative to line/block origin; measurement, overflow, hit testing and paint borrow it.
2. Editor presentation generations retain artifact handles/ranges. Recording stores those handles and
   paint applies origin/clip/color without rebuilding glyph vectors or String identifiers.
3. Text caches use borrowed/interned text identity, byte-accounted LRU generations and no x/y key.
   Dirty text/style/width/font changes invalidate only their prepared run.
4. Runtime artifact coverage is authoritative. A remaining host fallback is one-pass `O(G+H)` and
   reports why direct projection was unavailable.
5. A prepared font publication service performs scan/I/O/hash/parse away from paint, deduplicates
   source bytes/faces and atomically advances generation when ready.
6. Glyphs cache compact atlas/resource handles. A bounded owner admits pixels only on misses, reports
   resident/admitted/evicted bytes and separates CPU fallback residency from GPU atlas state.

## Instrumentation and acceptance

Matrix: text `empty/Latin/CJK/RTL/ligature/combining/emoji`, glyphs `1/100/1k/10k`, labels
`unique/repeated/translated`, wrap `none/word`, width `stable/change`, scale `1/1.25/1.5/2`, fonts
`stable/change/fallback`, phase `0..7`, mode `record-only/CPU/GPU/snapshot`, plugin panes `0/16/128`.

| Evidence | Acceptance |
| --- | --- |
| layout hit/miss, key-owned text bytes and origin-fragmented misses | translation changes: zero reshape/reprojection |
| Runtime artifact/fallback reason, shape passes and G/H operations | main path one shape; remaining fallback `O(G+H)` |
| record-only glyph/font projection | zero CPU-only glyph DTO/raster-face projection |
| font scan/I/O/hash/parse time and bytes on paint thread | zero after startup/publication |
| raster entries/bitmap bytes/admit/evict and lock wait | byte bounded; stable prepared glyphs avoid global lookup |
| preference/font snapshot reads | M1 host run: `G+1 -> 1` preference reads, `G -> 1` font captures |
| Swash miss allocations/copied bytes | same-format output: second allocation/copy eliminated |
| CPU/RSS/frame latency/context switches/WPR power | same executable/workload before and after |
| RenderDoc atlas upload/draw/batch/GPU plus pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add cache/origin/fallback/font/raster/lock/residency counters and capture scale baseline. | attributable text costs |
| M1 | Hoist run-stable smoothing/font context and reuse same-format Swash output buffers. | static operation/allocation reduction, focused green |
| M2 | Make Runtime relative prepared artifact authoritative for measure/record/paint; hard-cut x/y/String cache identity. | translated reuse, zero record-only projection |
| M3 | Close artifact gaps and make retained fallback one-pass linear. | one shape main path; fallback `O(G+H)` |
| M4 | Publish prepared deduplicated fonts off paint. | zero paint-thread scan/I/O/hash/parse |
| M5 | Add bounded atlas/CPU residency and glyph-owned handles; remove clear-all/global per-glyph lookup. | bounded bytes, admission-only raster |
| M6 | Run managed text scale, WPR/power, CPU/GPU and RenderDoc/pixel matrix. | quantified accepted milestone |

## M1 implementation result

CPU glyph drawing now captures smoothing once per run. A stack-local `OnceCell<HostTextFontSnapshot>`
is passed through the glyph loop: an exact artifact-only run never resolves a host font, while the
first host fallback glyph initializes one snapshot and every later fallback glyph borrows it. Raster
cache identity now receives the captured smoothing explicitly; the standalone raster test/helper API
retains its previous behavior by capturing context once before entering the shared core.

Swash bitmap conversion now truncates and returns its transferred `Vec<u8>` for alpha, retained
subpixel and color output. It no longer allocates/copies a second same-format buffer. Grayscale
conversion of a subpixel mask still allocates its intentionally different one-byte-per-pixel output.
Glyph keys, phases, formats, bitmap Arc ownership, cache contents and pixel sampling are unchanged.

| Static CPU glyph work for G glyphs | Before | After | Change |
| --- | ---: | ---: | ---: |
| text-preference/smoothing captures per run | `G+1` | 1 | `G` eliminated |
| host font-set/snapshot captures for G host-fallback glyphs | `G` | 1 | `G-1` eliminated |
| host font captures for exact artifact-only run | 0 | 0 | unchanged |
| artifact-only pre-scan passes | 0 | 0 | unchanged through lazy `OnceCell` |
| second same-format Swash output allocation per miss | 1 | 0 | eliminated |
| same-format Swash copied output bytes per miss | `B` | 0 | eliminated |
| global raster cache lookups/locks | `G` | `G` | unchanged, owned by M5 |

These are deterministic source/operation counts, not elapsed-time or power claims. M0/M5/M6 still
must measure run distributions, lock wait, bitmap residency and frame/power impact.

Post-M1 direct owner/path-test scope:

- Rust files: 37/37
- lines: 7,226
- bytes: 233,175
- joined UTF-8 path, NUL and raw-source-bytes SHA256:
  `e61f937f10c0a10b67bb2ef8e60feec3690cd88320138412829dd63c806a4968`
- unchanged files: 35 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `paint_text/draw/glyphs.rs` | 151 | 4,541 | `d15dd9640b0eb3679d60107374ec6903e99d0f879e3115ea8841ffa7cedd9f06` |
| `paint_text/raster.rs` | 406 | 13,071 | `dd205b227f7ab49574703782eae60570ea1170dfaf4f0de0cade33affb6b8a86` |

Focused static contract:
`tools/tests/test_editor_paint_text_run_context_performance_contract.py`, 50 lines, 2,026 bytes,
SHA256 `09d577ceec2452adfff6d979332df31010cf31e053325ead61166da36834bef3`.

## Validation state

- Full direct owner/path-test review: passed, 37/37 Rust files.
- Runtime layout/session and generation-safe glyph artifact support paths: traced/read.
- Relevant Unreal shaped-cache, text-run/layout and font-cache sources: read and mapped above.
- Existing profiling tests prove the exact artifact path avoids the second shaping pass, but ignored
  timing/counter scale tests are not current acceptance evidence.
- M1 focused static contract: RED 3/3 before implementation; the no-pre-scan refinement returned RED
  1/3; final implementation is GREEN 3/3.
- Current owned editor performance-contract set: GREEN 85/85 across 35 modules.
- Broad editor performance-contract set: 112/117 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for both changed Rust files and scoped `git diff --check`: passed.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
