---
title: AI Runtime Perception Frame Product Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/ai/runtime/src/capability.rs
  - zircon_plugins/ai/runtime/src/module.rs
  - zircon_plugins/ai/runtime/src/perception.rs
  - zircon_plugins/ai/runtime/src/perception
  - zircon_plugins/ai/runtime/src/plugin.rs
  - zircon_plugins/ai/runtime/src/plugin
  - zircon_plugins/ai/runtime/src/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AIPerceptionSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AIPerceptionComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AISense_Sight.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/EnvironmentQuery/EnvQueryManager.cpp
---

# AI Runtime Perception Frame Product Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

This production scope is **11/11 Rust files**, **2,199 physical / 2,009 non-empty lines**, **79,484 bytes** and **6 inline tests**. At repository revision `f811b3bf474d70347199772a175422333dfb36f6`, its ordered fingerprint is `0e74b6152d1bcfcb03b9c44747beb9556557d212da81823732b4621903555043`.

| Boundary | Files | Static result |
|---|---:|---|
| Perception collection/scan/stimuli/adapter | 5 | Sampling, aging, pair cursor, physics query, hearing queue and snapshot ownership reviewed. |
| Plugin/frame assembly | 4 | Scene-system ordering, World access, manager publication and debug event construction reviewed. |
| Capability/module/LOD | 2 | Advertised product scope, startup and distance-frequency behavior reviewed. |

No EQS implementation exists in the current AI Rust scope despite EQS being part of the canonical product plan. Capability therefore cannot be considered complete.

## 2. Structural performance findings

### P0: the pair budget begins after unbounded frame work

`perception/scan.rs:109-147` collects every receiver/source from the World, builds receiver-age and hearing-receiver vectors, ages all stored stimuli and performs a nested receiver/source existence check before pair-budget enforcement. `collect_perception_samples` walks `world.node_records()` and dynamically decodes perception components for every node.

The pair cursor then traverses a Cartesian `R * S` slot space (`scan.rs:198-223`). A pair quota limits calls to `scan_pair`, but not collection, decoding, aging, vector construction or the pre-scan nested work. Worst-case refresh latency is proportional to `ceil(R*S/budget)` frames, and skipped pairs have no importance/age prioritization.

### P0: sight uses per-pair math/allocation and fails open without physics

For each admitted sight pair, `scan.rs:236-305` computes distance with a square root, converts degrees to radians, computes cosine, normalizes direction and allocates `excluded_entities: vec![receiver.entity]`. If no physics occlusion provider answers, `:281-285` records a fallback pair and marks the source visible. Missing product infrastructure therefore becomes optimistic perception instead of typed capability failure.

The query is synchronous on the scene update path. There is no persistent spatial index, source/listener registration delta, trace-time budget, async query receipt or pending-query priority structure.

### P0: the frame always deep-builds global debug state

After perception, `plugin/registration.rs:384-399` materializes every perception snapshot and replaces the manager's whole-world snapshot set. Behavior tick builds distance LOD maps, active-entity sets and targeted snapshots, clones report/tree/blackboard/perception/debug DTOs, then emits a full `AiBehaviorDebugSnapshot` every frame (`:415-503`) regardless of whether an editor consumer is attached.

LOD only reduces selected behavior ticks. It does not eliminate full active-agent discovery, snapshot reconstruction, stale report retention checks or global debug publication. This can make editor diagnostics dominate the very runtime it observes.

### P1: perception owns repeated dynamic World extraction

Receiver/source collection derives typed samples by scanning generic scene records rather than maintaining registered component/query sets. Camera-distance LOD separately looks up transforms per agent and computes Euclidean distance. These operations occur under `with_world_mut`, alongside integration tasks, event publication and debug extraction, preventing meaningful overlap with unrelated systems.

### P1: stimuli publication is snapshot replacement, not delta delivery

`PerceivedStimuli` ages the complete retained map and snapshots clone retained entries. The manager replaces a world's perception snapshots wholesale. Hearing has a budgeted pending queue, but receiver lists are rebuilt each frame and snapshot publication does not expose changed/forgotten deltas or consumer backpressure.

## 3. Unreal source constraints

Unreal supplies the relevant ownership model:

- `AIPerceptionSystem.cpp:160-235` admits source registrations through pending work, ages stimuli on a configured schedule, updates senses only when their progress time is due, caches listener locations only for sense updates, and processes only listeners with new stimuli.
- `AIPerceptionComponent.cpp:482-620` moves a pending-stimuli queue, updates affected perceptual entries and broadcasts only updated actors instead of rebuilding every listener snapshot every frame.
- `AISense_Sight.cpp:136-142` defines explicit trace-count and time-slice defaults. `:260-440` retains in-range/out-of-range/pending query arrays, prioritizes by age/importance, advances a cursor and respects trace/time budgets including async pending work.
- `EnvQueryManager.cpp:434-590` advances running EQS queries one step under `MaxAllowedTestingTime`, supports breadth/depth scheduling and accounts async/time work. Zircon currently has no equivalent EQS product path.

The transferable design is persistent registration/query state plus multiple budgets and delta publication. Unreal's numeric defaults are references, not Zircon acceptance targets.

## 4. Dependency-ordered optimization plan

### M0: fail closed and publish truthful capability

If physics sight, EQS or a named sense is unavailable, report typed partial capability and refuse dependent assets/operations. Missing occlusion must not silently mean visible. Debug capability must state whether a demand subscriber and bounded transport are active.

### M1: register typed changed inputs

Replace per-frame `node_records()` decoding with ECS query/change generations for receivers, sources and transforms. Precompute normalized forward vectors, squared ranges and cosine thresholds when receiver settings change. Retain source/listener membership by world generation.

### M2: install persistent prioritized query sets

Maintain in-range/out-of-range/pending sight queries keyed by stable listener/source handles. Reprioritize by importance, distance and age without recreating the Cartesian product. Bound collection, pair evaluation, traces and elapsed time separately; expose oldest-query age and deferral reasons.

### M3: separate extract, async query and commit

Extract immutable candidates briefly, dispatch physics/read-side work through generation-qualified scheduler jobs, then commit only current results and stimuli deltas. Define cancellation/replacement behavior for destroyed worlds/entities/providers. Keep synchronous main-lane work under an explicit budget.

### M4: publish changed stimuli and on-demand diagnostics

Age/update only scheduled or affected listeners, publish refreshed/forgotten deltas, and retain snapshots at consumers that request them. Gate behavior debug construction by active reader/session, selected agent/bounds and frequency; enforce frame/byte/primitive limits and overflow receipts.

### M5: implement EQS as a budgeted product pipeline

Compile query templates, retain running query instances, execute one bounded step per scheduler admission, support async tests and publish generation-qualified results. Editor authoring/preview must consume the same compiled query artifact.

### M6: qualify scale and power

Measure listeners/sources `0/1/100/1k/10k`, in-range density, movement/change rate, sight/hearing mix, physics hit rate, async latency, debug readers `0/1`, and EQS breadth/depth modes. Report p50/p95/p99, collection/pair/trace/commit time, scanned/aged entries, allocations/bytes, oldest deferred age, CPU, wakeups, RSS and WPR energy estimates.

## 5. Acceptance gates

1. Stable frames perform zero generic World-node scans and zero component JSON decoding for unchanged perception membership.
2. Frame budgets include collection, pair selection, physics traces, elapsed time and publication, not only `scan_pair` calls.
3. Query latency is bounded/observable and prioritization prevents starvation as `R*S` grows.
4. Missing physics/EQS capability fails closed and cannot change perception semantics silently.
5. With no debug reader, behavior/perception debug snapshot construction and transport are zero.
6. World/entity/provider replacement cancels stale work and old generations cannot publish.
7. Current-source WPR/ETW and executable scenarios prove scale/power behavior before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **11/11 complete** for the captured fingerprint.
- Unreal reference-source comparison: complete for perception scheduling, sight budgets and EQS step ownership.
- Cargo/tests: pending because the managed Windows validation session is not executable.
- WPR/ETW/power: pending because no launchable current-source executable exists; no cross-engine power parity is claimed from static code.
- RenderDoc: not a CPU/perception profiler; use it only after a current-source viewport overlay/rendering scenario exists.
- Protected ledgers, milestone commit and WeCom completion remain pending.
