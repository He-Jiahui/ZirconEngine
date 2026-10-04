---
title: Animation Plugin Assembly Manager Editor Dist Current-Source Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/animation/runtime/src/*.rs
  - zircon_plugins/animation/runtime/src/manager
  - zircon_plugins/animation/editor/src
  - zircon_plugins/animation/dist/src
status: static_complete_dynamic_pending
canonical_owners:
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstanceProxy.cpp
---

# Animation Plugin Assembly Manager Editor Dist Current-Source Performance Review

## 1. Coverage

The assembly/product-boundary scope is **19/19 Rust files**, **1,567 physical / 1,437 non-empty lines**, **57,490 bytes** and **11 inline tests**. At repository revision `1538a67d526d4c8dff93aa96e189751c06f80ad6`, its captured worktree fingerprint is `dc342d53a562d51cd298995ef1f1118fabee0d485b94fa1fe6dec8c92f691f24`.

Together with the evaluation and state/capability records, this completes static production coverage for **137/137 Animation Rust files**, **12,766 physical / 11,731 non-empty lines**, **437,863 bytes** and **37 inline tests** after the scoped M0. The composite fingerprint is `6c716c681787e0f4076916509ce62595735e70facb68569fa3f0806ac7cfe867`.

| Folder | Files | Static review result |
|---|---:|---|
| Runtime root | 7 | Capability, module, plugin, system and public surface reviewed. |
| Runtime manager | 6 | Playback configuration plus source-asset graph/state/clip evaluation reviewed. |
| Editor | 5 | Declarative authoring view/drawer/templates and inspector registrations reviewed. |
| Dist | 1 | Stateless native registration carrier reviewed. |

## 2. Structural performance findings

### P0: runtime exposes two animation algorithms

`DefaultAnimationManager` publicly evaluates raw graph/state-machine/clip assets through `manager/{graph,state_machine,pose,sampling}.rs`, while `AnimationEvaluationPipeline` owns separate compiled graph, state-machine, clip and pose logic. The manager graph path performs recursive linear node lookup and source-string allocation; its state-machine path linearly searches states/transitions and clones the complete parameter map into the result; its clip sampler performs repeated validation/search with different behavior from the compiled evaluator.

This is a product and correctness problem before micro-optimization. Callers can select different algorithms under the same animation domain. One compiled artifact/instance service must own runtime evaluation; authoring-only source inspection must be named and isolated as such.

### P0: native Dist and Editor surfaces do not provide equivalent behavior

The source runtime registers the actual scene system and events. The native Dist entry is stateless, exports registration metadata, has no systems/events/commands/bridge methods and explicitly says evaluation remains hosted by the runtime module. Capability negotiation must distinguish a host-carried source runtime from an independently executable native provider.

The Editor registers an Animation view, drawer, templates and three inspector customizations. No animation authoring state, compiler job, preview-generation receipt or operation implementation is owned here. Registering a `.zui` document is not evidence that blend-space/mask/graph editing reaches the runtime artifact.

### P1: unconditional immediate startup and all-target scheduling lack workload policy

The driver and two manager services use `StartupMode::Immediate`; the scene system is registered in `PostUpdate` for client, server and editor targets and runs every scene tick. Partial capability status is appropriately cautious, but there is no target-specific runtime feature selection, demand activation, fixed budget or relevance gate.

Playback settings are protected by a global mutex. Store updates in-memory state before persistence, so a failed config write leaves the service and durable state divergent. Poison recovery accepts the inner value. This is not a demonstrated hot loop, but it needs transaction/currentness semantics before editor controls write settings repeatedly.

### P1: scheduling dependencies describe order, not data ownership

The animation system is placed after `zircon.scene.world_transform`; the pipeline later publishes physics targets and scene transforms through additional World phases. No explicit extract/evaluate/commit resource contract tells the scheduler which data can run concurrently or when physics/render consumers see a pose generation. A string ordering anchor cannot substitute for ownership and task dependencies.

## 3. Reference-engine constraints

Unreal separates live object access from worker evaluation through `FAnimInstanceProxy`: game-thread `PreUpdate` gathers required state, `UpdateAnimation`/`EvaluateAnimation` operate through the proxy, and `PostUpdate` publishes worker results. `USkeletalMeshComponent` owns parallel task dispatch/completion and update-rate policy. Zircon needs the same clarity of ownership while retaining ECS and Rust services.

The reference does not support keeping a public raw evaluator beside the compiled scene pipeline, advertising registration-only native behavior as equivalent execution, or treating editor documents as compiled-product integration.

## 4. Dependency-ordered optimization plan

### M0: publish truthful capability matrices

For source, native, client, server and editor profiles, declare whether registration, compilation, evaluation, events, IK, masks, skinning and authoring/preview are executable. Fail unavailable contributions during profile resolution or disable their UI. Do not infer runtime behavior from a manifest-only native entry.

### M1: converge the manager contract

Replace raw asset evaluation methods with compile/install/instance/update/query contracts backed by the same immutable artifacts used by the scene pipeline. If source inspection helpers remain, move them behind explicitly authoring-only APIs. Remove the duplicate raw graph/state/clip algorithm after callers migrate.

### M2: make activation and target policy explicit

Start evaluation services on admitted demand and attach target policy: dedicated server can skip pose materialization unless gameplay requires it; editor preview uses its own clock/relevance; clients use visibility/LOD/budget. Register scheduler read/write resources and dependency handles for extract, worker evaluation, physics integration and render publication.

### M3: close authoring and native products

Bind every editor surface to typed operations, generation-qualified compiler jobs and exact preview/runtime artifact receipts. Either provide a versioned native bridge that invokes equivalent runtime work or describe Dist as a registration carrier whose required host implementation is an admitted dependency.

### M4: qualify startup and steady-state behavior

Measure plugin resolution/registration/startup, zero-entity steady ticks, target-profile activation, settings writes/failures, editor open/edit/compile/preview and native/source parity. Record p50/p95/p99, allocations/bytes, systems activated, zero-work ticks, task/wait time, stale generations, CPU, wakeups, RSS and power.

## 5. Acceptance gates

1. Exactly one compiled animation algorithm owns runtime graph/state/clip semantics across manager, scene, editor preview and packaging.
2. Source and native capability reports distinguish host-provided execution from provider-owned execution and fail closed when unavailable.
3. Editor surfaces have executable typed operations and install the exact compiled generation preview/runtime consumes.
4. Zero-entity and disabled-target frames perform no animation scan/evaluation/publication work.
5. Scheduler dependencies express data ownership and allow worker overlap without mutable World waits.
6. Current-source source/native/client/server/editor matrices pass dynamic parity before protected-ledger promotion.

## 6. Validation status

- Per-production-Rust-file static review: **19/19 complete** for the captured fingerprint.
- Product caller/capability tracing: complete for runtime registration, manager, editor and Dist surfaces.
- `rustfmt --check --config skip_children=true`: Runtime production, Dist, four Editor files and the scoped target-table contract pass; existing `editor/src/tests.rs` has one formatting-only diff and was preserved.
- Cargo/tests and current-source executable: pending because the managed Windows validation session is unavailable.
- WPR/ETW and power: pending; no launchable current-source executable exists.
- Protected ledgers, milestone commit and quantified WeCom completion remain pending.
