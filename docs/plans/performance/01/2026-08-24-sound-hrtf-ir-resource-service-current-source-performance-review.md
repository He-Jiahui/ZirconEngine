---
title: Sound HRTF and IR Resource Service Current-Source Performance Review
date: 2026-08-24
status: static_complete_source_fix_dynamic_pending
scope:
  - zircon_plugins/sound/runtime/src/service_types/hrtf_profiles.rs
  - zircon_plugins/sound/runtime/src/service_types/impulse_responses.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/ConvolutionAlgorithm.h
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/UniformPartitionConvolution.cpp
---

# Sound HRTF and IR Resource Service Current-Source Performance Review

## 1. Status and frozen scope

The remaining Sound production resource-service slice completed E3 current-worktree static review over **2/2 Rust files** at revision `c02a7fb7c4b90381b9e701008bc8a2898fc09263`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| HRTF profile and impulse-response resource service | 2/2 | 81 / 72 | 2,870 | 0 / 0 | `74764428af26f4fec5667f2bb8f6373ec31db6651dbcb370744ebb5ae5b638c9` |

Both files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped `git diff --check` passes. This review removes one redundant deep clone from `remove_impulse_response_impl`. The existing ray-cache behavior test was inspected but could not be executed because managed Windows Cargo and a launchable current-source Sound artifact remain unavailable. ETW, power and audio-output evidence remain pending.

## 2. Per-file review ledger

| File | Static result |
|---|---|
| `hrtf_profiles.rs` | Validates payload before locking, then stores unbounded owned kernels. Every add/replace/remove invalidates all HRTF render states. Listing deep-clones every kernel and sorts on every call. |
| `impulse_responses.rs` | Validates finite samples before locking, but accepts unbounded formatless raw vectors. Static and ray-generated IRs share one ID map without source ownership. Removal updates both maps and ray status. |

## 3. Direct optimization completed

Before this review, removing any impulse response cloned the entire `ray_traced_impulse_responses` map, including every descriptor's owned sample vector, only to pass an immutable map to `refresh_ray_tracing_status`. The adjacent ray-provider clear path already proves that disjoint state fields can be borrowed directly.

The removal path now passes `&state.ray_traced_impulse_responses` directly. The redundant descriptor-map clone count is **1 -> 0** and copied sample elements are **sum of all cached ray IR samples -> 0** for this operation. Remaining status refresh cost is `O(R)` scalar descriptor visits for `R` cached descriptors; replacing it with incremental truthful status belongs to the structural plan.

Behavioral coverage already exists in `runtime/src/tests/ray_tracing/cache.rs`: after clear, the ray descriptor list is empty, status is `WaitingForGeometryProvider`, and removing the static IR returns Unknown. That test was not executed in this session and is not claimed passing.

## 4. Structural findings

### P0: resource admission is unbounded and cannot prepare a correct provider object

HRTF descriptors own two arbitrary-length kernels plus notes. Static IRs are only `(id, Vec<f32>)`, without sample rate, channel layout, frame count, duration, normalization, source locator, provider identity or prepared partition size. Validation scans finite values but enforces no count/byte/time budget. Both maps can grow for the manager lifetime.

Admission must resolve a versioned asset/provider receipt, validate exact format, compute raw and prepared bytes, enforce count/frame/byte budgets and prepare provider-specific immutable data before publication. Raw payloads are not the render-thread object.

### P0: static and ray-generated IR ownership can contradict each other

Ray submission writes the same ID into both the raw IR map and ray descriptor map. A later `set_impulse_response` with that ID replaces raw samples but leaves the ray descriptor and `RayTraced` status unchanged. Metadata can therefore claim ray-derived cell/source/listener/ray count while rendering would read unrelated static samples. Removal happens to clear both, but replacement has no ownership rule.

Use a single canonical IR entry with explicit origin/provider/generation and one immutable sample/prepared handle. Replacement is a validated generation transaction; dependent graph sends observe old or new last-good atomically.

### P0: every HRTF profile change invalidates every source/profile state

Loading, replacing or removing one profile calls `hrtf_states.clear()`. The cost is `O(H)` destruction for all active HRTF render states, including unrelated profiles, and it discards convolution history/tails. Repeated authoring or hot reload can turn a local asset edit into global voice churn and audible discontinuity.

Index states by profile generation. Prepare the new profile outside shared state, publish it, and invalidate only dependents of the replaced generation. Active sources retain last-good until a frame-boundary switch/crossfade; unrelated profiles are untouched.

### P1: catalog reads deep-copy large DSP payloads under the manager lock

`hrtf_profiles_impl` clones every full descriptor, including both kernels and notes, while holding the global manager mutex, then sorts the result. An Editor polling this catalog pays `O(total kernel samples + P log P)` and blocks unrelated Sound operations. Repository scan found no production caller today, so this is latent rather than measured product cost.

Publish immutable lightweight metadata rows and a generation cursor. Kernel data is accessed through explicit shared resource handles; full export is an explicit bounded operation outside the audio state lock.

### P1: validation rescans kernels without compilation or render reachability

HRTF validation scans both kernels once for finiteness and again for any non-zero sample. That control-path cost is secondary, but the accepted descriptor is never converted to a direction-indexed, sample-rate-compatible provider representation. Earlier complete spatial review found no production render consumer for this engine path. Optimizing the two scans before provider admission would accelerate storage of an unusable representation.

Combine scalar validation with provider preparation when it is naturally single-pass, but prioritize correct directional data, resampling policy, partitioning, state/tail continuity and actual render reachability.

## 5. Unreal-primary policy adopted

- `ConvolutionAlgorithm.h:19-32` makes block size, input/output channels, number of IRs and maximum IR samples explicit construction settings.
- `UniformPartitionConvolution.cpp:16-49` derives block size from FFT size and allocates a bounded number of partitions from the configured maximum IR length.
- `UniformPartitionConvolution.cpp:123-149,190-194` processes only active IR blocks, skips inverse FFT for zero outputs and enforces the configured maximum when setting an IR.
- `AudioMixerSourceManager.cpp:1427-1443` initializes per-source HRTF state through a valid selected spatial provider rather than a global raw-kernel map.
- `AudioMixerSourceManager.cpp:3184-3199` instruments HRTF render cost and verifies provider validity/channel support on the actual audio path.
- `AudioMixerDevice.cpp:1851-1895` enables reverb processing only when the selected provider returns a valid effect.

Zircon adopts explicit provider settings/budgets, prepared partitioned resources, per-source/profile state, actual-path instrumentation and provider validity. It must still measure its own algorithms and devices.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Resource identity and budgets | One canonical HRTF/IR asset identity with origin, provider, format, generation and raw/prepared byte budgets. | Oversize/wrong-format/colliding-origin admission fails before state mutation; exact residency is observable. |
| M1 Prepared immutable resources | Decode/resample/normalize/partition outside the manager lock and publish shared last-good handles. | Render thread receives no raw `Vec` ownership or allocation; replacement failure preserves old sound. |
| M2 Selective dependency invalidation | Profile/IR generation reverse indices and frame-boundary switch/tail policy. | Editing one profile visits only its dependents; unrelated voice state/tails remain unchanged. |
| M3 Lightweight catalogs | Immutable metadata generations and explicit full-payload export. | Stable Editor polling copies zero kernel/sample bytes and does not take the global audio-state lock. |
| M4 Truthful incremental status | Remove full-map status scans and distinguish static/provider/ray origins. | Add/replace/remove are expected `O(1)` metadata updates; status matches the actually rendered generation. |
| M5 Dynamic qualification | Fixed 1/32/256-source HRTF and short/long IR workloads with hot reload. | Record prepare/switch P50/P95/P99, callback HRTF/convolution time, copied/allocated bytes, lock wait, RSS, underruns, CPU, wakeups and power. |

## 7. Acceptance limits

This report completes static review of the last two Sound production service files and records one scoped source fix. It does not accept the Sound module, HRTF, convolution or ray-tracing product path. Rust behavior tests, current-source runtime/Editor workflows, WPR/power and any applicable RenderDoc capture remain required, so no Git milestone commit or quantified WeCom notification is warranted.
