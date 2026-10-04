---
title: Sound Output and Kira Bridge Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_no_source_change
scope:
  - zircon_plugins/sound/runtime/src/output
  - zircon_plugins/sound/runtime/src/kira_bridge
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioMixerDevice.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerDevice.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceBuffer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Public/AudioDeviceNotificationSubsystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioDeviceNotificationSubsystem.cpp
---

# Sound Output and Kira Bridge Current-Source Algorithm Performance Review

## 1. Status and frozen scope

Sound output and the Kira bridge completed E3 current-worktree static review over **26/26 Rust files** at revision `7fe97290fd3b0350c2c0f404fd00ad2d18f1335d`:

| Module folders | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| `runtime/src/{output,kira_bridge}` | 26/26 | 2,671 / 2,470 | 91,990 | 2 / 1 | `06818cd315f2b19e241e953b22356f85146b0e3a187e59f744f85f8b646c5776` |

The fingerprint is SHA-256 over sorted `repository-relative-path|sha256(file-bytes)` rows joined by LF. All 26 files pass standalone `rustfmt --check --edition 2021 --config skip_children=true`; scoped `git diff --check` passes and the scope is clean. No production source changed.

Managed Windows Cargo is unavailable to this session, so Rust unit tests and release benchmarks were not run. No launchable current-source Editor/runtime artifact exists, and `wpr.exe`, `renderdoccmd.exe` and `qrenderdoc.exe` are unavailable in the selected environment. WPR/ETW, real CPAL device, callback/xrun, memory high-water, power and rendered-product evidence remain pending. RenderDoc cannot qualify CPU/audio-thread behavior even after a product artifact exists.

## 2. Per-folder review ledger

| Module folder | Reviewed files | Static result |
|---|---|---|
| `output` | `catalog.rs`, `descriptor_validation.rs`, `mod.rs`, `status.rs` | Catalog delegates to synchronous CPAL enumeration; descriptor validation is bounded; latency is requested/estimated rather than measured. |
| `output/lifecycle` | `config.rs`, `mod.rs`, `start_stop.rs`, `status.rs`, `storage.rs` | State transitions are small, but callback/render/xrun counters have no producer and therefore expose false healthy zeroes. |
| `kira_bridge` | `device.rs`, `graph_compile.rs`, `manager.rs`, `mod.rs`, `playback_data.rs` | Owns CPAL selection, graph compilation, all Kira handles and whole-clip conversion. Device identity/config probing, graph scale and clip residency are not production-ready. |
| `kira_bridge/graph_compile` | `routes.rs` | Memoized route closure still clones expanded downstream vectors; dense/long routing graphs have superlinear work and storage. |
| `kira_bridge/graph_validation` | `effect.rs`, `graph.rs`, `mod.rs`, `ordering.rs`, `references.rs`, `track.rs`, `values.rs` | Numeric/history bounds and deterministic cycle detection are sound local guards. The computed topology is discarded before handle construction; the release benchmark is ignored and only covers ready-queue selection. |
| `kira_bridge/manager` | `graph.rs`, `lifecycle.rs`, `playback.rs` | Applies Kira operations synchronously, reserves staged capacities, scans/sorts all playback handles on polling and has no voice allocator/virtualization. |
| `kira_bridge/manager/graph` | `transaction.rs` | Stages structural handles but repeats subtree scans and parent ordering; active playback rejects structural edits, so the staging complexity does not yet provide live graph continuity. |

Related caller review covered `service_types/{manager_state,mixer_graph,output_device}`, `engine/state/playback.rs`, completion call sites, graph tests and current optimize plans. Those files are context, not claimed as completed by this 26-file ledger.

## 3. Structural performance findings

### P0: the runtime has no audio supervisor boundary

`DefaultSoundManager` owns one `Arc<Mutex<SoundEngineState>>` containing Kira, graphs, clips, playbacks, sources and other domains. Device startup holds the config guard and state mutex while creating the CPAL/Kira backend, compiling/installing the graph and replaying configured sources. Graph commit applies Kira handles while the same state mutex is held. Device enumeration is a synchronous `cpal::Host::output_devices()` walk and can run on the caller/UI thread.

The required shape is a device/backend actor plus immutable graph generation and bounded command ingress. Expensive enumerate/probe/prepare/compile/decode work occurs outside gameplay/UI locks; a short publish step swaps last-good generation. The audio callback never takes the global state mutex, allocates, logs or calls foreign code.

### P0: completion polling turns ordinary API traffic into O(V log V) serialized work

`drain_finished_playbacks` scans every Kira handle, allocates a `Vec`, sorts finished IDs and removes them. `poll_kira_completions` is called before most playback/source query and control methods while holding the global state mutex. Mapping a completed Kira playback back to a source then linearly scans all sources. With V handles, C completions and S sources, one poll is O(V + C log C + C*S); repeated status/control calls multiply that cost even when nothing finishes.

Completion must be produced once at the audio/runtime boundary into a bounded generation-qualified ring or dense slot event list, then consumed by world/session cursors. Stable playback/source slots provide O(1) owner lookup. Query methods read immutable status pages and must not trigger global cleanup.

### P0: graph compile/update contains several independent superlinear algorithms

The current graph pipeline repeatedly reconstructs graph indexes instead of compiling one reusable immutable generation:

- validation computes a deterministic topological order, then discards it;
- `build_graph_handles` and staged handle construction recover parent order through `pending.iter().position` plus `Vec::remove`, O(T^2) for T tracks;
- expanded send closure clones cached downstream route vectors. A chain is O(T^2) copied route entries; a dense acyclic send graph can approach O(T^3) traversal/copy work and O(T^2) stored routes;
- diff creates before/after lookup maps and expanded sends again, while `track_depth` and `has_ancestor_in` rebuild the parent map per query;
- `subtree_ids` repeatedly scans all parents until convergence and is invoked again per rebuilt root in both diff and transaction paths;
- the public mutation path clones the graph, compiles outside the mutex, applies Kira commands inside the mutex and clones the accepted graph again.

The existing normal test accepts graph diff P95 below 100 ms, active Kira mutation below 250 ms, and a 1,000-track public lock budget of 255 ms. Those thresholds detect catastrophic regressions but are not MVP frame/audio-control acceptance gates.

Required replacement: `AudioGraphCompiler` builds stable dense track slots, parent/child adjacency, one topological order, send adjacency/target slots and structural hashes once. Parameter-only changes diff by generation/dirty slot. Structural preparation occurs off the control lock, followed by an audio-frame-boundary publish/crossfade; old generation remains last-good until commit succeeds. Qualification reports scale at 16/64/256/1,000 tracks and sparse/dense sends, including allocations and lock/command latency.

### P0: configured physical voice capacity scales as tracks times global voices

Default options are 64 tracks and 128 logical voices. Activation multiplies tracks and voices by three for staged generations, yielding 192 physical sub-track slots, 192 send-track slots and 384 sound slots for the main track. Every compiled sub-track is also built with `sound_capacity(384)`. At 64 tracks this requests capacity for up to **24,960 per-track sound slots** across main plus sub-tracks, although admission allows only 128 logical playbacks globally. This is a 195x slot-capacity ratio before Kira internal overhead is measured.

Do not tune the factor blindly. A global `VoiceAllocator` owns real/virtual/pending/stopping slots; tracks hold routing/mix state, not the full global voice budget each. Shadow graph capacity is budgeted separately and only for changed nodes. Measure backend allocation bytes/RSS before and after activation and graph sizes; acceptance binds observed memory to configured real voices, tracks and sends.

### P0: whole-clip PCM conversion prevents bounded residency

`SoundAsset` retains interleaved `Vec<f32>`, then `cached_static_sound_data` allocates another full `Arc<[kira::Frame]>`. Mono duplicates every sample into both channels. A long clip therefore retains source PCM and Kira stereo frames simultaneously, with no streaming threshold, chunk cache, async decode, eviction or residency budget in this layer.

The target is a cooked `AudioClipArtifact` with short prepared clips and long streamed clips, seek/chunk metadata, single-flight load, lease/pin/eviction and per-class budgets. Kira receives bounded pages or a custom streaming sound; decode workers publish ready pages without blocking the audio callback. Report encoded, decoded, Kira/page and duplicate high-water bytes separately.

### P1: device catalog and activation are optimistic rather than negotiated

Device IDs are derived from display names (`kira-cpal:{name}`), so duplicate names collide and rename/default changes break identity. Catalog descriptors copy the requested config instead of enumerating supported sample formats/rates/buffer ranges. Availability rejects only channel counts above two. Activation performs another linear enumeration and forces sample rate, channels and fixed buffer without a supported-config preflight. There is no cached catalog generation, hotplug/default-device notification, fallback, reopen backoff or last-good migration.

Create an `AudioDeviceSupervisor` state machine with stable backend device identity, supported-config negotiation and asynchronous catalog generation. Prepare a new stream and graph before publish; on loss/swap, preserve last-good when possible or enter explicit silent/headless fallback. Never report a requested format as device capability.

### P1: telemetry fields claim measurements that do not exist

`rendered_blocks`, `rendered_frames`, `callback_count`, `last_callback_sequence`, `next_callback_sequence` and `underrun_count` are initialized, reset and snapshotted but not updated by a production callback. Latency is `block_size_frames * latency_blocks`, while queued/capacity samples are `None`. A UI can read zero underruns/callbacks as health when the fields are actually unsupported.

The realtime side writes a preallocated atomic/SPSC telemetry page for callback sequence, frames, xrun/stream error, queue depth and command age. A collector publishes generation/window/staleness. Unsupported metrics use an explicit availability state; estimated and measured latency remain separate.

### P1: voice exhaustion and lifecycle degrade as API failure

When logical voice capacity is reached, play returns `BackendUnavailable`. There is no priority, concurrency group, audibility score, deterministic stealing, virtualization or promotion. Deactivation allocates and sorts all playback IDs. Structural graph changes are rejected whenever any non-stopped playback exists, so Editor/gameplay graph edits cannot use a shadow generation or preserve tails.

Add a deterministic global voice allocator and explicit graph/tail transition policy before increasing capacity. Metrics distinguish requested, admitted real, virtualized, stolen, rejected and stopping voices so comparisons cannot gain performance by silently playing less audio.

## 4. Unreal source evidence and adopted policy

This plan adopts structure demonstrated by the checked-in Unreal source, not Unreal naming:

- `AudioMixerSourceManager.h:54-69,328-333,465-478,718-730` defines render phases, command pumping, pending release, game-thread copies, a buffered command queue and an MPSC ingress.
- `AudioMixerSourceManager.cpp:793-808,1251-1266,1646-1667` preallocates dense source state and uses a free-source index stack for O(1) acquire/release instead of scanning a handle map on each query.
- `AudioMixerSourceManager.cpp:4002-4094,4120,4140-4299` pumps commands in the render update, updates game-thread copies and delivers completion from the render-side lifecycle.
- `AudioMixerDevice.h:359-362,534-537` and `AudioMixerDevice.cpp:2505-2533` expose render/game MPSC command queues. `AudioMixerDevice.cpp:1441-1489` detects stalls/device changes and drives a multi-call device swap.
- `AudioMixerSourceBuffer.h:13,38-45,76-107,163-182` bounds queued buffers, separates async read modes and owns decode task state.
- `AudioMixerSourceBuffer.cpp:411-616` uses cached first buffers, up to three rotating buffers and asynchronous realtime decode instead of requiring full-clip PCM duplication.
- `AudioDeviceNotificationSubsystem.h:51-110` and its implementation expose default-device, add/remove, state and switched notifications.

Zircon should therefore use dense stable voice slots, bounded cross-thread commands/events, immutable game/control snapshots, bounded async decode pages and an explicit device supervisor. It should not copy Unreal's feature count or assume its timing budgets; Zircon must measure its own current-source artifact.

## 5. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Product baseline | Launchable current-source minimal runtime and Editor audio scene; source/build/config/device identity recorded. | Real CPAL output or explicit headless policy; WPR profile and telemetry fields distinguish unavailable from zero. |
| M1 Ownership split | Device actor, immutable graph generation, per-session/world audio slot, bounded command/completion channels. | No device create/enumerate/graph build/decode under global manager mutex; callback has zero allocation/lock/log/foreign call. |
| M2 Graph compiler | Dense indexes, one topology, sparse send representation, dirty-slot diff and last-good publish. | 16/64/256/1,000-track sparse/dense curves report CPU, allocations and command/lock P50/P95/P99; no unexplained superlinear curve. |
| M3 Voice allocator | Global real/virtual/stopping slots, priority/concurrency/steal policy and O(1) owner mapping. | Query cost independent of V cleanup; completion loss zero within budget; overload degrades deterministically with reason telemetry. |
| M4 Clip residency | Prepared/streamed cook artifacts, single-flight, bounded page decode, lease/evict budgets. | Long music memory is bounded by page budget rather than duration; decode starvation/xrun and duplicate-byte high-water are reported. |
| M5 Device supervisor | Stable identity, format negotiation, catalog generation, hotplug/swap/recovery/backoff and last-good fallback. | Disconnect/default swap/unsupported format/stall tests preserve policy; UI remains responsive and reports transition generation. |
| M6 Observability | Callback/xrun/queue/command/voice/decode metrics through realtime-safe pages. | Unsupported is never encoded as zero; metrics include source, generation, clock, window and staleness. |
| M7 Product qualification | Runtime, Editor preview/Play, standalone/export and headless workloads on MVP scene. | Publish audio/control thread CPU, main-thread CPU, RSS/allocation, I/O, wakeups, xrun, latency, power and quality parity against the same workload/config. |

Frame/audio budgets must be derived from product buffer duration and update cadence. For example, a 256-frame block at 48 kHz is 5.33 ms, but the accepted callback budget must reserve platform/backend safety margin and be proven by xrun-free soak; it is not automatically 5.33 ms. WPR/ETW qualifies CPU scheduling, waits, wakeups and power. RenderDoc is only relevant when Sound Editor UI or audio-reactive rendering enters a frame, and cannot close audio algorithm gates.

## 6. Direct-fix decision

No production edit is made in this pass. Reusing the existing topological order or removing one sort would be locally simple, but would optimize a graph representation that the required immutable-generation compiler replaces. Changing completion polling without the supervisor/event ownership would also risk observable ordering and cleanup semantics. The current environment cannot execute Rust regression tests or a real Kira/CPAL artifact, so an audio-core edit would lack the required behavioral and dynamic evidence.

Static review is complete only for these 26 files. Sound service, engine/DSP, source/spatial/event/timeline, import/cook, Editor and dist folders remain separate review/acceptance scopes. No Git milestone commit or quantified WeCom notification is warranted until a current-source dynamic milestone passes.
