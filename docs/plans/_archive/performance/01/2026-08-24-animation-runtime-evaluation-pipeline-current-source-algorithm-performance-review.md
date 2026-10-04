---
title: Animation Runtime Evaluation Pipeline Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/animation/runtime/src/channel_sampling
  - zircon_plugins/animation/runtime/src/evaluation
status: static_complete_m0_implemented_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstanceProxy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SkeletalMeshComponent.cpp
---

# Animation Runtime Evaluation Pipeline Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

The post-M0 production scope is **68/68 Rust files**, **8,837 physical / 8,183 non-empty lines**, **304,453 bytes** and **25 inline tests**. The captured worktree is based on repository revision `1538a67d526d4c8dff93aa96e189751c06f80ad6`; its ordered `path + NUL + raw bytes + NUL` SHA-256 is `efe888d3d708632b7629f7e4e3ba4ad5779bc8d17e62ff71e777672027ca720b`.

All Rust files under `channel_sampling` and `evaluation`, including every nested folder, were indexed and reviewed. Existing concurrent worktree edits in the evaluation pipeline were treated as current source and preserved. Integration-test files under `runtime/tests` are not promoted by this record because the managed Windows validation session is unavailable.

| Folder | Files | Static review result |
|---|---:|---|
| `channel_sampling` | 3 | Binary key lookup is sound after validation; no segment cursor or compressed channel representation exists. |
| `evaluation` root | 12 | Skeleton/target compilation, pose pool and public diagnostics reviewed. |
| `evaluation/clip_evaluator` | 15 | Revision caches are bounded by entries, but invalidation/stat collection and final-pose materialization remain per sample. |
| `evaluation/compiled_animation_clip` | 3 | Dense target slots are positive; channel payloads are cloned into the compiled clip. |
| `evaluation/compiled_graph` | 5 | Source names lower to slots, but evaluation still recursively walks graph paths. |
| `evaluation/pipeline` | 26 | ECS scan, scheduling, graph/state evaluation, events, physics and scene commit reviewed end to end. |
| `evaluation/pose_buffer` | 4 | SOA scratch reuse is positive; output ownership still reconstructs named bone vectors. |

## 2. Structural performance findings

### P0: worker submission blocks inside the mutable World phase

`tick_animation_world` acquires the mutable World at `tick.rs:191` and calls `sample_direct_clip_pose_requests` before releasing it. The worker helper round-robins requests into at most four fresh vectors, creates one synchronous channel per shard, ignores the scheduler submission result, and immediately waits on every receiver at `direct_clip_worker.rs:84`. A failed task panics instead of producing a bounded diagnostic.

This is synchronous fan-out/fan-in under exclusive scene ownership, not parallel animation scheduling. It prevents unrelated World work from overlapping evaluation and makes the owner thread inherit the slowest shard plus queue delay. Graphs, state machines and layers are still evaluated serially in the same phase. The pipeline reacquires mutable World ownership at least ten times in `tick.rs` and sixteen times across the pipeline, further fragmenting extract/evaluate/commit.

### P0: compiled graphs are not compiled execution programs

`CompiledAnimationGraph::evaluate` calls recursive `collect_clips` from the output. A shared subgraph is evaluated once per incoming path rather than once per node, so a DAG can scale with path count and approach exponential work. The frame cache linearly searches a bounded `VecDeque`, compares complete parameter maps, and clones the parameters into the cache. Graph timing resolution separately constructs a clip signature and resolves resource snapshots before it can decide whether timing is cached.

The manager-facing graph implementation remains another recursive source-asset evaluator. It performs linear node lookup during recursion and allocates visited names and result vectors. Runtime therefore has two authorities with different cost and semantics instead of one immutable topological program.

### P1: per-instance cache maintenance and final pose materialization erase scratch reuse

Every `AnimationClipEvaluator::sample_clip` drains resource events into a new `Vec`, even when there are none. After every sample it sums pool misses across every cached skeleton, producing O(instances * cached_skeletons) bookkeeping. Up to four worker evaluators each own skeleton/clip caches, pose pools, subscriptions and diagnostics, so the same resources can be compiled and retained four times.

The pose pool reuses only SOA scratch. Every result constructs a new `Vec<AnimationPoseBone>` and clones every bone name. Presentation, physics-target publication and transform application then copy or rebuild pose-shaped collections again. State-machine layers allocate two new `PoseBuffer` values per layer instead of borrowing the existing pool.

### P1: scan and commit are allocation-heavy and not change-driven

The five cached ECS queries are a good baseline and avoid a raw full-World scan. However, every frame still creates ordered seen-entity sets, staging vectors, pose-source sets and playback maps, clones graph/state parameter maps, and requests resource snapshots/revisions per entity. Playback times are fetched before the scan and again inside it. Disabled or missing-manager frames repeatedly clear and republish empty state instead of doing one transition update.

Sequence application loads owned assets before the World phase, takes and filters the whole sequence cache, and applies property mutations serially. Pose application first builds a transform-update vector and then mutates scene nodes bone by bone. The expected stable-frame property is not yet defined: zero compilation, zero resource-event allocation, zero unchanged pose copies and zero transform writes.

### P1: cache limits are entry counts, not memory or work budgets

Skeleton, clip, graph, timing, state-machine and diagnostic caches have fixed entry limits, which is better than unbounded retention. They do not enforce compiled bytes, joints, tracks, channels, keys or dependency fan-out. Eviction scans ordered maps for the least-recent sequence. There is no per-frame animation time budget, relevance/LOD policy or server-specific reduction path.

## 3. Unreal source constraints

Unreal is the primary structural reference, not a surface API template:

- `AnimInstance.cpp:796-934` performs game-thread pre-update and post-update around proxy state. Worker-produced notifies are explicitly dispatched after parallel completion on the game thread.
- `AnimInstance.cpp:1047-1138` routes Update and Evaluate through an any-thread proxy and uses a scoped transient pose lifetime for an evaluation pass.
- `AnimInstanceProxy.cpp:476-607` copies game-thread-dependent state in `PreUpdate` and commits in `PostUpdate`; `UpdateAnimationNode` runs through the proxy rather than mutating the live scene.
- `SkeletalMeshComponent.cpp:329-429` defines separate task-graph evaluation and game-thread completion tasks. `2915-3001` decides update/evaluation skipping and dispatches without synchronously waiting inside the setup phase when parallel work is valid.

Zircon should adopt the phase ownership: short game-thread extract, immutable worker input, dependency-tracked worker Update/Evaluate, and short owner-thread commit. It should not copy Unreal's object model.

## 4. Dependency-ordered optimization plan

### M0: remove proven compile-time redundant lookup - implemented

Use the already-built unique-name index in `SkeletonTargetTable` instead of rescanning every bone for each legacy leaf-name track. Preserve unresolved and ambiguous error semantics with the existing contract tests. This reduces legacy clip target resolution from O(tracks * bones) comparisons to O(tracks * log bones) with the current ordered map. It does not claim to solve frame evaluation.

Implemented in `evaluation/skeleton_target_table.rs`: the redundant `bone_names: Box<[String]>` is removed and unique/ambiguous/unresolved resolution reads `bone_name_indices` directly. Quantified static delta is one duplicate array of `B` bone-name strings per compiled skeleton to zero, and legacy leaf-name resolution from up to `T * B` string comparisons to `T` ordered-map lookups. A missing-name behavior case was added beside the existing ambiguous-name contract in `runtime/tests/animation_target_table_contract.rs`.

### M1: establish one animation instance/proxy and one compiled asset generation

Create a per-world/per-entity animation instance containing dense parameter slots, state, cached triangle, pose buffers and dependency generations. Keep skeleton names/paths in immutable metadata shared by all instances. Consolidate clip, graph and state-machine compiled artifacts behind one generation-qualified cache with byte/count budgets; worker shards must not duplicate caches or subscriptions.

### M2: lower graphs and state machines to bounded programs

Compile reachable graph nodes into one non-recursive topological program and evaluate each node at most once per instance. Replace full parameter-map equality with generation/change masks and dense slots. Compile clip timing, event spans, masks, nested-machine references and transition expressions into the same dependency artifact.

### M3: split extract, parallel Update/Evaluate and commit

Extract immutable instance inputs and resource-generation handles while briefly owning the World. Submit dependency-tracked batches through the runtime scheduler, continue other schedule work, and commit only after the task dependency completes. Never block on an ad hoc receiver while a mutable World borrow is active. Fail task admission/completion through typed bounded diagnostics.

Use stable work partitioning by measured cost such as joints, active nodes, layers and clips rather than round-robin instance count. Expose queue delay, worker time, owner wait, shard imbalance and fallback reason.

### M4: make pose ownership dense and change-driven

Keep local/component transforms in reusable dense arrays keyed by stable bone slots. Store names and paths once with the skeleton artifact. Publish versioned pose handles to rendering and physics, and materialize named DTOs only for diagnostics/editor inspection. Apply only changed scene bindings and avoid rebuilding physics targets or presentation maps when generations match.

### M5: add relevance, LOD and time budgets

Add target/profile policy for server, client and editor preview. Budget by measured update/evaluation/commit cost, visibility/significance, minimum quality and dependency groups. Support skip, interpolated update and reduced-work modes without violating root motion, events or physics ownership.

### M6: qualify the current-source runtime

Measure fixed matrices for entities `0/1/100/1k/10k`, bones `32/128/256`, graph nodes `10/100/1k`, DAG fan-out, layers `0/4/16`, clips/events and cache churn. Report p50/p95/p99 for scan, queue, update, evaluate and commit; World lock time; allocations/bytes; compiled/cache bytes; duplicate compiles; skipped/reduced work; pose copies; transform writes; CPU, wakeups, RSS and power.

## 5. Acceptance gates

1. No owner thread waits on animation worker completion while holding mutable World access.
2. Each reachable graph node executes at most once per instance evaluation; deep graphs are non-recursive and explicitly bounded.
3. Stable frames perform zero compilation, zero resource-event vector allocation, zero bone-name cloning and zero unchanged transform writes.
4. Compiled resources and pose storage are shared by generation and bounded by entries, bytes and input complexity.
5. Animation budget policy proves quality/relevance behavior on client, server and editor preview.
6. Managed benchmarks report scale curves and p50/p95/p99 with no worse-than-linear instance/joint growth after compilation.
7. A current-source executable passes correctness parity and WPR/ETW timing/power capture before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **68/68 complete** for the captured fingerprint.
- Scoped M0: implemented; `rustfmt --check` and scoped `git diff --check` pass, with legacy unique-name lookup O(T * B) -> O(T * log B) and duplicate name-array cardinality B -> 0.
- Source behavior and caller tracing: complete; worker wait, graph recursion, cache ownership and pose-copy paths were traced end to end.
- Cargo/tests: **pending** because the managed Windows validation session is not executable; the added/existing target-table contracts were not executed and raw Cargo was not substituted.
- WPR/ETW: **pending** because no current-source executable exists.
- RenderDoc: **not applicable yet**; CPU scheduling and pose ownership must be measured with CPU tools. It becomes relevant only after a real rendered skinning path exists.
- Protected `review.md`/`pending.md`, milestone commit and WeCom completion: unchanged/pending until dynamic acceptance.
