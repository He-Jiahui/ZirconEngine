# Sound HRTF Profile Loading Design

## Summary

Add deterministic HRTF profile loading to the sound runtime. The current listener `hrtf_profile` field only activates an ear-offset preview path. This slice gives that profile ID a concrete runtime data source: loaded left/right FIR kernels that spatial rendering can apply when a matching profile is available.

## Scope

This slice covers:

- neutral HRTF profile descriptor DTOs in `zircon_runtime::core::framework::sound`,
- `SoundManager` APIs to load, remove, and list HRTF profiles,
- sound runtime storage for validated HRTF profiles,
- deterministic left/right kernel application during source spatial rendering,
- fallback to the existing ear-offset preview when no loaded profile matches the listener,
- focused runtime tests and documentation updates.

This slice does not cover:

- SOFA, CIPIC, Steam Audio, Resonance, or other production HRTF database parsing,
- azimuth/elevation interpolation between many kernel measurements,
- high-performance partitioned binaural convolution,
- personalized HRTF calibration.

## Architecture

`zircon_runtime::core::framework::sound` owns a serializable `SoundHrtfProfileDescriptor` with profile ID, display name, sample rate, left/right FIR kernels, and optional notes. It remains an inert contract; it does not parse files or apply DSP.

`zircon_plugins/sound/runtime` owns validation, storage, and DSP use. Profiles live in `SoundEngineState` beside impulse responses and other runtime audio data. During spatial rendering, `apply_source_environment` checks the active listener. If `listener.hrtf_profile` names a loaded profile, the runtime applies the profile kernels to the source buffer. If not, the existing deterministic ear-offset preview remains the fallback.

## Data Flow

1. Host or editor calls `load_hrtf_profile(profile)` with finite left/right kernel samples.
2. Runtime validates non-empty ID/display name, non-zero sample rate, non-empty finite kernels, and at least one non-zero kernel sample.
3. Runtime stores or replaces the profile by ID.
4. Listener descriptors continue to reference profiles by string ID.
5. Spatial rendering resolves the listener profile ID against loaded profiles.
6. A loaded profile applies left/right FIR gains/delays to the source buffer before the normal pan/gain stage; missing profiles fall back to the preview path.

## Error Handling

Invalid profiles return `SoundError::InvalidParameter` with precise details. Removing a missing profile returns a typed unknown-HRTF error. Rendering never fails because a listener references an unloaded profile; it falls back to preview so authoring sessions remain audible.

## Testing

Add focused runtime tests for:

- valid HRTF profiles can be loaded and listed,
- invalid empty/non-finite kernels are rejected,
- removing a missing profile returns a typed error,
- a listener referencing a loaded profile changes left/right output deterministically,
- a listener referencing an unloaded profile still uses the existing preview fallback.

Validation should use sound runtime formatting, neutral sound rustfmt, focused spatial/HRTF tests when the workspace compiles, and whitespace checks. If unrelated dirty graphics code still blocks Cargo before sound compilation, record the exact blocker.

## Remaining Follow-Up After This Slice

After this slice, the HRTF gap becomes production HRTF database loading, interpolation, and optimized binaural convolution. The other remaining sound gaps are CPAL/platform OS audio adapter, production DSP/resampler/convolution tuning, geometry-backed occlusion/ray traversal IR generation, dynamic event plugin-code execution, and editor-host live operation execution.
