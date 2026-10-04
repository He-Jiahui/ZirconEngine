---
title: Sound State Playback and Source Service Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_shared_changes_preserved_dynamic_pending
scope:
  - zircon_plugins/sound/runtime/src/engine/state
  - zircon_plugins/sound/runtime/src/service_types/playback*
  - zircon_plugins/sound/runtime/src/service_types/source*
  - zircon_plugins/sound/runtime/src/service_types/clip_assets.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/external_sources.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/source.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ActiveSound.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/AudioDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp
---

# Sound State Playback and Source Service Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The Sound state/playback/source service slice completed E3 current-worktree static review over **28/28 Rust files** at revision `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`:

| Module slice | Files | Physical / non-empty lines | Bytes | Embedded tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `engine/state` plus selected `service_types` playback/source ownership | 28/28 | 2,166 / 1,972 | 75,127 | 1 / 0 | `cda1c9b2eeb3f902b13efde0f81e5028a756185072b405b06c9f61a2eb3d5694` |

All 28 files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped `git diff --check` passes. Shared changes are preserved: `engine/state/storage.rs` only moves an import into rustfmt order, while `service_types/clip_assets.rs` adds a pre-AssetManager cache hit and one embedded test. This review binds to those current bytes and does not claim ownership of either edit.

Managed Windows Cargo and a launchable current-source artifact remain unavailable, so the embedded test, the larger `runtime/src/tests` playback/source tree, real AssetManager/Kira/CPAL behavior, WPR/ETW and power evidence were not executed. This slice is not a product acceptance.

## 2. Per-folder review ledger

| Module | Reviewed files | Static result |
|---|---|---|
| `engine/state` | all 7 files | One global state owns every clip, playback, source, world journal, listener, volume, graph, device, timeline/event and acoustic cache. Snapshots deeply project several collections. Completion reconciliation has no reverse source index. |
| service roots/ownership | `mod.rs`, `manager_state.rs`, `manager_trait.rs` | Manager clones share separate config/state mutexes; no session/world owner, supervisor or generation transaction exists. |
| clip service | `clip_assets.rs` | Current early cache hit is a valid local improvement. Concurrent misses can still duplicate asset load, full decode and Kira conversion; no unload/lease/eviction exists. |
| direct playback | `playback.rs`, `playback_status.rs`, `playback_validation.rs`, `playback_controls.rs` and its 5 children | Every status/control first polls all Kira completions under the global mutex. Finished history is unbounded and empty queries scan it. |
| source voice | `sources.rs`, `source_controls.rs`, `source_seek.rs`, `source_status.rs` | Descriptor update always stops and recreates playback; status/control is mutex-bound; completion lookup scans all sources. |
| source-adjacent service | `acoustics.rs`, `external_sources.rs` | Listener/volume/external data is stored under the global mutex but has no Kira execution path; external playback is rejected. |
| public forwarding | `manager_trait/playback.rs`, `manager_trait/source.rs` | Broad synchronous APIs expose implementation work directly to arbitrary caller threads and do not declare latency, affinity, ticket or backpressure. |

Context review also covered Sound module registration, public runtime contracts, product call sites, Editor live-output construction and the prior output/Kira bridge report. These context files are not counted in this 28-file ledger.

## 3. Structural performance findings

### P0: the cheap current baseline is a missing product path

The Sound module registers an immediate empty driver and lazy manager factory, but no runtime scene/system/supervisor starts output, loads clips, creates/updates sources, updates listener/volume state or drains completion. Repository-wide production call-site scan outside this implementation found zero `.load_clip`, `.play_clip`, `.create_source`, `.update_source`, `.update_listener`, `.update_volume` or finished-drain consumers. The Editor live-output controller is exported, but its checked-in constructors are inside inline tests rather than product extension activation.

Therefore current CPU/power cannot be compared with another engine: Zircon is not executing the same behavior. M0 must first build one fixed product scene and lifecycle through real runtime and Editor paths, then freeze source/build/config/device/workload identity before profiling.

### P0: one descriptor update destroys and restarts the source voice

`update_source_impl` validates, removes the voice, stops its Kira playback, replaces the entire descriptor and calls `sync_source_voice_in_state`. The replacement restarts the clip from authored `start_seconds`; it does not preserve current sample position. Gain, speed, mute, playing, output route, transform or spatial-only changes all take the same stop/recreate path. If recreation fails, the previous live voice has already been stopped and last-good behavior is lost.

This is fatal for a future scene extraction system: sending changed transform/listener state per frame would continuously restart and allocate voices, while position is not applied to Kira anyway. Required design is a compiled source binding with field dirty bits and stable voice slot. Transform/spatial/volume/rate updates become bounded commands; only input artifact or incompatible graph changes replace a voice, with preserved cursor/crossfade and last-good rollback.

### P0: cleanup is query-triggered and finished history grows without a consumer

Most playback/source controls and status calls invoke `poll_kira_completions` while holding the global state mutex. The underlying scan/sort cost is documented in the output/Kira bridge review. Reconciliation then linearly scans all sources for every unmatched playback. Completed records append to `finished_playbacks` or `finished_sources`; without explicit drain calls those vectors grow for the manager lifetime. `playback_empty` and `source_empty` linearly scan the accumulated history, so their cost grows with total historical completions, not current voices.

Move completion production to a fixed audio-runtime pump with a bounded ring and loss/age diagnostics. Maintain playback-slot -> owner-slot mapping for O(1) reconciliation. Per-world/session cursors consume immutable completion pages; retention is bounded by count/bytes/age, and absence of a consumer cannot leak indefinitely.

### P0: global state has no world/session lifetime

The manager state mixes Kira/device, clips, direct playback, sources, listeners, volumes, parameters, graph, meters, timeline, events and acoustics. Only gameplay emission journals are keyed by `WorldHandle`; sources/listeners/volumes themselves are not. Journals, clips, external blocks and other maps have no world/session close path. Editor preview, Play, standalone runtime and scene replacement would share the same manager state and stale IDs.

Identifiers are monotonic raw integers without generation/owner epoch. Some increments use `+= 1`, one uses `saturating_add`; neither provides exhaustion or stale-handle semantics. Required shape is global device/clip-residency ownership plus per-session/per-world `AudioWorldSlot` with generation-qualified dense handles and explicit close. Old callbacks/completions cannot mutate a replacement world.

### P0: clip cache hit improved, but miss handling still duplicates the expensive path

The current shared edit checks `clip_ids_by_locator` before resolving the project manager, which avoids repeated work for an exact cached string and should be preserved. On a concurrent miss, each caller can still resolve/load the asset and construct `LoadedClip` independently before the second locked dedup check. `LoadedClip::new` retains the full `SoundAsset` and allocates the full Kira frame representation, so losing callers can pay the complete decode/conversion and peak memory anyway.

Use a canonical `AssetUri + artifact generation` key and an AssetManager-owned single-flight ticket. `AudioClipResidencyManager` returns leases to prepared/streamed artifacts, records pending/hit/error-negative-cache states and accounts encoded/decoded/page/Kira bytes. Clip unload/evict occurs after voice/editor leases release; hot reload publishes a new generation while existing voices follow explicit policy.

### P1: listener, volume and external source APIs perform storage work without audible behavior

Listener and volume updates only validate and insert descriptors into global hash maps. Source playback always passes centered pan and never consumes position, listener or volume data. External blocks replace a full stored block under the same mutex, but a playing external source is rejected as unsupported. A producer can therefore spend copy/move, validation and lock time with no render consumer.

Capability admission must report these paths as Unsupported until an executable provider exists. Real implementation uses scene change extraction, stable listener/emitter slots and bounded realtime-safe external rings with timestamps, format negotiation, underrun/overflow/backpressure and shutdown. Unsupported state should not retain arbitrary audio blocks.

### P1: snapshots and status are deep, synchronous observations

`SoundEngineState::snapshot` clones the full mixer graph, then clones every source descriptor, automation binding, dynamic-event catalog, meters and ray-tracing status while the caller holds the state lock. Playback/source status also performs completion cleanup and Kira handle queries before constructing a result. These are not passive reads and can block mutation/control callers.

Publish immutable generation-qualified status pages. Default observation returns `Arc` pages or cursor deltas; full capture is explicit and budgeted. Querying one playback/source is O(1), has no cleanup side effect and reports freshness/clock.

### P1: gameplay emission has a bounded queue but incomplete lifecycle

The per-world `VecDeque` correctly caps events at `SOUND_GAMEPLAY_EMISSION_CAPACITY` and reports missed events, which is a useful pattern to preserve. However, world entries are never retired, source creation performs real-time clock lookup while holding state, and returned reads clone all selected events. The event is emitted only on source creation, not on later playback/audibility transitions.

Move this journal into `AudioWorldSlot`, retire it on world close and publish events from the same admitted voice/source transition that owns the generation. Readers use bounded cursor pages; telemetry distinguishes capacity eviction from consumer lag.

### P1: manager registration ignores resolved plugin/project options

The lazy module factory constructs `DefaultSoundManager::from_weak_core`, which uses `SoundConfig::default()` rather than resolved package/project options. Config and state are separate mutexes without one generation transaction. Runtime behavior and benchmark configuration can therefore differ from manifest/editor settings even before output starts.

Introduce one typed resolved audio-config generation with documented precedence. Device/graph/world owners prepare against the same generation and publish applied/failed/last-good status; tests and profiler receipts record that generation.

## 4. Unreal source evidence and adopted policy

- `ActiveSound.h:293-336,387-473,583-635` ties active sound state to world/audio component identity, concurrency, virtualization, playback time and transform rather than storing all worlds in an unqualified global source map.
- `AudioDevice.cpp:3435-3499` routes listener updates onto the audio thread and then updates virtual loops; it does not synchronously mutate mixer state from an arbitrary caller.
- `AudioDevice.cpp:2646-2696,3259-3309` owns concurrency culling and inactive-loop virtualization. Lines 5345-5555 admit/virtualize/reject/re-realize sounds on the audio thread and preserve effective playback time on realization.
- `AudioMixerSourceManager.cpp:793-808,1251-1266,1646-1667` uses dense source arrays plus a free index stack for bounded acquire/release. Lines 4002-4120 update render state, game-thread copies and completion in the render lifecycle.
- `AudioMixerSourceBuffer.h:13,38-45,76-107` and `AudioMixerSourceBuffer.cpp:411-616` use bounded rotating buffers and asynchronous realtime decode for streaming inputs.

Zircon should adopt the demonstrated ownership split: world-qualified logical sounds, dense real voice slots, audio-thread commands, bounded completion/status pages, virtualization and bounded streaming. It must not copy Unreal's feature set or timing numbers; equivalent workloads and Zircon measurements decide acceptance.

## 5. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product path | Runtime supervisor and scene extraction create one real world slot; Editor preview/Play constructs a separate slot. | Fixed scene audibly plays and stops through product code; headless policy explicit; source/build/config/device receipt recorded. |
| M1 State ownership | Global device/residency plus per-world/session slots and generation-qualified dense IDs. | World close releases sources/listeners/volumes/events; stale callback/ID is rejected; preview and Play do not share gameplay state. |
| M2 Source diff | Compile descriptor fields to dirty commands; preserve cursor/last-good and crossfade incompatible changes. | Per-frame transform updates cause zero voice restart; gain/rate/mute/route latency and allocations are bounded and measured. |
| M3 Completion plane | Fixed pump, O(1) owner mapping and bounded cursor ring. | Cost depends on current changes, not API calls or historical completions; overflow/loss/age visible; soak has bounded memory. |
| M4 Clip residency | Canonical-key single-flight, prepared/streamed artifacts, leases and eviction. | Concurrent miss performs one load/decode; long-clip memory bounded by page budget; reload and cancellation preserve last-good. |
| M5 Spatial/external truth | Real listener/emitter/volume execution or explicit Unsupported; bounded external input adapter. | No accepted update performs useless storage-only work; transform/external latency, underrun/overflow and output quality verified. |
| M6 Observation/config | Immutable status pages and one resolved config generation. | O(1) status has no cleanup side effect; full snapshot explicit; receipts bind applied config and freshness. |
| M7 Dynamic qualification | Runtime/Editor/standalone/export/headless MVP workloads and overload/scene/device transitions. | Publish audio/control/main CPU, command latency, restarts, real/virtual voices, RSS/allocation, I/O, wakeups, xrun, power and audible parity. |

## 6. Direct-fix decision

The shared early cache hit in `clip_assets.rs` is the only simple local optimization in this slice and is preserved. It does not close single-flight or residency. Adding a reverse map, capping finished vectors or diffing a few descriptor fields independently would create partial ownership beside the required world-slot/voice allocator/completion design and cannot be regression-tested in the current session. No additional production source edit is made.

Static review is complete only for this 28-file slice. Timeline/automation, dynamic events/executors, mixer/device services, acoustic DSP, import/cook, Editor and dist remain separate review/acceptance scopes. No Git milestone commit or quantified WeCom notification is warranted.
