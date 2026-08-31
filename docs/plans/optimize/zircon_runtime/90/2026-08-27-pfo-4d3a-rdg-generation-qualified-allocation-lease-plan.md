# PFO-4d3a RDG generation-qualified allocation lease plan

Status: `source_implemented_static_checks_passed_dynamic_validation_and_measured_optimization_pending`

## Scope

This slice replaces raw WGPU texture/buffer ownership transfers between `TransientResourcePool`
and `RenderGraphExecutionResources` with opaque, generation-qualified allocation leases. The lease
owns the descriptor key, physical descriptor, native allocation, sampled identity where applicable,
last-used frame, and optional last-use submission ticket.

The slice is an ownership/correctness convergence. It does not change the free-list lookup or
retirement algorithms and therefore makes no performance-improvement claim.

## Structural review

The current implementation already provides compiler-owned alias slots, explicit external imports,
an explicit persistent-extraction acquisition class, device-epoch invalidation, and completion-gated
reuse. Three plan gaps remain:

1. The pool returns raw `wgpu::Texture` / `wgpu::Buffer` values and graph execution stores those raw
   values directly. Generation and last-use submission facts remain in adjacent pool state rather
   than traveling with the physical allocation.
2. The free pools are `BTreeMap<DescriptorKey, Vec<_>>`; acquire/release are logarithmic in the
   number of descriptor classes, not amortized O(1).
3. `end_frame` scans the entire retained history for age and budget eviction, while
   `collect_completed_submissions` queries every pending resource separately even when resources
   share one submission ticket.

Items 2 and 3 are performance candidates. They require real profile evidence before implementation.
This plan records them but does not optimize them.

## Existing owner selected

`zr_rhi` already defines non-forgeable `TextureHandle` / `BufferHandle` values qualified by device
id, device generation, registry namespace, slot, and slot generation. Production
`zr_rhi_wgpu::WgpuResourceRegistry` is the single persistent native registry and already owns
last-use ticket retirement.

The native scene renderer cannot adopt those handles in this slice because its transitional
`WgpuNativeRecorderLease` exposes only a frame-scoped device/encoder callback and intentionally does
not expose registry lookup. Moving transient RDG resources into the persistent registry now would
require converting every graph executor to neutral command recording in the same change.

The temporary RDG allocation lease therefore remains graph-local and is not a second public
registry: it has no global lookup table, no copyable public handle, no queue/submission authority,
and no native accessor outside graph execution ownership. Its boundary is designed so the later
neutral executor cutover can replace the lease payload with `TextureHandle` / `BufferHandle`
without changing graph lifetime classification.

## Implementation contract

- `TransientTextureAllocation` and `TransientBufferAllocation` are move-only graph-local owners.
- Every allocation carries the active `(DeviceId, DeviceGeneration)` epoch and validates that epoch
  when acquired from or returned to the pool.
- Descriptor key, descriptor, byte size, frame age, and optional last-use ticket live on the lease.
- Texture descriptor compatibility includes the declared view-format set, and native creation passes
  that set to WGPU; a pool hit cannot reuse a texture missing a requested view-format declaration.
- Pool completion promotes only terminal `Completed` leases back to the free buckets; abnormal
  terminal results and query errors drop the lease fail-closed.
- Abort returns unsubmitted leases immediately after graph command encoders have been dropped.
- Successful submission stamps the exact scene submission ticket before pending retirement.
- External texture/buffer imports remain in separate maps and are never converted into pool leases.
- Persistent extraction remains a distinct graph lifetime/acquire class and is copied into retained
  history before the frame submission is accepted; the source lease still retires on that ticket.

## Static acceptance

- Raw WGPU resource types no longer appear in pool entry/pending ownership fields.
- `RenderGraphExecutionResources` owned transient maps store allocation leases; imported maps keep
  external WGPU leases.
- Direct native access is private to allocation methods consumed only by graph execution resources.
- Acquire/abort/submit/device-epoch paths preserve the same lifecycle ordering.
- Focused source contract tests cover generation qualification, last-use ticket stamping, external
  separation, and persistent extraction classification.
- Touched Rust files pass `rustfmt --check` and scoped diff checks.

## Deferred dynamic and performance acceptance

- Cargo compile/unit integration, WGPU validation, real PNG, RenderDoc capture, 300-frame profile,
  VRAM and power evidence remain pending under `docs/tests/runtime/render`.
- No O(1), bottleneck-removal, power-parity, or optimal-complexity claim is permitted from this
  source-only slice.
- A later measured slice may replace descriptor buckets with hash lookup and retirement scans with
  ticket buckets plus an age wheel only after profile counters demonstrate material frame cost.
