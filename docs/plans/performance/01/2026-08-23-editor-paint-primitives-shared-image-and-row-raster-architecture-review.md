---
title: Editor paint primitives shared-image and row-raster performance review
date: 2026-08-23
module: zircon_editor retained-host paint_primitives
priority: MVP-P0 editor viewport recording and primitive paint
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate resource handles and cached draw elements; Slint software row raster
---

# Goal

Keep primitive paint proportional to visible commands, newly changed resource bytes and edge coverage.
A captured viewport already owned by `Arc<[u8]>` must cross paint recording by shared ownership, GPU
recording must never execute CPU raster work, and software rounded primitives must not rebuild geometry
for every pixel in their bounding box.

## Reviewed source

- Rust files: 26/26
- lines: 1,740
- bytes: 50,753
- joined normalized UTF-8 path, NUL and raw-source-bytes SHA256:
  `f194323f41ec52711f9c1edd7dbb9003a9f0187680a8fdb26b33f8d827df4d69`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

Scope: `paint_primitives.rs` and `paint_primitives/**`.

Supporting production paths traced/read: `HostRgbaFrame` constructors/recording state, chrome CPU
image replay, retained template image paint, native pane viewport paint, `HostViewportImageData`,
paint-text measurement/layout/cache and normal command-stream extraction.

This current-source review supersedes the primitive counts and current findings in
`2026-07-17-editor-paint-frame-primitives-static-review.md`; the older report remains useful history
for the one-border-command repair and PERF-MVP-155 provenance.

## Correct foundations to retain

1. `HostRgbaFrame` exposes distinct pixel-backed and recording-only constructors. Normal GPU command
   extraction uses recording-only storage and therefore does not allocate or touch a full RGBA frame.
2. Rect, border and image primitives resolve active/explicit clips and reject empty target pixel
   bounds before command emission or CPU rasterization.
3. Recording-only square and rounded borders emit one typed border command, including wide borders;
   they do not expand to four quads per edge layer.
4. Opaque identity CPU images use contiguous row copies; scaled images use center-sampled bilinear
   filtering and premultiplied-alpha interpolation without transparent-RGB bleed.
5. Retained template images already pass `Arc<[u8]>` through the shared recording API. Atlas recording
   keeps page pixels in the atlas payload instead of duplicating the source image.
6. Opaque rect fills and separator lines operate on continuous row spans. Coverage tests preserve
   fractional rounded edges and symmetry.

## Structural findings

### P0: viewport paint discards shared ownership and copies the full capture

`HostViewportImageData` stores captured CPU pixels as `Option<Arc<[u8]>>`, and presentation snapshots
also retain the viewport object by `Arc`. Its `rgba()` accessor returns only `Option<&[u8]>`.
`native_panes/viewport.rs` therefore calls the non-shared primitive; record-only image recording runs
`Arc::from(rgba)` and copies the entire capture into another allocation on every accepted viewport
repaint.

For a W by H capture the redundant work is exactly `4WH` bytes plus allocation per recorded frame:
1,920x1,080 is 8,294,400 bytes and 3,840x2,160 is 33,177,600 bytes. M1 makes the accessor return the
existing `&Arc<[u8]>` and routes viewport paint through the shared primitive. Pixel identity, resource
generation, clip and command shape remain unchanged; the command takes one O(1) Arc owner.

### P0: the raw recording API permits payload hashing and full pixel ownership per command

`ImageRecordingMetadata::ResourceKey` copies bytes into a new Arc whenever a pixel-backed frame is
recorded. Without an explicit resource key it also hashes the whole payload before copying it. Current
template paint already uses the shared API; chrome replay uses the raw API only on a non-recording CPU
frame. After M1, the product viewport caller is shared as well.

M2 makes shared generation-qualified image ownership the only recording contract and confines raw
borrowed slices to immediate CPU raster. The old raw recording path is hard-cut after tests/callers
migrate; a wrapper that silently copies would preserve the defect.

### P0: software rounded geometry is rebuilt across the full target rectangle

Rounded fill iterates every pixel in the bounding target. Each call to coverage rechecks frame
visibility, reclamps radius and reconstructs center/half extents; edge pixels evaluate a 4x4 sample
grid. Rounded borders perform outer and inner coverage, so an edge pixel can execute 32 signed-distance
evaluations, each repeating rectangle setup and `hypot` work. Large software/snapshot rounded panels
therefore scale with area and expensive per-pixel geometry, not primarily with rows and edge coverage.

M3 prepares outer/inner rounded geometry once and rasterizes per row: calculate left/right edge spans,
blend only antialiased edge pixels and fill interior runs as slices. GPU recording remains one typed
rounded command. Pixel parity must be proven before replacing the current reference raster.

### P0: text-marker sizing measures before the paint layout measures again

`draw_text_bars_clipped` calls Runtime `measure_text_size` to determine its frame width, then
`draw_text_with_size_and_style` performs Runtime `layout_text` for the same text. Even a layout-cache
hit first constructs an owned String cache key and locks a global cache. Hierarchy rows, close prompts,
fallback panes and debug overlays all use this marker path.

M4 returns one prepared text layout/metrics artifact that supplies both width and drawing. This is
owned jointly with `paint_text/**`; adding a second marker-local cache is rejected.

### P1: identity image fast path reads opaque pixels twice

The identity fast path scans every visible source pixel to prove alpha 255 and then copies the same
rows. This is preferable to bilinear blending for known opaque images but still costs one validation
pass per draw. M5 stores opacity/format metadata with the generation-qualified image resource so known
opaque resources copy once; unknown raw slices keep the safe validation path.

### P1: CPU pixel APIs repeatedly borrow the frame per rounded pixel

Rounded coverage writes call `frame.width()` and `frame.as_bytes_mut()` through `write_pixel` for each
accepted pixel. M3's row raster holds frame width/storage once and blends contiguous slices. Unsafe
pointer arithmetic is unnecessary.

### P2: small clip/FrameRect clones are not an allocation bottleneck

`effective_clip` returns owned `FrameRect` values and several recording paths clone them. These are
fixed-size stack values with no heap payload. They can converge with the canonical geometry owner, but
must not displace full-image copies, repeated shaping or rounded pixel geometry in priority.

## Reference-engine source basis

Unreal direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElements.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElementTypes.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering/DrawElementPayloads.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElementTypes.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIResourceManager.cpp`

Slate culls boxes before enqueue, stores typed box/rounded/border elements in the window element list
and resolves a brush to an `FSlateResourceHandle`/resource proxy; it does not copy source texture bytes
into each draw element. Viewports are enqueued through a shared viewport interface. Cached element data
retains source draw elements and render batches across invalidation.

Software-algorithm source read:

- `dev/slint/internal/renderers/software/scene.rs`
- `dev/slint/internal/renderers/software/draw_functions.rs`

Slint stores one prepared `RoundedRectangle` with radius, border width/colors and clip amounts, then
draws one scanline at a time. Its row function calculates curve intersections, blends narrow
antialiasing ranges and fills interior/border slices. Zircon need not copy its fixed-point details, but
must adopt the same prepared-geometry/row-span complexity boundary for the software backend.

## Target architecture

1. Paint commands carry compact generation-qualified image handles and O(1) shared pixel owners only
   when an upload is actually needed. Immediate CPU raster may borrow raw slices without recording.
2. GPU recording, CPU replay and snapshot modes are explicit targets; impossible hybrid branches are
   removed after callers/tests converge.
3. Rounded rect/border commands prepare geometry once. GPU submits typed primitives; software raster
   evaluates curve intersections per row and touches individual pixels only at antialiased edges.
4. Text measurement and drawing consume one prepared layout artifact. A generation-owned layout cache
   is shared with Runtime UI rather than shadowed by marker helpers.
5. Image resources carry opacity/format metadata so identity copy does not rescan known generations.

## Instrumentation and acceptance

Matrix: viewport `640x360/1080p/4K`, images `opaque/translucent/identity/scaled`, rounded bounds
`1/100/1k/10k`, radius `0/4/32`, border `0/1/8`, damage `outside/edge/interior/full`, text rows
`1/1k/10k`, backend `GPU/softbuffer/snapshot`.

| Evidence | Acceptance |
| --- | --- |
| viewport pixel copied/shared bytes and Arc owner count | stable capture copy bytes `4WH -> 0`; one shared owner |
| command/resource hash and upload bytes | zero steady raw-payload hash/copy/upload |
| rounded geometry evaluations, edge pixels and span pixels | setup O(commands), intersections O(rows), samples O(edge pixels) |
| text measure/layout/cache-key/lock counts | one prepared layout per changed text/layout generation |
| identity alpha-scan/copied bytes | known opaque generation scans `P -> 0`, copies `P` |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/upload/GPU and pixel/text parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add viewport copied/shared-byte, raw hash/copy, rounded evaluation/span and text layout counters; capture. | attributable baseline |
| M1 | Preserve captured viewport `Arc<[u8]>` through primitive recording. | per repaint copy bytes `4WH -> 0` |
| M2 | Hard-cut raw-slice recording in favor of generation-qualified shared image handles. | zero product raw payload copy/hash |
| M3 | Add prepared rounded geometry and row-span software raster. | setup O(commands), curve work O(rows/edges) |
| M4 | Share one prepared text layout between marker sizing and draw. | duplicate measure/layout eliminated |
| M5 | Add image opacity metadata and explicit paint targets; converge geometry ownership. | known opaque one-pass copy |
| M6 | Run managed scale, interaction, WPR/power and RenderDoc/pixel/text matrix. | quantified accepted milestone |

## M1 implementation result

`HostViewportImageData::rgba()` now returns `Option<&Arc<[u8]>>` rather than erasing shared ownership
to `Option<&[u8]>`. Native viewport paint routes captured CPU pixels through
`draw_shared_rgba_image_clipped_with_resource_key`; GPU-only viewport products keep the existing
external-resource command path.

The shared primitive records `Arc::clone` of the existing capture allocation. Resource key, dimensions,
clip, damage rejection and image bytes are unchanged. No compatibility accessor remains that can
silently return the viewport pixels as a raw recording slice.

| Deterministic captured-frame work per accepted recording | Before | After | Change |
| --- | ---: | ---: | ---: |
| pixel allocation/copy count | 1 | 0 | eliminated |
| copied pixel bytes | `4WH` | 0 | eliminated |
| 1,920x1,080 copied bytes | 8,294,400 | 0 | -7.91 MiB |
| 3,840x2,160 copied bytes | 33,177,600 | 0 | -31.64 MiB |
| command pixel owner operation | full allocation/copy | one `Arc::clone` | O(1) |

These are exact ownership/byte counts from the API path, not elapsed-time or upload claims. Resource
residency still decides whether the shared bytes are uploaded, and M0/M6 must measure that backend work.

The 26 direct primitive files remain unchanged after M1 and retain the reviewed 1,740 lines, 50,753
bytes and joined SHA256
`f194323f41ec52711f9c1edd7dbb9003a9f0187680a8fdb26b33f8d827df4d69`.

| Changed supporting owner | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `host_contract/data/viewport_image.rs` | 119 | 3,798 | `939eb344bd1374f726f756a5bc3e3513d30a6f684f06e7d6d5506046ec139735` |
| `paint_workbench_renderer/native_panes/viewport.rs` | 39 | 1,223 | `8b50659232e6121640fd25f31349988fcb1480fe0c68a10e91b9402ffcdf8225` |

Focused static contract:
`tools/tests/test_editor_viewport_shared_capture_performance_contract.py`, 35 lines, 1,308 bytes,
SHA256 `6f4f4f098a3c7c83f7701d123354f1dbad9c9fa8c78db2537468b3f5f9a0f797`.

## Validation state

- Full direct owner review: passed, 26/26 Rust files.
- Viewport/template/chrome image callers, frame modes and text layout boundary: traced/read.
- Unreal resource/draw-element/invalidation and Slint software row-raster sources: read and mapped.
- M1 focused static contract: RED 2/2 before implementation, GREEN 2/2 after implementation.
- Existing primitive Rust tests already prove the shared recording primitive reuses the same Arc pixel
  allocation; integration Rust behavior is not claimed passing until managed Cargo is executable.
- Current owned editor performance-contract set: GREEN 79/79 across 33 modules.
- Broad editor performance-contract set: 106/111 passed; its five failures are the unchanged known
  missing `component_showcase_state.rs`, missing `workbench_projection.rs`, missing `available_slots`,
  preview resize `.roots.clone()` and UI asset root dirty-helper `.roots.clone()` findings.
- `rustfmt --check` for both changed supporting Rust files and scoped `git diff --check`: passed.
- Managed Rust, WPR and RenderDoc validation remain pending because the managed Cargo Session is
  terminal `archived` with `cargo_session_not_executable`; no elapsed-time, GPU or power claim exists.

This module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
