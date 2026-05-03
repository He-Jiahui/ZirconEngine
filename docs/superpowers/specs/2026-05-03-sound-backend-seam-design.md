# Sound Backend Seam Design

## Summary

Add a real backend seam for the sound runtime without binding this slice to a platform audio device. The goal is to turn the existing pull-only software output path into a backend-adapter contract that a later CPAL or OS callback implementation can use directly, while keeping validation deterministic in headless CI.

## Scope

This slice covers:

- neutral runtime DTOs for backend capabilities, backend callback pulls, and backend session health,
- sound-runtime adapter state that owns backend lifecycle counters and callback accounting,
- a deterministic software/null backend adapter that pulls blocks from the existing mixer path,
- manager APIs that expose backend capability listing and backend callback polling,
- focused runtime tests and documentation updates.

This slice does not cover:

- CPAL, WASAPI, CoreAudio, ALSA, PulseAudio, or WebAudio device integration,
- real audio callback threads,
- device enumeration from the host OS,
- production low-latency scheduling or lock-free ring buffers.

## Architecture

`zircon_runtime::core::framework::sound` remains platform-neutral. It owns serializable descriptors and status types only. The new types should describe backend capabilities and callback results, not concrete OS handles or callback closures.

`zircon_plugins/sound/runtime` owns concrete backend-adapter behavior. The existing `SoundOutputDeviceRuntimeState` should become the lifecycle owner for backend sessions or delegate to a small focused adapter module. The adapter validates descriptors, starts/stops sessions, pulls exactly configured block sizes from `render_mix`, records rendered blocks/frames, records underruns and errors, and returns a neutral callback report.

The first adapter is a deterministic software/null backend. It uses the configured output descriptor and existing mixer renderer, so it proves the runtime contract without requiring physical audio devices.

## API Shape

Add neutral DTOs for:

- backend capability summary: backend id, display name, whether it is real-time capable, whether it is deterministic/headless, supported sample-rate/channel/block-size ranges, and notes,
- backend callback report: device id, backend id, requested frames, rendered frames, sample count, underrun flag, sequence index, and optional error text,
- backend session status: current descriptor, state, callback counters, underrun counters, rendered counters, and last error.

Extend `SoundManager` with small backend-facing methods:

- list available backend capabilities,
- pull one backend callback block for the configured device,
- query backend session status if the existing output-device status is not expressive enough.

Keep existing `configure_output_device`, `start_output_device`, `stop_output_device`, `output_device_status`, and `render_output_device_block` behavior intact. New APIs should build on them rather than replacing them in this slice.

## Data Flow

1. Host configures `SoundOutputDeviceDescriptor` with a backend id such as `software-null`.
2. Runtime validates the descriptor against generic output rules and the selected backend capability.
3. Host starts the output device.
4. A backend callback asks the sound manager to pull one configured block.
5. Runtime calls the existing mixer render path for the configured block size.
6. Runtime records callback sequence, rendered frames, sample count, underrun state, and errors.
7. Runtime returns a neutral callback report and audio block for tests or future backend glue.

## Error Handling

Invalid descriptors continue to return `SoundError::InvalidParameter` with precise messages. Pulling a callback while stopped returns `SoundError::BackendUnavailable`. Selecting an unsupported backend returns a typed backend-unavailable error rather than silently falling back. Render errors are recorded into backend status and returned to the caller.

## Testing

Add focused sound runtime tests for:

- capability listing includes the deterministic software/null backend,
- configuring and starting the backend allows a callback pull,
- callback reports exact requested/rendered frames and sample counts,
- stopped callbacks return backend unavailable,
- unsupported backend ids fail during configuration or start,
- render/accounting errors update backend status without corrupting future lifecycle state.

Validation should use sound-runtime formatting and focused sound tests. If Cargo is still blocked by unrelated dirty render-pipeline changes, record the exact external blocker and keep sound formatting/whitespace evidence fresh.

## Remaining Follow-Up After This Slice

After this seam is complete, the remaining sound backend gap becomes a concrete CPAL/platform adapter implementation on top of the stable callback contract. Other sound gaps remain production DSP quality, HRTF profile loading, geometry-backed ray traversal IR generation, dynamic event plugin-code execution, and editor-host live operation execution.
