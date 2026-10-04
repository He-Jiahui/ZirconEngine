---
title: Editor host presenter retained-artifact and backend-parity performance review
date: 2026-08-23
module: zircon_editor retained-host GPU and softbuffer presenters
priority: MVP-P0 editor paint, resize fallback and performance evidence integrity
status: source_reviewed_m0_actual_damage_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation/cached elements with Slint software dirty regions
---

# Goal

Make GPU and softbuffer presenters consume the same retained, generation-owned render artifact and
report the work they actually submit. Presentation diagnostics, resize behavior and fallback recovery
must not rebuild or clone the full editor scene, enlarge every dirty region, or publish optimistic
paint counters that disagree with the submitted draw list.

## Reviewed source

- owner Rust files: 32/32
- lines: 2,229
- bytes: 74,761
- source-only SHA256 over lexicographically sorted owner files:
  `816cb1491aa8d52482c1ab072e88d2fcb15b8094b63b2282f99c1d9b211371d0`
- post-M0 owner lines/bytes/SHA256: 2,230 / 74,824 /
  `65d269e6b73b633e9fe0aa46190a9a28fff7917fdfa7a5207908478126f88048`
- owning commit at review: `5f9704056761542857d74e733ce516f434de03dd`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `host_contract/presenter/*.rs` | 9/9 | 375 | 12,743 |
| `host_contract/presenter/gpu/**` | 5/5 | 1,002 | 32,668 |
| `host_contract/presenter/host_chrome_presenter/**` | 1/1 | 83 | 2,654 |
| `host_contract/presenter/softbuffer/**` | 17/17 | 769 | 26,696 |

All owner production and colocated test files were read in full. The direct call chain was also read
through window redraw/resize/retry, chrome command generation, RHI surface presentation, profile
artifact capture and performance counter gating. Those supporting owners are not counted here.

## Correct foundations to retain

1. GPU is the default native presenter and softbuffer is an explicit recovery/profile backend.
   Runtime-shared GPU surfaces remain behind a typed factory rather than leaking graphics handles into
   the window host.
2. GPU region presentation is gated on an initialized surface cache. Retryable no-submit does not
   publish a cache baseline; resize invalidates the ordinary damage baseline.
3. GPU native resize retains one versioned command draw-list per transaction, retargets only the
   physical surface size and rebuilds the ordinary baseline after the transaction.
4. Softbuffer retains a same-size RGBA backbuffer. Valid region streams repaint and copy only clipped
   rows, then call `present_with_damage`; invalid/out-of-bounds damage safely promotes to full.
5. Image residency is queried while command streams/draw lists are prepared, and GPU statistics expose
   uploads, shared resolves, compiled draws, visibility scans, batching, buffers, text and image caches.
6. The large GPU counter batch is compiled out in non-profiling builds and early-outs when capture is
   inactive. It is not a normal-product allocation hotspot.
7. Surface errors are explicit: retryable acquisition is distinct from fatal RHI/softbuffer failure.

## Structural findings

### P0: bootstrap/full GPU work is reported as a narrow region

When a narrow damage request arrives before `surface_cache_initialized`, GPU command construction
correctly ignores the damage and emits a full bootstrap stream. The diagnostic path nevertheless uses
the original requested rectangle and sets `region_present = damage.is_some()`, so it records only that
rectangle's pixels and increments region paint even though the submitted draw-list is full. The
existing test verifies the full draw-list and then expects the contradictory narrow counters.

This invalidated painted-pixel, full/region and amplification baselines after startup, resize and
retry. The first M0 change now captures `stream.damage()` before moving the stream into its draw-list
and uses that submitted damage for full/region and painted-pixel diagnostics. The cold cache contract
moved RED to GREEN and the existing Rust expectations now require a full bootstrap followed by a warm
region patch. Requested damage still needs a separate counter; it must never stand in for submitted
work.

### P0: presenters rebuild command artifacts from the complete presentation on every present

Both backends accept `&HostWindowPresentationData` and call a full command-stream entry point for each
present. Damage limits emitted commands, but the presenter has no generation-owned prepared artifact
contract; it cannot prove that stable layout/style/text/image ranges were already compiled. GPU RHI
caches can reuse later compiled data, but editor-side traversal and command extraction still happen
before the RHI sees the draw-list. Softbuffer then interprets that stream into its CPU frame.

This is the final consumer side of the adjacent chrome-command and retained-range reviews. M1 changes
the presenter input to a prepared immutable artifact containing source generation, retained command
ranges/resources, exact damage and target overlay. Presenter work becomes backend submission and
backend-local cache management, not full presentation discovery.

### P0: the softbuffer diagnostics overlay clones scene state and invalidates itself

Every softbuffer present calls `plan_present_for_diagnostics`. It clones the complete presentation DTO
to replace one debug string, builds a new overlay string from counters, and may iterate up to eight
times until overlay width, expanded damage and painted-pixel text stabilize. Because present counters
change every frame, a narrow unrelated region can also repaint the top-bar overlay. In verbose mode,
`presentation_summary` formats the complete summary before checking whether it matches the previous
one.

The clone is mostly shallow today, but it duplicates a wide ownership shape and forces command
generation to treat diagnostics as normal scene content. M2 makes diagnostics a small transient draw
layer with an independent generation and sampling policy. It can extend exact damage when visible,
but it cannot require cloning `HostWindowPresentationData` or feed its own same-frame pixel count back
through an iterative layout loop. Verbose summaries key off source generation before formatting.

### P0: native resize optimization exists only for GPU

The trait default for `present_during_native_resize` calls ordinary full `present`. GPU overrides it;
softbuffer does not. Each softbuffer resize clears the backbuffer, then rebuilds the full command
stream, CPU-rasterizes the entire current surface, converts/copies every pixel and submits full damage
for every intermediate native size. Complexity is O(K*N + sum(width*height)) for K size events and
scene size N.

M3 hard-cuts the conservative trait default and requires every backend to implement the shared
`NativeResizeTransaction` from the window review. Softbuffer retains one prepared scene snapshot per
transaction; intermediate sizes may scale/copy that snapshot but may not rediscover or repaint the
entire editor scene.

### P1: softbuffer has two surface-size authorities

The event loop already coalesces the latest native size and calls `presenter.resize`, yet each
softbuffer present calls `window.surface_size()` again and can resize itself. This adds a native query
to every fallback frame and preserves two ordering authorities. M3 passes an accepted metrics
generation through the presenter contract; the backend may validate on explicit surface failure, not
poll native size in normal present.

### P1: CPU raster and softbuffer use different pixel representations

The retained CPU frame stores RGBA8 while softbuffer requires packed `u32`. Each damaged pixel is
decoded and repacked in nested scalar loops after rasterization. Region clipping is correct and limits
the loop, so this is lower priority than scene rebuild/damage amplification. M4 measures conversion
bytes and cycles, then selects one of: native packed backing storage, a typed row converter using
validated vectorization, or direct raster into mapped rows. Pixel-format and alpha parity are required.

### P1: one rectangle remains the presenter damage ABI

GPU draw-lists, softbuffer repaint/copy and retry accept one optional rectangle. The adjacent redraw
and frame-geometry plans own `DamageRegionSet`; presenter M1/M5 must consume that exact bounded set and
preserve it through scissor/copy/submission instead of reducing it to a bounding rectangle.

## Unreal and secondary source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`
- `dev/slint/internal/renderers/software/lib.rs`

Unreal acquires one window draw buffer, asks each `SWindow` to paint through its persistent
invalidation root, and then submits the resulting draw buffer (`SlateApplication.cpp:1288-1305,
1501-1562`; `SWindow.cpp:2149`). `FSlateInvalidationRoot` chooses slow rebuild, fast invalid-list
update or no widget repaint (`SlateInvalidationRoot.cpp:356-424`). `FSlateCachedElementData` retains
per-widget cached element lists and only lists with new data (`DrawElements.h:152-230`). The
transferable rule is that presenter submission consumes retained invalidation output; it does not
reconstruct the source scene to discover work.

Unreal does not provide a directly comparable product softbuffer backend in this tree. Slint is used
only for that secondary implementation detail: its software renderer owns `PartialRenderingState`, a
bounded `DirtyRegion`, previous-frame dirty state and explicit new/reused/swapped buffer modes, then
renders/fills only those physical dirty ranges (`software/lib.rs:533-705`). This supports preserving
retained dirty ownership in CPU fallback; it is not used to override the Unreal-led editor architecture.

## Target architecture

1. Publish one immutable `PreparedHostPresent` per accepted source/target generation: retained command
   ranges, resources, exact damage set, target size/projection and transient layers.
2. GPU consumes the artifact into cached/owned draw-lists; softbuffer consumes the same artifact into
   retained CPU ranges/backing storage. No backend receives a full presentation DTO.
3. Record requested, prepared, submitted and presented damage separately. Full promotion has an
   explicit reason; actual submitted work owns painted-pixel/full/region counters.
4. Move diagnostics overlay and verbose summary outside normal presentation data. Key work by source
   and diagnostics generations and sample counters without a self-referential same-frame loop.
5. Require an explicit native-resize transaction implementation from every backend; remove the trait
   default full present and per-present native size polling.
6. Propagate bounded multi-region damage through command extraction, GPU scissors, software raster,
   buffer copy and surface present.
7. After structural work, choose a measured native softbuffer pixel path and delete redundant RGBA to
   packed-pixel conversion where the accepted backend supports it.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| requested/prepared/submitted/presented regions and pixels | separately reported; submitted counters match draw-list |
| source/range command visits and artifact builds | changed ranges only; stable source build zero |
| presentation DTO clones and overlay stabilization iterations | zero DTO clones; bounded non-iterative transient layer |
| resize command/scene/bitmap builds per backend | one source artifact per transaction |
| softbuffer raster/copy/format-convert bytes and cycles | exact dirty rows; measured implementation choice |
| GPU uploads/draws/scissors/cache hits/time | current-source RenderDoc and internal counter agreement |
| CPU/RSS/p95 latency/context switches/package energy | same executable/workload before and after |
| pixels, diagnostics text, retry and final resize layout | exact parity |

Matrix: backend `runtime GPU/standalone GPU/softbuffer`; cache `cold/warm/resized/retry`; source nodes
`1/1K/10K`; damage `none/one/opposite-2/8/64/full`; resize events `1/100/1K`; surface
`720p/1080p/4K`; overlay `off/stable/changing/verbose`; images `none/resident/upload`; retry `0/1/5`.

WPR owns CPU, allocation, wake, copy, disk-free present and package-energy evidence. RenderDoc owns
GPU draw/upload/scissor/time and pixel parity only. All artifacts and target directories stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Correct actual submitted-damage diagnostics and add requested/prepared/submitted counters. | cold/resize/retry full work cannot report narrow pixels |
| M1 | Introduce generation-owned `PreparedHostPresent` and retained command/range inputs. | stable source has zero command discovery |
| M2 | Extract diagnostics/verbose summary into a transient generation layer. | zero full DTO clones and self-invalidating loop |
| M3 | Require shared resize transactions and one size authority on all backends. | one scene artifact per K resize events |
| M4 | Select and implement the measured softbuffer backing/conversion path. | exact dirty bytes, lower CPU, pixel parity |
| M5 | Propagate multi-region damage and run WPR/power/RenderDoc matrix. | quantified current-source acceptance |

## Validation state

- Owner source review: passed, 32/32 Rust files.
- Window resize/retry, command generation, RHI surface, profiling gate and reference-engine consumers:
  read and mapped.
- M0 actual-damage source correction is applied. Its new static contract moved RED 0/1 to GREEN 1/1;
  adjacent window/presenter contracts pass 12/12 total.
- Changed Rust files pass independent `rustfmt --check` and scoped `git diff --check`.
- The discovered performance-contract suite passes 116/121. Its five failures are unchanged: two
  missing test-support files, missing `available_slots`, preview resize `.roots.clone()` and UI-asset
  root `.roots.clone()`.
- Managed Rust tests did not run, so M0's Rust behavior and requested/prepared/submitted counter matrix
  remain pending.
- M1-M5 remain architecture/dynamic work; no local softbuffer cache or overlay wrapper is accepted as
  a substitute for the shared artifact contracts.
- Managed Cargo, WPR and RenderDoc remain pending because the managed Cargo Session is terminal
  `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M5 pass on one source/executable/workload fingerprint.
