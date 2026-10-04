---
title: Sound Spatial Environment Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_shared_fix_preserved
scope:
  - zircon_plugins/sound/runtime/src/engine/math.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf
  - zircon_plugins/sound/runtime/src/engine/occlusion
  - zircon_plugins/sound/runtime/src/engine/source_environment
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
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSource.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/SoundWaveDecoder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/AudioFFT.cpp
  - dev/Fyrox/fyrox-sound/src/renderer/hrtf.rs
  - dev/godot/servers/audio/audio_filter_sw.h
  - dev/godot/servers/audio/audio_filter_sw.cpp
---

# Sound Spatial Environment Current-Source Algorithm Performance Review

## 1. Status and frozen scope

This slice completed E3 current-worktree static review over **38/38 Rust files** at revision `9a217cce07c574cbec8dda70b3e1142eeedbc9a9`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| spatial/environment engine | 38/38 | 1,267 / 1,143 | 40,316 | 3 / 1 | `44f668ed5a1d791ebaed4d89d8b9fee3d09d7d2b477ef8cc37dd88caaed5f4c6` |

All 38 files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped diff check passes. The existing shared edit in `source_environment/volume/filter.rs` is preserved: it removes one full-buffer clone from the low-pass block, guards zero channels and adds correctness/performance tests. Those tests were inspected but not executed because managed Windows Cargo is unavailable. No additional source edit is made by this review.

Managed Cargo, a launchable current-source product, a real audio callback workload, WPR/ETW, power measurement and audible-quality comparison remain unavailable. RenderDoc is not applicable to CPU audio DSP and must not be used as audio evidence.

## 2. Per-folder review ledger

| Module | Reviewed files | Static result |
|---|---:|---|
| `engine/hrtf` | 6/6 | Direct FIR and preview paths allocate/copy per block, lack correct continuous delay/filter history and do not derive HRIR from source direction. String profile IDs are cloned for hot lookup keys. |
| `engine/occlusion` | 5/5 | Each query linearly scans ray-traced descriptors; selection changes gain metadata but no physics/acoustic query is owned here. No generation cache or worker budget exists. |
| `engine/source_environment` | 25/25 | Listener/volume selection is repeated linear scan; direct convolution truncates tails at block boundaries; Doppler changes volume rather than pitch; callback-safe state and ownership are absent. |
| engine roots | 2/2 | Modules compile, but repository-wide call-site review found no production consumer of the environment application, active-listener resolver or HRTF-tail query. Several DSP/filter modules remain test-only. |

## 3. Product call-site truth

Repository-wide Sound runtime search found definitions and internal helper calls only for `apply_source_environment`, `active_listener_for` and `hrtf_tail_pending_for_source`; no render/output path invokes them. HRTF, occlusion, filter and convolution helpers only call within this isolated implementation. The current product therefore has no comparable spatial-audio workload: the measured runtime cost is effectively absent because the feature is not connected.

This is the first P0 finding. Dynamic profiling cannot prove these algorithms efficient, and wiring them into Kira merely to create a workload would activate incorrect DSP and unbounded callback work. M0 must first choose the supported MVP surface and connect it through a real backend/render owner.

## 4. Structural algorithm findings

### P0: the current DSP must not be integrated as the product spatial backend

The implementation combines stateless block transforms, transient allocations, hash/string lookup and potentially unbounded direct convolution. It has no persistent per-source processor object, audio-frame command boundary, scratch ownership, quality tier or overload policy. These are callback-domain architecture requirements, not optional micro-optimizations.

For MVP, support only capabilities that can be made correct and bounded: distance attenuation, equal-power stereo pan/channel mapping and stable pitch-based Doppler. HRTF, convolution reverb and ray-traced occlusion must report Unsupported until a proven processor/provider and control-to-audio handoff are active.

### P0: low-pass state resets at every audio block

`low_pass_block` starts its accumulator at zero for every block and channel. The allocation-removal edit is locally useful, but it preserves the old block-local output rather than continuous filter state. Each block therefore begins with a new transient. A correct implementation owns history per source/channel, interpolates parameter changes and survives block boundaries.

Unreal keeps persistent per-source `FInterpolatedLPF` objects in `AudioMixerSourceManager.h` and processes them in the source render path. Godot's `audio_filter_sw` processor retains filter histories (`ha*`/`hb*`) and interpolates coefficients. Zircon should adopt persistent processor state, not add more loops to this stateless helper.

### P0: HRTF is neither directional nor stream-continuous

`apply_loaded_hrtf_profile` copies the dry block, performs direct FIR `O(frames * channels * taps)`, appends history and front-drains a `Vec`. Fixed left/right kernels are selected by profile ID, not interpolated from listener-relative source direction. The preview path also copies the block and applies only intra-block delays, zero-filling each new block instead of retaining delay history. This can repeat discontinuities and does not implement moving-source HRTF.

Fyrox's checked-in HRTF renderer uses an HRIR sphere plus a persistent `HrtfProcessor`, current/previous sampling vectors, previous channel samples and previous/current distance gains. Its module documentation explicitly treats HRTF as convolution/FFT-heavy and calls out crossfade continuity. Zircon should use an equivalent proven stateful library/adapter with reusable scratch and direction interpolation; profile string lookup cannot occur per callback block.

### P0: convolution loses the impulse-response tail and has unbounded direct cost

`add_convolution_send` copies the dry block and convolves only samples available in that block. It stores no overlap/history, so an impulse response tail is truncated and restarted at every block. Cost is direct `O(sources * frames * channels * taps)` and the same source buffer can receive both volume and source-send convolution. Longer authored IRs therefore multiply callback work without a declared bound while still producing wrong output.

Do not optimize this loop. Replace it with a proven stateful partitioned/FFT convolution backend or keep convolution unavailable. Unreal's checked-in source tree supplies optimized FFT primitives under `SignalProcessing`; that is evidence for the primitive and ownership direction, not evidence that Zircon may copy an unverified Unreal reverb budget.

### P0: Doppler changes gain instead of playback rate

`doppler_preview_gain` maps a velocity ratio to at most a ten-percent volume change. Doppler is a pitch/playback-rate effect; gain modulation is semantically wrong and cannot be accepted by waveform comparison. Unreal's `FMixerSource::UpdatePitch` sends pitch to the voice, while `SoundWaveDecoder` applies pitch scale to resampling/frame advancement.

Compute a clamped pitch ratio on the control/update stage, smooth it, and publish the scalar through the backend's voice command. Acceptance must check frequency/pitch and continuity, not only sample amplitude.

### P1: scene and acoustic selection repeat full scans

The strongest volume influence performs a full volume scan and square root, and the same source path can call it twice. Occlusion linearly scans all ray-traced IR descriptors and selects by specificity/ray count. Listener selection is also linear. Naively invoking these helpers per source per block yields `O(sources * (volumes + acoustic descriptors))` control work on the audio callback.

Scene/ECS extraction should resolve the active listener once per output/session generation, spatially query changed sources/volumes on a worker/control stage, batch physics/acoustic requests under a frame budget and publish compact immutable parameters keyed by stable source slots. The audio thread then performs `O(active sources)` bounded scalar/stateful DSP without scene locks, entity traversal or descriptor scans.

### P1: pan and environment composition have no quality/ownership contract

The stereo pan path uses linear amplitude coefficients, not an equal-power law or an output-layout channel map. Environmental stages mutate the same buffer sequentially and can duplicate convolution, but there is no declared order, wet/dry normalization, tail ownership or bypass transition. Abrupt parameter/profile changes have no ramp/crossfade guarantee.

Compile one backend-specific per-source processing plan off the callback thread. It owns stage order, channel layout, persistent state, ramp duration, scratch upper bound and tail lifetime; unavailable authored features produce explicit diagnostics rather than silent approximations.

### P1: existing benchmark proves only a local allocation delta

The shared ignored performance test models removal of 2,048 allocations and 4,194,304 transient bytes across 21 pairs at 256 stereo frames and asks for a 15% P95 win. That is useful regression coverage for the clone removal, but it is unexecuted and compares against the same block-reset filter semantics. It cannot accept the product algorithm or callback budget.

## 5. Unreal-primary policy adopted

- `AudioMixerSourceManager.cpp:818-826` caches the spatialization plugin rather than resolving a provider per sample/block.
- Source initialization/release around `AudioMixerSourceManager.cpp:1090-1154,1394-1449` owns per-source plugin state and lifetime.
- `SetSpatializationParams` and LPF updates around `AudioMixerSourceManager.cpp:2042,2087-2104` cross through source-manager ownership.
- Plugin spatialization/occlusion `ProcessAudio` calls around `AudioMixerSourceManager.cpp:3119-3231` execute inside a defined render phase; persistent source effects/LPF follow around `3257-3556`.
- `AudioMixerSourceManager.cpp:4048-4062` gives the render plugin an all-sources-processed phase instead of arbitrary scene calls from DSP.
- `AudioMixerSource.cpp:1406-1432` and `SoundWaveDecoder.cpp:153-168,330-356` route pitch to voice decoding/resampling.

The adopted rule is command-owned per-source processor state plus an explicit provider capability. Unreal is the primary structural reference; Fyrox supplies concrete HRTF state evidence and Godot supplies concrete continuous filter-state evidence. Performance thresholds still must come from Zircon's own source/build/device/workload receipts.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability truth | Real runtime/Editor call graph declares MVP spatial features and unavailable advanced features. | Current-source product reaches attenuation/pan/pitch path; HRTF/convolution/ray occlusion cannot masquerade as Ready. |
| M1 Scene extraction | Stable source/listener slots, change generations, spatial index and batched physics/acoustic jobs. | No scene/entity/descriptor full scan or lock on audio callback; work scales with changed/active sources. |
| M2 Backend processor | Persistent per-source state, immutable processing plan, bounded commands/scratch and tail lifecycle. | Callback has zero steady-state allocation/string/hash/scene access and bounded work at declared source/block/profile limits. |
| M3 MVP spatial correctness | Equal-power/channel-layout pan, continuous attenuation and pitch-based smoothed Doppler. | Deterministic waveform/frequency/continuity fixtures across block boundaries and moving sources. |
| M4 Filter correctness | Persistent multi-channel LPF state and interpolated coefficient changes. | Split-block output matches continuous reference within tolerance; no boundary transient or allocation. |
| M5 Advanced providers | Proven directional HRTF and partitioned/FFT convolution adapters with explicit quality tiers. | Direction sweep, impulse-tail, moving-source crossfade and overload/bypass tests pass; unsupported providers remain unavailable. |
| M6 Dynamic qualification | MVP runtime and Editor preview under source motion, volume churn and overload. | Record audio/control/main CPU, callback P50/P95/P99/max, allocations, wakeups, underruns, latency, RSS, power and waveform/frequency parity with exact receipt. |

## 7. Direct-fix decision

The shared in-place low-pass edit is preserved because it removes a measurable transient copy without expanding scope. It remains provisional and must not be treated as algorithm acceptance. No further source edit is made: changing filter history, HRTF/convolution engines or Doppler semantics requires a new per-source/backend ownership contract and product integration tests. Tuning the isolated helpers would optimize a temporary and partly incorrect architecture.

Static review is complete only for these 38 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
