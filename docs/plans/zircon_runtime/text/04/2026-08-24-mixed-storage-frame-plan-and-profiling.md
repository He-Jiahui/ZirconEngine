---
kind: architecture_and_profiling_plan
status: architecture_review_complete_measurement_plan_complete_implementation_complete_validation_pending
origin_plan: docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
related_failure: docs/plans/zircon_runtime/text/04/failure-2026-07-18-native-bitmap-source-cache-idle-flush-and-linear-maintenance.md
---

# Text04 Mixed Storage Frame Plan And Profiling

## Decision

The native bitmap atlas must retain one canonical frame submission. It must not
turn every contiguous storage-format run into a cloned atlas and a rebuilt frame
plan. Draw order remains the original painter order. Texture/bind-group changes
are selected while replaying that order, so an `AlphaMask -> Color -> AlphaMask`
sequence remains three ordered draw segments without creating three frame plans.

This follows the relevant Unreal Slate invariants without copying its object
model: a shaped glyph first reuses its glyph/atlas cache identity, cache mutation
is centralized, and a requested flush occurs at a safe owner boundary. See
`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp`:
`GetShapedGlyphFontAtlasData`, `ConditionalFlushCache`, and `FlushCache`.

## Current-Source Evidence

| Owner | Current behavior | Structural consequence |
|---|---|---|
| `text/native_bitmap_atlas/storage.rs:23-46` | Splits contiguous formats and calls `from_frame_submission` once per run. | The number of submissions is `R`, the number of ordered format segments. |
| `storage.rs:138-205` | Clones `GlyphAtlasSet`, then filters/collects every major frame array for each run. | At least `O(R * G)` work for `G` frame glyphs; the atlas clone is materially larger than an order token. |
| `storage.rs:206-257` | Rebuilds dirty-page commands, batches, and GPU draw plan per run. | Upload and draw compilation are duplicated instead of sharing a frame-level result. |
| `native_bitmap_atlas/frame.rs:154-266` | Creates and aggregates `storage_submissions`; readiness is derived from their repeated projections. | Public report encodes the split implementation detail rather than resource ownership. |
| `scene_renderer/ui/text.rs:396-405,505-523` | Materializes renderer submissions and source-byte vectors for every run. | CPU vectors and renderer handoff scale with `R`. |
| `atlas_renderer/renderer.rs:222-279,317-351` | Prepares an instance buffer, upload plan, and shadow commit for every run. | Repeated GPU preparation and non-atomic frame shadow completion. |

The existing `Alpha -> Color -> Alpha` regressions prove required painter order,
but their assertion that there are exactly three storage submissions is an old
implementation detail. It must be replaced, not preserved as a compatibility
surface.

## Hard-Cut Design

1. Keep `NativeBitmapAtlasFrame` as the sole owner of the canonical
   `GlyphAtlasBitmapRenderSubmissionPlan`, source-image slice, atlas snapshot,
   and final shadow commit.
2. Replace `NativeBitmapAtlasStorageSubmission` and
   `GlyphAtlasBitmapRendererStorageSubmission` with a frame-level renderer
   input that borrows the canonical plan and source images. Do not retain a
   compatibility wrapper or per-format clone path.
3. Build prepared upload data once. Route each upload request to the atlas
   resource identified by its page format; reject a missing resource or stale
   generation before committing any page shadow.
4. Upload resource/page groups can be coalesced because they have no painter
   visibility. Render draw commands remain in source order and switch the
   atlas bind group only when the command requires another format resource.
5. Write one instance buffer for the whole frame. A draw segment stores an
   instance range and resource/pipeline key; its order is immutable.
6. Commit one combined page-shadow result only after all frame uploads are
   accepted. Any failed group fails closed and leaves the prior shadow state
   valid for the next frame.
7. Replace report fields that overload submission count with:
   `canonical_frame_plan_count`, `storage_resource_count`,
   `ordered_draw_segment_count`, `prepared_upload_group_count`,
   `instance_buffer_write_count`, and `shadow_commit_count`.

The source-cache font-byte snapshot remains a separate FontDatabase/FontBlob
ownership dependency. This change must not add a second font-byte, cache, slot,
or page owner in order to work around that unresolved foundation boundary.

## Required Regressions

- Alternating `AlphaMask/Color` at 1, 100, 1,000, and 10,000 glyphs preserves
  every painter-order token and every glyph instance range.
- The same workloads create one canonical frame plan and one frame shadow
  commit; they create at most one prepared upload group per actual page/resource
  group, never per ordered draw segment.
- Repeated formats share their atlas resource while retaining non-adjacent draw
  segments. `storage_resource_count` is two for Alpha/Color; ordered segments
  may equal glyph count for an alternating fixture.
- Upload failure, stale face epoch, missing format resource, and partial page
  failure do not commit a partial shadow or claim native replacement readiness.
- Existing persistent slots, page residency, DPI keys, placeholders, clipping,
  and Alpha/Color/Subpixel routes retain their current behavior.
- A deterministic WGPU framebuffer scene overlays alpha and color glyphs in an
  alternating order and compares the hard-cut frame with the pre-recorded
  baseline pixels before any proof PNG is exported.
- The final native-layout PNG must enter through `ScreenSpaceUiTextBatch`, its
  canonical shaped glyph identities, and `native_bitmap_atlas_glyph_runs` before
  reaching the atlas. The existing direct `NativeBitmapAtlasGlyphRun` WGPU scene
  remains a renderer/raster regression only: it proves real Swash output and GPU
  composition, but cannot by itself accept UI shaping, alignment, wrapping, or
  font-handle propagation.

## Measurement Protocol

### CPU Baseline And Post-Cut

Before deleting the old projection, run its deterministic fixture in an ignored
31-sample exporter in release mode. Warm each fixture, alternate workload order,
use nearest-rank p50/p95, and print raw samples plus these counters:

- glyph count `G`, ordered segment count `R`, distinct resource count `F`;
- frame-plan builds, deep atlas clones, source-byte vector materializations,
  prepared upload plans, instance-buffer writes, and shadow commits;
- CPU duration for frame projection, upload planning, renderer preparation, and
  total prepare path.

Repeat unchanged after the hard cut. The historical path is test-only baseline
evidence and must be deleted with the production hard cut; it is not a runtime
compatibility implementation. No fixed nanosecond threshold is valid before
recording the baseline on a controlled machine. The mandatory structural gates
are linear frame-plan ownership and zero deep atlas clones in the new path.

### Resource-Dispatch Follow-Up

The first canonical-frame implementation must also avoid reintroducing the
removed run partitioning cost inside GPU upload dispatch. A resource-loop that
filters every binding and collects a temporary vector for each resource costs
`O(F * B)` scans and allocates up to `F` short-lived vectors, where `F` is the
number of atlas formats and `B` is the upload binding count. It also obscures
the preflight-to-write atomicity boundary.

The corrected dispatch builds a fixed `GlyphAtlasFormat` resource table once,
preflights every binding before any queue write, and then replays bindings
directly against the selected texture. Its CPU work is `O(F + B)`, its extra
heap allocation count is zero, and a missing or duplicate format still rejects
the complete frame before the first GPU write. The scale fixture must record
the binding count, resource-table construction count, dispatch temporary-vector
allocation count, and missing-resource rejection count at 1, 100, 1,000, and
10,000 bindings.

### GPU And Power Qualification

Use the existing `profile_scope!` around the frame prepare/upload path together
with WGPU timestamp queries where the adapter supports them. Capture a matching
RenderDoc frame for each 1k and 10k workload, recording draw segments, bind-group
switches, `Queue::write_texture` bytes, instance-buffer bytes, GPU pass time,
and atlas VRAM. For power, collect a fixed-duration warm steady-state trace with
the same resolution, DPI, font collection, driver, adapter, and frame cap; report
the tool, sampling interval, ambient constraints, and raw trace. Do not compare
power or claim parity with Unreal until equivalent workload and image-quality
settings are recorded for both engines.

### Validation Order

1. Run the new deterministic unit and scale regressions through the managed
   Windows Cargo target pool.
2. Run the ignored 31-sample exporter and store text output in the plan record,
   not under `target` and not as a screenshot.
3. Run the WGPU product framebuffer test only after the shared RHI compile path
   is healthy. A real PNG may be written only by the passing product exporter to
   `docs/tests/runtime/text`.
4. Record the exact Cargo job, adapter, driver, resolution, feature set, raw
   measurements, and any WGPU/RenderDoc failure in the failure record.

## Current Status

Architecture review, measurement planning, and the canonical-frame implementation are complete.
The mixed-resource writer now builds a fixed format table, preflights the complete binding list,
and replays it without per-resource binding vectors; the renderer passes resource borrows without
materializing a resource vector. Focused source regressions cover missing and duplicate format
rejection. Scoped `rustfmt --check`, scoped `git diff --check`, and retired-dispatch source scans
pass. Managed Cargo validation, runtime timing, GPU timestamps, RenderDoc, power trace,
framebuffer acceptance, and a real-rendered PNG remain pending; no performance or visual result
is claimed by this record.

The native input hard cut has removed source-text, glyphon layout-buffer, `TextArea`, and renderer
fallback contracts from the atlas. A post-cut source review found the remaining integration gap:
artifact-backed horizontal `UiResolvedTextLayout` batches were routed to SDF before native
projection, even though their canonical artifact already carries glyph identity, font handles,
advances, offsets, and baseline. The correction is recorded in
`2026-08-24-native-layout-artifact-input-review.md`. This finding creates no measurement result
and remains subject to the same managed validation sequence.

The 2026-08-24 artifact-input repair is now implemented: `native_glyph_run.rs` consumes the
Text03 artifact's glyph sequence through the same batched handle-resolution and raster-key path
as ordinary canonical shaped batches. Horizontal artifact layouts no longer take an automatic SDF
route; vertical, visual-fallback, and distance-field-effect routes remain SDF. The ignored CJK
framebuffer scene now enters through `layout_text`, the normal UI render planner, and the native
projection rather than constructing a renderer batch itself. Focused source contracts,
`rustfmt --check`, and scoped `git diff --check` pass. Managed Cargo, WGPU screenshot export,
timings, GPU trace, RenderDoc, and power validation remain pending; no runtime result is claimed.

The 2026-08-24 post-cut review found that the ignored CJK native-layout exporter had constructed a
direct `NativeBitmapAtlasGlyphRun` from test-local font metrics. It has now been rerouted through
`ScreenSpaceUiTextBatch -> canonical shaped glyphs -> native_bitmap_atlas_glyph_runs`, using the
same shared font-database snapshot as rasterization; it retains real Swash/WGPU framebuffer
coverage and writes only to `docs/tests/runtime/text`. The managed WGPU run and resulting PNG are
still pending, so this implementation change is not an acceptance image or a performance result.
The separate low-level direct glyph-run renderer regression remains valid for renderer ownership,
but cannot stand in for this pipeline proof. Neither path permits renderer-local shaping or a
glyphon fallback.

On 2026-08-24, the actual ignored native-layout WGPU proof was brought under the
same target-directory guard as the native atlas proof. It now rejects relative
`CARGO_TARGET_DIR` values instead of resolving them against the workspace, and on
Windows rejects roots outside the coordinator-managed `D:`, `E:`, and `F:` drives.
Focused source contracts cover the relative and `C:` negative cases plus the
approved-root cases. This completes proof-output path governance only; it does
not constitute a Cargo, WGPU, framebuffer, performance, power, or PNG result.
