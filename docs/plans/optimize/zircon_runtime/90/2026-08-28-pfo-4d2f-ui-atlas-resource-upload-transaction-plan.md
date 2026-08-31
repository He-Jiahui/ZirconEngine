# PFO-4d2f UI Atlas Resource Upload Transaction

Status: `source_implemented_static_review_complete_dynamic_validation_pending`

Date: 2026-08-28

## Architecture Review

The screen-space UI renderer is an internal `zircon_runtime::graphics` subsystem. Its atlas
resource mutation is a leaf consumer of the existing generation-qualified WGPU submission owner;
it must not expose queue, poll, flush, or submit authority to text, atlas, or graph-pass code.

The remaining UI texture path has three structural defects:

1. SDF/MSDF and native bitmap atlas preparation call `Queue::write_texture` while UI draw recording
   is still fallible. Buffer uploads now belong to the outer frame transaction, but texture uploads
   bypass that transaction and its submission ledger.
2. `ScreenSpaceUiSdfAtlas::mark_prepared_pages_uploaded` clears the full-page retry intent before
   backend acceptance. Native bitmap preparation also calls `finish_bitmap_atlas_frame` and commits
   page shadows before the outer scene frame accepts or records any upload ticket.
3. A physical SDF or bitmap atlas texture may be replaced during preparation. If the frame then
   fails, the replacement remains selected while only prospective CPU identities were advanced or
   dropped. The next stable frame can therefore sample an uninitialized replacement. Bitmap atlas
   growth also needs replay of every committed page shadow, not only the newly dirty glyph rect.

The current algorithms are bounded by actual upload work after structural corrections. SDF command
generation and page/command matching use monotonic cursors over key-ordered owners instead of one
binary search per command. Bitmap staging hashes source identities and groups copies by page before
visiting that page's upload regions; full-replay membership uses an indexed set while preserving
resident-page order. This removes the previous cross-page `commands * all copies` scan. This review
found no evidence for a new sort, byte diff, or per-glyph registration layer. The required
optimization is transaction ownership, complete-batch validation, indexed page-local staging, and
full-shadow replay after physical replacement.

The owner-recovery review also found that the SDF material plan intentionally retains one default
material when it has no text or decoration draws. `material_count > 0` therefore cannot represent
active render contents: using it would promote every abandoned empty UI frame to a forced SDF
replay. Recovery admission must instead follow sampled atlas slots, generated vertices, or actual
draws. The atlas owner's own dirty/full-page state survives an empty plan, so this narrower predicate
does not weaken later texture initialization when text becomes visible again.

## Reference Alignment

Unreal Slate keeps font/texture atlas mutation on the render/RHI owner. `FSlateFontAtlasRHI` and
`FSlateTextureAtlasRHI` move copied atlas data into render-thread commands and mutate the RHI
texture there; Slate RDG separately publishes volatile geometry through `QueueBufferUpload` before
the consuming passes. Zircon must preserve the same single render-owner/order shape while retaining
its stronger fallible prepare/accept/commit semantics.

The local Lumen compute reference confirms explicit resource-state and command-list ordering, but
its D3D12 command queue is not an authority API for UI producers. `dev/Graphics` and the current
Zircon submission service reinforce that feature code should publish owned upload descriptions and
leave native queue operations to the backend.

## Target Design

1. Add `WgpuResourceUploadBatch`, an ownership-move packet containing buffer and texture batches.
   The WGPU submission service accepts it under one Copy ticket, applies all buffer writes then all
   texture writes under its single queue lock, and flushes the following graphics packet after both.
   Existing buffer-only and texture-only APIs remain ordinary constructors/consumers of the same
   packet semantics; no queue handle escapes.
2. Hard-cut `ScreenSpaceUiPreparedBufferUpload` to a complete `ScreenSpaceUiPreparedUpload` that
   owns both neutral batches and one move-only renderer generation token. Direct and compiled paths
   attach the complete UI resource packet to the existing frame upload acceptance point and commit
   UI state only after the scene command packet receives a ticket and the complete frame submission
   transaction validates that ticket ordering.
3. Convert atlas binding plans into immutable `WgpuTextureUpload` entries. One staged page payload
   is converted to `Arc<[u8]>` once and shared by texture ranges and the delayed page-shadow commit;
   preparation must not clone the page payload per range or retain borrowed source memory.
4. Keep native bitmap atlas ownership pending inside `ScreenSpaceUiTextSystem`. Success commits the
   candidate atlas/page shadow after frame acceptance. Abort or a dropped outer token invalidates
   candidate upload pages before the next preparation and preserves the committed atlas as the
   source of truth. Bitmap and SDF owners retain separate recovery debt, so an empty frame or an
   unrelated UI/text route may settle its own token without clearing another owner's required
   full replay.
5. Delay SDF `mark_prepared_pages_uploaded` until the same commit. A renderer-generation retry or
   physical atlas replacement forces full SDF/MSDF page upload. A dropped incremental SDF upload
   also promotes to owner-local full replay because the CPU cache transition is already prepared
   while the neutral texture batch was not submitted.
6. When a bitmap physical atlas is replaced, build full-page replay commands for every committed
   replayable page shadow, then overlay current dirty glyph copies. This remains one staging pass
   per replayed page and avoids reconstructing old glyphs or scanning source caches.

## Complexity And Performance Contract

- Resource-batch merge is `O(buffer writes + texture writes)` ownership movement with no payload
  clone and one logical Copy ticket.
- SDF atlas command generation and validation are
  `O(resident page metadata + dirty page metadata + dirty page bytes + upload ranges)`.
- Bitmap staging is `O(source count + copy count + sum(page commands * page copies) + staged bytes)`;
  committed-shadow pages cap upload regions at eight and physical replacement emits one full-page
  command, so the normal/retry paths reduce to `O(source count + copy count + staged bytes)` with a
  fixed page-local factor instead of the former cross-page quadratic scan.
- Physical replacement/retry is intentionally `O(committed atlas page bytes)`, because a new
  texture must be initialized completely before it can replace the old physical identity.
- Stable UI frames must produce zero atlas texture bytes and no UI-only ticket.
- This source design makes no runtime, power, or bottleneck-removal claim before dynamic evidence.

## Milestones

### M1 Mixed Resource Upload Foundation

Implementation status: `source_complete_static_review_complete_dynamic_validation_pending`.

- Add the mixed native packet and one-ticket submission-service path.
- Preserve separate buffer/texture write and rejection metrics while counting one admitted packet
  and one retained-byte total.
- Add source and unit guards for empty rejection, one ticket, payload ownership, write order, and
  upload-before-command flush order.

### M2 Atlas Neutral Upload Planning

Implementation status: `source_complete_static_review_complete_dynamic_validation_pending`.

- Replace atlas Queue writers with owned texture batches.
- Share staged page payloads with delayed bitmap shadow commits.
- Add full committed-shadow replay for bitmap physical replacement and full-page SDF retry.

### M3 UI Frame Transaction Integration

Implementation status: `source_complete_static_review_complete_dynamic_validation_pending`.

- Extend the UI prepared token across direct and compiled products.
- Commit bitmap/SDF state only after scene-ticket admission and full transaction validation.
- Cover dropped preparation, overlapping preparation, foreign attachment, failed admission,
  physical replacement, retry ordering, and empty-frame retry-debt preservation.

### M4 Testing And Product Evidence

Implementation status: `pending`; no Cargo, WGPU, screenshot, RenderDoc, profile, power, or product
acceptance claim is recorded by this source slice.

- Run the managed Windows compile/unit gate only after M1-M3 source slices are complete.
- Capture real rendered PNGs under `docs/tests/runtime/render`, then inspect the same product frame
  in RenderDoc from `D:\Tools\renderdoc`.
- Measure 1K/10K/100K UI vertices plus bitmap/SDF atlas churn: prepare CPU p50/p95/p99, staged and
  uploaded bytes, range count, Copy ticket count, native writes/submits, GPU frame time, VRAM peak,
  and process/GPU power where available.
- Debug from the lowest shared RHI/text owner, rerun upward, and only then record an accepted
  milestone or create a milestone commit/coordinator/WeCom update.

## Source Acceptance Boundary

- Production UI `Queue::write_texture` and `Queue::write_buffer` counts are both zero.
- Direct and compiled rendering admit one mixed frame resource packet and commit one UI token after
  ledger success.
- Atlas physical replacement replays committed content; dropped/admission-failed frames cannot
  publish page shadows or clear retry intent.
- Exact touched-file rustfmt, scoped diff checks, structure guards, and independent review pass.
- Cargo, real WGPU, screenshots, RenderDoc, profile, VRAM, and power remain pending until M4; source
  counts are not accepted as performance evidence.

## Current Progress

- `WgpuResourceUploadBatch` owns buffer and texture batches under one Copy ticket and one retained
  payload budget while retaining domain-specific write metrics.
- Native bitmap and SDF/MSDF atlas upload planning emits owned neutral texture ranges; production UI
  code has no direct `Queue::write_buffer` or `Queue::write_texture` call.
- Direct and compiled products each merge one frame resource packet, then delay bitmap, SDF, shadow,
  GPU-scene, HZB, exposure, and mesh-indirect state publication until the scene submission ticket
  passes transaction validation.
- Bitmap replacement replays committed shadows; SDF preparation rejects incomplete/non-canonical
  batches; failed UI frames preserve owner-scoped full-upload retry debt across empty and unrelated
  UI/text-route frames. SDF commands use a monotonic page merge, while bitmap sources and page
  copies are indexed before command staging.
- Bitmap and SDF recovery debt covers each active text owner's atlas and dynamic-buffer cache state;
  bitmap glyph copies additionally continue through the existing page/raster requeue path. An
  owner may clear debt only after committing either an empty canonical state that invalidates its
  upload caches or a complete forced replay, so inactive routes do not cause per-frame uploads.
- Complete owned buffer/texture payloads now use explicit infallible move constructors; upload
  write counts cross the admission boundary as typed values instead of being recovered through an
  enum `unreachable!` branch. The same review changed the SDF whole-batch fallback invariant to a
  fail-closed graphics error, leaving no production `expect`/`unreachable!` in this transaction
  slice and adding no payload copy or extra hot-path traversal.
- SDF owner recovery now ignores the always-present default material when atlas slots, vertices,
  and draws are all empty. This prevents abandoned empty UI frames from creating an unrelated SDF
  replay/upload while retaining atlas-local dirty/full-page state for later visible text.
- Upload admission deliberately charges every submitted write range rather than deduplicating
  shared application `Arc` allocations: WGPU still stages each `Queue::write_*` call separately,
  so pointer-based deduplication would understate native transfer pressure even though CPU source
  ownership is shared without cloning.
- Exact touched-file `rustfmt` and scoped source-contract checks passed on 2026-08-28. Dynamic
  compilation, real WGPU execution, RenderDoc capture, screenshots, and performance/power evidence
  remain M4 work.

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
