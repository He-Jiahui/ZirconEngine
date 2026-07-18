---
related_code:
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/automation/target/parameter_values.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/base_parameters.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/coordinates.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/mod.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/device.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_validation/mod.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/lifecycle.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph/transaction.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/playback.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs
  - zircon_plugins/sound/runtime/src/poison_recovery.rs
  - zircon_plugins/sound/runtime/src/engine/filter/mod.rs
  - zircon_plugins/sound/runtime/src/engine/filter/tests.rs
  - zircon_plugins/sound/runtime/src/engine/state/graph.rs
  - zircon_plugins/sound/runtime/src/engine/state/playback.rs
  - zircon_plugins/sound/runtime/src/engine/state/source.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/output/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sync.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/source_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/source_seek.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
implementation_files:
  - zircon_plugins/sound/runtime/src/automation/target/parameter_values.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/base_parameters.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/coordinates.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/graph_compile/routes.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/device.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/lifecycle.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/graph/transaction.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/manager/playback.rs
  - zircon_plugins/sound/runtime/src/kira_bridge/playback_data.rs
  - zircon_plugins/sound/runtime/src/poison_recovery.rs
  - zircon_plugins/sound/runtime/src/engine/filter/mod.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sync.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/source_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/source_seek.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
plan_sources:
  - user: 2026-07-17 implement the plugin plans with infrastructure and MVP first
  - docs/plans/zircon_plugins/02-sound.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_plugins/sound/runtime/src/tests/kira_bridge.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/catalog/hardcut.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/compilation.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/diff.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/hardcut.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/routing.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/graph/unsupported.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/lifecycle/backend.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/lifecycle/ownership.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/playback_data/cache.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/playback_data/range.rs
  - zircon_plugins/sound/runtime/src/tests/kira_bridge/source/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/kira_graph_sync.rs
  - zircon_plugins/sound/runtime/src/tests/playback/range.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/tests.rs
  - zircon_plugins/sound/runtime/src/engine/filter/tests.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/settings.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/active_runtime.rs
doc_type: module-detail
---

# Sound Runtime Plugin

## Purpose

The Sound runtime is the first-party provider for the neutral contracts in
`zircon_runtime::core::framework::sound`. Sound M1 hard-cuts the former software mixer,
direct CPAL output stack, producer thread, and ring-buffer renderer. Kira 0.12.2 is the
only audio execution owner. The plugin retains contract mapping, graph validation,
authoring state, source/playback identity, and typed runtime errors.

The runtime framework does not depend on Kira types. Kira is isolated below
`src/kira_bridge/`, so future dependency upgrades do not leak into shared DTOs or
editor/plugin registration contracts.

## Ownership Boundaries

- `kira_bridge/manager.rs` owns only the `KiraEngine` handle tables and mounts the
  lifecycle, graph, and playback behavior modules.
- `manager/lifecycle.rs` creates and drops `AudioManager`, applies configured resource
  limits, and converts backend startup failures into `SoundError`.
- `manager/graph.rs` owns installed graph state, parameter updates, and full rebuilds
  required by output-format changes.
- `manager/graph/transaction.rs` owns atomic incremental track/send/subtree staging.
- `manager/playback.rs` is the only owner of Kira `StaticSoundHandle` allocation and
  playback controls.
- `graph_compile.rs` validates a neutral graph once and produces the compiled graph plus
  its diff. Service modules do not repeat graph compilation.
- `automation/target/parameter_values.rs` owns typed parameter conversions, while
  `effect/base_parameters.rs` owns the enabled/bypass/wet fields shared by effect kinds.
  No behavior is hidden behind `common` or `helpers` modules.
- `descriptor_validation/coordinates.rs` owns finite coordinate validation shared by
  source, listener, and volume descriptors.
- `engine/state/` owns neutral authoring/runtime records. It never owns an audio thread
  or renders samples.

No production Sound file is allowed to combine these behavior families in a crate root
or exceed the repository production-file budget. Mutex poison recovery is centralized in
`poison_recovery.rs`; production call sites do not use poison `expect` or `unwrap`.

## Lifecycle And Configuration

`SoundConfig` is derived from `SoundPluginOptions`. The shared default, runtime package
manifest, and static `plugin.toml` all select `kira-cpal`. Direct `cpal` dependencies are
retired; Kira owns CPAL integration and the audio thread.

Output activation applies the complete runtime configuration:

- `block_size_frames` becomes Kira's internal buffer size;
- `max_voices` is enforced globally by the playback binding table and configures main and
  child track sound capacity;
- `max_tracks` is the logical graph limit checked before Kira allocation;
- physical track/send capacity reserves three bounded generations at the manager; each
  nested parent reserves three generations of its actual direct-child count, and a child
  topology edit rebuilds that affected parent subtree so its bounded capacity grows with the
  topology instead of allocating `max_tracks` slots in every leaf;
- `master_gain` is applied to Kira's main track and remains effective across graph edits.

Output configuration and start both hold `config -> state` while validating and applying
the Kira manager transition. Global gain uses the same order and commits the neutral config
only after Kira accepts the value, so concurrent start/configuration cannot expose a split
reported gain and rendered gain. Device discovery preserves the backend's original channel
count and layout. M1 advertises and activates only mono/stereo; wider devices remain visible
as unavailable with a typed diagnostic instead of being silently clamped to stereo.

Stopping or reconfiguring output drains Kira playback identities into the neutral
finished-event boundary before handles are discarded. Authoring records remain available
for an output restart and clip-backed sources are rebuilt against the new manager.

## Graph Compile And Commit Model

Graph authoring and active execution share one revision-checked two-phase commit, but the
prepared work depends on the output lifecycle state:

1. Snapshot the `Arc<SoundMixerGraph>`, revision, and Kira active state under the state mutex.
2. Clone and mutate outside the mutex. An inactive manager runs only neutral graph validation,
   so a project may author valid M2 effects and pre-effect routes before an output exists. An
   active manager additionally compiles the M1 Kira representation and its diff exactly once.
3. Reacquire the state mutex and retry if either the revision or Kira active state changed.
4. When active, apply the prepared Kira plan; then replace the neutral graph and commit the
   mutation metadata. When inactive, replace only the validated neutral graph.

Output activation holds the state mutex, starts Kira, and compiles/synchronizes the complete
current neutral graph before marking the device started. Consequently an authored M2 surface
is accepted while inactive but produces the typed M1 unsupported diagnostic at activation,
never a partially active graph. This model prevents concurrent edits or lifecycle transitions
from overwriting each other and keeps graph cloning, validation, compilation, and diff
allocation out of the shared state lock.

Pure gain/mute changes send Kira parameter commands with the standard 10 ms linear
`Tween`. Incremental structural edits first allocate missing send tracks and replacement
track/subtree handles into local tables. Allocation failure drops only staged handles and
leaves the installed Kira and neutral graphs unchanged. After every fallible operation
succeeds, the transaction swaps staged handles and drops removed handles. Only an output
format change uses a full staged rebuild.

A complete mixer import prepares every fallible authoring registry before mutating Kira.
Source descriptors without ids are assigned stable ids in input order, and those ids are
written into the committed neutral graph as well as the runtime source map. The old graph
and source bindings stay available until the new graph and preconfigured sources are both
installed. A stop or bind failure restores the previous graph and sources; if restoration
itself fails, Kira is deactivated and a typed backend error is returned instead of exposing
split Kira/neutral state.

## Send Routing Contract

Kira `SendTrack` output does not automatically pass through a logical target `Track`.
The compiler therefore projects the target track's effective M1 gain into the send track:
target gain/mute multiplied by every parent gain/mute through the master. Changes to any
member of that chain emit `SetSendVolume` for the existing Kira send handle.

Post-effect routes are expanded recursively: a send that enters a downstream track also
contributes to every post-effect send reachable from that track, with memoized chain gain
and mute projection. This keeps chained buses audible without compiling a second graph or
allocating route state while holding the public manager mutex.

M1 executes post-effect send routing. Pre-effect sends and target effects may be retained in
inactive neutral authoring state, but output activation and active graph mutation return typed
`UnsupportedAdvancedFeature` until Sound M2 supplies a Kira effect-route mapping; they are
never silently bypassed. Frame-capture tests use a custom Kira backend to verify the rendered
contribution of source send gain, target gain/mute, parent gain, and active parent updates.

Gain, mute, and send-gain changes update existing Kira handles while playback continues.
M1 does not migrate live handles between rebuilt parent/track trees, so structural graph
edits return a typed unsupported error while any Kira playback is genuinely active. A
naturally stopped handle does not block the edit and remains available for the normal
finished-event drain, avoiding both silent playback retirement and stale-handle lockout.

## Playback And Sources

Loaded clips retain an `Arc<SoundAsset>` and one cached Kira `StaticSoundData`, avoiding a
full asset/frame copy while the global state lock is held for every play. Non-looping
`duration_seconds` is encoded as a Kira playback slice rather than tracked only in neutral
metadata.

Kira playback handles implement play, stop, pause, resume, seek, gain, and playback-rate
controls. Natural Kira completion is drained exactly once into typed
`SoundPlaybackFinished` records. Deactivation also returns detached playback ids so the
neutral state cannot retain ghost playbacks.

Clip-backed `SoundSource` records map to hidden Kira playback handles. Source gain, speed,
pause/resume, seek, stop, and natural completion operate on those handles. Inactive output
may store source authoring data for later activation. External and synth sources currently
return typed unsupported errors when active because M1 has no Kira streaming/custom sound
adapter; no metadata-only success is reported.

Playback and source status expose absolute clip positions even though Kira reports and seeks
inside a slice with relative seconds. The shared conversion boundary subtracts the slice start
before a Kira seek and restores it when reading status, so activating output cannot reset a
sliced source cursor from its absolute start frame to zero.

## Performance Invariants

- Inactive graph mutation runs neutral validation without Kira compilation; active mutation
  compiles exactly once per attempt.
- State revision plus Kira active-state comparison provides retry consistency for concurrent
  edits and output lifecycle transitions.
- Structural changes stage only affected tracks/subtrees; format changes alone rebuild the
  complete graph.
- Cached `StaticSoundData` shares frame storage through `Arc`.
- Logical capacities fail before backend exhaustion; physical staging capacity is bounded.
- The milestone benchmark drives an active MockBackend Kira graph, records p50/p95 time and
  allocator calls for add, update, remove, and send mutations at 10, 100, and 1000 tracks,
  and uses thread-local allocation accounting so parallel library tests cannot pollute the
  measured budget.
- A MockBackend active public-commit harness uses the same production revision/active-state CAS
  and lock-held Kira apply primitive to record state-mutex hold p50/p95 for the same
  add/update/remove/send families and scales. Its linear per-track time budget is separate from
  the end-to-end active Kira allocation benchmark so lock cost and total backend cost remain
  independently visible.

## Hard-Cut Scope

The following execution owners are retired and are not compatibility surfaces:

- `engine/render/**`;
- `engine/dsp_state/**` and the old effect-chain executor;
- `output/cpal/**`, `output/software.rs`, and `output/ring_buffer.rs`;
- direct CPAL dependency/features and software-null/software-test/software-preview devices.

The remaining `engine/dsp/` numerical helpers are test-only M2 migration inputs, not an active
software mixer. Their deterministic goldens remain mounted so later Kira custom effects can
preserve the accepted numerical contracts. The folder-backed `engine/filter/` owner is also
test-only during M1; it preserves biquad mode, finite-value, shelf gain, frequency response,
channel isolation, and cross-block state goldens without entering the production module graph.
M2 must move the accepted numerical contracts into Kira effects rather than restoring an M1
effect-chain fallback.

The retired test topology is replaced by responsibility, not by preserving the former
renderer under test-only names:

- output queue/callback latency belongs to Kira and is represented by lifecycle and device
  catalog status tests;
- graph mix, gain, mute, parent, and send behavior is verified through Kira backend frame
  capture rather than the retired software renderer;
- effect numerical goldens remain mounted, while active M1 graphs reject effect execution
  with a typed M2 diagnostic until the Kira built-in/custom effect mapping is delivered.

## Test Coverage

The Sound M1 testing stage is pending fresh managed execution after the final source edits.
The required batch is:

- focused Kira bridge, graph-sync, frame-capture, lifecycle, source, cache/range, capacity,
  transaction, and DSP golden tests;
- the complete locked `zircon_plugin_sound_runtime` test suite;
- locked metadata for the root and plugin workspaces;
- the root `zircon_runtime` `scene::` upward gate required by the lockfile handoff;
- independent review with zero Critical and zero Important findings.

The document must not be read as acceptance evidence until the numbered M1 output record
contains terminal managed job ids, raw test counts, lock hashes, and the independent review.

## Follow-Up

Sound M2 owns Kira built-in/custom effect mapping, including the retained DSP goldens.
Sound M3 owns ECS spatial update and 3D policy. Sound M4 owns HRTF and the asset pipeline.
Sound M5 owns Timeline/Clock automation and editor mixer tooling. M1 intentionally rejects
those advanced execution surfaces rather than preserving the retired renderer as a fallback.
