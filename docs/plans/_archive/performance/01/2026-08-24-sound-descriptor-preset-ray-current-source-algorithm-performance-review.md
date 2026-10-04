---
title: Sound Descriptor Preset and Ray Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_no_source_change
scope:
  - zircon_plugins/sound/runtime/src/descriptor_validation
  - zircon_plugins/sound/runtime/src/presets
  - zircon_plugins/sound/runtime/src/ray_tracing
  - zircon_plugins/sound/runtime/src/service_types/ray_tracing_convolution.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/acoustics.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp
  - dev/UnrealEngine/Engine/Plugins/AudioGameplayVolume/Source/AudioGameplayVolume/Private/AudioGameplayVolumeSubsystem.cpp
  - dev/UnrealEngine/Engine/Plugins/AudioGameplayVolume/Source/AudioGameplayVolume/Private/AudioGameplayVolumeProxy.cpp
---

# Sound Descriptor Preset and Ray Current-Source Algorithm Performance Review

## 1. Status and frozen scope

This slice completed E3 current-worktree static review over **25/25 Rust files** at revision `3e56d81da5e572849b51c50506ec65ec35fcf608`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| descriptor validation, built-in presets, ray/IR cache and acoustics service boundary | 25/25 | 791 / 729 | 26,690 | 0 / 0 | `53aa60e0033f06a4740c2d8b83a4cff2587ab60c037cf793bb5bed77b702a74d` |

All files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; the scope is clean and scoped diff check passes. Managed Windows Cargo, a launchable current-source Sound/scene executable, ETW, power and RenderDoc evidence remain unavailable. No source changed.

## 2. Per-module review ledger

| Module | Reviewed files | Static result |
|---|---|---|
| `descriptor_validation` | 13/13 | Scalar/range validation is generally explicit, but source graph validation rebuilds the same track index for every source and multiple full scans occur while the global manager state is held. |
| `presets` | 6/6 | Catalog construction is small, but all presets are eagerly built and two presets author effects that the active backend rejects. Default preset configuration is not applied. |
| `ray_tracing` | 4/4 | The module validates and stores externally supplied IR metadata; it performs no ray tracing, geometry capture, scheduling or convolution preparation. Budget fields are not enforced. |
| ray/acoustics service boundary | 2/2 | Public operations execute under the manager lock; snapshots deep-clone descriptors and sample vectors. There is no scene-lifecycle invalidation contract. |

## 3. Structural findings

### P0: advertised ray tracing is an unbounded external IR ingestion cache

`submit_ray_traced_impulse_response` receives a fully formed `SoundRayTracedImpulseResponseDescriptor`. No production path derives it from scene geometry, issues physics/ray queries, schedules work, builds an impulse response, partitions convolution data or hands a prepared kernel to the render backend. Repository-wide production call review found no caller outside the Sound service and tests.

The name and `RayTraced` status therefore overstate the implementation. Until a concrete acoustics provider exists, this surface must be reported as Unavailable or as a narrowly named external-IR cache. It cannot be used to claim ray-traced audio capability or performance.

### P0: configured convolution/ray budgets do not constrain runtime state

`SoundConvolutionBudget::{max_impulse_responses,max_partition_frames,rays_per_update}` is stored in configuration and exposed as package options, but none of those fields is consumed by the production ray/IR path. Submission has no maximum descriptor count, sample count, channel/frame layout, IR duration, byte residency or per-update work quota.

The cache can grow with every unique IR and retain arbitrary sample vectors. This is both a memory-exhaustion path and a blocker for deterministic frame/audio deadlines. Admission must compute exact resident/prepared bytes and reject, evict or defer work through an explicit policy before taking ownership.

### P0: each submitted IR is retained twice before backend preparation

The provider inserts `descriptor.samples.clone()` into `impulse_responses` and then stores the original descriptor in `ray_traced_impulse_responses`. This duplicates the entire sample payload, approximately `2 * samples.len() * size_of::<f32>()` plus collection overhead, before any FFT partitions or backend copies. Full descriptor snapshots clone the payload again.

Use one canonical immutable sample allocation, such as a provider-owned prepared IR object with shared immutable storage. Metadata indices reference that object; public snapshots return bounded summaries or explicit shared handles, not deep copies.

### P0: built-in presets can persist graphs that the active provider cannot execute

The Music/SFX preset includes a limiter and Spatial Room includes a reverb. The current active Kira graph surface rejects those advanced effects, yet the preset catalog is unconditional and the package exposes unsupported capabilities. `default_mixer_preset` has no production application site. A project may therefore discover or persist a graph that later fails only at activation.

Preset discovery, default selection, persistence and activation must be derived from the same applied provider generation. Unsupported presets remain unavailable/read-only with a reason; every selected graph is compiled and admitted before it becomes project state.

### P1: source graph validation is `O(S*T)` allocation/hash work

`validate_source_descriptor_for_graph` collects all graph track IDs into a `HashSet` for each source. `configured_sources` calls it once per configured source. For `S` sources and `T` tracks this repeats `S` allocations and `S*T` track visits, although the track set is invariant for one graph transaction.

Build a validated graph index once, then validate all sources against borrowed typed track/effect/parameter tables. The target preparation complexity is `O(T + total_bindings + total_sends)` rather than `O(S*T + total_bindings + total_sends)`.

### P1: validation and deep scans execute inside one coarse manager lock

External blocks scan every sample for finiteness; HRTF kernels are scanned for both finiteness and non-zero content; source parameters are matched by strings; listener/source track membership uses linear or freshly built indices. The service acquires mutable manager state before these validations, so malformed or large payloads extend contention for unrelated graph, source, timeline and output operations.

Parse and validate untrusted payload shape, size and finite values before the state lock. Compile names to typed slots and use immutable generation indices. Under the lock, only recheck generation/ownership and atomically publish the prepared object.

### P1: status is derived by rescanning metadata and does not describe executed work

Every submit/clear rescans all descriptors to compute the maximum recorded `rays_traced`, then publishes that value as `rays_per_update`. It is neither a rate nor evidence of a current provider update. A caller can also set acoustic status directly, independently of scene/provider/cache state.

Status must be provider-owned and monotonic per generation: requested, queued, active, last-good, failed and unavailable. Counters report actual submitted/completed/cancelled rays/jobs, IR build time, resident bytes, eviction and age. Maintain aggregates incrementally instead of rescanning all entries.

### P1: scene/source/listener lifecycle is disconnected from acoustic cache lifecycle

Validation checks that referenced sources/listeners/volumes exist at submission time, but no production path removes or invalidates cached descriptors when those owners are removed, moved, reparented or change world generation. Stale data can survive and still produce `RayTraced` status.

Acoustic data requires world/source/listener/geometry generation keys and explicit invalidation. Removal is a bounded command to the provider; late jobs compare generations and are discarded. Last-good data may remain only under a declared stale-age policy.

### P1: no product caller or dynamic qualification target exists

Repository-wide scan found built-in preset and ray/IR operations used only by Sound implementation code and tests. There is no current-source runtime/editor workflow that constructs geometry, selects a provider, submits work and renders the result. Consequently there is no honest WPR, power or RenderDoc target for this slice.

RenderDoc would only qualify later GPU ray/convolution dispatch, resource lifetime and output parity. It cannot prove CPU scheduling, lock contention, wakeups or audio deadline behavior; those require ETW/WPR plus engine counters and audio-thread underrun telemetry.

## 4. Positive baseline retained

Descriptor validation rejects non-finite scalar/sample data, invalid coordinates, missing graph ownership and zero ray counts. The ray cache replaces by stable IR ID and clear removes both current maps. These checks are useful boundary behavior; the plan moves them into a compiled, budgeted admission pipeline rather than deleting them.

## 5. Unreal-primary policy adopted

- `AudioMixerDevice.cpp:1277-1305` initializes audio plugins from the actual source count, sample rate, buffer size and device, and initializes only valid provider interfaces before the source manager.
- `AudioDevice.cpp:156-180` reports available spatial plugins separately from the active plugin. Authored choices are not treated as active capability.
- `AudioGameplayVolumeSubsystem.cpp:430-458,462-493,497-541` creates, updates and removes an audio-thread representation through explicit audio-thread commands.
- `AudioGameplayVolumeSubsystem.cpp:618-627,669-709` builds a transient proxy list and performs listener-volume search on the audio thread with explicit profiling scope.
- `AudioGameplayVolumeProxy.cpp:35-53,137-168` performs bounds culling before the more expensive physics-body distance query and stops at the first containing primitive.

Zircon adopts actual-provider capability, scene-to-audio proxy generations, early broad-phase culling and explicit thread ownership. It does not copy Unreal implementation details or invent Unreal-equivalent performance numbers; it measures its own providers and workloads.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability and preset truth | Rename/downgrade the current external-IR cache and filter presets/options by applied provider. | No unsupported preset or ray/convolution feature can be persisted, activated or reported Ready. |
| M1 Compiled validation indices | Build graph track/effect/parameter indices once; validate/prepare payloads outside the manager lock. | Source preparation is `O(T + B + sends)`; lock-held time and allocations are benchmarked. |
| M2 Acoustic provider contract | Define scene geometry snapshot, query backend, task affinity, deadline/cancellation and result-generation contracts. | A selected provider exists and produces IR data from a versioned world; absent providers remain Unavailable. |
| M3 Bounded IR residency | One canonical immutable/prepared sample allocation with byte/count/frame budgets and eviction/rejection. | Peak resident bytes stay within configured budget; no duplicate raw sample payload; oversize admission is deterministic. |
| M4 Lifecycle and proxy model | Versioned world/source/listener/volume proxies with add/update/remove invalidation. | Removed or stale owners cannot publish results; late jobs are cancelled/discarded; audio thread never queries mutable world objects. |
| M5 Truthful status and diagnostics | Provider-owned generation state and incremental actual-work counters. | Status reports queued/completed/cancelled work, build latency, resident bytes, cache hit/eviction, stale age and failures. |
| M6 Dynamic qualification | Current-source MVP and optional-provider workloads with repeatable scenes. | Record frame/audio callback P50/P95/P99, lock wait, allocations, rays/jobs, IR build, underruns, RSS, CPU, wakeups and power; RenderDoc only when GPU work exists. |

## 7. Direct-fix decision

No production edit is made. Hoisting the track `HashSet` is mechanically small, but the surrounding transaction currently admits presets/providers that are not executable and Rust validation is unavailable. It is the first tested M1 change after M0 capability admission, accompanied by an allocation/complexity regression benchmark.

Static review is complete only for these 25 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
