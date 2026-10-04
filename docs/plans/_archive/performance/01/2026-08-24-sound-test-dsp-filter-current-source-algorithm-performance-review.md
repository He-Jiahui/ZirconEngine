---
title: Sound Test DSP and Filter Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_no_source_change
scope:
  - zircon_plugins/sound/runtime/src/engine/dsp
  - zircon_plugins/sound/runtime/src/engine/filter
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/DynamicsProcessor.h
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/DynamicsProcessor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/Phaser.h
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/Phaser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/Delay.h
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/Delay.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/ReverbFast.h
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/ReverbFast.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Public/DSP/BiQuadFilter.h
  - dev/Fyrox/fyrox-sound/src/dsp/filters.rs
  - dev/Fyrox/fyrox-sound/src/effects/reverb.rs
  - dev/godot/servers/audio/audio_filter_sw.h
  - dev/godot/servers/audio/audio_filter_sw.cpp
---

# Sound Test DSP and Filter Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Sound test DSP/filter slice completed E3 current-worktree static review over **17/17 Rust files** at revision `39f7f45c5671b1b8515685198f000989a0f1d82a`:

| Module | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `engine/dsp` | 10/10 | 667 / 613 | 18,691 | 26 / 0 | included below |
| `engine/filter` | 7/7 | 434 / 402 | 13,318 | 10 / 0 | included below |
| total | 17/17 | 1,101 / 1,015 | 32,009 | 36 / 0 | `89b9bbebe742954cc19036c928cffdb242aa0346dd6b80c4db2306bcc0f38edf` |

All files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped diff check passes and the scope is clean. Managed Windows Cargo is unavailable, so the 36 tests were inspected but not executed. No production source changed.

## 2. Reachability and ownership result

`engine/mod.rs` declares both modules behind `#[cfg(test)]`. Repository-wide call-site review found every DSP/filter function is called only by its colocated tests. The production Kira bridge does not consume `meter_for`, `compressor_block`, `phaser_block`, delay/reverb/convolution, stereo pan or the stateful biquad. Consequently:

- these 32 KB do not contribute production callback cost;
- their tests increase test compilation/execution and maintenance cost only;
- passing their golden outputs does not prove any advertised Sound effect works in runtime or Editor;
- they duplicate effect concepts already represented by public Sound descriptors and backend validation.

This test-only split also explains a correctness inversion: `engine/filter` owns persistent per-channel biquad history and tests split-block continuity, while the production environment low-pass uses a separate block-reset helper. Correct logic exists only in an unreachable test module.

## 3. Algorithm findings

### P0: golden tests canonize placeholder algorithms rather than product contracts

Several test names state that an M2 golden is preserved, but the golden is tied to local implementation samples:

- `phaser_block` multiplies samples by an LFO-derived gain. It is tremolo/amplitude modulation, not a phaser.
- `limit` clamps individual samples. It is a clipper, not a dynamics limiter with envelope/lookahead behavior.
- `reverb_block` adds three feed-forward delayed taps. It is not a feedback/diffusion reverb.
- `modulated_delay` reads only dry input history. Its `feedback` parameter scales the delayed dry signal but never writes feedback into the delay line.
- `pan_stereo` uses linear attenuation and mid/side width without an equal-power or channel-layout contract.

These tests should not be migrated unchanged into production. Replace them with backend-independent invariants: continuity across arbitrary block splits, finite/stable output, channel isolation, declared latency/tail, no steady-state allocation, parameter-ramp continuity, frequency/impulse/envelope response and overload policy.

### P0: maintaining a second DSP engine would deepen the wrong architecture

Public graph descriptors already accept dynamics, filters, modulation, reverb, waveshaping and stereo controls, while the active Kira M1 backend rejects unsupported effects. Making these test helpers production-visible would create a second processing engine without source/track/submix ownership, command scheduling, scratch budgets or provider capability negotiation.

There must be one backend processing-plan boundary. Either the selected backend/provider implements an effect correctly and advertises it, or authoring/runtime reports it Unsupported. Test reference processors may exist only when they are demonstrably independent oracles, not cheaper aliases for the product effect.

### P1: test DSP allocates and moves history in callback-shaped loops

Modulation, reverb and convolution clone the entire input block. `SoundHistoryState::remember` appends and front-drains a `Vec`, moving retained samples and potentially allocating. Direct convolution is `O(frames * channels * impulse taps)`. If copied into product, these become callback allocations and unbounded per-block work.

Unreal's `FDelay` owns a pre-sized audio buffer with read/write indices (`Delay.h:71-80`, initialization in `Delay.cpp:58-69`). Dynamics owns per-channel lookahead delay and envelope followers (`DynamicsProcessor.h:98-105`; initialization in `DynamicsProcessor.cpp:226-237`). Zircon needs fixed-capacity/circular processor storage prepared outside the callback.

### P1: effect names omit required signal-processing structures

Unreal's phaser owns an LFO, per-channel all-pass filter stages and feedback samples (`Phaser.h:53-63`; APF initialization and coefficient modulation in `Phaser.cpp:34-46,113-141`). Its dynamics processor exposes attack, release, knee, peak mode and lookahead (`DynamicsProcessor.h:46-73,98-129`). Fast reverb separates early and late reflections and uses delay/all-pass subsystems (`ReverbFast.h:7-12,36-74`).

These are structural differences, not parameter polish. Zircon should rename any intentionally simple utility to its true operation (clip, tremolo, feed-forward multi-tap) or remove it when the public feature name implies a processor it does not implement.

### P1: stateful biquad is the strongest local component but still lacks product-grade transitions

The biquad preserves `x1/x2/y1/y2` per channel, computes standard filter modes and verifies split-block continuity. That matches the persistent-state direction in Unreal's biquad and Godot/Fyrox filter processors. However, coefficients are recomputed from descriptor values on every call, channel-count change reallocates/reset state, and abrupt parameter changes have no interpolation/crossfade. It remains unreachable.

Move the concept, not this test module wholesale: prepare coefficients on control change, pre-size state for the negotiated layout, ramp coefficients or wet/dry bypass safely, and attach the processor to a stable backend effect/source slot.

### P1: meter semantics are incomplete and cannot back Editor telemetry

`meter_for` scans only the first two channels, reports block peak/RMS with no ballistic smoothing/hold/window and has no production publisher. Calling it for each Editor poll would either rescan/copy buffers or force UI cadence into the render path. The audio backend should accumulate bounded per-track/submix meters into immutable generation pages; Editor reads pages without touching audio buffers.

## 4. Unreal-primary policy adopted

- DSP processor state is initialized for a known sample rate, channel layout and upper bound; callback processing reuses it.
- A phaser is an all-pass network with modulated coefficients and feedback, not amplitude modulation.
- Dynamics capability names imply envelope, attack/release, ratio/knee and declared lookahead/latency behavior.
- Delay/reverb tail storage uses persistent bounded buffers and explicit flush/bypass semantics.
- Product effect availability follows the backend/provider, while tests assert signal contracts rather than a placeholder's exact samples.

Fyrox and Godot support the same stateful filter/reverb direction, but Unreal remains the primary engineering reference. Zircon must derive its own CPU/latency/power budgets from a current-source executable.

## 5. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability inventory | Map every public effect to active backend support, planned provider or Unsupported. | No test-only helper is presented as product capability; runtime and Editor agree. |
| M1 Test contract cleanup | Replace placeholder golden samples with independent response/continuity/allocation/latency contracts. | Tests fail against wrong-name algorithms and pass against selected provider/reference fixtures. |
| M2 Test-only code decision | Delete misleading duplicate DSP or retain only truthful, named reference utilities. | No dead duplicate product architecture; test/build time and ownership recorded. |
| M3 Processor lifecycle | Backend owns fixed-capacity state, parameter generations, ramps, scratch and tails. | Zero steady-state callback allocation/history drain; block-split output is stable and deterministic. |
| M4 Effect admission | Dynamics/filter/modulation/reverb/stereo features are provider-profile capabilities. | Unsupported effects fail before authoring commit; supported effects run through one render owner. |
| M5 Dynamic qualification | Current-source runtime/Editor exercises supported effects at source/track/submix scales. | Record audio/control/main CPU, callback P50/P95/P99/max, allocations, latency/tails, underruns, RSS, wakeups, power and response parity. |

## 6. Direct-fix decision

No source edit is made. Deleting 17 test-only files without Cargo would remove useful evidence such as biquad split-block continuity, while promoting or locally tuning them would entrench a duplicate and partly misnamed DSP engine. The safe next implementation is a capability/test-contract hard cutover after the backend processor owner is selected.

Static review is complete only for these 17 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
