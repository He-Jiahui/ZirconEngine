---
title: Sound Automation Timeline Current-Source Algorithm Performance Review
date: 2026-08-24
status: static_complete_dynamic_pending_shared_optimizations_preserved
scope:
  - zircon_plugins/sound/runtime/src/automation
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_plugins/sound/runtime/src/mixer_configuration
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/timeline_sequences.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/automation_timeline.rs
canonical_owners:
  - docs/plans/optimize/zircon_plugins/11-first-party-sound-source-runtime-editor-dist-catalog-mixer-spatial-reverb-timeline-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/48-runtime-wide-state-next-state-transition-hook-history-schedule-scope-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AudioMixer/Private/AudioMixerSourceManager.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/AudioModulationSystem.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/AudioModulationSystem.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/SoundControlBusMixProxy.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/AudioModulation/Source/AudioModulation/Private/SoundControlBusMixProxy.cpp
---

# Sound Automation Timeline Current-Source Algorithm Performance Review

## 1. Status and frozen scope

The complete Sound automation/timeline transaction slice completed E3 current-worktree static review over **37/37 Rust files** at revision `39f7f45c5671b1b8515685198f000989a0f1d82a`:

| Module slice | Files | Physical / non-empty lines | Bytes | Tests / ignored | Current fingerprint |
|---|---:|---:|---:|---:|---|
| automation, timeline, mixer configuration and service surface | 37/37 | 1,936 / 1,756 | 71,647 | 7 / 3 | `0c5d95942bc34b60c2e73a974773cc90f8bf1832deb7669b2121b8a8a03a57b4` |

The scope includes three existing shared edits: `timeline/advance.rs` preallocates sequence/track vectors and avoids cloning an unused binding path; `service_types/automation_timeline.rs` projects only target/parameter instead of cloning the complete binding; `mixer_configuration/timeline.rs` queries the existing binding map instead of building a transient set. They are preserved. The other 34 files pass standalone rustfmt; those three shared files have import/assert formatting drift and are not rewritten by this review. Scoped diff check has line-ending warnings only.

Managed Windows Cargo, a current-source product executable, WPR/ETW and power evidence remain unavailable. The seven tests and three ignored release gates were inspected but not executed. No source edit is made by this review.

## 2. Per-module review ledger

| Module | Files | Static result |
|---|---:|---|
| `automation` | 22/22 | Curve sampling revalidates and linearly scans all keys per sample; string parameters dispatch at runtime; graph targets clone/validate/replace the whole graph per value; active Kira rejects every target. |
| `timeline` | 5/5 | Entire schedule is taken and advanced under one state mutex; each track allocates/clones result/application data; error is non-atomic and can drop the schedule. Schedule/remove are linear. |
| `mixer_configuration` | 7/7 | Configuration rebuilds registries and retains valid runtime state. Direct binding-map retention is a good local change; configuration is not the playback scheduler. |
| service and trait surface | 3/3 | Every operation is synchronous; advance holds the global Sound mutex for all sequences/tracks and returns a deeply allocated report. No affinity, ticket, cancellation, budget or generation exists. |

## 3. Product call-site truth

Repository-wide scan outside Sound implementation/tests found no runtime, app, Editor or plugin product caller of bind/apply/schedule/advance APIs. When Kira is active, `ensure_automation_execution_available` returns `UnsupportedAdvancedFeature` before every target mutation. When Kira is inactive, values mutate authoring/state metadata but cannot prove audible automation.

The current low workload is absence of product integration, not performance acceptance. Editor timeline UI and runtime world time must share a compiled time-domain contract before dynamic measurement is meaningful.

## 4. Structural performance and correctness findings

### P0: the active backend advertises a timeline surface it cannot execute

All track, effect, source, listener, volume and synth targets pass through one guard that rejects active Kira with “enabled by Sound M5.” The trait nevertheless exposes synchronous bind, arbitrary curve sample and timeline advance methods. A product caller can successfully author/schedule metadata and fail only during playback.

Capability admission must happen when binding/sequence is compiled. Unsupported target/backend pairs fail before persistence or scheduling. A supported target compiles to a stable backend slot plus typed parameter command; no string/graph lookup remains in the tick path.

### P0: timeline advance is non-atomic and can erase the schedule on error

`advance_timeline_sequences` removes the complete `state.timeline_sequences` with `mem::take`. Any later curve, binding or target error returns through `?` before restoring retained/unprocessed playbacks. Earlier parameter mutations remain applied, while the remaining schedule is dropped. `unbind_automation_impl` removes a binding without retiring dependent sequences, so a subsequent advance can deterministically hit this path. Active Kira rejection makes the first target another deterministic failure.

Required design: compile/validate all sequence bindings and target capabilities first; evaluate into a side buffer without live mutation; publish one bounded batch atomically; commit cursors only after acknowledgement. On failure, the prior schedule and parameter generation remain authoritative.

### P0: graph automation is full-graph reconstruction per sampled parameter

For track/effect targets, every value clones `state.graph`, linearly finds the track/effect, mutates one scalar, runs full `validate_graph`, then replaces the graph. A frame with `A` graph-bound tracks therefore approaches `O(A * (graph clone + graph validation))` while the global Sound mutex is held. Multiple properties of the same object repeat the work independently.

Separate structural graph edits from realtime parameters. Compile stable track/effect slots and parameter IDs at graph generation time. Timeline evaluation coalesces last value per slot for the current quantum and publishes a compact scalar command batch; structural edits alone prepare/validate a new immutable graph generation.

### P0: curve validation and search repeat for every track on every tick

`sample_automation_curve` performs a full finite/order validation `O(keys)`, then a second linear window scan `O(keys)`. Scheduled sequences already validate curves on admission, yet advance repeats both passes. Sequential playback ignores temporal locality and maintains no key cursor. Direct `apply_automation_curve_sample` accepts arbitrary curves, which is a separate uncompiled API problem.

Compile curves once into immutable segments. For monotonic playback maintain a cursor for amortized `O(1)` advance; seek/scrub uses binary search `O(log keys)`. Loop wrap resets/locates the cursor explicitly. Validation belongs to import/schedule generation, not each sample.

### P1: the whole workload runs synchronously under one global state mutex

`advance_timeline_sequences_impl` holds the manager state mutex across every sequence, curve sample, binding lookup, target clone, descriptor/graph validation and mutation. This can block playback/source/device APIs and any caller thread, including Editor UI or main/world update. There is no task ownership or per-frame budget.

Time-domain evaluation belongs on the runtime animation/task scheduler or a dedicated control stage. It reads immutable compiled sequences, produces a bounded command batch and never owns the audio callback. The Sound control actor applies/coalesces commands and publishes applied generation/status.

### P1: each tick constructs reports and clone-heavy applications unconditionally

Advance allocates a report vector, per-sequence samples and applications vectors, clones every target/parameter and clones sequence IDs. The shared capacity changes remove growth reallocations and unused path copies, but still allocate `2 * sequences` inner vectors per call and emit every sample even when no diagnostic consumer exists.

Use reusable evaluation scratch and fixed/declared batch capacity. Observability publishes counters and optional sampled traces behind a budget; the normal tick returns a compact generation/status rather than a complete sample tree. Overflow follows an explicit coalescing/drop/error policy.

### P1: binding/sequence lifecycle is not compiled or generation-safe

Binding replaces by ID without validating target existence/backend capability or invalidating dependent compiled sequences. Scheduling linearly searches sequence IDs and stores full owned curves; removal retains/scans all entries; querying clones every full sequence. Raw binding IDs have no generation, so replacement silently changes the meaning of an already scheduled track.

Use immutable compiled sequence generations, generation-bearing handles and indexed schedule slots. Rebinding creates a new generation and either recompiles dependents transactionally or leaves the previous last-good generation active.

### P1: shared microbenchmarks do not cover the dominant path

The three ignored gates measure capacity growth, unused-path clone removal and transient binding-set removal. The target projection benchmark applies only `SynthParameter` while Kira is inactive. It does not execute track/effect graph clone/validation, backend commands, contention, rollback, Editor scrubbing or real cadence. Results are unexecuted and cannot accept M5.

Retain them as local regressions after managed execution, but add complexity/cadence tests at key/track/sequence/graph scales and product traces with the active backend.

## 5. Unreal-primary policy adopted

- `AudioMixerSourceManager.cpp:1980-2123` routes pitch, volume, spatial and filter scalar updates as audio-mixer commands; it does not clone the authored mixer graph for each value.
- `AudioMixerSourceManager.cpp:4007-4023,4227-4299` pumps bounded render phases/command queues; the header owns source and MPSC queues plus fill observability.
- `AudioModulationSystem.h:100,178-181,258-262` separates modulation processing-thread ownership and an MPSC command queue.
- `AudioModulationSystem.cpp:529-539` establishes processing-thread identity and drains commands at the modulation update boundary.
- `SoundControlBusMixProxy.h:83-95` owns proxy stages and time-based update; `SoundControlBusMixProxy.cpp:180-187,229-265` updates target/fade state in cached proxies and mixes current values into bus proxies.

Zircon adopts compiled proxy/slot ownership, command batching and time-based smoothing. It does not copy Unreal's thresholds; acceptance numbers must come from Zircon's current-source workload receipt.

## 6. Required optimization plan

| Milestone | Required result | Acceptance gate |
|---|---|---|
| M0 Capability truth | Backend/target/parameter support resolved before bind/schedule. | Active runtime cannot accept a sequence it will reject on first advance. |
| M1 Compiled sequence | Validated immutable curves, typed parameter slots, binding generations and seek cursors. | Sequential sample amortized `O(1)`, seek `O(log keys)`; no string/full validation in tick. |
| M2 Transaction semantics | Side-buffer evaluation, atomic command batch and last-good cursor/parameter generation. | Any invalidation/overflow/backend failure preserves prior schedule and applied values. |
| M3 Scheduler ownership | Runtime time domain evaluates off UI/main critical section; Sound control actor applies batches. | No global Sound mutex across curve evaluation/graph validation; declared per-frame budget/cancellation. |
| M4 Parameter/graph split | Realtime scalar slots update without graph clone; structural edits prepare new graph generations. | `A` scalar tracks produce at most one coalesced command per changed slot, not `A` graph rebuilds. |
| M5 Observation | Compact generation/counters plus budgeted trace pages. | Normal tick has reusable bounded scratch; full sample report is opt-in and capped. |
| M6 Dynamic qualification | Runtime and Editor play/scrub/loop/rebind/overload current-source timelines. | Record evaluation/control/audio/main CPU, mutex/queue P50/P95/P99, allocations, commands/coalescing, missed deadlines, RSS, wakeups, power and audible parity. |

## 7. Direct-fix decision

The three shared local optimizations are preserved, but no additional source edit is made. The dominant fix changes compilation, lifecycle, transaction and thread ownership; a binary search or another clone removal inside the current synchronous API would leave the `A * graph` behavior and destructive error path intact. The three dirty shared files also cannot be safely reformatted or transactionally rewritten without managed Cargo and coordination with their owner.

Static review is complete only for these 37 files. Dynamic acceptance, a Git milestone commit and quantified WeCom notification are not warranted.
