# Sound DSP/HRTF Quality Design

## Summary

Improve the deterministic sound runtime quality path by making filter DSP stateful and making loaded HRTF profile FIR rendering continuous across render blocks. This slice stays inside existing neutral contracts: `SoundFilterEffect` keeps its current cutoff/resonance/gain fields, and `SoundHrtfProfileDescriptor` keeps explicit left/right kernels.

## Scope

This slice covers:

- replacing the current block-local filter approximation with per-channel biquad runtime state,
- supporting low-pass, high-pass, band-pass, notch, low-shelf, and high-shelf filter modes through the existing descriptor,
- keeping filter state across consecutive `render_mix` calls for each `(track, effect)` instance,
- keeping loaded HRTF profile FIR tail across consecutive `render_mix` calls for each source/listener/profile path,
- extracting HRTF profile rendering into a focused engine module so render orchestration does not absorb another DSP responsibility,
- focused runtime tests and architecture/session documentation.

This slice does not cover:

- OS audio backend scheduling,
- SIMD or partitioned convolution,
- SOFA/CIPIC database parsing,
- cross-source HRTF interpolation over spherical datasets,
- geometry-backed occlusion or ray traversal.

## Architecture

`zircon_runtime::core::framework::sound` remains the neutral DTO layer. No new public API is needed for this slice because the existing filter and HRTF descriptors already express the data being improved.

`zircon_plugins/sound/runtime` owns concrete DSP state. `SoundEffectRuntimeState` gains biquad state keyed by existing `SoundEffectStateKey`. `SoundEngineState` gains HRTF render state keyed by source/listener/profile identity, with stale entries pruned during render-state synchronization.

`engine/dsp.rs` remains the track/effect DSP executor and delegates filter math to a focused `engine/filter.rs`. Loaded HRTF profile rendering moves to `engine/hrtf.rs`, leaving the render orchestration module as the owner for source, listener, volume, and routing behavior.

## Reference Evidence

- `dev/Fyrox/fyrox-sound/src/dsp/filters.rs` uses Audio EQ Cookbook biquad coefficients and persistent per-filter sample history.
- `dev/Fyrox/fyrox-sound/src/effects/filter.rs` stores one biquad per stereo channel and retunes on sample-rate/parameter changes.
- `dev/Fyrox/fyrox-sound/src/renderer/hrtf.rs` keeps previous left/right samples and previous sampling vector/distance gain for HRTF continuity.
- `dev/UnrealEngine/Engine/Source/Runtime/SignalProcessing/Private/Filter.cpp` keeps per-channel biquad state and optimizes mono processing by caching coefficients/state inside the loop.
- `dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp` gates HRTF through spatialization plugin state, so Zircon keeps HRTF as a runtime-owned profile path rather than a framework DTO behavior.
- `dev/bevy/crates/bevy_audio/src/audio.rs` explicitly documents simple panning instead of HRTF; Zircon intentionally goes beyond Bevy here while retaining Bevy-style playback controls.

## Behavior

Filter rendering computes stable biquad coefficients from normalized cutoff, resonance/quality, and gain. The runtime clamps cutoff below Nyquist and clamps quality/gain to deterministic finite ranges. Each channel has independent history so multi-channel filters do not bleed state between channels.

Loaded HRTF rendering applies left and right FIR kernels to the current source buffer plus remembered tail samples from previous blocks. The state key includes source ID, listener ID, and profile ID so changing listeners or profiles naturally starts a new FIR history. Missing profiles still fall back to the deterministic ear-offset preview path.

## Testing

Focused runtime tests should cover:

- low-pass filter attenuates after an impulse and keeps state across one-frame blocks,
- high-pass filter rejects DC and produces transient output,
- shelf filter gain affects output through the existing `gain_db` field,
- loaded HRTF profile applies FIR tails across blocks,
- missing HRTF profile continues to use the preview fallback instead of stale loaded-profile state.

Validation uses sound runtime formatting, neutral rustfmt for changed framework files if any, focused `dsp_state` and `spatial` test filters, and whitespace checks over sound/docs/session files.

## Remaining Follow-Up

After this slice, DSP/HRTF quality still has deeper production gaps: SIMD, partitioned convolution, denormal handling, high-order resampling, and real HRTF database parsing/interpolation.
