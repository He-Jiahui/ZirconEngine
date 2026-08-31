# Temporal History Domain Architecture Review (2026-08-29)

## Status

- Review: complete for the current WGPU scene-history owner and submit path.
- Source implementation: complete for the P0-2 domain-state/read-snapshot/write-intent/post-submit-commit hard cut.
- Dynamic validation: pending; the earlier managed `frame_history_` validator produced no output before the 244-second outer timeout, and the focused `history_domains_commit` request produced no output before a 47.5-second short timeout.
- GPU/RenderDoc/power evidence: pending. No performance or product-acceptance claim is made by this record.

## Scope

This review covers the persistent histories owned by `SceneFrameHistoryTextures`:

1. TAA scene color.
2. Hybrid global illumination lighting and temporal metadata.
3. Ambient occlusion.
4. Screen-space reflection.
5. HZB furthest mip chain.
6. Exposure.
7. Volumetric scattering.

The objective is to remove the global all-or-nothing validity model and establish an Unreal-aligned, per-domain history contract before further temporal quality or performance optimization.

## Current-source findings

### F1: one global validity bit controls unrelated domains

`prepare_history_textures` currently derives one `history_available` value from viewport publication, a monolithic allocation match, and recreation state. That value is forwarded through compiled-scene execution, graph binding, scene passes, post processing, SSR, HZB, AO, and volumetrics.

Consequences:

- A TAA allocation or camera discontinuity can invalidate AO, SSR, HZB, exposure, GI, and volumetrics together.
- AO, SSR, and HZB have no owned content-validity state, so a bound texture can be mistaken for valid history.
- Allocation compatibility, content validity, and resource binding availability are treated as one concept.
- Reset attribution cannot identify which domain was rejected or why.

### F2: history state is committed before scene submission succeeds

`copy_history_textures` mutates CPU history state while recording commands:

- TAA flips the read/write slot.
- GI and volumetric history set their validity flags.
- Exposure swaps read/write buffers.

The actual scene submission occurs later in `submit_compiled_scene_frame`. If command encoding, submission admission, or a pre-submit path fails, CPU state can describe writes that were never accepted by the queue. This violates the required meaning of `last_successful_frame`.

The existing renderer already demonstrates the correct transaction shape for environment cubemap uploads, reflection probes, IBL writeback, realtime IBL, and transient retirement: prepare during encoding, commit only after `SubmissionTicket` is returned.

### F3: allocation ownership is monolithic

Any mismatch in target size, HZB layout, TAA key, or volumetric quality replaces the entire `SceneFrameHistoryTextures` allocation. This couples domains with different extents and formats and causes avoidable texture recreation and cold starts.

P0-2 will first separate validity and commit semantics while retaining the allocation bundle. Per-domain allocation replacement remains a follow-up because it changes GPU resource lifetime and budget behavior and requires measured evidence.

## Unreal and Lumen reference

The local Unreal Engine reference uses domain-owned history state rather than whole-frame equality:

- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/PostProcess/TemporalAA.cpp`: TAA derives a camera-cut path from its own `InputHistory.IsValid()`, `View.bCameraCut`, and resource shape, then publishes its own output history.
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadiosity.cpp`: radiosity invalidates indirect-lighting history when a required atlas is missing or its extent/format changes.
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp`: screen-probe history separately validates required resources, lighting format, closure count, and optional short-range histories.
- `dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenViewState.h`: reflection, screen-probe, ReSTIR, radiance-cache, and radiosity histories retain distinct state and frame metadata.
- `dev/LumenInUE5.5.4WithComputeShader/Private/TemporalReprojection.hlsl`: temporal reuse is decided locally from reprojection/depth/visibility evidence, not from equality of unrelated scene payloads.

The required Zircon behavior is therefore domain-local invalidation plus a shared camera-cut signal, not a single global history switch.

## Target contract

### Domain identity

`SceneHistoryDomain` is a closed enum with seven values: `TaaSceneColor`, `HybridGlobalIllumination`, `AmbientOcclusion`, `ScreenSpaceReflection`, `HzbFurthest`, `Exposure`, and `VolumetricScattering`.

### Persistent state

Each domain owns `SceneHistoryDomainState`:

- `generation`: increments only when a submitted write for that domain is committed.
- `valid`: true only when a committed resource can be sampled by the next frame.
- `last_successful_frame`: the renderer frame generation that last committed the domain.
- `reset_reason`: the most recent reason that made the domain invalid.

The state is fixed-size and indexed by the enum. Reads and updates are O(1); no per-frame heap allocation or hash lookup is permitted.

### Reset reasons

The initial closed reason set is:

- `NeverProduced`
- `PreviousFrameUnavailable`
- `CameraCut`
- `AllocationChanged`
- `FeatureDisabled`
- `SourceUnavailable`
- `StructuralCompatibilityChanged`

Reasons are diagnostics and policy evidence. They are not inferred from texture existence.

### Read snapshot

At frame prepare, the owner captures an immutable `SceneHistoryAvailability` snapshot. Each consumer queries only its own domain. Resource binding may still bind a fallback texture, but shader-side history use must follow domain validity.

### Write intent and commit

History copies return `SceneHistoryWriteIntent`, a fixed bit set containing only domains whose copy/resolve command was encoded successfully. Encoding must not mutate persistent generation, validity, or ping-pong ownership.

After `submit_graphics_command_buffers_with_frame_diagnostics_and_surface` returns a `SubmissionTicket`, the scene submission owner commits the intent with the current `frame_generation`. Commit performs TAA/exposure ping-pong transitions and records validity/generation/last-successful-frame for the written domains.

On every pre-submit error, dropping the intent leaves persistent history unchanged. On a submitted frame that later reports an auxiliary finalization error, the history write is still committed because the GPU work has already entered the queue.

## Domain invalidation policy

| Domain | Shared camera cut | Allocation mismatch | Missing encoded source | Feature disabled |
|---|---:|---:|---:|---:|
| TAA scene color | invalidate | invalidate | invalidate | invalidate |
| Hybrid GI | invalidate | invalidate | invalidate | invalidate |
| Ambient occlusion | invalidate | invalidate | invalidate | invalidate |
| Screen-space reflection | invalidate | invalidate | invalidate | invalidate |
| HZB furthest | invalidate | invalidate | invalidate | invalidate |
| Exposure | preserve | invalidate | invalidate | invalidate |
| Volumetric scattering | invalidate | invalidate | invalidate | invalidate |

Exposure is not a reprojected spatial signal, so a camera cut must not reset it. Target recreation still resets it because its owned buffer pair is replaced.

The first implementation may invalidate all spatial domains when the retained viewport history is unavailable. It must not use that signal for exposure, and it must retain per-domain reasons so later policies can converge without another ownership rewrite.

When every history consumer is disabled, prepare still visits an already allocated history target long enough to commit `FeatureDisabled` for each domain. It does not allocate a new history target solely to record that disabled state; a viewport with no existing target remains allocation-free.

## Implementation sequence

1. Add the fixed-size domain state table, availability snapshot, reset reasons, and write-intent bit set beside `SceneFrameHistoryTextures`.
2. Move TAA, GI, volumetric, AO, SSR, HZB, and exposure validity into the domain table while preserving existing resource accessors.
3. Make history copy encoding return a write intent without flipping or validating persistent state.
4. Carry the intent into `CompiledSceneFrameSubmissionContext` and commit it immediately after successful scene submission.
5. Replace the global read flag in graph/resource/post-process contexts with the per-domain availability snapshot.
6. Add source-level and unit coverage for independent invalidation, no pre-submit mutation, post-submit commit, generation monotonicity, and exposure camera-cut preservation.
7. Only after dynamic correctness evidence, profile CPU submit cost, history-copy GPU passes, residency, bandwidth, frame time, and power before splitting physical allocations.

## Performance measurement plan

No optimization result is claimed before measurement. The post-correctness matrix is:

- Static complexity: O(1) domain lookup/update and one fixed seven-domain scan at prepare/commit.
- CPU: median/P95 history prepare, binding, copy-intent construction, and post-submit commit over at least 1,000 frames.
- GPU: timestamp each history-copy region and report bytes copied per domain.
- Memory: resident bytes per domain and recreation count.
- Quality/reset: valid-frame ratio and reset counts grouped by domain/reason.
- Power: steady 1080p and 1440p samples after warmup, compared with the same feature set before the hard cut and with documented engine-class expectations.

Acceptance requires that the structural rewrite removes unrelated-domain cold starts without introducing measurable submit-thread regression. GPU and power values must come from real WGPU/RenderDoc/tool captures stored outside `C:`; screenshots and capture evidence belong under `docs/tests/runtime/render` only after a running product frame is verified.

## Implemented source delta

- Added a fixed seven-entry `SceneHistoryDomainStates` table with per-domain `generation`, `valid`, `last_successful_frame`, and `reset_reason`.
- Added `SceneHistoryAvailability`, `SceneHistoryWriteIntent`, and a non-`Copy` `SceneHistoryFrameTransaction`; all lookup/update operations are fixed-index O(1), with fixed seven-domain prepare/commit scans and no heap allocation.
- Removed TAA, GI, and volumetric duplicate validity truth from their resource holders.
- Replaced TAA/GI/AO/SSR/HZB/volumetric read decisions with their own availability bits. Exposure remains outside spatial camera-cut invalidation and requests a default-buffer reset only when its own domain is invalid.
- Removed the invalid-frame clone plus `without_history_resources()` path. A cold frame now executes seed passes and publishes fresh domain writes instead of deleting the work needed to recover history.
- `copy_history_textures` now returns a write intent and never flips or validates persistent state during command encoding.
- `submit_compiled_scene_frame` commits exposure reset, TAA/exposure ping-pong ownership, domain validity, generation, and `last_successful_frame` immediately after the backend returns a scene `SubmissionTicket`, before auxiliary IBL finalization can report a post-submit error.
- Added pure CPU state-machine coverage for independent invalidation, commit-only validity, source-unavailable reset, generation/last-successful-frame, and reseed precedence; added source contracts for copy-time immutability and submit-before-history-commit ordering.
- Added the public read-only `RenderHistoryDomainsReport` projection without exposing scene texture ownership. The projection is written to `RenderGraphExecutionRecord` only after the history transaction commits, then flows through `RenderStats` into `DiagnosticStore`.
- Added 43 fixed-path diagnostic samples per submitted frame: one history-target-present flag plus `valid`, `generation`, `last_successful_frame_present`, `last_successful_frame`, `active_reset_reason_code`, and `frame_reset_reason_code` for each of the seven domains. The recording path performs no frame-time string construction or heap-backed domain lookup.
- `active_reset_reason_code` describes why a domain remains invalid after commit; `frame_reset_reason_code` preserves a reset event even when the same submitted frame successfully reseeds the domain. Code `0` means no reset; `1..7` map to `NeverProduced`, `PreviousFrameUnavailable`, `CameraCut`, `AllocationChanged`, `FeatureDisabled`, `SourceUnavailable`, and `StructuralCompatibilityChanged`.
- Added the crate-internal `RenderFrameHistoryInput` boundary object so runtime passes the authoritative framework invalidation reason with the handle and availability bit. The framework now distinguishes `CameraCut` from structural `FrameInputsChanged`; scene maps these to the spatial-domain `CameraCut` and `StructuralCompatibilityChanged` reasons respectively instead of collapsing both into `PreviousFrameUnavailable`.

Static evidence on 2026-08-29:

- Exact `rustfmt --edition 2021 --config skip_children=true --check`: passed for 21 touched Rust files.
- Scoped `git diff --check`: passed.
- `cargo metadata --locked --no-deps --format-version 1`: passed.
- Production source guards: copy-time flip/set count `0`; whole-frame history stripping/cloning count `0`; volumetric duplicate `valid: bool` count `0`.
- Largest touched production file remains `render.rs` at 906 lines, below the repository 1,000-line modularization gate. No new code/artifact was written under `C:`.
- The diagnostics follow-up passed exact `rustfmt --check` on 13 files, scoped `git diff --check`, and locked Cargo metadata. Its focused managed test request produced no output before the 54.1-second outer timeout, so this is source/static evidence only.

## Open items

- Managed Cargo/WGPU execution of the P0-2 tests after the validator lane becomes available; source/static completion is not dynamic acceptance.
- Capture the new per-domain diagnostics over the static/motion/cut/resize/provider-switch sequence and quantify valid ratio plus resets by reason.
- Per-domain physical allocation keys and independent recreation, contingent on measured recreation/budget pressure.
- RenderDoc marker/timestamp proof and real rendered screenshots.

## Physical allocation audit (2026-08-29)

The current constructor materializes one `SceneFrameHistoryTextures` bundle per `FrameHistoryHandle`. The bundle owns two `Rgba16Float` TAA scene-color textures, one `Rgba16Float` Hybrid GI color texture, one `Rgba16Float` Hybrid GI temporal-metadata texture, one `Rgba8Unorm` AO texture, one `Rgba16Float` SSR texture, one `Rgba16Float` HZB texture with its mip chain, two exposure storage buffers, and an optional `Rgba16Float` 3D volumetric texture. Resource creation is still bundle-wide even when only one consumer is enabled; the domain table currently separates validity and commit semantics, not physical allocation lifetime.

For a 1920x1080 target, one full-resolution `Rgba16Float` 2D surface is `1920 * 1080 * 8 = 16,588,800` bytes (15.82 MiB before backend tiling/alignment). The two TAA surfaces are therefore 31.64 MiB, GI color plus metadata another 31.64 MiB, SSR 15.82 MiB, and AO 7.91 MiB, for a static full-resolution subtotal of about 87.01 MiB before HZB mips, exposure buffers, volumetric 3D history, row alignment, and driver residency overhead. This is a planning upper bound, not a measured allocation report.

The allocation hypothesis is therefore narrower than “split every domain immediately”: if capture data shows low simultaneous feature occupancy or repeated bundle recreation caused by one domain's extent/quality change, independent physical keys should reduce peak resident bytes and recreation churn. If captures show most frames use the full bundle and recreation is rare, splitting may add bind-group/materialization overhead without improving frame time. The experiment must compare bundle-wide and per-domain candidates under identical scene, resolution, provider, quality, and warmup conditions.

Measurement gates before any physical split:

- record per-domain requested/enabled occupancy, creation/replacement count, resident bytes, and first-use latency over at least 1,000 submitted frames;
- timestamp history initialization clears, per-domain copy regions, and bind-group rebuilds with GPU timestamps, then report p50/p95/p99 CPU and GPU cost;
- run 1080p and 1440p static, camera-pan, camera-cut, resize, TAA-only, GI-only, SSR-only, all-features, and feature-toggle sequences;
- compare peak and steady-state VRAM, upload/copy bytes, valid-frame ratio, reset reason counts, and power after warmup;
- only implement a split when the measured reduction exceeds the added bind/materialization cost and the dynamic WGPU sequence proves no stale cross-domain reads.

No physical allocation optimization is claimed or implemented in this audit. The existing fixed-domain state/diagnostic work is the prerequisite for collecting these measurements without conflating unrelated history resets.
