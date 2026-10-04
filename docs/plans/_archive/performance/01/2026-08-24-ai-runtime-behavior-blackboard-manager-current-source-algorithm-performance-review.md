---
title: AI Runtime Behavior Blackboard Manager Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/ai/runtime/src/behavior_tree.rs
  - zircon_plugins/ai/runtime/src/behavior_tree
  - zircon_plugins/ai/runtime/src/blackboard.rs
  - zircon_plugins/ai/runtime/src/blackboard
  - zircon_plugins/ai/runtime/src/manager.rs
  - zircon_plugins/ai/runtime/src/manager
  - zircon_runtime/src/core/framework/ai
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/59-runtime-task-execution-job-scheduler-handle-dependency-cancellation-thread-budget-timer-shutdown-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/60-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BlackboardComponent.cpp
---

# AI Runtime Behavior Blackboard Manager Current-Source Algorithm Performance Review

## 1. Coverage and execution truth

The production execution scope is **32/32 Rust files**, **8,640 physical / 7,948 non-empty lines**, **307,038 bytes** and **41 inline tests**. At repository revision `f811b3bf474d70347199772a175422333dfb36f6`, its ordered `workspace-relative path + NUL + raw bytes + NUL` SHA-256 is `a020f29f4a7438b4ab9eef5d325e805b8ab97e8f00484d67ff29b01917c52e96`.

The nine public AI contract files under `zircon_runtime/src/core/framework/ai` were also traced through the runtime behavior and script integration call chain. Existing concurrent AI edits were treated as current source and preserved. No Cargo execution or dynamic timing is claimed.

| Boundary | Files | Static result |
|---|---:|---|
| Behavior tree compiler/catalog/executor | 17 | Dense preorder, subtree/observer indices, node semantics, recursion and runtime-state ownership reviewed. |
| Blackboard layout/store/observer | 5 | Dense storage, validation, ingress/egress conversion and change notification reviewed. |
| Manager registration/snapshot/tick | 10 | Global ownership, generation publication, leases, per-agent work and LOD scheduling reviewed. |

## 2. Structural performance findings

### P0: one agent tick resolves every registered tree implementation

`manager/tick.rs:55-95` locks the process-wide manager, finds the requested tree linearly, then flattens implementation slots from the complete compiled-tree generation. It resolves and allocates owners for all those slots before acquiring an execution lease, even though the agent can only execute the reachable root/subtree program. With `A` ticked agents, `T` registered trees and `S` total implementation slots, admission contains an avoidable **O(A * S)** catalog/lease path rather than O(reachable program dependencies).

The same admission clones a schema and stored perception, validates the complete blackboard input and removes/reinserts blackboard and instance state (`manager/tick.rs:97-150`). Dense slots and scratch buffers are useful local improvements, but they do not remove this outer algorithm.

### P0: one global mutex owns all worlds while scheduling remains serial

`DefaultAiManager` stores all worlds in one mutex. `manager/tick.rs:345-379` scans active entries, allocates candidate/request vectors and clones per-agent perception; `:381-389` then ticks every request serially. The runtime integration host holds `&mut World`, so changing the loop to a parallel iterator would be unsound and would merely move contention.

The required boundary is extract/evaluate/commit: briefly extract immutable, generation-qualified inputs; run pure/read-only prepared programs on scheduler jobs according to declared node affinity; commit navigation, animation, script and event side effects on their owning lanes. World mutation must not be smuggled into generic worker tasks.

### P0: the catalog advertises behaviors that do not exist

The standard catalog exposes `SetBlackboard`, `EmitEvent` and `UpdateBlackboardDistance`. In `behavior_tree/executor.rs:387-402`, distance update delegates to a generic service result, while set-blackboard and emit-event delegate to generic task evaluation. They do not perform their named mutation/event behavior. Tests can inject `result` or `service_result` parameters and call that node semantics, but this is not product execution.

This makes both optimization and performance measurement invalid: a cheap placeholder appears faster than the behavior users selected. Capability must be partial or unavailable until each named node has typed compiled inputs, actual side effects, failure semantics and integration coverage.

### P1: compiled execution retains authoring strings and linear parameter lookup

`zircon_runtime/src/core/framework/ai/behavior_tree.rs:121-149` models node IDs, implementation IDs, child IDs and parameter keys as owned strings. Compiled nodes still retain parameter DTO arrays, and hot helpers search them linearly (`executor/support.rs:115-140`). Instance state uses `BTreeMap<String, Vec<_>>`, string tree stacks and string diagnostics (`executor.rs:46-100`). Weighted-child lookup scans parameter keys by prefix/index for each candidate.

Compilation has already added dense node indices, parent/subtree targets and observer indices. The next step is a prepared immutable program: interned/stable IDs, typed operands, reachable implementation-owner set, packed per-node state layout, precomputed weights/policies and bounded diagnostics. Authoring DTOs should not remain the execution representation.

### P1: traversal has no explicit work or depth contract

Subtrees, reactive selector rechecks, observers and aborts can recursively revisit programs. Cycle validation exists, but there is no product-facing maximum depth, node-evaluation budget, abort/restart budget or over-budget continuation receipt. A legal large tree or oscillating observer graph can monopolize the scene update lane.

`Parallel` is a behavior-tree composite semantic, not evidence of worker execution. Its children can include mutable integration tasks, so scheduler parallelism must be based on declared read/write effects and affinity, not the node name.

## 3. Unreal source constraints

Unreal is the primary structural reference for ownership and scheduling:

- `BehaviorTreeComponent.cpp:1698-1881` accumulates elapsed time, returns when the next requested tick is not due, ticks only active auxiliary/parallel/task work, and processes execution only when requested. `:1923-1947` disables ticking when no work remains or schedules the next interval.
- `BehaviorTreeManager.cpp:263-310` caches one initialized tree template and packs instance memory using the compiled node memory requirements. It does not rediscover every registered node owner per agent tick.
- `BlackboardComponent.cpp:120-176` creates a dense key-offset table and packed value memory at schema initialization. `:331-490` notifies observers by changed key, supports pause/queue, and deduplicates notifications.

Zircon should adopt those lifetime properties: compiled-once programs, packed instance state, demand scheduling and changed-key propagation. It should not copy UObject layout or Unreal's exact constants.

## 4. Dependency-ordered optimization plan

### M0: make behavior capability truthful

Mark named nodes without product semantics unavailable/partial and fail their compilation. Add one executable contract test per advertised node before restoring capability. Reject unknown or missing integration owners at compile/install time rather than every agent tick.

### M1: compile one prepared program generation

Produce immutable generation-qualified programs with stable node/tree IDs, typed operands, packed state offsets, precomputed child weights/policies, observer routes and the exact reachable implementation-owner set. Acquire provider-generation leases once per installed program/instance generation, not from the global catalog per tick.

### M2: establish world/session ownership

Shard manager state by world/session generation. Add install, replacement, unload, cancellation/join and shutdown receipts. Keep immutable compiled programs shareable while instances, blackboards, perceptions and pending delta remain world-owned.

### M3: make blackboard ingress and publication incremental

Validate schema/type at admission or external mutation boundaries. Use stable slot IDs and changed-slot lists internally; queue/deduplicate observer notifications; publish debugger/editor DTOs only for demanded agents/keys/generations. Stable ticks must not clone or revalidate the full blackboard.

### M4: schedule extract/evaluate/commit

Extract current agent inputs and immutable program handles without holding mutable World access. Dispatch pure/read-only program slices through Runtime59 with explicit dependencies, cancellation and budgets. Commit integration requests on declared main/world/script/navigation/animation lanes and resume agents from receipts.

### M5: bound execution and diagnostics

Add depth, evaluated-node, observer-abort, restart, integration-request and diagnostic-byte budgets with continuation state. Instrument lock wait/hold, extract, program evaluate, integration wait/commit, observer work and publication separately.

### M6: qualify algorithms and product semantics

Measure agents `0/1/100/1k/10k`, trees `1/10/1k`, nodes `1/100/10k`, active depth, observer fan-out and blackboard slots/changes. Report p50/p95/p99, allocations/bytes, evaluated nodes, lock time, task/wait time, CPU, wakeups, RSS and power. Include script/navigation/animation integration and over-budget continuation.

## 5. Acceptance gates

1. Agent tick cost depends on the reachable prepared program, not every registered tree/provider.
2. Stable blackboards perform zero schema rebuild, full-entry validation and unchanged-slot publication.
3. Every advertised standard node executes its named product behavior or fails closed at compile/admission.
4. Worlds do not contend on one process-global AI state lock; handles and work are owner/generation qualified.
5. Scheduler jobs never retain mutable World access and integration effects commit through explicit affinity lanes.
6. Depth, node, abort and diagnostic work are bounded with observable continuation/defer reasons.
7. Current-source WPR/ETW evidence confirms scale behavior before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **32/32 complete** for the captured fingerprint.
- Unreal reference-source comparison: complete for behavior scheduling, tree template memory and blackboard observers.
- Direct source optimization: intentionally deferred; 27 overlapping AI/shared files are already modified, and the safe change requires a prepared-program/world-ownership contract rather than a local loop patch.
- Cargo/tests: pending because the managed Windows validation session is not executable; raw Cargo was not substituted.
- WPR/ETW/power: pending because no launchable current-source executable exists.
- RenderDoc: not applicable to this CPU execution phase.
- Protected ledgers, milestone commit and WeCom completion remain pending.
