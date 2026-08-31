# PFO-4d1t UI Dynamic Buffer Upload Transaction

Status: `source_implemented_static_checks_and_independent_review_passed_dynamic_wgpu_validation_pending`

Date: 2026-08-28

## Structural Review

The production screen-space UI path still owns the last six direct `wgpu::Queue::write_buffer`
calls in graphics: generic geometry vertices, image vertices, bitmap-atlas instances, bitmap-atlas
viewport parameters, SDF vertices, and SDF materials. These calls are spread across four renderer
owners, but the direct scene path records all of them after its frame buffer batch has already been
accepted, while the compiled path records them inside a graph pass without attaching them to the
graph upload result.

The fatal issue is not six native calls in isolation. Each writer currently advances one or more
CPU-side reuse identities during preparation: payload hashes, weak retained-plan identities,
viewport transforms, uploaded materials, or compiled SDF frame inputs. If a later UI preparation,
render-graph pass, upload admission, or scene submission fails, a replacement buffer may remain
uninitialized while a retry treats it as synchronized. UI buffer ownership is therefore split from
the frame transaction and its reuse algorithm is not failure-safe.

The current hashing and packing work is already linear in payload size. Generic and image geometry
are generated once per invalidated retained segment, atlas instances and SDF materials are packed
contiguously, and SDF vertices are one contiguous POD slice. No sort, sparse map, per-vertex
registration, or byte-diff scan is justified without runtime profile evidence. The structural fix
must retain `O(payload bytes + invalidated segments)` preparation and ownership-move batch append.

## Reference Alignment

Unreal Slate keeps buffer creation and mutation on the render/RHI owner. Its legacy
`TSlateElementVertexBuffer::PreFillBuffer` performs grow-only capacity preparation before the
render-thread lock/copy/unlock, and `FSlateUpdatableInstanceBuffer::Update` moves CPU data into an
enqueued render command rather than exposing the RHI queue to the producer. The current Slate RDG
path in `SlateRHIRenderingPolicy.cpp` creates graph-tracked volatile vertex/index buffers and calls
`GraphBuilder.QueueBufferUpload`, binding those resources only in graph draw passes.

Zircon must preserve that ownership shape with its existing neutral `WgpuBufferUploadBatch` and
single frame upload ticket. It must not copy the explicit D3D12 upload/command-list API from
`dev/LumenInUE5.5.4WithComputeShader`; that project is a pass/resource/barrier reference, not an
authority model for UI uploads.

## Design

1. Introduce a screen-space UI prepared buffer-upload object that owns one immutable
   `WgpuBufferUploadBatch` plus move-only commit data for all reuse identities changed by the frame.
2. Every UI buffer writer may create/grow its physical buffer during preparation, but it appends
   payload/range ownership to the prepared batch and does not publish the next hash, viewport,
   material shadow, retained-plan identity, or compiled-frame identity before commit.
3. The direct scene path records UI before accepting `frame_buffer_uploads`, appends the prepared UI
   batch, then commits UI state only after backend upload acceptance and ledger registration.
4. The compiled graph pass returns the prepared UI batch and commit data through graph execution;
   the outer frame merges it into the same upload ticket and applies the commit only after backend
   acceptance and ledger registration.
5. A replacement buffer always produces a full payload upload even when bytes equal the committed
   shadow. Dropping a preparation leaves the old committed identity intact so the next frame retries.
6. Exactly one outstanding UI preparation is allowed per renderer owner. The reservation is held
   through the outer frame and released by successful commit or drop, preventing a later physical
   replacement from invalidating an older pending commit.
7. Bitmap/SDF atlas texture writes remain in the typed texture-upload domain for PFO-4d2 follow-up.
   This slice removes raw buffer writes without disguising texture queue authority as buffer work.
8. Focused tests must cover stable reuse, changed payload accounting, buffer replacement, dropped
   preparation retry, overlapping preparation rejection, and commit-after-accept ordering in both
   direct and compiled products.

## Performance Measurement Plan

Dynamic validation will capture text-free geometry, image-heavy UI, bitmap text, and SDF text at
1K/10K/100K UI vertices. Record UI prepare CPU p50/p95/p99, invalidated segment count, hash bytes,
packed bytes, upload ranges/bytes, frame upload ticket count, native buffer writes/copies, GPU frame
time, VRAM peak, and process/GPU power where exposed. Stable retained frames must produce zero UI
buffer bytes and zero UI-specific ticket; invalidated work must grow linearly with payload and
invalidated segments. RenderDoc must show the upload before the consuming scene submission.

No source count is evidence that the bottleneck or power target has been met. Algorithm changes
beyond transaction ownership require profile evidence that identifies hashing, packing, range
count, or allocation as the actual cost.

## Source Acceptance Boundary

Focused source acceptance requires zero production UI `queue.write_buffer`, one prepared UI upload
transaction, direct and compiled attachment to the existing frame batch, post-accept commit, and
abort/retry plus ordering guards. Exact touched-file rustfmt and scoped diff checks are required.
Cargo, real WGPU, product PNG, RenderDoc, profile, VRAM, and power remain pending until the managed
Windows validation lane is available; the status must stay source-only until those artifacts exist.

## Completed Source Work

1. The six production UI buffer writers now publish `WgpuBufferUpload::from_bytes` entries into one
   prepared UI batch. Production `queue.write_buffer` in the UI renderer is zero; bitmap/SDF atlas
   texture writes intentionally remain in the separate texture-upload follow-up.
2. `ScreenSpaceUiRenderer` owns one outstanding preparation and one unforgeable owner identity. A
   prepared/committed generation mismatch forces all six child owners to upload in full, so a
   dropped frame cannot reuse prospective hashes, viewport state, material shadows, or retained
   plans for an unaccepted buffer.
3. Direct rendering records UI, attaches its batch, accepts the one frame upload ticket, records it
   in the submission ledger, and only then commits UI reuse state. Compiled rendering moves the
   token through pass and stage results and follows the same outer acceptance/commit ordering.
4. Product framebuffer tests now admit their prepared neutral buffer batches through
   `RenderBackend`; structure guards reject raw UI buffer writes and lock the six neutral sites plus
   direct/compiled ordering. Unit tests cover dropped-preparation retry, overlap rejection,
   cross-transaction attachment rejection, and the requirement that commit follows attachment.
5. Focused counts are: production UI raw buffer writes `0`; neutral UI buffer upload sites `6`;
   direct attach/commit `1/1`; compiled take/commit `1/1`; abort retry, overlap rejection, and
   append-before-commit and foreign-transaction rejection tests `1/1/1/1`. Exact touched-file
   rustfmt and scoped diff checks passed.
6. Cargo, real WGPU, product PNG, RenderDoc, 1K/10K/100K profile, VRAM, and power remain pending.
   This slice makes no runtime performance or power claim.
7. Independent review found one direct-path cleanup defect: UI attachment, upload admission, or
   submission-ledger failure returned without deferring the GPU timer frame or releasing the
   realtime-IBL scheduler token. All three exits now perform the same cleanup as the surrounding
   direct and compiled failure paths before returning. Independent recheck confirmed all three
   failure exits perform timer defer plus realtime-IBL abort, prepared-token drop preserves full
   retry, and the three product framebuffer paths flush neutral uploads before draw submission. No
   new correctness issue was found in the static recheck.
